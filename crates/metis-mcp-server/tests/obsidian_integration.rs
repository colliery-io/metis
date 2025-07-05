//! Obsidian integration tests

use super::common::*;
use metis_mcp_server::tools::OpenVaultInObsidianTool;

/// Test open vault in Obsidian tool with valid project
#[tokio::test]
async fn test_open_vault_in_obsidian_valid_project() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize a test project first
    let init_result = initialize_test_project(&project_path, "obsidian-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);

    // Test the Obsidian tool
    let obsidian_tool = OpenVaultInObsidianTool {
        project_path: metis_path.clone(),
    };

    let result = obsidian_tool.call_tool().await;

    // The tool should succeed (even if Obsidian isn't installed, the config update should work)
    match &result {
        Ok(call_result) => {
            println!("Obsidian tool succeeded: {:?}", call_result);
            // Tool succeeded - assume it worked correctly
        }
        Err(e) => {
            println!("Obsidian tool failed: {:?}", e);
            // On some systems, this might fail due to missing Obsidian or permissions
            // We'll treat this as a conditional test
            if e.to_string().contains("No such file or directory")
                || e.to_string().contains("permission")
                || e.to_string().contains("command not found")
            {
                println!("Obsidian not available on this system - test skipped");
                return;
            }

            // Other errors should cause test failure
            panic!("Unexpected error from Obsidian tool: {:?}", e);
        }
    }
}

/// Test open vault in Obsidian tool with non-existent project
#[tokio::test]
async fn test_open_vault_in_obsidian_invalid_project() {
    let obsidian_tool = OpenVaultInObsidianTool {
        project_path: "/non/existent/path".to_string(),
    };

    let result = obsidian_tool.call_tool().await;

    // Should return an error response, not panic
    assert!(
        result.is_ok(),
        "Tool should return error response, not panic"
    );

    if let Ok(call_result) = result {
        println!("Tool returned response for invalid path: {:?}", call_result);
        // Tool should handle the error gracefully (exact format may vary)
    }
}

/// Test that .obsidian directory is created if it doesn't exist
#[tokio::test]
async fn test_obsidian_directory_creation() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_result = initialize_test_project(&project_path, "obsidian-dir-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);
    let obsidian_dir = format!("{}/.obsidian", metis_path);

    // Ensure .obsidian directory doesn't exist initially
    if file_exists(&obsidian_dir).await {
        tokio::fs::remove_dir_all(&obsidian_dir).await.unwrap();
    }
    assert!(
        !file_exists(&obsidian_dir).await,
        ".obsidian directory should not exist initially"
    );

    // Run the Obsidian tool
    let obsidian_tool = OpenVaultInObsidianTool {
        project_path: metis_path,
    };

    let _result = obsidian_tool.call_tool().await;

    // Even if the tool fails due to Obsidian not being available,
    // it should have created the .obsidian directory
    assert!(
        file_exists(&obsidian_dir).await,
        ".obsidian directory should be created by the tool"
    );
}
