/// Safe Highrise chat hex palette (readable against Highrise's dark chat UI).
pub const SAFE_COLORS: &[(&str, &str, &str)] = &[
    ("accent", "38BDF8", "Sky blue — headers, titles, highlights"),
    ("blue", "3B82F6", "Bright blue — song titles, links, info"),
    ("success", "22C55E", "Emerald green — confirmations & active states"),
    ("warning", "FFC93D", "Warm gold — warnings & up-next alerts"),
    ("amber", "F59E0B", "Amber — secondary warnings"),
    ("danger", "EF4444", "Red — errors & blocks"),
    ("coral", "FF4D6D", "Soft red — danger/moderation alerts"),
    ("body", "E5E7EB", "Light gray — primary readable body text"),
    ("white", "FFFFFF", "Pure white — crisp contrast"),
    ("bronze", "CD7F32", "Bronze VIP badge color"),
    ("silver", "C0C0C0", "Silver VIP badge color"),
    ("gold", "FFD700", "Gold VIP badge color"),
    ("purple", "B266FF", "Lifetime VIP / royal accent"),
];

pub const HIGHRISE_MAX_MSG_LEN: usize = 250;

struct DocEntry {
    id: &'static str,
    section: &'static str,
    title: &'static str,
    body: &'static str,
}

const DOCS: &[DocEntry] = &[
    DocEntry {
        id: "bot_ws_overview",
        section: "bot_api",
        title: "Highrise Bot WebSocket Protocol",
        body: "Endpoint: wss://highrise.game/web/botapi?events=<comma_separated_events>\n\
               Headers: api-token: <token>, room-id: <room_id>\n\
               Keepalive: send KeepaliveRequest every 15s.\n\
               Correlation: attach `rid` UUID to requests; match incoming `rid` for acks or Error.\n\
               Requirement: bot account must own the room or have Designer rights.",
    },
    DocEntry {
        id: "bot_ws_events",
        section: "bot_api",
        title: "Bot WebSocket Subscribe Events (12/12)",
        body: "SessionMetadata (on_ready), ChatEvent (on_chat / on_whisper via whisper: bool), \
               MessageEvent (on_dm), UserJoinedEvent (on_join), UserLeftEvent (on_leave), \
               UserMovedEvent (on_move with Position | AnchorPosition), EmoteEvent (on_emote), \
               ReactionEvent (on_reaction), TipReactionEvent (on_tip), VoiceEvent (on_voice), \
               RoomModeratedEvent (on_moderate), ChannelEvent (on_channel).",
    },
    DocEntry {
        id: "bot_ws_requests",
        section: "bot_api",
        title: "Bot WebSocket Publish Endpoints (30/30 + Extras)",
        body: "AnchorHitRequest, BuyItemRequest, BuyRoomBoostRequest, BuyVoiceTimeRequest, \
               ChangeRoomPrivilegeRequest, ChannelRequest, ChatRequest, CheckVoiceChatRequest, \
               EmoteRequest, FloorHitRequest, GetBackpackRequest, ChangeBackpackRequest, \
               GetConversationsRequest, GetInventoryRequest, GetMessagesRequest, \
               GetRoomPrivilegeRequest, GetRoomUsersRequest, GetUserOutfitRequest, \
               GetWalletRequest, InviteSpeakerRequest, KeepaliveRequest, \
               LeaveConversationRequest, MessageMediaRequest, ModerateRoomRequest, \
               MoveUserToRoomRequest, ReactionRequest, RemoveSpeakerRequest, \
               SendMessageRequest, SendBulkMessageRequest, SetOutfitRequest, \
               TeleportRequest, TipUserRequest, IndicatorRequest.",
    },
    DocEntry {
        id: "webapi_overview",
        section: "web_api",
        title: "Highrise Public Web REST API",
        body: "Base URL: https://webapi.highrise.game (override with HR_WEBAPI_URL).\n\
               Cursor pagination: starts_after, ends_before, sort_order (asc|desc), limit.\n\
               Rate limits: HTTP 429 with Retry-After header -> Error::RateLimited.\n\
               Endpoints (12): GET /users/{id}, GET /users, GET /rooms/{id}, GET /rooms, \
               GET /posts/{id}, GET /posts, GET /items/{id}, GET /items, GET /items/search, \
               GET /grabs/{id}, GET /grabs, POST /conversations/{id}/messages (requires api-token header).",
    },
    DocEntry {
        id: "rich_text_colors",
        section: "guides",
        title: "Highrise Rich Text, Safe Hex Codes & Bold Styling",
        body: "Color tags: use <#RRGGBB>text (6-char uppercase hex, no # before <). Example: <#38BDF8>Title\\n<#E5E7EB>Body.\n\
               Bold text: Markdown **bold** does NOT render in Highrise; use Unicode Mathematical Sans-Serif Bold \
               (supported by format_highrise_text tool).\n\
               Message limit: keep each chat/whisper <= 250 chars and chunk on newlines.\n\
               Whisper vs Chat: use whisper() for command errors/private replies and chat() for room announcements.",
    },
    DocEntry {
        id: "empty_room_hibernation",
        section: "guides",
        title: "Empty Room Hibernation (code 1006) & Auto-Reconnect",
        body: "Highrise closes empty room instances and drops bot WebSockets with code 1006.\n\
               highrise-rs Client::run() automatically reconnects with exponential backoff (2s..30s) \
               and stops only on fatal auth/permission errors (invalid token, missing designer rights).",
    },
];

pub fn search_docs(query: &str, section: Option<&str>) -> String {
    let q = query.trim().to_ascii_lowercase();
    let mut hits = Vec::new();
    for doc in DOCS {
        if let Some(sec) = section {
            if !doc.section.eq_ignore_ascii_case(sec) {
                continue;
            }
        }
        if q.is_empty()
            || doc.title.to_ascii_lowercase().contains(&q)
            || doc.body.to_ascii_lowercase().contains(&q)
            || doc.id.to_ascii_lowercase().contains(&q)
        {
            hits.push(format!("### {} [{}]\n{}", doc.title, doc.id, doc.body));
        }
    }
    if hits.is_empty() {
        "No matching Highrise docs found. Try queries like: websocket, webapi, hex, bold, 1006, events, requests.".to_string()
    } else {
        hits.join("\n\n")
    }
}

pub fn get_guide(topic: &str) -> String {
    match topic.trim().to_ascii_lowercase().as_str() {
        "quickstart" => r#"use highrise_rs::{BotHandler, BoxFuture, Client, Config, Highrise, SessionMetadata, User};

struct MyBot;
impl BotHandler for MyBot {
    fn on_ready<'a>(&'a self, api: &'a Highrise, meta: SessionMetadata) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            let _ = api.chat(format!("Online in {}", meta.room_info.room_name)).await;
        })
    }
    fn on_chat<'a>(&'a self, api: &'a Highrise, user: User, msg: String) -> BoxFuture<'a, ()> {
        Box::pin(async move {
            if msg == "!ping" { let _ = api.whisper(&user.id, "Pong!").await; }
        })
    }
}

#[tokio::main]
async fn main() {
    let cfg = Config::new(std::env::var("API_TOKEN").unwrap(), std::env::var("ROOM_ID").unwrap());
    Client::new(cfg, MyBot).run().await.unwrap();
}"#.to_string(),
        "webapi" => r#"use highrise_rs::{PageQuery, SortOrder, WebApi};

async fn fetch_example() -> highrise_rs::Result<()> {
    let web = WebApi::new();
    let user = web.get_user("6ab12b37b8baea2da8891b5f").await?;
    let items = web.search_items(&PageQuery { limit: Some(10), sort_order: SortOrder::Desc, ..Default::default() }, "hoodie", None).await?;
    println!("{} ({} items)", user.user.username, items.items.len());
    Ok(())
}"#.to_string(),
        "colors" | "hex" | "formatting" => {
            let mut out = String::from("Safe Highrise Hex Palette (use <#HEX>text):\n");
            for (name, hex, desc) in SAFE_COLORS {
                out.push_str(&format!("- {name}: <#{hex}> ({desc})\n"));
            }
            out.push_str("\nTip: Keep messages <= 250 chars per send and use Unicode bold instead of **markdown**.");
            out
        }
        "moderation" => "BotHandler::on_moderate + Highrise::kick(&uid), mute(&uid, secs), ban(&uid, secs), unban(&uid), set_privilege(&uid, RoomPermissions { moderator, designer }).".to_string(),
        "economy" => "Use tip(&uid, GoldBar::G10), wallet(), inventory(), buy_item(&item_id), buy_room_boost(amount), buy_voice_time().".to_string(),
        _ => "Available topics: quickstart, webapi, colors, moderation, economy.".to_string(),
    }
}

pub fn get_endpoint(name: &str) -> String {
    let n = name.trim().to_ascii_lowercase();
    match n.as_str() {
        "chatrequest" | "chat" | "whisper" => {
            "Publish: ChatRequest { _type: \"ChatRequest\", message, whisper_target_id: Option<String> }\n\
             Rust SDK: api.chat(msg) | api.whisper(target_user_id, msg)"
        }
        "emoterequest" | "emote" => {
            "Publish: EmoteRequest { _type: \"EmoteRequest\", emote_id, target_user_id }\n\
             Rust SDK: api.emote(emote_id, target_user_id)"
        }
        "teleportrequest" | "teleport" => {
            "Publish: TeleportRequest { _type: \"TeleportRequest\", user_id, destination: { x, y, z, facing } }\n\
             Rust SDK: api.teleport(user_id, x, y, z, Facing::FrontRight)"
        }
        "getroomusersrequest" | "room_users" => {
            "Publish: GetRoomUsersRequest {}\n\
             Response: GetRoomUsersResponse { content: [(User, Position|AnchorPosition)] }\n\
             Rust SDK: api.room_users() -> Result<Vec<RoomUser>>"
        }
        "usermovedevent" | "on_move" => {
            "Subscribe: UserMovedEvent { user: User, position: Position | AnchorPosition }\n\
             Rust SDK: BotHandler::on_move(api, user, EntityPosition::Floor|Anchor)"
        }
        "tipreactionevent" | "on_tip" => {
            "Subscribe: TipReactionEvent { sender: User, receiver: User, item: CurrencyItem { type, amount } }\n\
             Rust SDK: BotHandler::on_tip(api, sender, receiver, item)"
        }
        "get /items/search" | "search_items" => {
            "Web API: GET https://webapi.highrise.game/items/search?query=...&limit=20&sort_order=desc\n\
             Rust SDK: web.search_items(&page, query, category) -> Result<SearchItemsResponse>"
        }
        "post /conversations/{id}/messages" | "send_conversation_message" => {
            "Web API: POST https://webapi.highrise.game/conversations/{id}/messages (header: api-token)\n\
             Rust SDK: web.send_conversation_message(conversation_id, api_token, content)"
        }
        _ => return search_docs(name, None),
    }
    .to_string()
}

/// Converts ASCII letters and digits into Unicode Mathematical Sans-Serif Bold
/// so bold renders inside Highrise chat/whispers (since Highrise ignores `**`).
pub fn to_highrise_bold(text: &str) -> String {
    text.chars()
        .map(|ch| match ch {
            'A'..='Z' => char::from_u32(ch as u32 + 120_211).unwrap_or(ch),
            'a'..='z' => char::from_u32(ch as u32 + 120_205).unwrap_or(ch),
            '0'..='9' => char::from_u32(ch as u32 + 120_764).unwrap_or(ch),
            _ => ch,
        })
        .collect()
}

/// Formats and chunks text safely for Highrise chat (`<#RRGGBB>` + <=250 char chunks).
pub fn format_highrise_text(text: &str, hex: Option<&str>, bold: bool) -> serde_json::Value {
    let body = if bold {
        to_highrise_bold(text)
    } else {
        text.to_string()
    };

    let clean_hex = hex.map(|h| h.trim().trim_start_matches('#').to_ascii_uppercase());
    let hex_valid = clean_hex
        .as_ref()
        .map(|h| h.len() == 6 && h.chars().all(|c| c.is_ascii_hexdigit()))
        .unwrap_or(true);

    let formatted = match (&clean_hex, hex_valid) {
        (Some(h), true) => format!("<#{h}>{body}"),
        _ => body,
    };

    let chunks = chunk_message(&formatted, HIGHRISE_MAX_MSG_LEN);
    serde_json::json!({
        "formatted": formatted,
        "hex_valid": hex_valid,
        "char_count": formatted.chars().count(),
        "chunks": chunks,
        "chunk_count": chunks.len(),
    })
}

fn chunk_message(text: &str, max_chars: usize) -> Vec<String> {
    if text.chars().count() <= max_chars {
        return vec![text.to_string()];
    }
    let mut chunks = Vec::new();
    let mut current = String::new();
    for line in text.split('\n') {
        let candidate_len = if current.is_empty() {
            line.chars().count()
        } else {
            current.chars().count() + 1 + line.chars().count()
        };
        if candidate_len <= max_chars {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        } else {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            for part in line.chars().collect::<Vec<_>>().chunks(max_chars) {
                chunks.push(part.iter().collect());
            }
        }
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}
