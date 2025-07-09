use std::path::PathBuf;

/// Core application state that persists across the application lifecycle
#[derive(Debug, Clone)]
pub struct CoreAppState {
    pub workspace_dir: Option<PathBuf>,
    pub workspace_valid: bool,
    pub sync_complete: bool,
}

impl CoreAppState {
    pub fn new() -> Self {
        Self {
            workspace_dir: None,
            workspace_valid: false,
            sync_complete: false,
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
    }

    pub fn invalidate_workspace(&mut self) {
        self.workspace_valid = false;
        self.sync_complete = false;
    }
}

impl Default for CoreAppState {
    fn default() -> Self {
        Self::new()
    }
}