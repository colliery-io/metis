use crate::constants::{DATABASE_FILE_NAME, METIS_DIR_NAME};
use anyhow::Result;
use std::path::{Path, PathBuf};

/// Service for detecting and validating Metis workspaces
pub struct WorkspaceDetectionService;

impl WorkspaceDetectionService {
    pub fn new() -> Self {
        Self
    }

    /// Find the nearest Metis workspace by traversing up the directory tree
    pub fn find_workspace(&self) -> Result<Option<PathBuf>> {
        let mut current_dir = std::env::current_dir()?;

        loop {
            let metis_dir = current_dir.join(METIS_DIR_NAME);

            if let Some(validated_dir) = self.validate_workspace(&metis_dir)? {
                return Ok(Some(validated_dir));
            }

            // Try parent directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        Ok(None)
    }

    /// Find workspace starting from a specific directory
    pub fn find_workspace_from(&self, start_path: &Path) -> Result<Option<PathBuf>> {
        let mut current_dir = start_path.to_path_buf();

        loop {
            let metis_dir = current_dir.join(METIS_DIR_NAME);

            if let Some(validated_dir) = self.validate_workspace(&metis_dir)? {
                return Ok(Some(validated_dir));
            }

            // Try parent directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        Ok(None)
    }

    /// Validate that a directory is a proper Metis workspace
    pub fn validate_workspace(&self, metis_dir: &Path) -> Result<Option<PathBuf>> {
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Ok(None);
        }

        let db_path = metis_dir.join(DATABASE_FILE_NAME);
        if !db_path.exists() {
            return Err(anyhow::anyhow!(
                "Metis workspace found at {:?} but database missing. Run 'metis sync'.",
                metis_dir
            ));
        }

        Ok(Some(metis_dir.to_path_buf()))
    }

    /// Check if a path is within a Metis workspace
    pub fn is_in_workspace(&self, path: &Path) -> Result<bool> {
        Ok(self.find_workspace_from(path)?.is_some())
    }

    /// Get the workspace root for a given path
    pub fn get_workspace_root(&self, path: &Path) -> Result<Option<PathBuf>> {
        if let Some(metis_dir) = self.find_workspace_from(path)? {
            // Return the parent of .metis directory (the actual project root)
            if let Some(parent) = metis_dir.parent() {
                return Ok(Some(parent.to_path_buf()));
            }
        }
        Ok(None)
    }
}

impl Default for WorkspaceDetectionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_workspace_missing_directory() {
        let service = WorkspaceDetectionService::new();
        let temp_dir = TempDir::new().unwrap();
        let metis_dir = temp_dir.path().join(METIS_DIR_NAME);

        let result = service.validate_workspace(&metis_dir).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_validate_workspace_missing_database() {
        let service = WorkspaceDetectionService::new();
        let temp_dir = TempDir::new().unwrap();
        let metis_dir = temp_dir.path().join(METIS_DIR_NAME);
        
        fs::create_dir_all(&metis_dir).unwrap();

        let result = service.validate_workspace(&metis_dir);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("database missing"));
    }

    #[test]
    fn test_validate_workspace_valid() {
        let service = WorkspaceDetectionService::new();
        let temp_dir = TempDir::new().unwrap();
        let metis_dir = temp_dir.path().join(METIS_DIR_NAME);
        
        fs::create_dir_all(&metis_dir).unwrap();
        fs::File::create(metis_dir.join(DATABASE_FILE_NAME)).unwrap();

        let result = service.validate_workspace(&metis_dir).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), metis_dir);
    }

    #[test]
    fn test_find_workspace_traversal() {
        let service = WorkspaceDetectionService::new();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let metis_dir = project_root.join(".metis");
        let nested_dir = project_root.join("src").join("deep").join("nested");
        
        fs::create_dir_all(&metis_dir).unwrap();
        fs::File::create(metis_dir.join(DATABASE_FILE_NAME)).unwrap();
        fs::create_dir_all(&nested_dir).unwrap();

        let result = service.find_workspace_from(&nested_dir).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), metis_dir);
    }

    #[test]
    fn test_get_workspace_root() {
        let service = WorkspaceDetectionService::new();
        let temp_dir = TempDir::new().unwrap();
        let project_root = temp_dir.path();
        let metis_dir = project_root.join(".metis");
        let nested_dir = project_root.join("src").join("deep");
        
        fs::create_dir_all(&metis_dir).unwrap();
        fs::File::create(metis_dir.join(DATABASE_FILE_NAME)).unwrap();
        fs::create_dir_all(&nested_dir).unwrap();

        let result = service.get_workspace_root(&nested_dir).unwrap();
        assert!(result.is_some());
        assert_eq!(result.unwrap(), project_root);
    }
}