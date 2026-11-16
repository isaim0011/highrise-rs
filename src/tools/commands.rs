use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};

use thiserror::Error;
use tokio::sync::RwLock;

use crate::api::Highrise;
use crate::types::User;

/// Ordered role hierarchy matching production multi-bot permissions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Role {
    #[default]
    Member = 0,
    Vip = 1,
    Mod = 2,
    Owner = 3,
    Creator = 4,
}

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("command is disabled")]
    Disabled,
    #[error("requires {0:?} role or higher")]
    InsufficientRole(Role),
    #[error("wait {0}s before using this command again")]
    Cooldown(u64),
    #[error("{0}")]
    Failed(String),
}

pub type CmdFuture<'a> = Pin<Box<dyn Future<Output = Result<(), CommandError>> + Send + 'a>>;
pub type CmdFn = Arc<dyn Fn(CommandContext) -> CmdFuture<'static> + Send + Sync>;

/// Metadata for a registered command.
#[derive(Clone)]
pub struct CommandMeta {
    pub name: String,
    pub category: String,
    pub description: String,
    pub usage: String,
    pub min_role: Role,
    pub cooldown: Duration,
    pub aliases: Vec<String>,
    pub enabled: bool,
}

/// Invocation context passed into every command handler.
#[derive(Clone)]
pub struct CommandContext {
    pub api: Highrise,
    pub user: User,
    pub role: Role,
    pub is_whisper: bool,
    pub prefix: String,
    pub command: String,
    pub args: Vec<String>,
}

impl CommandContext {
    /// Extracts `@username` mentions from `args` (with leading `@` stripped).
    pub fn mentions(&self) -> Vec<&str> {
        self.args
            .iter()
            .filter_map(|a| a.strip_prefix('@'))
            .filter(|s| !s.is_empty())
            .collect()
    }

    /// Returns `args[idx]` stripped of any leading `@`.
    pub fn target_arg(&self, idx: usize) -> Option<&str> {
        self.args.get(idx).map(|s| s.trim_start_matches('@')).filter(|s| !s.is_empty())
    }
}

#[derive(Default)]
struct Overrides {
    min_role: HashMap<String, Role>,
    enabled: HashMap<String, bool>,
    cooldown: HashMap<String, Duration>,
    user_allow: HashMap<String, HashSet<String>>,
    user_deny: HashMap<String, HashSet<String>>,
    cooldowns: HashMap<(String, String), Instant>,
}

struct Registered {
    meta: CommandMeta,
    handler: CmdFn,
}

/// Dynamic command router supporting multi-prefix, longest-match subcommands,
/// aliases, role hierarchy, per-user allow/deny overrides, and cooldowns.
#[derive(Clone)]
pub struct CommandManager {
    prefixes: Vec<String>,
    commands: Arc<RwLock<HashMap<String, Registered>>>,
    aliases: Arc<RwLock<HashMap<String, String>>>,
    overrides: Arc<RwLock<Overrides>>,
}

impl CommandManager {
    pub fn new(prefixes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            prefixes: prefixes.into_iter().map(Into::into).collect(),
            commands: Arc::new(RwLock::new(HashMap::new())),
            aliases: Arc::new(RwLock::new(HashMap::new())),
            overrides: Arc::new(RwLock::new(Overrides::default())),
        }
    }

    pub async fn register<F, Fut>(&self, meta: CommandMeta, handler: F)
    where
        F: Fn(CommandContext) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<(), CommandError>> + Send + 'static,
    {
        let key = meta.name.to_ascii_lowercase();
        let mut cmds = self.commands.write().await;
        let mut als = self.aliases.write().await;
        for alias in &meta.aliases {
            als.insert(alias.to_ascii_lowercase(), key.clone());
        }
        cmds.insert(
            key,
            Registered {
                meta,
                handler: Arc::new(move |ctx| Box::pin(handler(ctx))),
            },
        );
    }

    pub async fn add_alias(&self, alias: impl Into<String>, target_cmd: impl Into<String>) {
        self.aliases
            .write()
            .await
            .insert(alias.into().to_ascii_lowercase(), target_cmd.into().to_ascii_lowercase());
    }

    pub async fn set_min_role(&self, cmd: &str, role: Role) {
        self.overrides.write().await.min_role.insert(cmd.to_ascii_lowercase(), role);
    }

    pub async fn set_enabled(&self, cmd: &str, enabled: bool) {
        self.overrides.write().await.enabled.insert(cmd.to_ascii_lowercase(), enabled);
    }

    pub async fn set_cooldown(&self, cmd: &str, cooldown: Duration) {
        self.overrides.write().await.cooldown.insert(cmd.to_ascii_lowercase(), cooldown);
    }

    pub async fn allow_user(&self, cmd: &str, user_id: impl Into<String>) {
        let k = cmd.to_ascii_lowercase();
        let uid = user_id.into();
        let mut ov = self.overrides.write().await;
        if let Some(d) = ov.user_deny.get_mut(&k) {
            d.remove(&uid);
        }
        ov.user_allow.entry(k).or_default().insert(uid);
    }

    pub async fn deny_user(&self, cmd: &str, user_id: impl Into<String>) {
        let k = cmd.to_ascii_lowercase();
        let uid = user_id.into();
        let mut ov = self.overrides.write().await;
        if let Some(a) = ov.user_allow.get_mut(&k) {
            a.remove(&uid);
        }
        ov.user_deny.entry(k).or_default().insert(uid);
    }

    /// Dispatches a chat/whisper message if it starts with a configured prefix and
    /// matches a registered command or alias (longest multi-word match wins).
    pub async fn dispatch(
        &self,
        api: &Highrise,
        user: User,
        role: Role,
        text: &str,
        is_whisper: bool,
    ) -> Option<Result<(), CommandError>> {
        let trimmed = text.trim();
        let prefix = self.prefixes.iter().find(|p| trimmed.starts_with(p.as_str()))?;
        let rest = trimmed[prefix.len()..].trim();
        if rest.is_empty() {
            return None;
        }

        let tokens: Vec<&str> = rest.split_whitespace().collect();
        let cmds = self.commands.read().await;
        let als = self.aliases.read().await;

        // Longest-match subcommand resolution (e.g., "music fit save" > "music").
        let mut matched: Option<(String, usize)> = None;
        for len in (1..=tokens.len().min(4)).rev() {
            let candidate = tokens[..len].join(" ").to_ascii_lowercase();
            let resolved = als.get(&candidate).cloned().unwrap_or(candidate);
            if cmds.contains_key(&resolved) {
                matched = Some((resolved, len));
                break;
            }
        }
        let (cmd_name, consumed) = matched?;
        let reg = cmds.get(&cmd_name)?;
        let meta = reg.meta.clone();
        let handler = Arc::clone(&reg.handler);
        drop(cmds);
        drop(als);

        let mut ov = self.overrides.write().await;
        let enabled = ov.enabled.get(&cmd_name).copied().unwrap_or(meta.enabled);
        if !enabled {
            return Some(Err(CommandError::Disabled));
        }

        let denied = ov.user_deny.get(&cmd_name).is_some_and(|s| s.contains(&user.id));
        if denied && role < Role::Creator {
            return Some(Err(CommandError::InsufficientRole(Role::Creator)));
        }

        let explicitly_allowed = ov.user_allow.get(&cmd_name).is_some_and(|s| s.contains(&user.id));
        let min_role = ov.min_role.get(&cmd_name).copied().unwrap_or(meta.min_role);
        if !explicitly_allowed && role < min_role {
            return Some(Err(CommandError::InsufficientRole(min_role)));
        }

        let cd = ov.cooldown.get(&cmd_name).copied().unwrap_or(meta.cooldown);
        if !cd.is_zero() && role < Role::Owner {
            let key = (cmd_name.clone(), user.id.clone());
            if let Some(last) = ov.cooldowns.get(&key) {
                let elapsed = last.elapsed();
                if elapsed < cd {
                    let wait = (cd - elapsed).as_secs().max(1);
                    return Some(Err(CommandError::Cooldown(wait)));
                }
            }
            ov.cooldowns.insert(key, Instant::now());
        }
        drop(ov);

        let ctx = CommandContext {
            api: api.clone(),
            user,
            role,
            is_whisper,
            prefix: prefix.clone(),
            command: cmd_name,
            args: tokens[consumed..].iter().map(|s| (*s).to_string()).collect(),
        };
        Some(handler(ctx).await)
    }

    /// Returns help lines visible to the given `role` (respecting dynamic overrides).
    pub async fn help_for(&self, role: Role) -> Vec<String> {
        let cmds = self.commands.read().await;
        let ov = self.overrides.read().await;
        let prefix = self.prefixes.first().map(String::as_str).unwrap_or("/");
        let mut out = Vec::new();
        for (name, reg) in cmds.iter() {
            let enabled = ov.enabled.get(name).copied().unwrap_or(reg.meta.enabled);
            let min_role = ov.min_role.get(name).copied().unwrap_or(reg.meta.min_role);
            if enabled && role >= min_role {
                out.push(format!("{prefix}{} — {}", reg.meta.usage, reg.meta.description));
            }
        }
        out.sort();
        out
    }
}
