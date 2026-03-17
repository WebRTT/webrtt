// Integration tests for WebRTT edge node.
// These tests simulate two clients connecting to the server.
// They require the server to be running and API keys configured.
// Gate behind #[ignore] for CI — run locally with: cargo test -- --ignored

#[cfg(test)]
mod tests {
    use futures::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message as WsMessage;
    use uuid::Uuid;
    use webrtt_protocol::{codec, messages::*};

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64
    }

    #[tokio::test]
    #[ignore]
    async fn full_session_two_clients() {
        let url = "ws://127.0.0.1:3001";
        let session_id = Uuid::new_v4();
        let speaker_a = Uuid::new_v4();
        let speaker_b = Uuid::new_v4();

        // Connect Speaker A
        let (mut ws_a, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("failed to connect speaker A");

        // Send SESSION_INIT for Speaker A
        let init_a = WebRTTMessage::SessionInit(SessionInit {
            session_id,
            speaker_id: speaker_a,
            source_lang: LanguageCode::PtBr,
            target_lang: LanguageCode::EnUs,
            timestamp_ms: now_ms(),
        });
        ws_a.send(WsMessage::Binary(codec::encode(&init_a).unwrap()))
            .await
            .unwrap();

        // Expect SESSION_READY
        let msg = ws_a.next().await.unwrap().unwrap();
        if let WsMessage::Binary(bytes) = msg {
            let decoded = codec::decode(&bytes).unwrap();
            assert!(matches!(decoded, WebRTTMessage::SessionReady(_)));
        } else {
            panic!("expected binary message");
        }

        // Connect Speaker B
        let (mut ws_b, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("failed to connect speaker B");

        let init_b = WebRTTMessage::SessionInit(SessionInit {
            session_id,
            speaker_id: speaker_b,
            source_lang: LanguageCode::EnUs,
            target_lang: LanguageCode::PtBr,
            timestamp_ms: now_ms(),
        });
        ws_b.send(WsMessage::Binary(codec::encode(&init_b).unwrap()))
            .await
            .unwrap();

        let msg = ws_b.next().await.unwrap().unwrap();
        if let WsMessage::Binary(bytes) = msg {
            let decoded = codec::decode(&bytes).unwrap();
            assert!(matches!(decoded, WebRTTMessage::SessionReady(_)));
        }

        // End session
        let end = WebRTTMessage::SessionEnd(SessionEnd {
            session_id,
            timestamp_ms: now_ms(),
        });
        ws_a.send(WsMessage::Binary(codec::encode(&end).unwrap()))
            .await
            .unwrap();
        ws_b.send(WsMessage::Binary(codec::encode(&end).unwrap()))
            .await
            .unwrap();
    }
}
