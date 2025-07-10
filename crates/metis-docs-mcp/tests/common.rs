//! Common utilities and helper functions for integration tests


use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

