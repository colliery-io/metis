use crate::formatting::ToolOutput;
use metis_core::application::services::workspace::initialization::WorkspaceInitializationService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "initialize_project",
    description = "Initialize a new Metis project by creating a 'metis' subdirectory at the specified path. Sets up project configuration including short code generation (format: PREFIX-TYPE-NNNN).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct InitializeProjectTool {
    /// Path where the '.metis' subdirectory will be created (e.g., "/path/to/my-project" creates "/path/to/my-project/.metis/")
    pub project_path: String,
    /// Optional project prefix for document short codes, up to 6 characters (e.g., "PROJ", "ACME", "TEST"). If not provided, defaults to "PROJ"
    pub prefix: Option<String>,
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
        let result = WorkspaceInitializationService::initialize_workspace_with_prefix(
            project_path,
            project_name,
            self.prefix.as_deref(),
        )
        .await
        .map_err(|e| CallToolError::new(e))?;

        // Get the configured prefix to include in response, limiting to 6 characters
        let configured_prefix = {
            let prefix = self.prefix.as_deref().unwrap_or("PROJ").to_uppercase();
            if prefix.len() > 6 {
                prefix.chars().take(6).collect()
            } else {
                prefix
            }
        };

        let output = ToolOutput::new()
            .header("Project Initialized")
            .success(&format!(
                "Initialized Metis workspace at {}",
                result.metis_dir.display()
            ))
            .table(
                &["Field", "Value"],
                vec![
                    vec![
                        "Metis Directory".to_string(),
                        result.metis_dir.to_string_lossy().to_string(),
                    ],
                    vec![
                        "Database".to_string(),
                        result.database_path.to_string_lossy().to_string(),
                    ],
                    vec![
                        "Vision".to_string(),
                        result.vision_path.to_string_lossy().to_string(),
                    ],
                    vec!["Project Prefix".to_string(), configured_prefix],
                ],
            )
            .build_result();

        Ok(output)
    }
}
