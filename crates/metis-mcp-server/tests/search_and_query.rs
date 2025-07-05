//! Search and query operation tests

use super::common::*;
use metis_mcp_server::tools::{ListDocumentsTool, SearchDocumentsTool};

/// Test listing documents
#[tokio::test]
async fn test_list_documents() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_result = initialize_test_project(&project_path, "list-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);

    // Test listing all documents
    let list_tool = ListDocumentsTool {
        project_path: metis_path,
        document_type: None,
        phase: None,
        limit: None,
    };

    let list_result = list_tool.call_tool().await;
    assert!(
        list_result.is_ok(),
        "Document listing failed: {:?}",
        list_result.err()
    );

    if let Ok(call_result) = list_result {
        println!("List documents succeeded: {:?}", call_result);
        // Should find at least the vision document that was created during initialization
    }
}

/// Test searching documents
#[tokio::test]
async fn test_search_documents() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_result = initialize_test_project(&project_path, "search-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    let metis_path = format!("{}/metis", project_path);

    // Test searching for vision content
    let search_tool = SearchDocumentsTool {
        project_path: metis_path,
        query: "vision".to_string(),
        document_type: None,
        limit: None,
    };

    let search_result = search_tool.call_tool().await;
    assert!(
        search_result.is_ok(),
        "Document search failed: {:?}",
        search_result.err()
    );
}

// TODO: Move more search and query tests here from the original file
// - Filtered search tests
// - Advanced query tests
// - Performance tests
