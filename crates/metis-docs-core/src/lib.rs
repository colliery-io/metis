//! Metis - A design-first software development documentation system
//!
//! Metis implements the Flight Levels methodology for hierarchical documentation
//! management, providing core functions for creating, validating, and transitioning
//! documents through their defined phases.

pub mod application;
pub mod dal;
pub mod domain;
pub mod error;

// Re-export main types for convenience
pub use application::Application;
pub use dal::Database;
pub use domain::documents::{
    adr::Adr,
    initiative::{Complexity, Initiative},
    strategy::{RiskLevel, Strategy},
    task::Task,
    vision::Vision,
    traits::{Document, DocumentValidationError},
    types::{DocumentId, DocumentType, Phase, Tag},
};
pub use error::{MetisError, Result};
