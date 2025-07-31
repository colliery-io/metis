//! Common utilities and helper functions for integration tests

use anyhow::Result;
use metis_core::dal::Database;
use metis_mcp_server::tools::InitializeProjectTool;
use tempfile::TempDir;

/// Create a temporary directory for testing
pub fn create_temp_dir() -> TempDir {
    TempDir::new().expect("Failed to create temp directory")
}

/// Helper struct for MCP server testing
pub struct McpTestHelper {
    pub temp_dir: TempDir,
    pub project_path: String,
    pub metis_dir: String,
}

impl McpTestHelper {
    pub fn new() -> Result<Self> {
        let temp_dir = create_temp_dir();
        let project_path = temp_dir.path().to_string_lossy().to_string();
        let metis_dir = format!("{}/.metis", project_path);

        Ok(Self {
            temp_dir,
            project_path,
            metis_dir,
        })
    }

    pub async fn initialize_project(&self) -> Result<()> {
        let init_tool = InitializeProjectTool {
            project_path: self.project_path.clone(),
        };

        let result = init_tool.call_tool().await;
        if result.is_err() {
            return Err(anyhow::anyhow!(
                "Failed to initialize project: {:?}",
                result
            ));
        }
        Ok(())
    }

    pub fn get_database(&self) -> Result<Database> {
        let db_path = format!("{}/metis.db", self.metis_dir);
        Database::new(&db_path).map_err(|e| anyhow::anyhow!("Database error: {}", e))
    }

    pub fn get_project_name(&self) -> String {
        self.temp_dir
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
