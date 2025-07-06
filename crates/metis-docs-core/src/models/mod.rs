//! Data models for the Metis document management system

pub mod document;

// Re-export main types for convenience
pub use document::{Complexity, Document, DocumentType, RelationshipType, RiskLevel};
