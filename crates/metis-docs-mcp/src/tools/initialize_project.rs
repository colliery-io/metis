use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use metis_core::application::services::workspace::initialization::WorkspaceInitializationService;
use std::path::Path;

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
    /// Path where the '.metis' subdirectory will be created (e.g., "/path/to/my-project" creates "/path/to/my-project/.metis/")
    pub project_path: String,
}

impl InitializeProjectTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = Path::new(&self.project_path);
        
        // Derive project name from the directory name
        let project_name = project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Metis Project");
        
        // Use the WorkspaceInitializationService to handle all the setup
        let result = WorkspaceInitializationService::initialize_workspace(project_path, project_name).await
            .map_err(|e| CallToolError::new(e))?;
        
        let response = serde_json::json!({
            "success": true,
            "message": format!("Initialized Metis workspace at {}", result.metis_dir.display()),
            "metis_directory": result.metis_dir.to_string_lossy(),
            "database_path": result.database_path.to_string_lossy(),
            "vision_path": result.vision_path.to_string_lossy(),
            "vision_created": true,
            "database_initialized": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }
}