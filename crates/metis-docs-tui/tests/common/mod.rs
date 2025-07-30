use anyhow::Result;
use metis_docs_tui::app::App;
use std::path::PathBuf;
use tempfile::TempDir;

/// Simple test helper for TUI tests
pub struct TestHelper {
    pub temp_dir: TempDir,
    pub project_path: PathBuf,
    pub metis_dir: PathBuf,
}

impl TestHelper {
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path().to_path_buf();
        
        // Initialize metis workspace
        metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
            &project_path,
            "Test Project"
        ).await?;
        
        let metis_dir = project_path.join(".metis");
        
        Ok(Self {
            temp_dir,
            project_path,
            metis_dir,
        })
    }
    
    pub fn create_app(&self) -> App {
        let mut app = App::new();
        
        // Set the workspace and mark as ready
        app.core_state.set_workspace(self.metis_dir.clone());
        app.core_state.set_sync_complete();
        
        // Initialize services
        app.document_service = Some(metis_docs_tui::services::DocumentService::new(self.metis_dir.clone()));
        app.sync_service = Some(metis_docs_tui::services::SyncService::new(self.metis_dir.clone()));
        app.transition_service = Some(metis_docs_tui::services::TransitionService::new(self.metis_dir.clone()));
        
        app
    }
}