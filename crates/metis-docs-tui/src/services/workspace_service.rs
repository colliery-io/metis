use anyhow::Result;
use std::path::PathBuf;

/// Service for workspace operations
pub struct WorkspaceService;

impl WorkspaceService {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_workspace(&self) -> Result<Option<PathBuf>> {
        let mut current_dir = std::env::current_dir()?;

        loop {
            let metis_dir = current_dir.join(".metis");

            if metis_dir.exists() && metis_dir.is_dir() {
                let db_path = metis_dir.join("metis.db");
                if db_path.exists() {
                    return Ok(Some(metis_dir));
                } else {
                    return Err(anyhow::anyhow!(
                        "Metis workspace found but database missing. Run 'metis sync'."
                    ));
                }
            }

            // Try parent directory
            match current_dir.parent() {
                Some(parent) => current_dir = parent.to_path_buf(),
                None => break, // Reached filesystem root
            }
        }

        // No workspace found after checking all parent directories
        Ok(None)
    }
}

impl Default for WorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}
