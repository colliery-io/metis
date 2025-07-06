//! Common utilities and helper functions for integration tests

use metis_mcp_server::tools::{InitializeProjectTool, MetisTools};
use tempfile::TempDir;
use tokio::fs;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Initialize a test project in the given directory
pub async fn initialize_test_project(
    project_path: &str,
    project_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let tool = InitializeProjectTool {
        project_path: project_path.to_string(),
        project_name: project_name.to_string(),
        description: Some(format!("Test project: {}", project_name)),
    };

    let result = tool.call_tool().await;
    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to initialize project: {:?}", e).into()),
    }
}

/// Check if a file exists at the given path
pub async fn file_exists(path: &str) -> bool {
    fs::metadata(path).await.is_ok()
}

/// Get all available tool names for verification
pub fn get_tool_names() -> Vec<String> {
    MetisTools::tools()
        .iter()
        .map(|tool| tool.name.clone())
        .collect()
}
