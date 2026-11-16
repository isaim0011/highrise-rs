# highrise-rs — Core SDK Plan

## Goal
Build a fast, lightweight, idiomatic Rust SDK (`highrise-rs`) for Highrise's Bot WebSocket API (`wss://highrise.game/web/botapi`), combining the ergonomics of the official Python `highrise-bot-sdk` (`BaseBot` trait hooks) with full event/request coverage (including `ReactionEvent` and `VoiceEvent` missing from `highrise.bot` v2.8.1).

## Architecture
- **Small, single-purpose modules** (each < 200 lines):
  - `src/error.rs`: Typed `Error` enum with fatal-vs-recoverable classification.
  - `src/types.rs`: Core models (`User`, `Position`, `Facing`, `GoldBar`, `Reaction`, `SessionMetadata`, etc.).
  - `src/protocol/events.rs`: Strongly-typed incoming `WsEvent` enum matching Highrise `_type` payloads.
  - `src/protocol/requests.rs`: Outgoing `WsRequest` payloads with `rid` correlation.
  - `src/transport.rs`: `tokio-tungstenite` (rustls) connection, 15s `KeepaliveRequest` loop, `rid` -> `oneshot` pending map with 10s timeout.
  - `src/api.rs`: High-level `Highrise` action handle (`chat`, `whisper`, `emote`, `teleport`, `room_users`, moderation, voice, etc.).
  - `src/client.rs`: `BotHandler` trait + `Client::run()` reconnect loop with jittered exponential backoff.
  - `src/lib.rs`: Clean public API exports.

## Verification Strategy
1. `cargo check` and `cargo clippy --all-targets -- -D warnings` (zero warnings).
2. `cargo test` covering protocol JSON round-trips, `rid` matching, and error classification.
