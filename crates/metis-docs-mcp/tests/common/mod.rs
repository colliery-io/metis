//! Common utilities and helper functions for integration tests

use anyhow::Result;
use metis_core::dal::Database;
use metis_mcp_server::tools::InitializeProjectTool;

// Re-export the shared test helper from core
pub use metis_core::tests::common::MetisTestHelper;

/// MCP-specific test helper that wraps the core helper
pub struct McpTestHelper {
    core_helper: MetisTestHelper,
}

impl McpTestHelper {
    pub async fn new() -> Result<Self> {
        let core_helper = MetisTestHelper::new().await?;
        Ok(Self { core_helper })
    }

    /// Get project path as string (for backward compatibility)
    pub fn project_path(&self) -> String {
        self.core_helper.project_path_string()
    }

    /// Get metis directory as string (for backward compatibility)
    pub fn metis_dir(&self) -> String {
        self.core_helper.metis_dir_string()
    }

    pub async fn initialize_project(&self) -> Result<()> {
        let init_tool = InitializeProjectTool {
            project_path: self.core_helper.project_path_string(),
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
        self.core_helper.get_database()
    }

    pub fn get_project_name(&self) -> String {
        self.core_helper.temp_dir
            .path()
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
    }
}
