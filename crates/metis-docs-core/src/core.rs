//! Core document management functions for the Metis methodology
//!
//! This module provides the main public API for document creation, validation,
//! and rendering. It re-exports types and functions from specialized modules.

// Re-export main types and functions
pub use crate::context::{Complexity, DocumentContext, RiskLevel};
pub use crate::exit_criteria::{
    validate_exit_criteria, validate_exit_criteria_content, ExitCriteriaResult,
};
pub use crate::phases::{can_transition_to_phase, transition_phase};
pub use crate::render::render;
pub use crate::template::TemplateEngine;
pub use crate::validation::{validate, validate_content, ValidationResult};
