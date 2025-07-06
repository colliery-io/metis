use thiserror::Error;

#[derive(Debug, Error)]
pub enum McpServerError {
    #[error("Document not found: {document_path}")]
    DocumentNotFound { document_path: String },

    #[error("Invalid parameter: {param_name} - {message}")]
    InvalidParameter { param_name: String, message: String },

    #[error("Metis project not initialized at path: {path}")]
    ProjectNotInitialized { path: String },

    #[error("Core library error: {0}")]
    CoreLibrary(#[from] metis_core::MetisError),

    #[error("Configuration error: {0}")]
    Config(#[from] anyhow::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, McpServerError>;
