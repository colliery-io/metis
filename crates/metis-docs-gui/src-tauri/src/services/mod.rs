pub mod archive;
pub mod document;
pub mod project;
pub mod sync;
pub mod transition;
pub mod version;

// Re-export main service functions
pub use archive::archive_document;
pub use document::{
    create_document, get_available_parents, list_documents, read_document, search_documents,
    update_document,
};
pub use project::{get_project_config, initialize_project, load_project};
pub use sync::sync_project;
pub use transition::transition_phase;
pub use version::get_app_version;
