//! Work item types, workflows, transitions, and short codes.
//!
//! This module is pure domain logic — no database or object-store dependency —
//! so it can be unit-tested in isolation and reused by the DAL, service layer,
//! REST handlers, and CLI. See METIS-T-0127.

pub mod config;
pub mod short_code;
pub mod transition;

pub use config::{ConfigError, ItemTypeConfig, ProjectConfig};
pub use short_code::{format_short_code, parse_short_code, ShortCode, ShortCodeError};
pub use transition::{TransitionError, TransitionOutcome};
