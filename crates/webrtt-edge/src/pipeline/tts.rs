// ElevenLabs streaming TTS client.
//
// KEY REQUIREMENT: use the streaming endpoint with maximum latency optimization.
//
// Endpoint:
//   POST https://api.elevenlabs.io/v1/text-to-speech/{voice_id}/stream
//   ?optimize_streaming_latency=4
//   xi-api-key: {api_key}
//   Content-Type: application/json
//   Body: {
//     "text": "...",
//     "model_id": "eleven_turbo_v2",
//     "voice_settings": { "stability": 0.5, "similarity_boost": 0.75 }
//   }
//
// Returns: chunked MP3 audio stream.
// We collect all chunks and concatenate — or stream them directly to client.
//
// Model choice: eleven_turbo_v2 is faster than eleven_multilingual_v2
// Use eleven_multilingual_v2 only when the target language requires it.

use bytes::Bytes;
use futures::Stream;
use futures::StreamExt;

#[allow(dead_code)]
pub struct ElevenLabsClient {
    http: reqwest::Client,
    api_key: String,
    voice_id: String,
}

#[allow(dead_code)]
impl ElevenLabsClient {
    pub fn new(api_key: String, voice_id: String) -> Self {
        Self {
            http: reqwest::Client::new(),
            api_key,
            voice_id,
        }
    }

    pub async fn synthesize(&self, text: &str) -> anyhow::Result<Vec<u8>> {
        let mut stream = Box::pin(self.synthesize_stream(text).await?);
        let mut audio = Vec::new();
        while let Some(chunk) = stream.next().await {
            audio.extend_from_slice(&chunk?);
        }
        Ok(audio)
    }

    pub async fn synthesize_stream(
        &self,
        text: &str,
    ) -> anyhow::Result<impl Stream<Item = anyhow::Result<Bytes>>> {
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}/stream?optimize_streaming_latency=4",
            self.voice_id
        );

        let resp = self
            .http
            .post(&url)
            .header("xi-api-key", &self.api_key)
            .header("Content-Type", "application/json")
            .json(&serde_json::json!({
                "text": text,
                "model_id": "eleven_turbo_v2",
                "voice_settings": {
                    "stability": 0.5,
                    "similarity_boost": 0.75
                }
            }))
            .send()
            .await?
            .error_for_status()?;

        Ok(resp.bytes_stream().map(|r| r.map_err(Into::into)))
    }
}
