use std::fmt;

/// Application-specific error types
#[derive(Debug, Clone)]
pub enum AppError {
    WorkspaceError(String),
    DocumentError(String),
    SyncError(String),
    ValidationError(String),
    IoError(String),
    DatabaseError(String),
    UserInputError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::WorkspaceError(msg) => write!(f, "Workspace Error: {}", msg),
            AppError::DocumentError(msg) => write!(f, "Document Error: {}", msg),
            AppError::SyncError(msg) => write!(f, "Sync Error: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Validation Error: {}", msg),
            AppError::IoError(msg) => write!(f, "IO Error: {}", msg),
            AppError::DatabaseError(msg) => write!(f, "Database Error: {}", msg),
            AppError::UserInputError(msg) => write!(f, "Input Error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::IoError(err.to_string())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::DocumentError(err.to_string())
    }
}

impl From<metis_core::MetisError> for AppError {
    fn from(err: metis_core::MetisError) -> Self {
        match err {
            metis_core::MetisError::Database(e) => AppError::DatabaseError(e.to_string()),
            metis_core::MetisError::FileSystem(msg) => AppError::IoError(msg),
            metis_core::MetisError::DocumentNotFound { id } => AppError::DocumentError(format!("Document not found: {}", id)),
            metis_core::MetisError::InvalidPhaseTransition { .. } => AppError::ValidationError(err.to_string()),
            _ => AppError::DocumentError(err.to_string()),
        }
    }
}

/// Result type alias for application operations
pub type AppResult<T> = Result<T, AppError>;