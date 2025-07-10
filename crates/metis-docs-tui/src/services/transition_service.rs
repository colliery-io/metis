use anyhow::Result;
use std::path::PathBuf;
use metis_core::application::services::workspace::transition::PhaseTransitionService;

/// Service for document phase transitions
pub struct TransitionService {
    workspace_dir: PathBuf,
}

impl TransitionService {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    /// Transition a document to the next phase in its natural sequence
    pub async fn transition_to_next_phase(&self, document_id: String) -> Result<()> {
        let transition_service = PhaseTransitionService::new(&self.workspace_dir);
        
        let _result = transition_service.transition_to_next_phase(&document_id).await
            .map_err(|e| anyhow::anyhow!("Failed to transition document: {}", e))?;
        
        Ok(())
    }
}