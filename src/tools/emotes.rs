use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::Result;

const FREE_EMOTES_JSON: &str = include_str!("data/emotes_free.json");
const ALL_EMOTES_JSON: &str = include_str!("data/emotes_all.json");

/// Live upstream free-emotes JSON maintained at `Djxhknz/Ids-highrise`.
pub const DEFAULT_EMOTES_URL: &str =
    "https://raw.githubusercontent.com/Djxhknz/Ids-highrise/main/emotes.json";

/// Built-in emote pack selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EmotePack {
    /// 280 free Highrise emotes (usable by any bot/player without owning paid emotes).
    #[default]
    Free,
    /// Full 371+ catalog (free + paid emotes).
    All,
}

/// Highrise emote metadata (owned strings so catalogs can be imported dynamically).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Emote {
    pub id: String,
    pub name: String,
    #[serde(default = "default_duration")]
    pub duration: f32,
    #[serde(default)]
    pub is_free: bool,
}

fn default_duration() -> f32 {
    3.0
}

impl Emote {
    pub fn is_dance(&self) -> bool {
        self.id.starts_with("dance-") || self.id.contains("dance")
    }
}

/// Dynamic emote catalog with Free/All packs, JSON/URL import, blocklist, and lookup.
#[derive(Debug, Clone)]
pub struct Emotes {
    items: Vec<Emote>,
    blocked: HashSet<String>,
}

impl Default for Emotes {
    fn default() -> Self {
        Self::new()
    }
}

impl Emotes {
    /// Loads the default 280-emote `Free` pack.
    pub fn new() -> Self {
        Self::from_pack(EmotePack::Free)
    }

    /// Loads either `EmotePack::Free` (280 free emotes) or `EmotePack::All` (371 free+paid).
    pub fn from_pack(pack: EmotePack) -> Self {
        let free_ids: HashSet<String> = Self::parse_json(FREE_EMOTES_JSON)
            .unwrap_or_default()
            .into_iter()
            .map(|e| e.id.to_ascii_lowercase())
            .collect();

        let mut items = match pack {
            EmotePack::Free => Self::parse_json(FREE_EMOTES_JSON).unwrap_or_default(),
            EmotePack::All => Self::parse_json(ALL_EMOTES_JSON).unwrap_or_default(),
        };
        for item in &mut items {
            item.is_free = pack == EmotePack::Free || free_ids.contains(&item.id.to_ascii_lowercase());
        }

        Self {
            items,
            blocked: HashSet::new(),
        }
    }

    /// Builds a catalog from any JSON array of `{ id, name, duration }`.
    pub fn from_json(json_str: &str) -> Result<Self> {
        Ok(Self {
            items: Self::parse_json(json_str)?,
            blocked: HashSet::new(),
        })
    }

    /// Imports/merges emotes from a JSON string (deduplicating by `id`).
    pub fn import_json(&mut self, json_str: &str) -> Result<usize> {
        let incoming = Self::parse_json(json_str)?;
        let mut seen: HashSet<String> = self
            .items
            .iter()
            .map(|e| e.id.to_ascii_lowercase())
            .collect();
        let mut added = 0;
        for item in incoming {
            if seen.insert(item.id.to_ascii_lowercase()) {
                self.items.push(item);
                added += 1;
            }
        }
        Ok(added)
    }

    /// Fetches and merges an emote JSON array from a remote URL (e.g. a GitHub raw URL).
    pub async fn import_from_url(&mut self, url: &str) -> Result<usize> {
        let text = reqwest::get(url).await?.error_for_status()?.text().await?;
        self.import_json(&text)
    }

    /// Fetches the latest live emote list from `Djxhknz/Ids-highrise` (`DEFAULT_EMOTES_URL`),
    /// falling back to the bundled `pack` if the network is unavailable.
    pub async fn sync_pack(pack: EmotePack) -> Self {
        let mut catalog = match pack {
            EmotePack::Free => Self {
                items: Vec::new(),
                blocked: HashSet::new(),
            },
            EmotePack::All => Self::from_pack(EmotePack::All),
        };
        if catalog.import_from_url(DEFAULT_EMOTES_URL).await.is_err() && catalog.items.is_empty() {
            return Self::from_pack(pack);
        }
        if pack == EmotePack::Free {
            for item in &mut catalog.items {
                item.is_free = true;
            }
        }
        catalog
    }

    fn parse_json(json_str: &str) -> Result<Vec<Emote>> {
        Ok(serde_json::from_str::<Vec<Emote>>(json_str)?)
    }

    /// Blocks an emote by ID or name (e.g. `yes`, `no`, `hello`, `hi`).
    pub fn block(&mut self, id_or_name: impl Into<String>) {
        self.blocked.insert(id_or_name.into().to_ascii_lowercase());
    }

    /// Returns all non-blocked emotes.
    pub fn allowed(&self) -> Vec<&Emote> {
        self.items
            .iter()
            .filter(|e| {
                !self.blocked.contains(&e.id.to_ascii_lowercase())
                    && !self.blocked.contains(&e.name.to_ascii_lowercase())
            })
            .collect()
    }

    /// Returns only free non-blocked emotes.
    pub fn free_only(&self) -> Vec<&Emote> {
        self.allowed().into_iter().filter(|e| e.is_free).collect()
    }

    /// Returns only dance emotes that are not blocked.
    pub fn dances(&self) -> Vec<&Emote> {
        self.allowed().into_iter().filter(|e| e.is_dance()).collect()
    }

    /// Resolves an emote by 1-based number (`"1"`), exact ID, or case-insensitive name.
    pub fn resolve(&self, query: &str) -> Option<&Emote> {
        let q = query.trim();
        let list = self.allowed();
        if let Ok(idx) = q.parse::<usize>() {
            if idx >= 1 && idx <= list.len() {
                return Some(list[idx - 1]);
            }
        }
        let lower = q.to_ascii_lowercase();
        list.into_iter()
            .find(|e| e.id.eq_ignore_ascii_case(&lower) || e.name.eq_ignore_ascii_case(&lower))
    }

    /// Picks a pseudo-random emote from the allowed pool using a monotonic seed.
    pub fn pick_random(&self, dances_only: bool, seed: u64) -> Option<&Emote> {
        let pool = if dances_only { self.dances() } else { self.allowed() };
        if pool.is_empty() {
            return None;
        }
        let idx = (seed as usize) % pool.len();
        Some(pool[idx])
    }
}
