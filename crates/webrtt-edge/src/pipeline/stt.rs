// Deepgram Streaming STT client.
//
// KEY REQUIREMENT: one persistent WebSocket connection per session.
// Do NOT reconnect per utterance. Reconnecting adds 200ms+ of TLS handshake.
//
// Deepgram streaming endpoint:
//   wss://api.deepgram.com/v1/listen
//   ?encoding=linear16
//   &sample_rate=16000
//   &channels=1
//   &language={lang}
//   &interim_results=true
//   &endpointing=false
//
// Deepgram sends JSON results over the WebSocket as speech is processed.
// We parse them and forward partial transcripts to the speculation engine.

use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::{broadcast, mpsc};
use tokio_tungstenite::tungstenite::{self, http::Request};
use tracing::{debug, error, warn};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PartialTranscript {
    pub text: String,
    pub confidence: f32,
    pub is_final: bool,
}

#[allow(dead_code)]
pub struct DeepgramClient {
    audio_tx: mpsc::Sender<Vec<u8>>,
    pub transcript_rx: broadcast::Receiver<PartialTranscript>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepgramResponse {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    channel: Option<DeepgramChannel>,
    is_final: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepgramChannel {
    alternatives: Vec<DeepgramAlternative>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeepgramAlternative {
    transcript: String,
    confidence: f32,
}

#[allow(dead_code)]
impl DeepgramClient {
    pub async fn connect(api_key: &str, lang: &str) -> anyhow::Result<Self> {
        let url = format!(
            "wss://api.deepgram.com/v1/listen\
             ?encoding=linear16\
             &sample_rate=16000\
             &channels=1\
             &language={}\
             &interim_results=true\
             &endpointing=false",
            lang
        );

        let request = Request::builder()
            .uri(&url)
            .header("Authorization", format!("Token {}", api_key))
            .header("Host", "api.deepgram.com")
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header(
                "Sec-WebSocket-Key",
                tungstenite::handshake::client::generate_key(),
            )
            .body(())?;

        let (ws_stream, _) = tokio_tungstenite::connect_async(request).await?;
        let (mut ws_sink, mut ws_source) = ws_stream.split();

        let (audio_tx, mut audio_rx) = mpsc::channel::<Vec<u8>>(128);
        let (transcript_tx, transcript_rx) = broadcast::channel::<PartialTranscript>(64);

        // Write task: forward PCM audio to Deepgram
        tokio::spawn(async move {
            while let Some(pcm) = audio_rx.recv().await {
                if ws_sink
                    .send(tungstenite::Message::Binary(pcm))
                    .await
                    .is_err()
                {
                    break;
                }
            }
            // Send close message to signal end of audio
            let _ = ws_sink
                .send(tungstenite::Message::Text(
                    r#"{"type":"CloseStream"}"#.into(),
                ))
                .await;
        });

        // Read task: parse Deepgram JSON responses
        let tx = transcript_tx.clone();
        tokio::spawn(async move {
            while let Some(msg) = ws_source.next().await {
                match msg {
                    Ok(tungstenite::Message::Text(text)) => {
                        match serde_json::from_str::<DeepgramResponse>(&text) {
                            Ok(resp) => {
                                if let Some(channel) = resp.channel {
                                    if let Some(alt) = channel.alternatives.first() {
                                        if !alt.transcript.is_empty() {
                                            let partial = PartialTranscript {
                                                text: alt.transcript.clone(),
                                                confidence: alt.confidence,
                                                is_final: resp.is_final.unwrap_or(false),
                                            };
                                            debug!(
                                                text = %partial.text,
                                                confidence = partial.confidence,
                                                is_final = partial.is_final,
                                                "deepgram transcript"
                                            );
                                            let _ = tx.send(partial);
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "failed to parse deepgram response");
                            }
                        }
                    }
                    Ok(tungstenite::Message::Close(_)) => break,
                    Err(e) => {
                        error!(error = %e, "deepgram websocket error");
                        break;
                    }
                    _ => {}
                }
            }
        });

        Ok(Self {
            audio_tx,
            transcript_rx,
        })
    }

    pub async fn send_audio(&self, pcm: Vec<u8>) -> anyhow::Result<()> {
        self.audio_tx.send(pcm).await.map_err(Into::into)
    }
}
