//! Metis - A design-first software development documentation system
//!
//! Metis implements the Flight Levels methodology for hierarchical documentation
//! management, providing core functions for creating, validating, and transitioning
//! documents through their defined phases.

pub mod context;
pub mod core;
pub mod database;
pub mod error;
pub mod exit_criteria;
pub mod models;
pub mod phases;
pub mod project;
pub mod render;
pub mod sync;
pub mod template;
pub mod updates;
pub mod validation;

// Re-export main types for convenience
pub use core::{
    can_transition_to_phase, render, transition_phase, validate, validate_content,
    validate_exit_criteria, validate_exit_criteria_content, Complexity, DocumentContext,
    ExitCriteriaResult, RiskLevel, TemplateEngine, ValidationResult,
};
pub use database::{
    DocumentStore, QueryService, Relationship, RelationshipDirection, SearchResult,
};
pub use error::{MetisError, Result};
pub use models::*;
pub use project::{initialize_project, ProjectConfig, ProjectMetadata};
pub use sync::{SyncEngine, SyncError, SyncResult};
pub use updates::{update_blocked_by, update_document_content, update_exit_criterion};
