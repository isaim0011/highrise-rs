use std::collections::HashMap;

use serde::Serialize;

use crate::types::{Facing, MessageMedia, OutfitItem, Reaction, RoomPermissions};

/// Outgoing WebSocket request payloads serialized with Highrise's `_type` tag.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "_type")]
pub enum WsRequest {
    KeepaliveRequest {},
    ChatRequest {
        message: String,
        whisper_target_id: Option<String>,
    },
    SendMessageRequest {
        conversation_id: String,
        content: String,
        #[serde(rename = "type")]
        msg_type: &'static str,
    },
    EmoteRequest {
        emote_id: String,
        target_user_id: String,
    },
    ReactionRequest {
        target_user_id: String,
        reaction: Reaction,
    },
    TeleportRequest {
        user_id: String,
        destination: Destination,
    },
    FloorHitRequest {
        destination: Destination,
    },
    AnchorHitRequest {
        anchor: AnchorTarget,
    },
    ModerateRoomRequest {
        user_id: String,
        moderation_action: &'static str,
        #[serde(skip_serializing_if = "Option::is_none")]
        action_length: Option<u64>,
    },
    GetRoomUsersRequest {},
    GetUserOutfitRequest {
        user_id: String,
    },
    GetRoomPrivilegeRequest {
        user_id: String,
    },
    ChangeRoomPrivilegeRequest {
        user_id: String,
        permissions: RoomPermissions,
    },
    TipUserRequest {
        user_id: String,
        gold_bar: &'static str,
    },
    MoveUserToRoomRequest {
        user_id: String,
        room_id: String,
    },
    CheckVoiceChatRequest {},
    InviteSpeakerRequest {
        user_id: String,
    },
    RemoveSpeakerRequest {
        user_id: String,
    },
    SetOutfitRequest {
        outfit: Vec<OutfitItem>,
    },
    GetWalletRequest {},
    GetInventoryRequest {},
    BuyItemRequest {
        item_id: String,
    },
    BuyRoomBoostRequest {
        payment_method: &'static str,
        amount: u32,
    },
    SendBulkMessageRequest {
        user_ids: Vec<String>,
        content: String,
        #[serde(rename = "type")]
        msg_type: &'static str,
    },
    GetConversationsRequest {
        not_joined: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_id: Option<String>,
    },
    GetMessagesRequest {
        conversation_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        last_message_id: Option<String>,
    },
    LeaveConversationRequest {
        conversation_id: String,
    },
    ChannelRequest {
        message: String,
        tags: Vec<String>,
    },
    IndicatorRequest {
        icon: Option<String>,
    },
    GetBackpackRequest {
        user_id: String,
    },
    ChangeBackpackRequest {
        user_id: String,
        changes: HashMap<String, i32>,
    },
    BuyVoiceTimeRequest {
        payment_method: &'static str,
    },
    MessageMediaRequest {
        media: MessageMedia,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct Destination {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub facing: Facing,
}

#[derive(Debug, Clone, Serialize)]
pub struct AnchorTarget {
    pub entity_id: String,
    pub anchor_ix: u32,
}
