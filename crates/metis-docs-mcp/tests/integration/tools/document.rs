//! Document lifecycle tests (create, validate, transition)

use crate::common::*;
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
        strategy_id: Some("test-strategy".to_string()),
        initiative_id: None,
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
        strategy_id: Some("test-strategy".to_string()),
        initiative_id: None,
    };

    let initiative_result = initiative_tool.call_tool().await;
    assert!(
        initiative_result.is_ok(),
        "Initiative creation failed: {:?}",
        initiative_result.err()
    );

    // Test creating a task document - this would have caught the wildcard issue
    let task_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Test Task".to_string(),
        parent_title: Some("Test Initiative".to_string()),
        risk_level: None,
        complexity: None,
        decision_maker: None,
        stakeholders: None,
        strategy_id: Some("test-strategy".to_string()),
        initiative_id: Some("test-initiative".to_string()),
    };

    let task_result = task_tool.call_tool().await;
    assert!(
        task_result.is_ok(),
        "Task creation failed: {:?}",
        task_result.err()
    );

    // Verify actual file paths were created correctly (this would have caught the * issue)
    let strategy_path = format!("{}/strategies/test-strategy/strategy.md", metis_path);
    assert!(
        tokio::fs::metadata(&strategy_path).await.is_ok(),
        "Strategy file not created at expected path: {}",
        strategy_path
    );

    let initiative_path = format!("{}/strategies/test-strategy/initiatives/test-initiative/initiative.md", metis_path);
    assert!(
        tokio::fs::metadata(&initiative_path).await.is_ok(),
        "Initiative file not created at expected path: {}",
        initiative_path
    );

    let task_path = format!("{}/strategies/test-strategy/initiatives/test-initiative/tasks/test-task.md", metis_path);
    assert!(
        tokio::fs::metadata(&task_path).await.is_ok(),
        "Task file not created at expected path: {}",
        task_path
    );

    // Verify no wildcard directories were created
    let wildcard_path = format!("{}/strategies/*", metis_path);
    assert!(
        tokio::fs::metadata(&wildcard_path).await.is_err(),
        "Wildcard directory should not exist: {}",
        wildcard_path
    );
}

/// Test that invalid hierarchy parameters are rejected
#[tokio::test]
async fn test_invalid_hierarchy_parameters() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/metis", project_path);

    // Initialize project
    let init_result = initialize_test_project(&project_path, "invalid-hierarchy-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    // Test initiative without strategy_id should fail
    let invalid_initiative = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Invalid Initiative".to_string(),
        parent_title: Some("Some Strategy".to_string()),
        risk_level: None,
        complexity: Some("s".to_string()),
        decision_maker: None,
        stakeholders: None,
        strategy_id: None, // Missing required strategy_id
        initiative_id: None,
    };

    let result = invalid_initiative.call_tool().await;
    assert!(result.is_ok()); // Tool returns Ok but with error message
    
    // Check that the response contains an error about missing strategy_id
    let response = result.unwrap();
    let content = format!("{:?}", response);
    assert!(content.contains("strategy_id"), "Should require strategy_id for initiative");

    // Test task without initiative_id should fail
    let invalid_task = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Invalid Task".to_string(),
        parent_title: Some("Some Initiative".to_string()),
        risk_level: None,
        complexity: None,
        decision_maker: None,
        stakeholders: None,
        strategy_id: Some("test-strategy".to_string()),
        initiative_id: None, // Missing required initiative_id
    };

    let result = invalid_task.call_tool().await;
    assert!(result.is_ok()); // Tool returns Ok but with error message
    
    // Check that the response contains an error about missing initiative_id
    let response = result.unwrap();
    let content = format!("{:?}", response);
    assert!(content.contains("initiative_id"), "Should require initiative_id for task");
}

/// Test database sync and foreign key relationships
#[tokio::test]
async fn test_database_sync_and_relationships() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/metis", project_path);

    // Initialize project
    let init_result = initialize_test_project(&project_path, "db-sync-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    // Create a full hierarchy: strategy -> initiative -> task
    let strategy_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Database Test Strategy".to_string(),
        parent_title: Some("metis-vision".to_string()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        decision_maker: None,
        stakeholders: None,
        strategy_id: Some("database-test-strategy".to_string()),
        initiative_id: None,
    };

    let strategy_result = strategy_tool.call_tool().await;
    assert!(strategy_result.is_ok(), "Strategy creation failed");

    let initiative_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Database Test Initiative".to_string(),
        parent_title: Some("Database Test Strategy".to_string()),
        risk_level: None,
        complexity: Some("m".to_string()),
        decision_maker: None,
        stakeholders: None,
        strategy_id: Some("database-test-strategy".to_string()),
        initiative_id: None,
    };

    let initiative_result = initiative_tool.call_tool().await;
    assert!(initiative_result.is_ok(), "Initiative creation failed");

    let task_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Database Test Task".to_string(),
        parent_title: Some("Database Test Initiative".to_string()),
        risk_level: None,
        complexity: None,
        decision_maker: None,
        stakeholders: None,
        strategy_id: Some("database-test-strategy".to_string()),
        initiative_id: Some("database-test-initiative".to_string()),
    };

    let task_result = task_tool.call_tool().await;
    assert!(task_result.is_ok(), "Task creation failed");

    // Verify database was created and contains no foreign key errors
    let db_path = format!("{}/.metis.db", metis_path);
    assert!(
        tokio::fs::metadata(&db_path).await.is_ok(),
        "Database file should exist: {}",
        db_path
    );

    // TODO: Add actual database query tests to verify relationships were created correctly
}

/// Test ADR creation (independent documents)
#[tokio::test]
async fn test_adr_creation() {
    let temp_dir = create_temp_dir();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/metis", project_path);

    // Initialize project
    let init_result = initialize_test_project(&project_path, "adr-test").await;
    assert!(init_result.is_ok(), "Project initialization failed");

    // Test creating an ADR (should not require hierarchy parameters)
    let adr_tool = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "adr".to_string(),
        title: "Use PostgreSQL Database".to_string(),
        parent_title: None, // ADRs are independent
        risk_level: None,
        complexity: None,
        decision_maker: Some("Architecture Team".to_string()),
        stakeholders: None,
        strategy_id: None, // Should not require strategy_id
        initiative_id: None,
    };

    let adr_result = adr_tool.call_tool().await;
    assert!(
        adr_result.is_ok(),
        "ADR creation failed: {:?}",
        adr_result.err()
    );

    // Verify ADR was created in the correct location
    let adr_path = format!("{}/decisions", metis_path);
    assert!(
        tokio::fs::metadata(&adr_path).await.is_ok(),
        "ADR directory should exist: {}",
        adr_path
    );
}
