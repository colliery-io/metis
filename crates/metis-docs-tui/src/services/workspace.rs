use anyhow::Result;
use metis_core::application::services::workspace::WorkspaceDetectionService;
use std::path::PathBuf;

/// Service for workspace operations
pub struct WorkspaceService {
    detection_service: WorkspaceDetectionService,
}

impl WorkspaceService {
    pub fn new() -> Self {
        Self {
            detection_service: WorkspaceDetectionService::new(),
        }
    }

    pub async fn check_workspace(&self) -> Result<Option<PathBuf>> {
        self.detection_service.find_workspace()
    }
}

impl Default for WorkspaceService {
    fn default() -> Self {
        Self::new()
    }
}
