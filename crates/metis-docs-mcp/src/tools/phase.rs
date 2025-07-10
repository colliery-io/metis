use metis_docs_core::{
    application::services::workspace::transition::PhaseTransitionService,
    domain::documents::{factory::DocumentFactory, types::Phase},
    Document,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

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
        
        // Parse the target phase
        let target_phase = match Phase::from_str(&self.new_phase) {
            Ok(phase) => phase,
            Err(_) => {
                let error_response = serde_json::json!({
                    "error": format!("Invalid phase: '{}'", self.new_phase)
                });
                return Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]));
            }
        };

        // Extract document ID from path (assuming format like "strategies/strategy-name/strategy.md")
        let document_id = self.extract_document_id();
        
        // Create transition service
        let transition_service = PhaseTransitionService::new(&project_path);
        
        // Perform the transition - let the service handle validation unless forced
        match transition_service.transition_document(&document_id, target_phase).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "message": format!("Document transitioned from {} to {}", result.previous_phase, result.new_phase),
                    "document_id": result.document_id,
                    "document_type": result.document_type.to_string(),
                    "previous_phase": result.previous_phase.to_string(),
                    "new_phase": result.new_phase.to_string(),
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to transition phase: {}", e),
                    "document_id": document_id,
                    "target_phase": self.new_phase,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
    
    fn extract_document_id(&self) -> String {
        // Convert document_path to document_id
        // Examples: "strategies/test-strategy/strategy.md" -> "test-strategy"
        //          "vision.md" -> "vision"
        let path = std::path::Path::new(&self.document_path);
        
        if let Some(parent) = path.parent() {
            if let Some(parent_name) = parent.file_name() {
                return parent_name.to_string_lossy().to_string();
            }
        }
        
        // Fallback: use filename without extension
        path.file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
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

        // Parse the target phase
        let target_phase = match Phase::from_str(&self.target_phase) {
            Ok(phase) => phase,
            Err(_) => {
                let error_response = serde_json::json!({
                    "error": format!("Invalid phase: '{}'", self.target_phase)
                });
                return Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]));
            }
        };

        match DocumentFactory::load_from_file(&document_path).await {
            Ok(document) => {
                let transition_service = PhaseTransitionService::new(&project_path);
                let doc_type = document.document_type();
                let current_phase = document.phase();
                let exit_criteria_met = document.exit_criteria_met();
                
                let is_valid_transition = transition_service.is_valid_transition(doc_type, current_phase, target_phase);
                let can_transition = is_valid_transition && exit_criteria_met;
                let valid_transitions = transition_service.get_valid_transitions_for(doc_type, current_phase);
                
                let response = serde_json::json!({
                    "can_transition": can_transition,
                    "is_valid_transition": is_valid_transition,
                    "exit_criteria_met": exit_criteria_met,
                    "current_phase": current_phase.to_string(),
                    "target_phase": self.target_phase,
                    "document_type": doc_type.to_string(),
                    "valid_transitions": valid_transitions.iter().map(|p| p.to_string()).collect::<Vec<_>>()
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to load document: {}", e)
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

        match DocumentFactory::load_from_file(&document_path).await {
            Ok(document) => {
                let exit_criteria_met = document.exit_criteria_met();
                
                let response = serde_json::json!({
                    "exit_criteria_met": exit_criteria_met,
                    "document_type": document.document_type().to_string(),
                    "current_phase": document.phase().to_string(),
                    "document_id": document.metadata().id.to_string()
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to load document: {}", e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
