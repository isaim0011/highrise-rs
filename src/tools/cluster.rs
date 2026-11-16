use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{watch, RwLock};
use tokio::task::JoinSet;
use tracing::{error, info};

use crate::client::{BotHandler, Client, Config};

/// Configuration for one bot managed by `HighriseCluster`.
#[derive(Debug, Clone)]
pub struct BotSpec {
    pub name: String,
    pub config: Config,
}

impl BotSpec {
    pub fn new(name: impl Into<String>, config: Config) -> Self {
        Self {
            name: name.into(),
            config,
        }
    }
}

/// Live lifecycle state for a bot inside `HighriseCluster`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BotStatus {
    Starting,
    Running,
    Stopped,
    Failed(String),
}

/// Shared handle for inspecting and gracefully stopping a running `HighriseCluster`.
#[derive(Clone)]
pub struct ClusterHandle {
    shutdown_tx: watch::Sender<bool>,
    statuses: Arc<RwLock<HashMap<String, BotStatus>>>,
}

impl ClusterHandle {
    /// Signals every managed bot in the cluster to shut down gracefully.
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
    }

    /// Returns a snapshot of every bot's current status.
    pub async fn statuses(&self) -> HashMap<String, BotStatus> {
        self.statuses.read().await.clone()
    }
}

/// Multi-bot supervisor running N bots concurrently with unified shutdown & status tracking.
#[derive(Default)]
pub struct HighriseCluster {
    entries: Vec<(BotSpec, Arc<dyn BotHandler>)>,
}

impl HighriseCluster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a bot instance to run in the cluster.
    pub fn add_bot<H: BotHandler>(&mut self, spec: BotSpec, handler: H) -> &mut Self {
        self.entries.push((spec, Arc::new(handler)));
        self
    }

    /// Starts all registered bots concurrently and waits until all exit or `shutdown()` is called.
    pub async fn run(self) -> ClusterHandle {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);
        let statuses = Arc::new(RwLock::new(HashMap::new()));
        let handle = ClusterHandle {
            shutdown_tx,
            statuses: Arc::clone(&statuses),
        };

        let mut set = JoinSet::new();
        for (spec, handler) in self.entries {
            let name = spec.name.clone();
            let mut rx = shutdown_rx.clone();
            let status_map = Arc::clone(&statuses);
            status_map.write().await.insert(name.clone(), BotStatus::Starting);

            set.spawn(async move {
                info!(bot = %name, room = %spec.config.room_id, "cluster starting bot");
                status_map.write().await.insert(name.clone(), BotStatus::Running);
                let client = Client::new(spec.config, SharedHandler(handler));

                let final_status = tokio::select! {
                    res = client.run() => match res {
                        Ok(()) => BotStatus::Stopped,
                        Err(e) => {
                            error!(bot = %name, "bot exited with fatal error: {e}");
                            BotStatus::Failed(e.to_string())
                        }
                    },
                    _ = rx.changed() => {
                        info!(bot = %name, "cluster shutdown signal received");
                        BotStatus::Stopped
                    }
                };
                status_map.write().await.insert(name, final_status);
            });
        }

        while set.join_next().await.is_some() {}
        handle
    }
}

struct SharedHandler(Arc<dyn BotHandler>);

impl BotHandler for SharedHandler {
    fn on_ready<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        meta: crate::types::SessionMetadata,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_ready(api, meta)
    }
    fn on_chat<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        message: String,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_chat(api, user, message)
    }
    fn on_whisper<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        message: String,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_whisper(api, user, message)
    }
    fn on_dm<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user_id: String,
        conversation_id: String,
        is_new: bool,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_dm(api, user_id, conversation_id, is_new)
    }
    fn on_join<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        pos: crate::types::EntityPosition,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_join(api, user, pos)
    }
    fn on_leave<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_leave(api, user)
    }
    fn on_move<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        pos: crate::types::EntityPosition,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_move(api, user, pos)
    }
    fn on_emote<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        emote_id: String,
        receiver: crate::types::User,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_emote(api, user, emote_id, receiver)
    }
    fn on_reaction<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        user: crate::types::User,
        reaction: crate::types::Reaction,
        receiver: crate::types::User,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_reaction(api, user, reaction, receiver)
    }
    fn on_tip<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        sender: crate::types::User,
        receiver: crate::types::User,
        item: crate::types::CurrencyItem,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_tip(api, sender, receiver, item)
    }
    fn on_voice<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        users: Vec<(crate::types::User, String)>,
        seconds_left: u64,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_voice(api, users, seconds_left)
    }
    fn on_moderate<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        moderator_id: String,
        target_user_id: String,
        moderation_type: String,
        duration: Option<u64>,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0
            .on_moderate(api, moderator_id, target_user_id, moderation_type, duration)
    }
    fn on_channel<'a>(
        &'a self,
        api: &'a crate::api::Highrise,
        sender_id: String,
        message: String,
        tags: Vec<String>,
    ) -> crate::client::BoxFuture<'a, ()> {
        self.0.on_channel(api, sender_id, message, tags)
    }
}
