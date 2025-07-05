//! Error types for Metis operations

use thiserror::Error;

pub type Result<T> = std::result::Result<T, MetisError>;

#[derive(Debug, Error)]
pub enum MetisError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    #[error("Document not found: {id}")]
    DocumentNotFound { id: String },

    #[error("Invalid document type: {document_type}")]
    InvalidDocumentType { document_type: String },

    #[error("Invalid phase transition from {from} to {to} for document type {doc_type}")]
    InvalidPhaseTransition {
        from: String,
        to: String,
        doc_type: String,
    },

    #[error("Missing required field: {field}")]
    MissingRequiredField { field: String },

    #[error("Template not found: {template}")]
    TemplateNotFound { template: String },

    #[error("Validation failed: {message}")]
    ValidationFailed { message: String },

    #[error("Exit criteria not met: {missing_count} of {total_count} criteria incomplete")]
    ExitCriteriaNotMet {
        missing_count: usize,
        total_count: usize,
    },
}
