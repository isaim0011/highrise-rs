use serde::{Deserialize, Serialize};

/// Avatar facing direction in a Highrise room.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Facing {
    #[default]
    FrontRight,
    FrontLeft,
    BackRight,
    BackLeft,
}

/// 3D room coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    #[serde(default)]
    pub facing: Facing,
}

/// Seat/furniture anchor coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnchorPosition {
    pub entity_id: String,
    pub anchor_ix: u32,
}

/// User position variants reported by `UserJoined`, `Movement`, and `GetRoomUsers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum EntityPosition {
    Floor(Position),
    Anchor(AnchorPosition),
}

/// Basic Highrise user identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    #[serde(default)]
    pub username: String,
}

/// User + position tuple returned by `GetRoomUsers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RoomUser {
    pub user: User,
    pub position: EntityPosition,
}

/// Room metadata delivered in `SessionMetadata`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoomInfo {
    #[serde(default)]
    pub room_name: String,
    #[serde(default)]
    pub owner_id: String,
}

/// Session details emitted on successful room login (`Ready`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub user_id: String,
    #[serde(default)]
    pub room_info: RoomInfo,
    #[serde(default)]
    pub rate_limits: serde_json::Value,
    #[serde(default)]
    pub connection_id: String,
    #[serde(default)]
    pub sdk_version: String,
}

/// Built-in room reactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Reaction {
    Clap,
    Heart,
    Thumbs,
    Wave,
    Wink,
}

/// Tip currency item received in `TipReactionEvent`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrencyItem {
    #[serde(rename = "type", default)]
    pub kind: String,
    #[serde(default)]
    pub amount: u64,
}

/// Valid gold bar amounts for `TipUserRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoldBar {
    G1,
    G5,
    G10,
    G50,
    G100,
    G500,
    G1000,
    G5000,
    G10000,
}

impl GoldBar {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::G1 => "gold_bar_1",
            Self::G5 => "gold_bar_5",
            Self::G10 => "gold_bar_10",
            Self::G50 => "gold_bar_50",
            Self::G100 => "gold_bar_100",
            Self::G500 => "gold_bar_500",
            Self::G1000 => "gold_bar_1k",
            Self::G5000 => "gold_bar_5000",
            Self::G10000 => "gold_bar_10k",
        }
    }
}

/// Room privilege flags for moderation/designer management.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RoomPermissions {
    #[serde(default)]
    pub moderator: bool,
    #[serde(default)]
    pub designer: bool,
}

/// Equipped outfit item (matches Highrise Bot API `Item` schema).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutfitItem {
    #[serde(rename = "type")]
    pub item_type: String,
    pub id: String,
    #[serde(default = "default_one")]
    pub amount: u32,
    #[serde(default)]
    pub account_bound: bool,
    #[serde(default)]
    pub active_palette: i32,
}

/// Alias matching the official Highrise Bot API `Item` schema name.
pub type Item = OutfitItem;
/// Alias matching the official Highrise Bot API `Message` schema name.
pub type Message = DirectMessage;

/// Result returned by `BuyItemRequest`, `BuyRoomBoostRequest`, and `BuyVoiceTimeRequest`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BuyResult {
    Success,
    InsufficientFunds,
    OnlyTokenBought,
}

fn default_one() -> u32 {
    1
}

/// Direct-message conversation message schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DirectMessage {
    pub message_id: String,
    pub conversation_id: String,
    #[serde(rename = "createdAt", default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub sender_id: String,
}

/// Direct-message conversation thread schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    #[serde(default)]
    pub did_join: bool,
    #[serde(default)]
    pub unread_count: u32,
    #[serde(default)]
    pub last_message: Option<DirectMessage>,
    #[serde(default)]
    pub muted: bool,
}

/// Pre-signed media upload descriptor for `MessageMediaRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageMedia {
    #[serde(default)]
    pub media_id: Option<String>,
    #[serde(default)]
    pub upload_url: Option<String>,
    #[serde(default)]
    pub mime_type: String,
    #[serde(default)]
    pub width: Option<u32>,
    #[serde(default)]
    pub height: Option<u32>,
}

/// Room voice chat status returned by `CheckVoiceChatRequest`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct VoiceChatStatus {
    #[serde(default)]
    pub seconds_left: u64,
    #[serde(default)]
    pub auto_speakers: Vec<String>,
    #[serde(default)]
    pub users: Vec<(String, String)>,
}
