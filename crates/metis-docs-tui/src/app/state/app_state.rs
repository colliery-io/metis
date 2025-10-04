use std::path::PathBuf;
use metis_core::domain::configuration::FlightLevelConfig;

/// Core application state that persists across the application lifecycle
#[derive(Debug, Clone)]
pub struct CoreAppState {
    pub workspace_dir: Option<PathBuf>,
    pub workspace_valid: bool,
    pub sync_complete: bool,
    pub sync_in_progress: bool,
    pub flight_config: FlightLevelConfig,
}

impl CoreAppState {
    pub fn new() -> Self {
        Self {
            workspace_dir: None,
            workspace_valid: false,
            sync_complete: false,
            sync_in_progress: false,
            flight_config: FlightLevelConfig::default(), // Start with default (full) configuration
        }
    }

    pub fn is_ready(&self) -> bool {
        self.workspace_valid && self.sync_complete
    }

    pub fn set_workspace(&mut self, workspace_dir: PathBuf) {
        self.workspace_dir = Some(workspace_dir);
        self.workspace_valid = true;
    }

    pub fn set_sync_complete(&mut self) {
        self.sync_complete = true;
        self.sync_in_progress = false;
    }

    pub fn set_sync_in_progress(&mut self) {
        self.sync_in_progress = true;
        self.sync_complete = false;
    }

    pub fn set_flight_config(&mut self, config: FlightLevelConfig) {
        self.flight_config = config;
    }
}

impl Default for CoreAppState {
    fn default() -> Self {
        Self::new()
    }
}
