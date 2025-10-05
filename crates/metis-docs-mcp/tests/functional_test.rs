//! Functional integration tests for MCP tools

use metis_core::Database;
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

    // 1.5. Set full configuration to enable all document types for testing
    let db_path = format!("{}/.metis/metis.db", project_path);
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    config_repo.set("flight_levels", r#"{"strategies_enabled":true,"initiatives_enabled":true}"#).unwrap();

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

    // 4. Search for documents
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
    let _vision_id = project_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string();

    // Test transition phase tool
    // The vision document ID should be the project name (which is the temp dir name)
    let vision_id = project_name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    
    let transition_tool = TransitionPhaseTool {
        project_path: metis_path.clone(),
        document_id: vision_id,
        phase: Some("review".to_string()),
        force: None,
    };

    let result = transition_tool.call_tool().await;
    if let Err(e) = &result {
        println!("Phase transition error: {:?}", e);
    }
    assert!(result.is_ok(), "Phase transition should succeed: {:?}", result);
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
    let update_content = EditDocumentTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
        search: "{Why this vision exists and what it aims to achieve}".to_string(),
        replace: "This is an updated purpose statement.".to_string(),
        replace_all: None,
    };

    let result = update_content.call_tool().await;
    assert!(result.is_ok(), "Update document content should succeed");

    // Test updating another section
    let update_purpose = EditDocumentTool {
        project_path: metis_path.clone(),
        document_path: "vision.md".to_string(),
        search: "This is an updated purpose statement.".to_string(),
        replace: "Updated purpose: To create an amazing system for work management.".to_string(),
        replace_all: None,
    };

    let result = update_purpose.call_tool().await;
    if let Err(e) = &result {
        println!("Update purpose error: {:?}", e);
    }
    assert!(result.is_ok(), "Update purpose content should succeed: {:?}", result);
}
