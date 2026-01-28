use crate::formatting::ToolOutput;
use metis_core::{
    application::services::workspace::{PhaseTransitionService, WorkspaceDetectionService},
    domain::documents::types::{DocumentType, Phase},
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "transition_phase",
    description = "Transition a document to a new phase using its short code (e.g., PROJ-V-0001). If phase is not provided, transitions to the next valid phase automatically. IMPORTANT: You can only transition to adjacent phases - you cannot skip phases (e.g., todo->completed is invalid; must go todo->active->completed).",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Phase to transition to (optional - if not provided, transitions to next phase)
    pub phase: Option<String>,
    /// Force transition even if exit criteria aren't met
    pub force: Option<bool>,
}

impl TransitionPhaseTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let _db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Create the phase transition service
        let transition_service = PhaseTransitionService::new(metis_dir);

        // Perform the transition using short code directly
        let result = if let Some(phase_str) = &self.phase {
            // Transition to specific phase
            let target_phase = self.parse_phase(phase_str)?;
            transition_service
                .transition_document(&self.short_code, target_phase)
                .await
                .map_err(|e| CallToolError::new(e))?
        } else {
            // Auto-transition to next phase
            transition_service
                .transition_to_next_phase(&self.short_code)
                .await
                .map_err(|e| CallToolError::new(e))?
        };

        // Get phase progression for visual display
        let doc_type_str = result.document_type.to_string();
        let phases = self.get_phase_sequence(&doc_type_str);
        let current_index = phases
            .iter()
            .position(|p| *p == result.to_phase.to_string())
            .unwrap_or(0);

        let phase_strs: Vec<&str> = phases.iter().map(|s| s.as_str()).collect();

        let output = ToolOutput::new()
            .header("Phase Transition")
            .text(&format!(
                "{}: {} -> {}",
                self.short_code,
                result.from_phase,
                result.to_phase
            ))
            .blank()
            .phase_progress(&phase_strs, current_index)
            .build_result();

        Ok(output)
    }

    fn parse_phase(&self, phase_str: &str) -> Result<Phase, CallToolError> {
        match phase_str.to_lowercase().as_str() {
            "draft" => Ok(Phase::Draft),
            "review" => Ok(Phase::Review),
            "published" => Ok(Phase::Published),
            "discussion" => Ok(Phase::Discussion),
            "decided" => Ok(Phase::Decided),
            "superseded" => Ok(Phase::Superseded),
            "backlog" => Ok(Phase::Backlog),
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

    fn get_phase_sequence(&self, document_type: &str) -> Vec<String> {
        // Use DocumentType::phase_sequence() - the single source of truth
        let doc_type = match document_type {
            "vision" => Some(DocumentType::Vision),
            "strategy" => Some(DocumentType::Strategy),
            "initiative" => Some(DocumentType::Initiative),
            "task" => Some(DocumentType::Task),
            "adr" => Some(DocumentType::Adr),
            _ => None,
        };

        match doc_type {
            Some(dt) => dt.phase_sequence().iter().map(|p| p.to_string()).collect(),
            None => vec!["unknown".to_string()],
        }
    }
}
