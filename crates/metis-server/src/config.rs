//! Server configuration. Precedence: CLI flags > environment > built-in default.

use std::net::SocketAddr;
use std::path::PathBuf;

use metis_core::objects::ObjectStoreConfig;

/// Resolved server configuration.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Database URL. `postgres://…` selects Postgres; anything else SQLite.
    pub database_url: String,
    /// Where item bodies are stored.
    pub object_store: ObjectStoreConfig,
    /// Address to bind the HTTP server to.
    pub bind: SocketAddr,
    /// Local mode: localhost only, auth disabled, implicit single user.
    pub local: bool,
}

impl ServerConfig {
    /// Build config for team (server) mode from explicit values + env fallbacks.
    pub fn team(
        database_url: Option<String>,
        bind: Option<SocketAddr>,
        object_root: Option<PathBuf>,
    ) -> anyhow::Result<Self> {
        let database_url = database_url
            .or_else(|| std::env::var("DATABASE_URL").ok())
            .ok_or_else(|| anyhow::anyhow!("DATABASE_URL is required (or pass --database-url)"))?;
        let object_store = match object_root
            .or_else(|| std::env::var("METIS_OBJECT_ROOT").ok().map(PathBuf::from))
        {
            Some(root) => ObjectStoreConfig::Filesystem { root },
            None => ObjectStoreConfig::DbBlob,
        };
        let bind = bind
            .or_else(|| {
                std::env::var("METIS_BIND")
                    .ok()
                    .and_then(|s| s.parse().ok())
            })
            .unwrap_or_else(|| "0.0.0.0:7878".parse().unwrap());
        Ok(Self {
            database_url,
            object_store,
            bind,
            local: false,
        })
    }

    /// Build config for local (solo) mode: a SQLite file, db-blob store, bound
    /// to localhost, auth disabled.
    pub fn local(database_url: Option<String>, bind: Option<SocketAddr>) -> Self {
        Self {
            database_url: database_url.unwrap_or_else(|| "metis.db".to_string()),
            object_store: ObjectStoreConfig::DbBlob,
            bind: bind.unwrap_or_else(|| "127.0.0.1:7878".parse().unwrap()),
            local: true,
        }
    }
}
