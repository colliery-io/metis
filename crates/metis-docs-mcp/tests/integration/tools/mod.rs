//! Tests for tools module

// Module declarations
pub mod document;
pub mod obsidian;
pub mod phase;
pub mod project;
pub mod query;
pub mod update;

// Re-export all tools tests
pub use document::*;
pub use obsidian::*;
pub use phase::*;
pub use project::*;
pub use query::*;
pub use update::*;

use crate::common::*;
use metis_mcp_server::tools::MetisTools;

/// Test that MCP server tools are available
#[tokio::test]
async fn test_tools_available() {
    let tools = MetisTools::tools();

    // Verify we get tools back
    assert!(!tools.is_empty());

    // Verify key tools exist
    let tool_names = get_tool_names();
    assert!(tool_names.contains(&"initialize_project".to_string()));
    assert!(tool_names.contains(&"create_document".to_string()));
    assert!(tool_names.contains(&"list_documents".to_string()));
    assert!(tool_names.contains(&"search_documents".to_string()));
    assert!(tool_names.contains(&"update_document_content".to_string()));
    assert!(tool_names.contains(&"update_exit_criterion".to_string()));
    assert!(tool_names.contains(&"update_blocked_by".to_string()));
    assert!(tool_names.contains(&"transition_phase".to_string()));
    assert!(tool_names.contains(&"validate_exit_criteria".to_string()));
    assert!(tool_names.contains(&"validate_document".to_string()));
    assert!(tool_names.contains(&"open_vault_in_obsidian".to_string()));
}

/// Test initialize project tool directly
#[tokio::test]
async fn test_initialize_project_tool() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    let result = initialize_test_project(&project_path, "test-project").await;
    assert!(
        result.is_ok(),
        "Project initialization failed: {:?}",
        result.err()
    );

    // Verify essential files were created
    let metis_path = format!("{}/metis", project_path);
    assert!(
        file_exists(&format!("{}/vision.md", metis_path)).await,
        "Vision file not created"
    );
    assert!(
        file_exists(&format!("{}/.metis.db", metis_path)).await,
        "Database not created"
    );
}

/// Test project initialization is idempotent
#[tokio::test]
async fn test_initialize_project_idempotent() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize once
    let result1 = initialize_test_project(&project_path, "idempotent-test").await;
    assert!(result1.is_ok(), "First initialization failed");

    // Initialize again - should not fail
    let result2 = initialize_test_project(&project_path, "idempotent-test").await;
    assert!(
        result2.is_ok(),
        "Second initialization failed - not idempotent"
    );

    // Verify files still exist
    let metis_path = format!("{}/metis", project_path);
    assert!(
        file_exists(&format!("{}/vision.md", metis_path)).await,
        "Vision file missing after second init"
    );
}
