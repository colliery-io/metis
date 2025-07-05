//! Document lifecycle tests (create, validate, transition)

use super::common::*;
use metis_mcp_server::tools::CreateDocumentTool;

/// Test creating different document types
#[tokio::test]
async fn test_create_documents() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/metis", project_path);

    // Initialize project
    let init_result = initialize_test_project(&project_path, "doc-lifecycle-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    // Test creating a strategy document
    let strategy_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_title: Some("metis-vision".to_string()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        decision_maker: None,
        stakeholders: None,
    };

    let strategy_result = strategy_tool.call_tool().await;
    assert!(
        strategy_result.is_ok(),
        "Strategy creation failed: {:?}",
        strategy_result.err()
    );

    // Test creating an initiative document
    let initiative_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_title: Some("Test Strategy".to_string()),
        risk_level: None,
        complexity: Some("s".to_string()),
        decision_maker: None,
        stakeholders: None,
    };

    let initiative_result = initiative_tool.call_tool().await;
    assert!(
        initiative_result.is_ok(),
        "Initiative creation failed: {:?}",
        initiative_result.err()
    );
}

// TODO: Move more document lifecycle tests here from the original file
// - Document validation tests
// - Phase transition tests
// - Document relationship tests
