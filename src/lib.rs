//! Fast, lightweight async Rust SDK core for the Highrise Bot WebSocket API.

pub mod api;
pub mod client;
pub mod error;
pub mod mcp;
pub mod protocol;
pub mod transport;
pub mod types;
pub mod webapi;
pub mod webapi_models;

pub use api::Highrise;
pub use client::{BotHandler, BoxFuture, Client, Config};
pub use error::{Error, Result};
pub use protocol::{WsEvent, WsRequest};
pub use types::{
    AnchorPosition, BuyResult, Conversation, CurrencyItem, DirectMessage, EntityPosition, Facing,
    GoldBar, Item, Message, MessageMedia, OutfitItem, Position, Reaction, RoomInfo,
    RoomPermissions, RoomUser, SessionMetadata, User, VoiceChatStatus,
};
pub use webapi::{PageQuery, SortOrder, WebApi};
