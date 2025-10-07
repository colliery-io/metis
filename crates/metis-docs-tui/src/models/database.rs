#[derive(Debug, Clone)]
pub struct DatabaseDocument {
    pub id: String,
    pub title: String,
    pub document_type: metis_core::domain::documents::types::DocumentType,
    pub filepath: String,
    #[allow(dead_code)] // Used in filtering logic but not accessed directly in struct usage
    pub archived: bool,
}
