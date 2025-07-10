use metis_core::{
    application::services::workspace::PhaseTransitionService,
    domain::documents::{traits::Document, types::Phase},
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "check_phase_transition",
    description = "Check if a document can transition to a new phase",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CheckPhaseTransitionTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Document ID to check transition for
    pub document_id: String,
    /// Phase to check transition to
    pub target_phase: String,
}

impl CheckPhaseTransitionTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Validate metis workspace exists
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Metis workspace not found at {}. Run initialize_project first.",
                    metis_dir.display()
                ),
            )));
        }

        // Parse the target phase
        let target_phase = self.parse_phase(&self.target_phase)?;

        // Create the phase transition service
        let _transition_service = PhaseTransitionService::new(metis_dir);

        // Try to validate the transition - this will tell us if it's possible
        // We first need to find the document and get its current phase
        let discovery_service =
            metis_core::application::services::document::DocumentDiscoveryService::new(metis_dir);
        let discovery_result = discovery_service
            .find_document_by_id(&self.document_id)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Load current phase (reusing the same logic as the transition service)
        let current_phase = self
            .get_current_phase(&discovery_result.file_path, discovery_result.document_type)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Check if the transition would be valid by attempting validation
        let can_transition = match discovery_result.document_type {
            metis_core::domain::documents::types::DocumentType::Vision => {
                let vision = metis_core::Vision::from_file(&discovery_result.file_path)
                    .await
                    .map_err(|e| CallToolError::new(e))?;
                vision.can_transition_to(target_phase)
            }
            metis_core::domain::documents::types::DocumentType::Strategy => {
                let strategy = metis_core::Strategy::from_file(&discovery_result.file_path)
                    .await
                    .map_err(|e| CallToolError::new(e))?;
                strategy.can_transition_to(target_phase)
            }
            metis_core::domain::documents::types::DocumentType::Initiative => {
                let initiative = metis_core::Initiative::from_file(&discovery_result.file_path)
                    .await
                    .map_err(|e| CallToolError::new(e))?;
                initiative.can_transition_to(target_phase)
            }
            metis_core::domain::documents::types::DocumentType::Task => {
                let task = metis_core::Task::from_file(&discovery_result.file_path)
                    .await
                    .map_err(|e| CallToolError::new(e))?;
                task.can_transition_to(target_phase)
            }
            metis_core::domain::documents::types::DocumentType::Adr => {
                let adr = metis_core::Adr::from_file(&discovery_result.file_path)
                    .await
                    .map_err(|e| CallToolError::new(e))?;
                adr.can_transition_to(target_phase)
            }
        };

        let response = serde_json::json!({
            "can_transition": can_transition,
            "document_id": self.document_id,
            "current_phase": current_phase.to_string(),
            "target_phase": self.target_phase,
            "document_type": discovery_result.document_type.to_string(),
            "file_path": discovery_result.file_path.to_string_lossy(),
            "message": if can_transition {
                format!("Document can transition from {} to {}", current_phase, self.target_phase)
            } else {
                format!("Document cannot transition from {} to {} - transition is not valid for this document type", current_phase, self.target_phase)
            }
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

    fn parse_phase(&self, phase_str: &str) -> Result<Phase, CallToolError> {
        match phase_str.to_lowercase().as_str() {
            "draft" => Ok(Phase::Draft),
            "review" => Ok(Phase::Review),
            "published" => Ok(Phase::Published),
            "discussion" => Ok(Phase::Discussion),
            "decided" => Ok(Phase::Decided),
            "superseded" => Ok(Phase::Superseded),
            "todo" => Ok(Phase::Todo),
            "active" => Ok(Phase::Active),
            "blocked" => Ok(Phase::Blocked),
            "completed" => Ok(Phase::Completed),
            "shaping" => Ok(Phase::Shaping),
            "design" => Ok(Phase::Design),
            "ready" => Ok(Phase::Ready),
            "decompose" => Ok(Phase::Decompose),
            "discovery" => Ok(Phase::Discovery),
            _ => Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Unknown phase: {}", phase_str),
            ))),
        }
    }

    async fn get_current_phase(
        &self,
        file_path: &Path,
        doc_type: metis_core::domain::documents::types::DocumentType,
    ) -> Result<Phase, metis_core::MetisError> {
        use metis_core::domain::documents::types::DocumentType;
        match doc_type {
            DocumentType::Vision => {
                let vision = metis_core::Vision::from_file(file_path).await?;
                Ok(vision.phase()?)
            }
            DocumentType::Strategy => {
                let strategy = metis_core::Strategy::from_file(file_path).await?;
                Ok(strategy.phase()?)
            }
            DocumentType::Initiative => {
                let initiative = metis_core::Initiative::from_file(file_path).await?;
                Ok(initiative.phase()?)
            }
            DocumentType::Task => {
                let task = metis_core::Task::from_file(file_path).await?;
                Ok(task.phase()?)
            }
            DocumentType::Adr => {
                let adr = metis_core::Adr::from_file(file_path).await?;
                Ok(adr.phase()?)
            }
        }
    }
}
