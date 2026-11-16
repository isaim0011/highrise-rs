use std::collections::HashMap;

use serde_json::Value;

use crate::error::Result;
use crate::protocol::requests::{AnchorTarget, Destination, WsRequest};
use crate::transport::Transport;
use crate::types::{
    BuyResult, Conversation, CurrencyItem, DirectMessage, Facing, GoldBar, MessageMedia,
    OutfitItem, Reaction, RoomPermissions, RoomUser, VoiceChatStatus,
};

/// Ergonomic async handle for Highrise room actions (passed to `BotHandler` callbacks).
#[derive(Clone)]
pub struct Highrise {
    transport: Transport,
    pub bot_id: String,
}

impl Highrise {
    pub(crate) fn new(transport: Transport, bot_id: String) -> Self {
        Self { transport, bot_id }
    }

    /// Sends a public message in room chat.
    pub async fn chat(&self, message: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::ChatRequest {
                message: message.into(),
                whisper_target_id: None,
            })
            .await?;
        Ok(())
    }

    /// Sends a private whisper to a user in the room.
    pub async fn whisper(&self, target_user_id: impl Into<String>, message: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::ChatRequest {
                message: message.into(),
                whisper_target_id: Some(target_user_id.into()),
            })
            .await?;
        Ok(())
    }

    /// Replies to an in-game direct message conversation.
    pub async fn send_dm(&self, conversation_id: impl Into<String>, content: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::SendMessageRequest {
                conversation_id: conversation_id.into(),
                content: content.into(),
                msg_type: "text",
            })
            .await?;
        Ok(())
    }

    /// Plays an emote on a target user (or on the bot itself using `self.bot_id`).
    pub async fn emote(&self, emote_id: impl Into<String>, target_user_id: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::EmoteRequest {
                emote_id: emote_id.into(),
                target_user_id: target_user_id.into(),
            })
            .await?;
        Ok(())
    }

    /// Sends a reaction (`clap`, `heart`, `thumbs`, `wave`, `wink`) to a user.
    pub async fn react(&self, target_user_id: impl Into<String>, reaction: Reaction) -> Result<()> {
        self.transport
            .request(&WsRequest::ReactionRequest {
                target_user_id: target_user_id.into(),
                reaction,
            })
            .await?;
        Ok(())
    }

    /// Walks the bot to `(x, y, z)`.
    pub async fn walk(&self, x: f64, y: f64, z: f64, facing: Facing) -> Result<()> {
        self.transport
            .request(&WsRequest::FloorHitRequest {
                destination: Destination { x, y, z, facing },
            })
            .await?;
        Ok(())
    }

    /// Seats the bot on a furniture anchor.
    pub async fn sit(&self, entity_id: impl Into<String>, anchor_ix: u32) -> Result<()> {
        self.transport
            .request(&WsRequest::AnchorHitRequest {
                anchor: AnchorTarget {
                    entity_id: entity_id.into(),
                    anchor_ix,
                },
            })
            .await?;
        Ok(())
    }

    /// Teleports a user (or the bot) to `(x, y, z)`.
    pub async fn teleport(
        &self,
        user_id: impl Into<String>,
        x: f64,
        y: f64,
        z: f64,
        facing: Facing,
    ) -> Result<()> {
        self.transport
            .request(&WsRequest::TeleportRequest {
                user_id: user_id.into(),
                destination: Destination { x, y, z, facing },
            })
            .await?;
        Ok(())
    }

    /// Fetches all users currently in the room along with their positions.
    pub async fn room_users(&self) -> Result<Vec<RoomUser>> {
        let val = self.transport.request(&WsRequest::GetRoomUsersRequest {}).await?;
        let raw = val.get("content").cloned().unwrap_or(Value::Array(vec![]));
        let pairs: Vec<(crate::types::User, crate::types::EntityPosition)> =
            serde_json::from_value(raw)?;
        Ok(pairs
            .into_iter()
            .map(|(user, position)| RoomUser { user, position })
            .collect())
    }

    /// Fetches a user's currently equipped outfit items.
    pub async fn user_outfit(&self, user_id: impl Into<String>) -> Result<Vec<OutfitItem>> {
        let val = self
            .transport
            .request(&WsRequest::GetUserOutfitRequest {
                user_id: user_id.into(),
            })
            .await?;
        let raw = val.get("outfit").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(raw)?)
    }

    /// Reads a user's moderator/designer privileges in this room.
    pub async fn room_privilege(&self, user_id: impl Into<String>) -> Result<RoomPermissions> {
        let val = self
            .transport
            .request(&WsRequest::GetRoomPrivilegeRequest {
                user_id: user_id.into(),
            })
            .await?;
        let raw = val.get("content").cloned().unwrap_or(val);
        Ok(serde_json::from_value(raw)?)
    }

    /// Updates a user's moderator/designer privileges in this room.
    pub async fn set_privilege(
        &self,
        user_id: impl Into<String>,
        permissions: RoomPermissions,
    ) -> Result<()> {
        self.transport
            .request(&WsRequest::ChangeRoomPrivilegeRequest {
                user_id: user_id.into(),
                permissions,
            })
            .await?;
        Ok(())
    }

    pub async fn kick(&self, user_id: impl Into<String>) -> Result<()> {
        self.moderate(user_id, "kick", None).await
    }

    pub async fn mute(&self, user_id: impl Into<String>, seconds: u64) -> Result<()> {
        self.moderate(user_id, "mute", Some(seconds)).await
    }

    pub async fn ban(&self, user_id: impl Into<String>, seconds: u64) -> Result<()> {
        self.moderate(user_id, "ban", Some(seconds)).await
    }

    pub async fn unban(&self, user_id: impl Into<String>) -> Result<()> {
        self.moderate(user_id, "unban", None).await
    }

    async fn moderate(
        &self,
        user_id: impl Into<String>,
        moderation_action: &'static str,
        action_length: Option<u64>,
    ) -> Result<()> {
        self.transport
            .request(&WsRequest::ModerateRoomRequest {
                user_id: user_id.into(),
                moderation_action,
                action_length,
            })
            .await?;
        Ok(())
    }

    /// Tips a gold bar from the bot's wallet to a user.
    pub async fn tip(&self, user_id: impl Into<String>, gold_bar: GoldBar) -> Result<()> {
        self.transport
            .request(&WsRequest::TipUserRequest {
                user_id: user_id.into(),
                gold_bar: gold_bar.as_str(),
            })
            .await?;
        Ok(())
    }

    /// Moves a user to another room.
    pub async fn move_user_to_room(
        &self,
        user_id: impl Into<String>,
        room_id: impl Into<String>,
    ) -> Result<()> {
        self.transport
            .request(&WsRequest::MoveUserToRoomRequest {
                user_id: user_id.into(),
                room_id: room_id.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn voice_status(&self) -> Result<VoiceChatStatus> {
        let val = self.transport.request(&WsRequest::CheckVoiceChatRequest {}).await?;
        Ok(serde_json::from_value(val)?)
    }

    pub async fn invite_speaker(&self, user_id: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::InviteSpeakerRequest {
                user_id: user_id.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn remove_speaker(&self, user_id: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::RemoveSpeakerRequest {
                user_id: user_id.into(),
            })
            .await?;
        Ok(())
    }

    pub async fn set_outfit(&self, outfit: Vec<OutfitItem>) -> Result<()> {
        self.transport
            .request(&WsRequest::SetOutfitRequest { outfit })
            .await?;
        Ok(())
    }

    pub async fn wallet(&self) -> Result<Vec<CurrencyItem>> {
        let val = self.transport.request(&WsRequest::GetWalletRequest {}).await?;
        let raw = val.get("content").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn inventory(&self) -> Result<Vec<OutfitItem>> {
        let val = self.transport.request(&WsRequest::GetInventoryRequest {}).await?;
        let raw = val.get("items").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn buy_item(&self, item_id: impl Into<String>) -> Result<BuyResult> {
        let val = self
            .transport
            .request(&WsRequest::BuyItemRequest {
                item_id: item_id.into(),
            })
            .await?;
        let raw = val.get("result").cloned().unwrap_or(Value::String("success".into()));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn buy_room_boost(&self, amount: u32) -> Result<BuyResult> {
        let val = self
            .transport
            .request(&WsRequest::BuyRoomBoostRequest {
                payment_method: "bot_wallet_only",
                amount,
            })
            .await?;
        let raw = val.get("result").cloned().unwrap_or(Value::String("success".into()));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn send_bulk_dm(&self, user_ids: Vec<String>, content: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::SendBulkMessageRequest {
                user_ids,
                content: content.into(),
                msg_type: "text",
            })
            .await?;
        Ok(())
    }

    pub async fn conversations(&self, not_joined: bool, last_id: Option<String>) -> Result<Vec<Conversation>> {
        let val = self
            .transport
            .request(&WsRequest::GetConversationsRequest { not_joined, last_id })
            .await?;
        let raw = val.get("conversations").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn dm_messages(
        &self,
        conversation_id: impl Into<String>,
        last_message_id: Option<String>,
    ) -> Result<Vec<DirectMessage>> {
        let val = self
            .transport
            .request(&WsRequest::GetMessagesRequest {
                conversation_id: conversation_id.into(),
                last_message_id,
            })
            .await?;
        let raw = val.get("messages").cloned().unwrap_or(Value::Array(vec![]));
        Ok(serde_json::from_value(raw)?)
    }

    pub async fn leave_conversation(&self, conversation_id: impl Into<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::LeaveConversationRequest {
                conversation_id: conversation_id.into(),
            })
            .await?;
        Ok(())
    }

    /// Sends a hidden-channel IPC payload (e.g. bot-to-bot coordination).
    pub async fn send_channel(&self, message: impl Into<String>, tags: Vec<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::ChannelRequest {
                message: message.into(),
                tags,
            })
            .await?;
        Ok(())
    }

    /// Sets or clears the indicator icon above the bot's head.
    pub async fn set_indicator(&self, icon: Option<String>) -> Result<()> {
        self.transport
            .request(&WsRequest::IndicatorRequest { icon })
            .await?;
        Ok(())
    }

    /// Fetches the bot's own equipped outfit.
    pub async fn my_outfit(&self) -> Result<Vec<OutfitItem>> {
        self.user_outfit(self.bot_id.clone()).await
    }

    /// Fetches a user's world backpack counts.
    pub async fn backpack(&self, user_id: impl Into<String>) -> Result<HashMap<String, i32>> {
        let val = self
            .transport
            .request(&WsRequest::GetBackpackRequest {
                user_id: user_id.into(),
            })
            .await?;
        let raw = val.get("backpack").cloned().unwrap_or(Value::Object(Default::default()));
        Ok(serde_json::from_value(raw)?)
    }

    /// Modifies a user's world backpack item deltas.
    pub async fn change_backpack(
        &self,
        user_id: impl Into<String>,
        changes: HashMap<String, i32>,
    ) -> Result<()> {
        self.transport
            .request(&WsRequest::ChangeBackpackRequest {
                user_id: user_id.into(),
                changes,
            })
            .await?;
        Ok(())
    }

    /// Purchases voice chat time using the bot's wallet.
    pub async fn buy_voice_time(&self) -> Result<BuyResult> {
        let val = self
            .transport
            .request(&WsRequest::BuyVoiceTimeRequest {
                payment_method: "bot_wallet_only",
            })
            .await?;
        let raw = val.get("result").cloned().unwrap_or(Value::String("success".into()));
        Ok(serde_json::from_value(raw)?)
    }

    /// Requests a pre-signed media upload URL (`MessageMediaRequest`).
    pub async fn request_media_upload(&self, media: MessageMedia) -> Result<MessageMedia> {
        let val = self
            .transport
            .request(&WsRequest::MessageMediaRequest { media })
            .await?;
        let raw = val.get("media").cloned().unwrap_or(val);
        Ok(serde_json::from_value(raw)?)
    }
}
