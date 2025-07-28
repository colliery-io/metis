pub mod kanban;

pub use kanban::*;

// Add database document structure
#[derive(Debug, Clone)]
pub struct DatabaseDocument {
    pub id: String,
    pub title: String,
    pub document_type: metis_core::domain::documents::types::DocumentType,
    pub filepath: String,
    pub archived: bool,
}
