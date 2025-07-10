use anyhow::Result;
use std::path::PathBuf;

/// Service for workspace operations
pub struct WorkspaceService;

impl WorkspaceService {
    pub fn new() -> Self {
        Self
    }

    pub async fn check_workspace(&self) -> Result<Option<PathBuf>> {
        let current_dir = std::env::current_dir()?;
        let metis_dir = current_dir.join(".metis");
        
        if metis_dir.exists() && metis_dir.is_dir() {
            let db_path = metis_dir.join("metis.db");
            if db_path.exists() {
                Ok(Some(metis_dir))
            } else {
                Err(anyhow::anyhow!("Metis workspace found but database missing. Run 'metis sync'."))
            }
        } else {
            Err(anyhow::anyhow!("Not in a Metis workspace. Run 'metis init' to create one."))
        }
    }

}

impl Default for WorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}