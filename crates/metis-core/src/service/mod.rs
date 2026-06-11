//! The service layer: business logic over the DAL and object store.
//!
//! Services are the single implementation that the REST API, MCP tools, and CLI
//! all sit on (see METIS-I-0031 D3). They are synchronous (diesel-dualdb v1 is
//! sync); the server runs them via `spawn_blocking`. Each method takes a live
//! [`DualConnection`](diesel_dualdb::DualConnection) and an
//! [`Actor`](crate::actor::Actor); mutations run in a transaction and write an
//! `events` audit row.

pub mod item;
pub mod project;
pub mod query;
pub mod user;

pub use query::ItemFilter;

/// A failure in a service operation.
#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    /// A referenced entity does not exist.
    #[error("not found: {0}")]
    NotFound(String),
    /// Input failed a business-rule check.
    #[error("validation error: {0}")]
    Validation(String),
    /// An optimistic-concurrency precondition failed (e.g. stale `content_key`).
    #[error("conflict: {0}")]
    Conflict(String),
    /// A workflow transition was rejected.
    #[error(transparent)]
    Transition(#[from] crate::workflow::transition::TransitionError),
    /// The project configuration is invalid.
    #[error(transparent)]
    Config(#[from] crate::workflow::config::ConfigError),
    /// An object-store operation failed.
    #[error(transparent)]
    Object(#[from] crate::objects::ObjectError),
    /// A database operation failed.
    #[error("database error: {0}")]
    Db(#[from] diesel::result::Error),
}

/// Service result alias.
pub type Result<T> = std::result::Result<T, ServiceError>;
