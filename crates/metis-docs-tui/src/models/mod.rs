pub mod kanban;

pub use kanban::*;

// Add database document structure
#[derive(Debug, Clone)]
pub struct DatabaseDocument {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub document_type: metis_core::domain::documents::types::DocumentType,
    pub phase: String,
    pub filepath: String,
    pub parent_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}