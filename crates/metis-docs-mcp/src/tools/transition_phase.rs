use metis_core::{
    application::services::workspace::PhaseTransitionService, domain::documents::types::Phase,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "transition_phase",
    description = "Transition a document to a new phase",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Document ID to transition  
    pub document_id: String,
    /// New phase to transition to
    pub new_phase: String,
    /// Force transition even if exit criteria aren't met
    pub force: Option<bool>,
}

impl TransitionPhaseTool {
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
        let target_phase = self.parse_phase(&self.new_phase)?;

        // Create the phase transition service
        let transition_service = PhaseTransitionService::new(metis_dir);

        // Perform the transition
        let result = transition_service
            .transition_document(&self.document_id, target_phase)
            .await
            .map_err(|e| CallToolError::new(e))?;

        let response = serde_json::json!({
            "success": true,
            "document_id": result.document_id,
            "document_type": result.document_type,
            "from_phase": result.from_phase.to_string(),
            "to_phase": result.to_phase.to_string(),
            "file_path": result.file_path.to_string_lossy(),
            "forced": self.force.unwrap_or(false)
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
}
