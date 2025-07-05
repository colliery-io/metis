//! Error handling and edge case tests

use super::common::*;
use metis_mcp_server::tools::{CreateDocumentTool, ListDocumentsTool};

/// Test foreign key constraint handling
#[tokio::test]
async fn test_foreign_key_constraint_handling() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_result = initialize_test_project(&project_path, "fk-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);

    // Try to create a document with non-existent parent
    let orphan_doc = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Orphan Initiative".to_string(),
        parent_title: Some("Non Existent Strategy".to_string()),
        risk_level: None,
        complexity: Some("s".to_string()),
        decision_maker: None,
        stakeholders: None,
    };

    let orphan_result = orphan_doc.call_tool().await;

    // This should fail gracefully, not crash
    match orphan_result {
        Ok(call_result) => {
            println!("Tool handled orphan document: {:?}", call_result);
            // Tool should handle this gracefully, either by creating or rejecting
        }
        Err(e) => {
            println!("Tool failed gracefully for orphan document: {:?}", e);
        }
    }

    // The list tool should still work even if there are problematic documents
    let list_tool = ListDocumentsTool {
        project_path: metis_path,
        document_type: None,
        phase: None,
        limit: None,
    };

    let list_result = list_tool.call_tool().await;
    // The sync might fail due to foreign key constraint, but the tool should handle it
    match list_result {
        Ok(_) => println!("List tool handled constraints gracefully"),
        Err(e) => println!("List tool reported error: {:?}", e),
    }
}

// TODO: Move more error handling tests here from the original file
// - Invalid input validation tests
// - Missing file handling tests
// - Corrupted database recovery tests
