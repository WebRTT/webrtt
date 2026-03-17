# WebRTT — Reference Edge Node

The reference implementation of the [WebRTT protocol](https://github.com/webrtt/spec).

Written in Rust. Built for low-latency voice translation between two participants.

## How It Works

WebRTT edge node runs the full pipeline for each utterance:

```
AUDIO_CHUNK(s) → VAD → STT (Deepgram) → MT (DeepL) → TTS (ElevenLabs)
                                  ↓
                        HYPOTHESIS (speculative, during speech)
                                  ↓
                          COMMIT or REVERT (after speech ends)
```

Speculative hypotheses start flowing to the listener ~150ms after the speaker
begins talking — before the sentence is complete.

## Quickstart

### Requirements

- Rust 1.75+
- API keys: Deepgram, DeepL, ElevenLabs

### Run

```bash
git clone https://github.com/webrtt/webrtt
cd webrtt
cp .env.example .env
# fill in your API keys in .env
cargo run --bin webrtt-edge
```

The server starts on `ws://localhost:3001`.

### Test

```bash
cargo test --all
```

## Protocol

Full specification: [github.com/webrtt/spec](https://github.com/webrtt/spec)

## License

Apache 2.0
