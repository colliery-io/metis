//! Document update operation tests

use super::common::*;
use metis_mcp_server::tools::UpdateDocumentContentTool;

/// Test updating document content
#[tokio::test]
async fn test_update_document_content() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_result = initialize_test_project(&project_path, "update-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);

    // Test updating content
    let update_tool = UpdateDocumentContentTool {
        project_path: metis_path,
        document_path: "vision.md".to_string(),
        section_heading: "Vision Statement".to_string(),
        new_content: "Updated vision content for testing".to_string(),
    };

    let update_result = update_tool.call_tool().await;

    match &update_result {
        Ok(call_result) => {
            println!("Content update succeeded: {:?}", call_result);
        }
        Err(e) => {
            println!("Content update failed: {:?}", e);
        }
    }

    assert!(update_result.is_ok(), "Content update failed");
}

// TODO: Move more update operation tests here from the original file
// - Exit criteria update tests
// - Blocked by relationship update tests
// - Batch update tests
