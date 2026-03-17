use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use futures::{SinkExt, StreamExt};
use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::WebSocketStream;
use tracing::{debug, info, instrument, warn};
use tungstenite::Message as WsMessage;
use uuid::Uuid;

use crate::config::Config;
use crate::registry::SessionRegistry;
use crate::speculation::SpeculationEngine;
use webrtt_protocol::{codec, messages::*};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SessionState {
    WaitingForInit,
    Idle,
    Listening { speaker_id: Uuid },
    Processing { speaker_id: Uuid },
    Terminated,
}

#[allow(dead_code)]
pub struct SpeakerSlot {
    pub speaker_id: Uuid,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub tx: mpsc::Sender<Vec<u8>>,
}

#[allow(dead_code)]
pub struct Session {
    pub id: Uuid,
    pub state: SessionState,
    pub speakers: Vec<SpeakerSlot>,
    pub speculation: SpeculationEngine,
    pub created_at: Instant,
}

#[allow(dead_code)]
impl Session {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: SessionState::WaitingForInit,
            speakers: Vec::with_capacity(2),
            speculation: SpeculationEngine::new(),
            created_at: Instant::now(),
        }
    }

    pub fn other_speaker(&self, speaker_id: &Uuid) -> Option<&SpeakerSlot> {
        self.speakers.iter().find(|s| &s.speaker_id != speaker_id)
    }

    pub fn find_speaker(&self, speaker_id: &Uuid) -> Option<&SpeakerSlot> {
        self.speakers.iter().find(|s| &s.speaker_id == speaker_id)
    }
}

#[instrument(skip_all, fields(peer = %peer_addr))]
pub async fn handle_connection(
    ws_stream: WebSocketStream<tokio::net::TcpStream>,
    registry: Arc<SessionRegistry>,
    config: Arc<Config>,
    peer_addr: SocketAddr,
) -> anyhow::Result<()> {
    let (mut ws_sink, mut ws_source) = ws_stream.split();

    let (tx, mut rx) = mpsc::channel::<Vec<u8>>(64);

    tokio::spawn(async move {
        while let Some(bytes) = rx.recv().await {
            if ws_sink.send(WsMessage::Binary(bytes)).await.is_err() {
                break;
            }
        }
    });

    let mut session_arc: Option<Arc<RwLock<Session>>> = None;
    let mut my_speaker_id: Option<Uuid> = None;

    while let Some(msg) = ws_source.next().await {
        let msg = match msg {
            Ok(WsMessage::Binary(bytes)) => match codec::decode(&bytes) {
                Ok(m) => m,
                Err(e) => {
                    warn!(error = %e, "failed to decode message");
                    continue;
                }
            },
            Ok(WsMessage::Close(_)) => break,
            Ok(WsMessage::Ping(_)) => {
                continue;
            }
            _ => continue,
        };

        debug!(msg_type = ?std::mem::discriminant(&msg), "received message");

        match msg {
            WebRTTMessage::SessionInit(init) => {
                handle_session_init(
                    init,
                    &registry,
                    &config,
                    tx.clone(),
                    &mut session_arc,
                    &mut my_speaker_id,
                )
                .await?;
            }

            WebRTTMessage::SpeechStart(start) => {
                if let Some(arc) = &session_arc {
                    let mut session = arc.write().await;
                    session.state = SessionState::Listening {
                        speaker_id: start.speaker_id,
                    };
                    debug!(speaker = %start.speaker_id, "speech started");
                }
            }

            WebRTTMessage::AudioChunk(chunk) => {
                if let Some(arc) = &session_arc {
                    handle_audio_chunk(chunk, arc, &config).await;
                }
            }

            WebRTTMessage::SpeechEnd(end) => {
                if let Some(arc) = &session_arc {
                    handle_speech_end(end, arc, &config).await;
                }
            }

            WebRTTMessage::SessionEnd(_) => {
                if let Some(arc) = &session_arc {
                    let session = arc.read().await;
                    registry.remove(&session.id).await;
                    info!(session = %session.id, "session ended");
                }
                break;
            }

            _ => {
                warn!("unexpected message type from client");
            }
        }
    }

    if let Some(arc) = &session_arc {
        let session = arc.read().await;
        registry.remove(&session.id).await;
        info!(
            session = %session.id,
            duration_secs = session.created_at.elapsed().as_secs(),
            "connection closed"
        );
    }

    Ok(())
}

async fn handle_session_init(
    init: SessionInit,
    registry: &SessionRegistry,
    _config: &Config,
    tx: mpsc::Sender<Vec<u8>>,
    session_arc: &mut Option<Arc<RwLock<Session>>>,
    my_speaker_id: &mut Option<Uuid>,
) -> anyhow::Result<()> {
    let arc = match registry.get(&init.session_id).await {
        Some(existing) => existing,
        None => {
            let session = Session::new(init.session_id);
            registry.insert(session).await
        }
    };

    {
        let mut session = arc.write().await;
        session.speakers.push(SpeakerSlot {
            speaker_id: init.speaker_id,
            source_lang: init.source_lang,
            target_lang: init.target_lang,
            tx: tx.clone(),
        });
    }

    *session_arc = Some(Arc::clone(&arc));
    *my_speaker_id = Some(init.speaker_id);

    let ready = WebRTTMessage::SessionReady(SessionReady {
        session_id: init.session_id,
        timestamp_ms: now_ms(),
    });
    let bytes = codec::encode(&ready)?;
    tx.send(bytes).await.ok();

    info!(session = %init.session_id, speaker = %init.speaker_id, "session init complete");
    Ok(())
}

async fn handle_audio_chunk(
    chunk: AudioChunk,
    _session_arc: &Arc<RwLock<Session>>,
    _config: &Config,
) {
    debug!(
        seq = chunk.seq,
        bytes = chunk.audio.len(),
        "audio chunk received"
    );
}

async fn handle_speech_end(end: SpeechEnd, _session_arc: &Arc<RwLock<Session>>, _config: &Config) {
    debug!(speaker = %end.speaker_id, "speech ended");
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}
