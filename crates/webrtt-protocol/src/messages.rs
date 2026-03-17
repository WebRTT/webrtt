use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// All valid WebRTT message types.
/// Tagged with `type` field for MessagePack serialization.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WebRTTMessage {
    SessionInit(SessionInit),
    SessionReady(SessionReady),
    SessionEnd(SessionEnd),
    SpeechStart(SpeechStart),
    SpeechEnd(SpeechEnd),
    AudioChunk(AudioChunk),
    Hypothesis(Hypothesis),
    Commit(Commit),
    Revert(Revert),
    SessionContext(SessionContext),
    Error(WebRTTError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInit {
    pub session_id: Uuid,
    pub speaker_id: Uuid,
    pub source_lang: LanguageCode,
    pub target_lang: LanguageCode,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReady {
    pub session_id: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionEnd {
    pub session_id: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechStart {
    pub session_id: Uuid,
    pub speaker_id: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpeechEnd {
    pub session_id: Uuid,
    pub speaker_id: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioChunk {
    pub session_id: Uuid,
    pub speaker_id: Uuid,
    pub seq: u64,
    #[serde(with = "serde_bytes")]
    pub audio: Vec<u8>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hypothesis {
    pub session_id: Uuid,
    pub target_speaker_id: Uuid,
    pub checkpoint_id: Uuid,
    pub seq: u64,
    pub tokens: Vec<String>,
    pub confidence: f32,
    #[serde(with = "serde_bytes", default, skip_serializing_if = "Vec::is_empty")]
    pub audio: Vec<u8>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub session_id: Uuid,
    pub target_speaker_id: Uuid,
    pub checkpoint_id: Uuid,
    pub tokens: Vec<String>,
    #[serde(with = "serde_bytes")]
    pub audio: Vec<u8>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Revert {
    pub session_id: Uuid,
    pub target_speaker_id: Uuid,
    pub checkpoint_id: Uuid,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: Uuid,
    pub vocab: Vec<String>,
    pub domain: Option<String>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRTTError {
    pub session_id: Uuid,
    pub code: ErrorCode,
    pub message: String,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    SessionNotFound,
    LangNotSupported,
    AudioFormatInvalid,
    RateLimitExceeded,
    InternalError,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum LanguageCode {
    PtBr,
    EnUs,
    EsEs,
    FrFr,
    DeDe,
}

impl LanguageCode {
    /// Returns the BCP 47 string used by external APIs
    pub fn as_bcp47(&self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::EnUs => "en-US",
            Self::EsEs => "es-ES",
            Self::FrFr => "fr-FR",
            Self::DeDe => "de-DE",
        }
    }

    /// Returns the Deepgram language code
    pub fn as_deepgram(&self) -> &'static str {
        match self {
            Self::PtBr => "pt-BR",
            Self::EnUs => "en-US",
            Self::EsEs => "es",
            Self::FrFr => "fr",
            Self::DeDe => "de",
        }
    }

    /// Returns the DeepL target language code
    pub fn as_deepl(&self) -> &'static str {
        match self {
            Self::PtBr => "PT-BR",
            Self::EnUs => "EN-US",
            Self::EsEs => "ES",
            Self::FrFr => "FR",
            Self::DeDe => "DE",
        }
    }
}
