use anyhow::Result;
use metis_docs_tui::app::App;
use std::path::PathBuf;

// Re-export the shared test helper from core
pub use metis_core::tests::common::MetisTestHelper;

/// TUI-specific test helper that wraps the core helper
pub struct TestHelper {
    core_helper: MetisTestHelper,
}

impl TestHelper {
    pub async fn new() -> Result<Self> {
        let core_helper = MetisTestHelper::new().await?;
        Ok(Self { core_helper })
    }

    /// Get the metis directory path (for backward compatibility)
    pub fn metis_dir(&self) -> &PathBuf {
        &self.core_helper.metis_dir
    }

    pub fn create_app(&self) -> App {
        let mut app = App::new();

        // Set the workspace and mark as ready
        app.core_state.set_workspace(self.core_helper.metis_dir.clone());
        app.core_state.set_sync_complete();

        // Initialize services
        app.document_service = Some(metis_docs_tui::services::DocumentService::new(
            self.core_helper.metis_dir.clone(),
        ));
        app.sync_service = Some(metis_docs_tui::services::SyncService::new(
            self.core_helper.metis_dir.clone(),
        ));
        app.transition_service = Some(metis_docs_tui::services::TransitionService::new(
            self.core_helper.metis_dir.clone(),
        ));

        app
    }
}
