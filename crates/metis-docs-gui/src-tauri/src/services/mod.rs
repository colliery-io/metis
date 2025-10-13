pub mod project;
pub mod document;
pub mod archive;
pub mod transition;

// Re-export main service functions
pub use project::{initialize_project, load_project, get_project_config};
pub use document::{create_document, update_document, list_documents, read_document, search_documents, get_available_parents};
pub use archive::archive_document;
pub use transition::transition_phase;