//! # metis-core
//!
//! The storage and domain core of Metis 3.0 (see initiative METIS-I-0031). It
//! provides one query codebase that runs against PostgreSQL (team, self-hosted
//! deployments) or SQLite (solo/local mode) via [`diesel_dualdb`], a generated
//! relational schema, and — in later tasks — the workflow engine, object store,
//! data-access layer, and service layer that the server and CLI build on.
//!
//! ## Layers (filled in across METIS-I-0031 tasks)
//! - [`db`] — connection pooling + migrations (T-0126)
//! - [`schema`] — generated Diesel table definitions (T-0126)
//! - [`workflow`] — project config + transition engine + short codes (T-0127)
//! - [`objects`] — content-addressed object store (T-0128)
//! - [`models`] — row structs + DTOs; [`service`] — business logic (T-0129)
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod actor;
pub mod db;
pub mod models;
pub mod objects;
pub mod repo;
pub mod schema;
pub mod service;
pub mod workflow;

pub use actor::Actor;

/// Re-export of the dual-backend connection type for downstream crates.
pub use diesel_dualdb::DualConnection;
/// Re-export of the connection pool type.
pub use diesel_dualdb::Pool;
