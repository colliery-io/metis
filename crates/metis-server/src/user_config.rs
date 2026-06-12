//! User-level client configuration (`~/.config/metis/config.toml`).
//!
//! Holds how a developer machine reaches its Metis server: a `database_url`
//! (direct-DB mode, the T-0132 model) and a `token` used to attribute the
//! session's actor. `metis mcp` and `metis hook` fall back to this when their
//! flags / env are absent. Unknown fields are ignored so the schema can grow
//! (e.g. a future `server_url` for the hosted HTTP transport).

use std::path::PathBuf;

use serde::Deserialize;

/// Client config read from `~/.config/metis/config.toml`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct UserConfig {
    /// Database URL the client connects to (Postgres team DB or SQLite path).
    pub database_url: Option<String>,
    /// Bearer token resolved to the acting user for attribution.
    pub token: Option<String>,
    /// Default project slug, when a remote matches none or to scope commands.
    pub default_project: Option<String>,
}

impl UserConfig {
    /// The config file path: `$XDG_CONFIG_HOME/metis/config.toml`, else
    /// `$HOME/.config/metis/config.toml`.
    pub fn path() -> Option<PathBuf> {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))?;
        Some(base.join("metis").join("config.toml"))
    }

    /// Load the config, or a default (all `None`) if absent or unparseable.
    /// Never errors — a missing/garbled file must not break a hook.
    pub fn load() -> Self {
        Self::path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|s| Self::parse(&s).ok())
            .unwrap_or_default()
    }

    /// Parse config from a TOML string.
    pub fn parse(s: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(s)
    }
}
