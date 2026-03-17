#!/usr/bin/env bash
# Sends a pre-recorded WAV file as a WebRTT session.
# Requires: websocat, cargo build
#
# Usage: ./scripts/test_session.sh [path/to/audio.wav]
#
# This script is a placeholder — full implementation requires
# a binary tool that speaks the WebRTT MessagePack protocol.

set -euo pipefail

WAV_FILE="${1:-}"
SERVER="${WEBRTT_SERVER:-ws://127.0.0.1:3001}"

if [ -z "$WAV_FILE" ]; then
    echo "Usage: $0 <path/to/audio.wav>"
    echo ""
    echo "Sends PCM audio from a WAV file to the WebRTT edge node."
    echo "Server: $SERVER"
    exit 1
fi

if [ ! -f "$WAV_FILE" ]; then
    echo "Error: file not found: $WAV_FILE"
    exit 1
fi

echo "WebRTT test session"
echo "  Server: $SERVER"
echo "  Audio:  $WAV_FILE"
echo ""
echo "NOTE: This script requires a binary client that speaks MessagePack."
echo "      For now, use the integration tests: cargo test -p webrtt-test -- --ignored"
