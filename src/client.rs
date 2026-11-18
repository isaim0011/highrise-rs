use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Notify;
use tokio::time;
use tracing::{error, info, warn};

use crate::api::Highrise;
use crate::error::{Error, Result};
use crate::protocol::WsEvent;
use crate::transport::Transport;
use crate::types::{CurrencyItem, EntityPosition, Reaction, SessionMetadata, User};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Configuration for connecting a Highrise bot to a room.
#[derive(Debug, Clone)]
pub struct Config {
    pub api_token: String,
    pub room_id: String,
    pub reconnect_min: Duration,
    pub reconnect_max: Duration,
}

impl Config {
    pub fn new(api_token: impl Into<String>, room_id: impl Into<String>) -> Self {
        Self {
            api_token: api_token.into(),
            room_id: room_id.into(),
            reconnect_min: Duration::from_millis(500),
            reconnect_max: Duration::from_secs(3),
        }
    }
}

/// Event callbacks implemented by bot applications (all default to no-op).
#[allow(unused_variables)]
pub trait BotHandler: Send + Sync + 'static {
    fn on_ready<'a>(&'a self, api: &'a Highrise, meta: SessionMetadata) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_chat<'a>(&'a self, api: &'a Highrise, user: User, message: String) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_whisper<'a>(&'a self, api: &'a Highrise, user: User, message: String) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_dm<'a>(
        &'a self,
        api: &'a Highrise,
        user_id: String,
        conversation_id: String,
        is_new: bool,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_join<'a>(&'a self, api: &'a Highrise, user: User, pos: EntityPosition) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_leave<'a>(&'a self, api: &'a Highrise, user: User) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_move<'a>(&'a self, api: &'a Highrise, user: User, pos: EntityPosition) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_emote<'a>(
        &'a self,
        api: &'a Highrise,
        user: User,
        emote_id: String,
        receiver: User,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_reaction<'a>(
        &'a self,
        api: &'a Highrise,
        user: User,
        reaction: Reaction,
        receiver: User,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_tip<'a>(
        &'a self,
        api: &'a Highrise,
        sender: User,
        receiver: User,
        item: CurrencyItem,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_voice<'a>(
        &'a self,
        api: &'a Highrise,
        users: Vec<(User, String)>,
        seconds_left: u64,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_moderate<'a>(
        &'a self,
        api: &'a Highrise,
        moderator_id: String,
        target_user_id: String,
        moderation_type: String,
        duration: Option<u64>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
    fn on_channel<'a>(
        &'a self,
        api: &'a Highrise,
        sender_id: String,
        message: String,
        tags: Vec<String>,
    ) -> BoxFuture<'a, ()> {
        Box::pin(async {})
    }
}

/// Persistent bot runner with automatic exponential-backoff reconnects.
pub struct Client<H> {
    config: Config,
    handler: H,
}

impl<H: BotHandler> Client<H> {
    pub fn new(config: Config, handler: H) -> Self {
        Self { config, handler }
    }

    /// Runs the bot until a fatal auth/permission error occurs.
    pub async fn run(&self) -> Result<()> {
        let mut backoff = self.config.reconnect_min;

        loop {
            let started = std::time::Instant::now();
            match self.run_once().await {
                Ok(()) => {
                    // Clean server close (e.g. empty room hibernation); reset backoff.
                    backoff = self.config.reconnect_min;
                    warn!("disconnected from room; reconnecting in {:?}", backoff);
                }
                Err(Error::Fatal(msg)) => {
                    error!("fatal error, stopping reconnect loop: {msg}");
                    return Err(Error::Fatal(msg));
                }
                Err(e) => {
                    if started.elapsed() >= Duration::from_secs(30) {
                        backoff = self.config.reconnect_min;
                    }
                    warn!("connection error ({e}); reconnecting in {:?}", backoff);
                }
            }

            time::sleep(backoff).await;
            backoff = (backoff * 2).min(self.config.reconnect_max);
        }
    }

    async fn run_once(&self) -> Result<()> {
        let (transport, mut events) = Transport::connect(
            &self.config.api_token,
            &self.config.room_id,
            WsEvent::ALL_SUBSCRIPTIONS,
        )
        .await?;

        let mut api = Highrise::new(transport.clone(), String::new());
        let notify_reconnect = Arc::new(Notify::new());
        let mut watchdog: Option<tokio::task::JoinHandle<()>> = None;

        let res: Result<()> = loop {
            let item = tokio::select! {
                maybe = events.recv() => match maybe {
                    Some(it) => it,
                    None => break Ok(()),
                },
                _ = notify_reconnect.notified() => {
                    break Ok(());
                }
            };
            let evt = match item {
                Ok(e) => e,
                Err(e) => break Err(e),
            };
            match evt {
                WsEvent::SessionMetadata {
                    user_id,
                    room_info,
                    rate_limits,
                    connection_id,
                    sdk_version,
                } => {
                    info!(bot_id = %user_id, room = %room_info.room_name, "connected to highrise");
                    api = Highrise::new(transport.clone(), user_id.clone());
                    if let Some(h) = watchdog.take() {
                        h.abort();
                    }
                    let api_w = api.clone();
                    let bid = user_id.clone();
                    let nr = Arc::clone(&notify_reconnect);
                    // Ghost-session watchdog, diagnostics-only: confirmed in production
                    // (2026-11-18) that self-absence from room_users() does NOT correlate
                    // with real "connected but not rendered in-room" invisibility -- a
                    // forced reconnect on this signal fired in lockstep across all bots on
                    // a fixed ~5min cadence regardless of whether any bot was actually
                    // visibly broken, and did not fix a real invisibility incident that
                    // happened alongside it (only a manual process restart did). So this no
                    // longer forces a reconnect at all; it only logs API-call health. If a
                    // reliable ghost-session signal is found later, reintroduce the trigger
                    // here rather than reusing room_users() self-presence.
                    let _ = &nr;
                    watchdog = Some(tokio::spawn(async move {
                        time::sleep(Duration::from_secs(30)).await;
                        let mut err_streak: u32 = 0;
                        loop {
                            match api_w.room_users().await {
                                Ok(_users) => {
                                    err_streak = 0;
                                }
                                Err(e) => {
                                    err_streak = err_streak.saturating_add(1);
                                    warn!(bot_id = %bid, err_streak, "room_users presence check failed: {e}");
                                }
                            }
                            time::sleep(Duration::from_secs(20)).await;
                        }
                    }));

                    let meta = SessionMetadata {
                        user_id,
                        room_info,
                        rate_limits,
                        connection_id,
                        sdk_version,
                    };
                    self.handler.on_ready(&api, meta).await;
                }
                WsEvent::ChatEvent { user, message, whisper } => {
                    if whisper {
                        self.handler.on_whisper(&api, user, message).await;
                    } else {
                        self.handler.on_chat(&api, user, message).await;
                    }
                }
                WsEvent::MessageEvent {
                    user_id,
                    conversation_id,
                    is_new_conversation,
                } => {
                    self.handler
                        .on_dm(&api, user_id, conversation_id, is_new_conversation)
                        .await;
                }
                WsEvent::UserJoinedEvent { user, position } => {
                    self.handler.on_join(&api, user, position).await;
                }
                WsEvent::UserLeftEvent { user } => {
                    self.handler.on_leave(&api, user).await;
                }
                WsEvent::UserMovedEvent { user, position } => {
                    self.handler.on_move(&api, user, position).await;
                }
                WsEvent::EmoteEvent { user, emote_id, receiver } => {
                    self.handler.on_emote(&api, user, emote_id, receiver).await;
                }
                WsEvent::ReactionEvent { user, reaction, receiver } => {
                    self.handler.on_reaction(&api, user, reaction, receiver).await;
                }
                WsEvent::TipReactionEvent { sender, receiver, item } => {
                    self.handler.on_tip(&api, sender, receiver, item).await;
                }
                WsEvent::VoiceEvent { users, seconds_left } => {
                    self.handler.on_voice(&api, users, seconds_left).await;
                }
                WsEvent::RoomModeratedEvent {
                    moderator_id,
                    target_user_id,
                    moderation_type,
                    duration,
                } => {
                    self.handler
                        .on_moderate(&api, moderator_id, target_user_id, moderation_type, duration)
                        .await;
                }
                WsEvent::ChannelEvent {
                    sender_id,
                    msg,
                    tags,
                } => {
                    self.handler.on_channel(&api, sender_id, msg, tags).await;
                }
                WsEvent::KeepaliveResponse {} | WsEvent::Error { .. } | WsEvent::Unknown => {}
            }
        };

        if let Some(h) = watchdog.take() {
            h.abort();
        }
        res
    }
}
