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
