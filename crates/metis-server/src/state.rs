//! Shared application state and the blocking-DAL helper.

use std::sync::Arc;

use metis_core::objects::ObjectStore;
use metis_core::Pool;

use crate::config::ServerConfig;

/// Server state shared across handlers (cheap to clone — pool and store are
/// reference-counted).
#[derive(Clone)]
pub struct AppState {
    /// Connection pool (Postgres or SQLite, per config).
    pub pool: Pool,
    /// Object store backend for item bodies.
    pub store: Arc<dyn ObjectStore + Send + Sync>,
    /// Resolved configuration.
    pub config: Arc<ServerConfig>,
    /// In local mode, the implicit single user's id (auth is disabled).
    pub local_user: Option<uuid::Uuid>,
}

impl AppState {
    /// Run a synchronous DAL/service closure off the async runtime.
    ///
    /// diesel-dualdb is sync, so all database work goes through `spawn_blocking`
    /// with a pooled connection. The closure receives a live connection.
    pub async fn blocking<T, F>(&self, f: F) -> Result<T, crate::error::ApiError>
    where
        T: Send + 'static,
        F: FnOnce(&mut metis_core::DualConnection) -> Result<T, crate::error::ApiError>
            + Send
            + 'static,
    {
        let pool = self.pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| crate::error::ApiError::internal(format!("pool: {e}")))?;
            f(&mut conn)
        })
        .await
        .map_err(|e| crate::error::ApiError::internal(format!("join: {e}")))?
    }
}
