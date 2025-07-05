use metis_core::{transition_phase, validate_exit_criteria};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    /// Path to the document file (relative to project root)
    pub document_path: String,

    /// New phase to transition to
    pub new_phase: String,

    /// Force transition even if exit criteria aren't met
    pub force: Option<bool>,
}

impl TransitionPhaseTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);
        let force = self.force.unwrap_or(false);

        match transition_phase(&document_path, &self.new_phase, force).await {
            Ok(_) => {
                let response = serde_json::json!({
                    "message": format!("Document transitioned to phase '{}' successfully", self.new_phase),
                    "document_path": self.document_path,
                    "new_phase": self.new_phase,
                    "project_path": self.project_path,
                    "force": force
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to transition phase: {}", e),
                    "document_path": self.document_path,
                    "new_phase": self.new_phase,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

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

    /// Path to the document file (relative to project root)
    pub document_path: String,

    /// Phase to check transition to
    pub target_phase: String,
}

impl CheckPhaseTransitionTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        // Note: can_transition_phase function doesn't exist in metis-core, using validate_exit_criteria instead
        match validate_exit_criteria(&document_path).await {
            Ok(validation_result) => {
                let can_transition = validation_result.met;
                let response = serde_json::json!({
                    "can_transition": can_transition,
                    "message": if can_transition {
                        format!("Document can transition to phase '{}'", self.target_phase)
                    } else {
                        format!("Document cannot transition to phase '{}' - exit criteria not met", self.target_phase)
                    },
                    "document_path": self.document_path,
                    "target_phase": self.target_phase,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to check phase transition: {}", e),
                    "document_path": self.document_path,
                    "target_phase": self.target_phase,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

#[mcp_tool(
    name = "validate_exit_criteria",
    description = "Validate if a document's exit criteria are completed",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidateExitCriteriaTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,

    /// Path to the document file (relative to project root)
    pub document_path: String,
}

impl ValidateExitCriteriaTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        match validate_exit_criteria(&document_path).await {
            Ok(validation_result) => {
                let response = serde_json::json!({
                    "is_complete": validation_result.met,
                    "total_criteria": validation_result.total_criteria,
                    "completed_criteria": validation_result.completed_criteria,
                    "missing_criteria": validation_result.missing_criteria,
                    "message": if validation_result.met {
                        "All exit criteria are completed"
                    } else {
                        "Some exit criteria are still pending"
                    },
                    "document_path": self.document_path,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to validate exit criteria: {}", e),
                    "document_path": self.document_path,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
