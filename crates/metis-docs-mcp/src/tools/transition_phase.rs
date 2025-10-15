use metis_core::{
    application::services::workspace::PhaseTransitionService, application::Application,
    dal::Database, domain::documents::types::Phase,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "transition_phase",
    description = "Transition a document to a new phase using its short code (e.g., PROJ-V-0001). If phase is not provided, transitions to the next valid phase automatically",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct TransitionPhaseTool {
    /// Path to the .metis folder containing the document
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

        // Auto-sync after transition to update database
        self.sync_workspace(metis_dir).await?;

        let response = serde_json::json!({
            "success": true,
            "short_code": self.short_code,
            "document_id": result.document_id,
            "document_type": result.document_type,
            "from_phase": result.from_phase.to_string(),
            "to_phase": result.to_phase.to_string(),
            "file_path": result.file_path.to_string_lossy(),
            "forced": self.force.unwrap_or(false),
            "auto_synced": true
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

    async fn sync_workspace(&self, metis_dir: &Path) -> Result<(), CallToolError> {
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open database for sync: {}", e),
            ))
        })?;
        let app = Application::new(database);

        app.sync_directory(metis_dir)
            .await
            .map_err(|e| CallToolError::new(e))?;

        Ok(())
    }
}
