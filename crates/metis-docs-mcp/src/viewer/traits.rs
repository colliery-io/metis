use std::path::PathBuf;

/// Trait that all viewer backends must implement.
///
/// Each backend is responsible for opening files in its respective
/// viewer and reporting whether a file is already open (to avoid
/// tab/window sprawl).
pub trait DocumentViewer: Send + Sync {
    /// Open one or more document files in this viewer.
    fn open(&self, paths: &[PathBuf]) -> Result<(), ViewerError>;

    /// Check whether a file is already open in this viewer.
    /// Used for "look before you leap" — skip opening if already visible.
    fn is_open(&self, path: &PathBuf) -> Result<bool, ViewerError>;

    /// Human-readable name of this viewer backend (e.g., "VSCode", "System Editor").
    fn name(&self) -> &str;

    /// Check whether this viewer backend is available on the system.
    fn is_available(&self) -> bool;
}

/// Errors that can occur when interacting with a viewer backend.
#[derive(Debug, thiserror::Error)]
pub enum ViewerError {
    #[error("Viewer '{viewer}' is not available: {reason}")]
    NotAvailable { viewer: String, reason: String },

    #[error("Failed to open document in '{viewer}': {reason}")]
    OpenFailed { viewer: String, reason: String },

    #[error("Failed to check open status in '{viewer}': {reason}")]
    StatusCheckFailed { viewer: String, reason: String },

    #[error("No viewer backend available")]
    NoViewerAvailable,
}
