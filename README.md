# highrise-rs

Fast, async Rust SDK for the **Highrise Bot WebSocket API**, **Highrise Web REST API**, plus a built-in **`highrise_mcp`** stdio MCP server.

## Features
- **Full Bot WebSocket API**: All 30 Publish endpoints (plus `IndicatorRequest`, `ChangeBackpackRequest`, `SendBulkMessageRequest`, `upload_media`, `send_invite`) and all 12 Subscribe events.
- **Full Web REST API (`WebApi`)**: All 12 endpoints across `Users`, `Rooms`, `Posts`, `Items` (including `/items/search`), `Grabs`, and `Conversations`, with cursor pagination and HTTP `429` handling.
- **Production Bot Tools (`highrise_rs::tools`)**:
  - `CommandManager`: Multi-prefix, longest-match subcommands, aliases, `Role` hierarchy (`Member < Vip < Mod < Owner < Creator`), runtime overrides, and cooldowns.
  - `DanceFloorManager`: 3D bounding-box zones (`Fixed`, `SyncRandom`, `PerUserRandom`) + `set_zone_from_users()`.
  - `Emotes`: Live-syncing via `Emotes::sync_pack(EmotePack::Free | EmotePack::All)` from `Djxhknz/Ids-highrise` (with bundled `280` free & `371` free+paid fallback), `import_json()`, `import_from_url()`, `block()`, and lookup by number, ID, or name.
  - `ItemCatalog`: Live-syncing `untradable_items_new.json` from `Djxhknz/Ids-highrise` (`ItemCatalog::sync_from_github()`), `search()`, `by_category()`, and `to_outfit_item()`.
  - `HighriseCluster`: Multi-bot Tokio supervisor with `ClusterHandle::shutdown()` and live status tracking.
  - **Dynamic DX helpers**: `find_user("@name")`, `user_position("@name")`, `goto_user("@name")`, `bring_user("@target", "@dest")`, `emote_user("1", "@target", &emotes)`.
- **Built-in MCP Server (`highrise_mcp`)**: Offline docs, endpoint lookup, safe Highrise `<#RRGGBB>` hex color formatting, Unicode bold, and `<= 250`-char message chunking.

## Quickstart

```rust
use highrise_rs::{BotHandler, BoxFuture, Client, Config, Highrise, SessionMetadata, User};

struct EchoBot;

impl BotHandler for EchoBot {
    fn on_ready<'a>(&'a self, _api: &'a Highrise, meta: SessionMetadata) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            println!("Connected as {} in {}", meta.user_id, meta.room_info.room_name);
        })
    }

    fn on_chat<'a>(&'a self, api: &'a Highrise, user: User, message: String) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if message == "!ping" {
                let _ = api.whisper(&user.id, "Pong!").await;
            }
        })
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("API_TOKEN").expect("API_TOKEN required");
    let room_id = std::env::var("ROOM_ID").expect("ROOM_ID required");
    Client::new(Config::new(token, room_id), EchoBot).run().await.unwrap();
}
```

## MCP Server

```bash
cargo run --bin highrise_mcp
```

## Running multiple bots on one host (avoiding Multilogin collisions)

If you run two or more bots (distinct accounts/tokens) from the same machine and
have them join the same room, Highrise can flag the simultaneous sessions from
one outbound IP as a conflict: bots swap/kick each other, and you'll see
`server error event: Multilogin` in the logs. This is a platform-side,
IP-based check, not a bug in any one bot's token or room ID — it's a known
issue reported by multiple unrelated developers hosting multi-bot setups on a
single VPS.

As of `0.1.5`, set the `BIND_SOURCE_IP` environment variable per bot process to
give each one a distinct outbound IP (e.g. a secondary VNIC or a secondary
private IP with its own reserved public IP on a cloud VM):

```bash
# bot A
BIND_SOURCE_IP=10.0.0.21 ./bot-a

# bot B (different local IP, e.g. a secondary VNIC's address)
BIND_SOURCE_IP=10.0.0.253 ./bot-b
```

Both the bot-api websocket (`Transport::connect`) and the `WebApi` REST client
honor this automatically — no other code changes needed. Set it per-process
(systemd `Environment=`, not one shared `.env` read by every bot), or the fix
won't do anything.

## Changelog highlights

- **0.1.8** — Re-tuned the ghost-session watchdog: it now requires ~5 minutes
  of *continuous* self-absence from `room_users()` (15 misses at a 20s
  interval) before forcing a reconnect, instead of either the old 90s
  false-positive threshold (0.1.6 and earlier) or no automatic recovery at
  all (0.1.7). Real "connected but never rendered in-room" ghost sessions
  self-heal within a few minutes; normal transient roster gaps do not trip it.
- **0.1.7** — The internal `room_users()`-based "ghost session" watchdog no
  longer forced a reconnect on its own. Removed entirely because the old
  check fired constantly (every ~90s) on bots simply not being listed in
  their own room roster, which is normal — but this also meant genuine
  ghost sessions had no automatic recovery until 0.1.8.
- **0.1.5** — Added opt-in `BIND_SOURCE_IP` support (see above) and lowered
  the default reconnect backoff from `2s..30s` to `500ms..3s`, so multiple
  bots recover together within a few seconds of an empty-room hibernation
  instead of drifting tens of seconds apart.
