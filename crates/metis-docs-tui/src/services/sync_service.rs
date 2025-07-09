use anyhow::Result;
use std::path::PathBuf;
use metis_core::{
    application::Application,
    dal::Database,
};

/// Service for database synchronization operations
pub struct SyncService {
    workspace_dir: PathBuf,
}

impl SyncService {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    pub async fn sync_database(&self) -> Result<()> {
        let db_path = self.workspace_dir.join("metis.db");
        let db = Database::new(&db_path.to_string_lossy())
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        let app = Application::new(db);
        
        match app.sync_directory(&self.workspace_dir).await {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Sync failed: {}", e)),
        }
    }

    pub async fn is_sync_needed(&self) -> Result<bool> {
        // This is a simplified check - in reality you'd want to compare
        // file modification times with database timestamps
        let db_path = self.workspace_dir.join("metis.db");
        
        if !db_path.exists() {
            return Ok(true);
        }

        // For now, assume sync is always needed
        // TODO: Implement proper sync detection logic
        Ok(true)
    }
}