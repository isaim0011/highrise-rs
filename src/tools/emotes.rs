use std::collections::HashSet;

/// Standard Highrise emote metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Emote {
    pub id: &'static str,
    pub name: &'static str,
    pub duration_secs: f32,
    pub is_dance: bool,
}

/// Curated standard Highrise emotes with lookup by 1-based number, ID, or name.
#[derive(Debug, Clone)]
pub struct Emotes {
    items: Vec<Emote>,
    blocked: HashSet<String>,
}

const STANDARD_EMOTES: &[Emote] = &[
    Emote { id: "dance-macarena", name: "Macarena", duration_secs: 12.2, is_dance: true },
    Emote { id: "dance-tiktok8", name: "Savage Dance", duration_secs: 10.9, is_dance: true },
    Emote { id: "dance-tiktok2", name: "Don't Start Now", duration_secs: 10.4, is_dance: true },
    Emote { id: "dance-tiktok9", name: "Viral Groove", duration_secs: 11.8, is_dance: true },
    Emote { id: "dance-tiktok10", name: "Shuffle", duration_secs: 8.2, is_dance: true },
    Emote { id: "dance-blackpink", name: "Shut Down", duration_secs: 7.2, is_dance: true },
    Emote { id: "dance-russian", name: "Russian Dance", duration_secs: 10.2, is_dance: true },
    Emote { id: "dance-breakdance", name: "Breakdance", duration_secs: 17.6, is_dance: true },
    Emote { id: "dance-floss", name: "Floss", duration_secs: 21.3, is_dance: true },
    Emote { id: "dance-pennywise", name: "Pennywise", duration_secs: 1.2, is_dance: true },
    Emote { id: "dance-weird", name: "Weird Dance", duration_secs: 21.6, is_dance: true },
    Emote { id: "dance-zombie", name: "Zombie Dance", duration_secs: 12.9, is_dance: true },
    Emote { id: "dance-shoppingcart", name: "Let's Go Shopping", duration_secs: 4.3, is_dance: true },
    Emote { id: "dance-kawai", name: "Kawaii", duration_secs: 10.3, is_dance: true },
    Emote { id: "dance-orangejustice", name: "Orange Justice", duration_secs: 6.5, is_dance: true },
    Emote { id: "emote-wave", name: "Wave", duration_secs: 2.7, is_dance: false },
    Emote { id: "emote-hello", name: "Hello", duration_secs: 2.7, is_dance: false },
    Emote { id: "emote-hug", name: "Hug", duration_secs: 4.0, is_dance: false },
    Emote { id: "emote-kiss", name: "Kiss", duration_secs: 3.9, is_dance: false },
    Emote { id: "emote-shy", name: "Shy", duration_secs: 4.5, is_dance: false },
    Emote { id: "emote-laughing", name: "Laugh", duration_secs: 2.7, is_dance: false },
    Emote { id: "emote-sad", name: "Sad", duration_secs: 5.4, is_dance: false },
    Emote { id: "emote-angry", name: "Angry", duration_secs: 5.8, is_dance: false },
    Emote { id: "emote-tired", name: "Tired", duration_secs: 4.6, is_dance: false },
    Emote { id: "emote-greedy", name: "Greedy", duration_secs: 4.6, is_dance: false },
    Emote { id: "emote-model", name: "Model", duration_secs: 6.5, is_dance: false },
    Emote { id: "emote-bow", name: "Bow", duration_secs: 3.3, is_dance: false },
    Emote { id: "emote-curtsy", name: "Curtsy", duration_secs: 2.4, is_dance: false },
    Emote { id: "emote-cute", name: "Cute", duration_secs: 6.2, is_dance: false },
    Emote { id: "emote-energyball", name: "Energy Ball", duration_secs: 7.6, is_dance: false },
    Emote { id: "emote-float", name: "Float", duration_secs: 9.0, is_dance: false },
    Emote { id: "emote-frog", name: "Frog", duration_secs: 14.5, is_dance: false },
    Emote { id: "emote-teleporting", name: "Teleporting", duration_secs: 11.8, is_dance: false },
    Emote { id: "emote-gravity", name: "Gravity", duration_secs: 9.0, is_dance: false },
    Emote { id: "emote-swordfight", name: "Swordfight", duration_secs: 5.9, is_dance: false },
    Emote { id: "emote-celebrate", name: "Celebrate", duration_secs: 3.4, is_dance: false },
    Emote { id: "emote-clap", name: "Clap", duration_secs: 2.1, is_dance: false },
    Emote { id: "emote-thumbsup", name: "Thumbs Up", duration_secs: 2.7, is_dance: false },
    Emote { id: "idle-loop-tapdance", name: "Tap Loop", duration_secs: 6.3, is_dance: true },
    Emote { id: "idle-enthusiastic", name: "Enthusiastic", duration_secs: 15.9, is_dance: false },
    Emote { id: "idle-posh", name: "Posh", duration_secs: 21.9, is_dance: false },
    Emote { id: "sit-idle-cute", name: "Rest", duration_secs: 17.1, is_dance: false },
    Emote { id: "emoji-ghost", name: "Ghost", duration_secs: 3.5, is_dance: false },
    Emote { id: "emoji-poop", name: "Poop", duration_secs: 4.8, is_dance: false },
];

impl Default for Emotes {
    fn default() -> Self {
        Self::new()
    }
}

impl Emotes {
    pub fn new() -> Self {
        Self {
            items: STANDARD_EMOTES.to_vec(),
            blocked: HashSet::new(),
        }
    }

    /// Blocks an emote by ID or name (useful to suppress unwanted triggers like `yes`/`no`).
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

    /// Returns only dance emotes that are not blocked.
    pub fn dances(&self) -> Vec<&Emote> {
        self.allowed().into_iter().filter(|e| e.is_dance).collect()
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
