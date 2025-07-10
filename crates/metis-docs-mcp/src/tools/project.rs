use metis_docs_core::application::services::workspace::WorkspaceInitializationService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// list_projects tool removed - direct path approach doesn't need project discovery

#[mcp_tool(
    name = "initialize_project",
    description = "Initialize a new Metis project by creating a 'metis' subdirectory at the specified path",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitializeProjectTool {
    /// Path where the 'metis' subdirectory will be created (e.g., "/path/to/my-project" creates "/path/to/my-project/metis/")
    pub project_path: String,

    /// Name of the project
    pub project_name: String,

    /// Optional description for the project
    pub description: Option<String>,
}

impl InitializeProjectTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let base_path = PathBuf::from(&self.project_path);

        match WorkspaceInitializationService::initialize_workspace(&base_path, &self.project_name).await {
            Ok(result) => {
                let response = serde_json::json!({
                    "message": format!("Project '{}' initialized successfully", self.project_name),
                    "project_path": result.metis_dir,
                    "database_path": result.database_path,
                    "vision_path": result.vision_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to initialize project '{}': {}", self.project_name, e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
