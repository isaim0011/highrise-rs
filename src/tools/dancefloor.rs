use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::RwLock;
use tokio::time;
use tracing::debug;

use crate::api::Highrise;
use crate::error::{Error, Result};
use crate::tools::emotes::Emotes;
use crate::types::{EntityPosition, Position};

/// 3D axis-aligned dancefloor bounding box.
#[derive(Debug, Clone, PartialEq)]
pub struct DanceFloorZone {
    pub name: String,
    pub min_x: f64,
    pub max_x: f64,
    pub min_y: f64,
    pub max_y: f64,
    pub min_z: f64,
    pub max_z: f64,
}

impl DanceFloorZone {
    pub fn new(name: impl Into<String>, a: Position, b: Position) -> Self {
        Self {
            name: name.into(),
            min_x: a.x.min(b.x),
            max_x: a.x.max(b.x),
            min_y: a.y.min(b.y),
            max_y: a.y.max(b.y),
            min_z: a.z.min(b.z),
            max_z: a.z.max(b.z),
        }
    }

    pub fn contains(&self, pos: &Position) -> bool {
        pos.x >= self.min_x
            && pos.x <= self.max_x
            && pos.y >= self.min_y
            && pos.y <= self.max_y
            && pos.z >= self.min_z
            && pos.z <= self.max_z
    }
}

/// How the dancefloor assigns emotes to users standing inside active zones.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum DanceMode {
    /// Every dancer performs the exact same configured emote ID.
    Fixed(String),
    /// All dancers perform the same random emote together each tick.
    #[default]
    SyncRandom,
    /// Each dancer gets an independent random emote each tick.
    PerUserRandom,
}

/// Manages dancefloor zones, live user positions, and paced emote playback.
#[derive(Clone, Default)]
pub struct DanceFloorManager {
    inner: Arc<RwLock<State>>,
}

#[derive(Default)]
struct State {
    enabled: bool,
    mode: DanceMode,
    dances_only: bool,
    zones: Vec<DanceFloorZone>,
    positions: HashMap<String, Position>,
}

impl DanceFloorManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(State {
                enabled: true,
                mode: DanceMode::SyncRandom,
                dances_only: true,
                zones: Vec::new(),
                positions: HashMap::new(),
            })),
        }
    }

    pub async fn set_enabled(&self, enabled: bool) {
        self.inner.write().await.enabled = enabled;
    }

    pub async fn set_mode(&self, mode: DanceMode) {
        self.inner.write().await.mode = mode;
    }

    pub async fn set_dances_only(&self, dances_only: bool) {
        self.inner.write().await.dances_only = dances_only;
    }

    pub async fn add_zone(&self, zone: DanceFloorZone) {
        let mut st = self.inner.write().await;
        st.zones.retain(|z| !z.name.eq_ignore_ascii_case(&zone.name));
        st.zones.push(zone);
    }

    /// Dynamically creates/updates a dancefloor zone using two users' live room positions.
    pub async fn set_zone_from_users(
        &self,
        name: impl Into<String>,
        api: &Highrise,
        user_a: &str,
        user_b: &str,
    ) -> Result<DanceFloorZone> {
        let pos_a = api
            .user_position(user_a)
            .await?
            .ok_or_else(|| Error::Server(format!("position not found for {user_a}")))?;
        let pos_b = api
            .user_position(user_b)
            .await?
            .ok_or_else(|| Error::Server(format!("position not found for {user_b}")))?;
        let zone = DanceFloorZone::new(name, pos_a, pos_b);
        self.add_zone(zone.clone()).await;
        Ok(zone)
    }

    pub async fn remove_zone(&self, name: &str) -> bool {
        let mut st = self.inner.write().await;
        let before = st.zones.len();
        st.zones.retain(|z| !z.name.eq_ignore_ascii_case(name));
        st.zones.len() != before
    }

    pub async fn update_position(&self, user_id: impl Into<String>, pos: &EntityPosition) {
        let mut st = self.inner.write().await;
        match pos {
            EntityPosition::Floor(p) => {
                st.positions.insert(user_id.into(), *p);
            }
            EntityPosition::Anchor(_) => {
                // Seated users leave floor dancezones.
                st.positions.remove(&user_id.into());
            }
        }
    }

    pub async fn remove_user(&self, user_id: &str) {
        self.inner.write().await.positions.remove(user_id);
    }

    /// Returns all user IDs currently standing inside any configured dancefloor zone.
    pub async fn active_dancers(&self, exclude_bot_id: Option<&str>) -> Vec<String> {
        let st = self.inner.read().await;
        if !st.enabled || st.zones.is_empty() {
            return Vec::new();
        }
        st.positions
            .iter()
            .filter(|(uid, pos)| {
                exclude_bot_id.is_none_or(|b| b != uid.as_str())
                    && st.zones.iter().any(|z| z.contains(pos))
            })
            .map(|(uid, _)| uid.clone())
            .collect()
    }

    /// Runs one dancefloor tick (paced at 60ms per user to respect Highrise rate limits).
    pub async fn step(&self, api: &Highrise, emotes: &Emotes) {
        let (enabled, mode, dances_only, dancers) = {
            let st = self.inner.read().await;
            if !st.enabled || st.zones.is_empty() {
                return;
            }
            let users: Vec<String> = st
                .positions
                .iter()
                .filter(|(uid, pos)| {
                    uid.as_str() != api.bot_id.as_str()
                        && st.zones.iter().any(|z| z.contains(pos))
                })
                .map(|(uid, _)| uid.clone())
                .collect();
            (st.enabled, st.mode.clone(), st.dances_only, users)
        };

        if !enabled || dancers.is_empty() {
            return;
        }

        let base_seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(1);

        let shared_emote = match &mode {
            DanceMode::Fixed(id) => Some(id.as_str()),
            DanceMode::SyncRandom => emotes.pick_random(dances_only, base_seed).map(|e| e.id.as_str()),
            DanceMode::PerUserRandom => None,
        };

        for (ix, uid) in dancers.iter().enumerate() {
            let emote_id = match shared_emote {
                Some(id) => id,
                None => match emotes.pick_random(dances_only, base_seed.wrapping_add(ix as u64 * 31)) {
                    Some(e) => e.id.as_str(),
                    None => continue,
                },
            };
            if let Err(e) = api.emote(emote_id, uid).await {
                debug!("dancefloor emote failed for {uid}: {e}");
            }
            time::sleep(Duration::from_millis(60)).await;
        }
    }

    /// Spawns a background dancefloor loop task.
    pub fn spawn_loop(
        &self,
        api: Highrise,
        emotes: Arc<Emotes>,
        interval: Duration,
    ) -> tokio::task::JoinHandle<()> {
        let mgr = self.clone();
        tokio::spawn(async move {
            let mut ticker = time::interval(interval);
            loop {
                ticker.tick().await;
                mgr.step(&api, &emotes).await;
            }
        })
    }
}
