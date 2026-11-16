use serde::Deserialize;
use serde_json::Value;

use crate::types::{CurrencyItem, EntityPosition, Reaction, RoomInfo, User};

/// All incoming WebSocket frames from `wss://highrise.game/web/botapi`.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "_type")]
pub enum WsEvent {
    SessionMetadata {
        user_id: String,
        #[serde(default)]
        room_info: RoomInfo,
        #[serde(default)]
        rate_limits: Value,
        #[serde(default)]
        connection_id: String,
        #[serde(default)]
        sdk_version: String,
    },
    ChatEvent {
        user: User,
        message: String,
        #[serde(default)]
        whisper: bool,
    },
    UserJoinedEvent {
        user: User,
        position: EntityPosition,
    },
    UserLeftEvent {
        user: User,
    },
    UserMovedEvent {
        user: User,
        position: EntityPosition,
    },
    EmoteEvent {
        user: User,
        emote_id: String,
        receiver: User,
    },
    ReactionEvent {
        user: User,
        reaction: Reaction,
        receiver: User,
    },
    TipReactionEvent {
        sender: User,
        receiver: User,
        item: CurrencyItem,
    },
    MessageEvent {
        user_id: String,
        conversation_id: String,
        #[serde(default)]
        is_new_conversation: bool,
    },
    RoomModeratedEvent {
        #[serde(alias = "moderatorId", default)]
        moderator_id: String,
        #[serde(alias = "targetUserId", default)]
        target_user_id: String,
        #[serde(alias = "moderationType", default)]
        moderation_type: String,
        #[serde(default)]
        duration: Option<u64>,
    },
    ChannelEvent {
        sender_id: String,
        msg: String,
        #[serde(default)]
        tags: Vec<String>,
    },
    VoiceEvent {
        #[serde(default)]
        users: Vec<(User, String)>,
        #[serde(default)]
        seconds_left: u64,
    },
    KeepaliveResponse {},
    Error {
        message: String,
        #[serde(default)]
        rid: Option<String>,
    },
    /// Forward-compatible fallback for new Highrise event types.
    #[serde(other)]
    Unknown,
}

impl WsEvent {
    /// Full set of WebSocket event names subscribed on connection.
    pub const ALL_SUBSCRIPTIONS: &'static [&'static str] = &[
        "SessionMetadata",
        "ChatEvent",
        "UserMovedEvent",
        "UserJoinedEvent",
        "UserLeftEvent",
        "MessageEvent",
        "TipReactionEvent",
        "RoomModeratedEvent",
        "ChannelEvent",
        "EmoteEvent",
        "VoiceEvent",
        "ReactionEvent",
    ];
}
