use crate::error::CodecError;
use crate::messages::WebRTTMessage;

/// Encode a WebRTTMessage to MessagePack bytes.
pub fn encode(msg: &WebRTTMessage) -> Result<Vec<u8>, CodecError> {
    rmp_serde::to_vec_named(msg).map_err(CodecError::Encode)
}

/// Decode MessagePack bytes to a WebRTTMessage.
pub fn decode(bytes: &[u8]) -> Result<WebRTTMessage, CodecError> {
    rmp_serde::from_slice(bytes).map_err(CodecError::Decode)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::messages::*;
    use uuid::Uuid;

    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[test]
    fn roundtrip_session_init() {
        let msg = WebRTTMessage::SessionInit(SessionInit {
            session_id: Uuid::new_v4(),
            speaker_id: Uuid::new_v4(),
            source_lang: LanguageCode::PtBr,
            target_lang: LanguageCode::EnUs,
            timestamp_ms: now(),
        });
        let encoded = encode(&msg).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert!(matches!(decoded, WebRTTMessage::SessionInit(_)));
    }

    #[test]
    fn roundtrip_audio_chunk_with_binary() {
        let audio = vec![0u8, 1, 2, 3, 128, 255];
        let msg = WebRTTMessage::AudioChunk(AudioChunk {
            session_id: Uuid::new_v4(),
            speaker_id: Uuid::new_v4(),
            seq: 42,
            audio: audio.clone(),
            timestamp_ms: now(),
        });
        let encoded = encode(&msg).unwrap();
        let decoded = decode(&encoded).unwrap();
        if let WebRTTMessage::AudioChunk(chunk) = decoded {
            assert_eq!(chunk.audio, audio);
            assert_eq!(chunk.seq, 42);
        } else {
            panic!("wrong message type after roundtrip");
        }
    }

    #[test]
    fn roundtrip_hypothesis_empty_audio() {
        let msg = WebRTTMessage::Hypothesis(Hypothesis {
            session_id: Uuid::new_v4(),
            target_speaker_id: Uuid::new_v4(),
            checkpoint_id: Uuid::new_v4(),
            seq: 1,
            tokens: vec!["hello".into(), "how".into()],
            confidence: 0.85,
            audio: vec![],
            timestamp_ms: now(),
        });
        let encoded = encode(&msg).unwrap();
        let decoded = decode(&encoded).unwrap();
        assert!(matches!(decoded, WebRTTMessage::Hypothesis(_)));
    }

    #[test]
    fn roundtrip_all_message_types() {
        let session_id = Uuid::new_v4();
        let speaker_id = Uuid::new_v4();
        let checkpoint_id = Uuid::new_v4();
        let ts = now();

        let messages = vec![
            WebRTTMessage::SessionReady(SessionReady {
                session_id,
                timestamp_ms: ts,
            }),
            WebRTTMessage::SessionEnd(SessionEnd {
                session_id,
                timestamp_ms: ts,
            }),
            WebRTTMessage::SpeechStart(SpeechStart {
                session_id,
                speaker_id,
                timestamp_ms: ts,
            }),
            WebRTTMessage::SpeechEnd(SpeechEnd {
                session_id,
                speaker_id,
                timestamp_ms: ts,
            }),
            WebRTTMessage::Commit(Commit {
                session_id,
                target_speaker_id: speaker_id,
                checkpoint_id,
                tokens: vec!["hi".into()],
                audio: vec![0u8; 100],
                timestamp_ms: ts,
            }),
            WebRTTMessage::Revert(Revert {
                session_id,
                target_speaker_id: speaker_id,
                checkpoint_id,
                timestamp_ms: ts,
            }),
            WebRTTMessage::SessionContext(SessionContext {
                session_id,
                vocab: vec!["test".into()],
                domain: Some("medical".into()),
                timestamp_ms: ts,
            }),
            WebRTTMessage::Error(WebRTTError {
                session_id,
                code: ErrorCode::InternalError,
                message: "something went wrong".into(),
                timestamp_ms: ts,
            }),
        ];

        for msg in messages {
            let encoded = encode(&msg).unwrap();
            let _decoded = decode(&encoded).unwrap();
        }
    }
}
