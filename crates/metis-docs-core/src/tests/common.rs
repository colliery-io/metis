//! Common test utilities for Metis core and other crates

use anyhow::Result;
use crate::application::services::workspace::WorkspaceInitializationService;
use crate::dal::Database;
use std::path::PathBuf;

/// Shared test helper for workspace setup
pub struct MetisTestHelper {
    pub temp_dir: tempfile::TempDir,
    pub project_path: PathBuf,
    pub metis_dir: PathBuf,
    pub db_path: PathBuf,
}

impl MetisTestHelper {
    /// Create a new test helper with initialized workspace
    pub async fn new() -> Result<Self> {
        let temp_dir = tempfile::TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();

        // Initialize metis workspace
        WorkspaceInitializationService::initialize_workspace(&project_path, "Test Project")
            .await?;

        let metis_dir = project_path.join(".metis");
        let db_path = metis_dir.join("metis.db");

        Ok(Self {
            temp_dir,
            project_path,
            metis_dir,
            db_path,
        })
    }

    /// Create a new test helper with custom project name
    pub async fn with_project_name(project_name: &str) -> Result<Self> {
        let temp_dir = tempfile::TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();

        // Initialize metis workspace
        WorkspaceInitializationService::initialize_workspace(&project_path, project_name).await?;

        let metis_dir = project_path.join(".metis");
        let db_path = metis_dir.join("metis.db");

        Ok(Self {
            temp_dir,
            project_path,
            metis_dir,
            db_path,
        })
    }

    /// Get a database connection for testing
    pub fn get_database(&self) -> Result<Database> {
        Database::new(&self.db_path.to_string_lossy())
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))
    }

    /// Get the project path as a string
    pub fn project_path_string(&self) -> String {
        self.project_path.to_string_lossy().to_string()
    }

    /// Get the metis directory path as a string
    pub fn metis_dir_string(&self) -> String {
        self.metis_dir.to_string_lossy().to_string()
    }

    /// Ensure the workspace exists and is valid
    pub fn validate_workspace(&self) -> Result<()> {
        if !self.metis_dir.exists() {
            return Err(anyhow::anyhow!("Metis directory does not exist"));
        }

        if !self.db_path.exists() {
            return Err(anyhow::anyhow!("Database file does not exist"));
        }

        Ok(())
    }

    /// Create additional directories for testing
    pub fn create_test_subdirs(&self, subdirs: &[&str]) -> Result<Vec<PathBuf>> {
        let mut created_paths = Vec::new();
        
        for subdir in subdirs {
            let path = self.project_path.join(subdir);
            std::fs::create_dir_all(&path)?;
            created_paths.push(path);
        }
        
        Ok(created_paths)
    }

    /// Write a test file to the workspace
    pub fn write_test_file<P: AsRef<std::path::Path>>(&self, relative_path: P, content: &str) -> Result<PathBuf> {
        let full_path = self.project_path.join(relative_path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        std::fs::write(&full_path, content)?;
        Ok(full_path)
    }
}