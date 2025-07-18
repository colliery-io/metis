//! Functional integration tests for MCP tools

use metis_mcp_server::tools::*;
use std::fs;
use tempfile::tempdir;

#[tokio::test]
async fn test_initialize_project_functional() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    let tool = InitializeProjectTool {
        project_path: project_path.clone(),
    };

    let result = tool.call_tool().await;
    assert!(result.is_ok(), "Initialize project should succeed");

    // Verify .metis directory was created (note the dot!)
    let metis_dir = temp_dir.path().join(".metis");
    assert!(metis_dir.exists());
    assert!(metis_dir.is_dir());

    // Verify database was created
    let db_path = metis_dir.join("metis.db");
    assert!(db_path.exists());

    // Verify vision.md was created
    let vision_path = metis_dir.join("vision.md");
    assert!(vision_path.exists());

    // Verify vision content (should use temp directory name as project name)
    let vision_content = fs::read_to_string(&vision_path).unwrap();
    assert!(vision_content.contains("#vision"));
    assert!(vision_content.contains("#phase/draft"));
}

#[tokio::test]
async fn test_full_document_lifecycle() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // 1. Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
    };

    let result = init_tool.call_tool().await;
    assert!(result.is_ok(), "Initialize should succeed");

    // 2. Create a strategy (use .metis directory path)
    let metis_path = format!("{}/.metis", project_path);
    let project_name = temp_dir.path().file_name().unwrap().to_str().unwrap();
    let create_strategy = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: Some(project_name.to_string()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["dev_team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(result.is_ok(), "Create strategy should succeed");

    // 3. List documents
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
    };

    let result = list_tool.call_tool().await;
    assert!(result.is_ok(), "List documents should succeed");

    // 4. Validate the vision document
    let validate_tool = ValidateDocumentTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
    };

    let result = validate_tool.call_tool().await;
    assert!(result.is_ok(), "Validate document should succeed");

    // 5. Search for documents
    let search_tool = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: "Test".to_string(),
        document_type: None,
        limit: None,
    };

    let result = search_tool.call_tool().await;
    assert!(result.is_ok(), "Search documents should succeed");
}

#[tokio::test]
async fn test_phase_transition_workflow() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
    };

    let result = init_tool.call_tool().await;
    assert!(result.is_ok());

    // First, let's check if we can transition the vision from draft to review
    // The document ID is derived from the project name (temp directory name)
    let metis_path = format!("{}/.metis", project_path);
    let project_name = temp_dir.path().file_name().unwrap().to_str().unwrap();
    // The document ID is derived from the title, which is the project name
    // Using the DocumentId::from_title conversion logic
    let vision_id = project_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    // Check phase transition removed - use transition_phase tool instead

    // Validate exit criteria for the vision
    let validate_exit = ValidateExitCriteriaTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
    };

    let result = validate_exit.call_tool().await;
    assert!(result.is_ok(), "Validate exit criteria should succeed");
}

#[tokio::test]
async fn test_document_updates() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();

    // Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
    };

    let result = init_tool.call_tool().await;
    assert!(result.is_ok());

    // Update document content
    let metis_path = format!("{}/.metis", project_path);
    let update_content = UpdateDocumentContentTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
        section_heading: "Purpose".to_string(),
        new_content: "This is an updated purpose statement.".to_string(),
    };

    let result = update_content.call_tool().await;
    assert!(result.is_ok(), "Update document content should succeed");

    // Update blocked_by relationship
    let update_blocked = UpdateBlockedByTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
        blocked_by: vec!["external_approval".to_string()],
    };

    let result = update_blocked.call_tool().await;
    assert!(result.is_ok(), "Update blocked_by should succeed");
}
