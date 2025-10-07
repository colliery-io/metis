//! Comprehensive tests for MCP server behavior across different flight level configurations
//! These tests validate real-world usage scenarios for each configuration preset

use anyhow::Result;
use metis_mcp_server::tools::*;
use crate::common::McpTestHelper;

mod common;

/// Helper to extract short code from MCP response JSON
fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    if let Some(rust_mcp_sdk::schema::ContentBlock::TextContent(text_content)) = result.content.first() {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
            if let Some(short_code) = json.get("short_code").and_then(|v| v.as_str()) {
                return short_code.to_string();
            }
        }
    }
    panic!("Failed to extract short_code from MCP response");
}

/// Test MCP server behavior with default streamlined configuration
/// Streamlined: strategies disabled, initiatives enabled (Vision → Initiative → Task)
#[tokio::test]
async fn test_streamlined_configuration_workflows() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    // Verify default is streamlined (no need to set config)
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    let flight_config = config_repo.get_flight_level_config().map_err(|e| anyhow::anyhow!("Failed to get config: {}", e))?;
    assert!(!flight_config.strategies_enabled, "Should be streamlined by default");
    assert!(flight_config.initiatives_enabled, "Initiatives should be enabled in streamlined");
    
    println!("=== Test Streamlined Configuration (Default) ===");

    // 1. Strategy creation should fail
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let strategy_result = create_strategy.call_tool().await;
    assert!(strategy_result.is_err(), "Strategy creation should fail in streamlined mode");
    let error_msg = format!("{:?}", strategy_result.unwrap_err());
    assert!(error_msg.contains("strategy creation is disabled"), "Should mention strategy is disabled");
    assert!(error_msg.contains("streamlined mode"), "Should mention current mode");
    assert!(error_msg.contains("Available document types"), "Should list available types");

    // 2. Initiative creation should succeed WITHOUT parent_id
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None, // No parent provided - should use NULL strategy
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
    };
    
    let initiative_result = create_initiative.call_tool().await;
    assert!(initiative_result.is_ok(), "Initiative creation should succeed in streamlined mode without parent: {:?}", initiative_result);
    let initiative_short_code = extract_short_code(&initiative_result.unwrap());

    // 3. Task creation with initiative parent should succeed
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Test Task".to_string(),
        parent_id: Some(initiative_short_code), // Reference the initiative by short code
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let task_result = create_task.call_tool().await;
    assert!(task_result.is_ok(), "Task creation with initiative parent should succeed: {:?}", task_result);

    // 4. Task creation without parent should fail (initiatives are enabled)
    let create_orphan_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Orphan Task".to_string(),
        parent_id: None,
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let orphan_result = create_orphan_task.call_tool().await;
    assert!(orphan_result.is_err(), "Task without parent should fail when initiatives enabled");
    let error_msg = format!("{:?}", orphan_result.unwrap_err());
    assert!(error_msg.contains("requires a parent initiative"), "Should mention initiative requirement");

    println!("✅ Streamlined configuration workflows validated");
    Ok(())
}

/// Test MCP server behavior with direct configuration
/// Direct: both strategies and initiatives disabled (Vision → Task only)
#[tokio::test]
async fn test_direct_configuration_workflows() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    // Set direct configuration
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    config_repo.set("flight_levels", r#"{"strategies_enabled":false,"initiatives_enabled":false}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    println!("=== Test Direct Configuration ===");

    // 1. Strategy creation should fail
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: None,
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let strategy_result = create_strategy.call_tool().await;
    assert!(strategy_result.is_err(), "Strategy creation should fail in direct mode");

    // 2. Initiative creation should fail
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None,
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
    };
    
    let initiative_result = create_initiative.call_tool().await;
    assert!(initiative_result.is_err(), "Initiative creation should fail in direct mode");

    // 3. Task creation without parent should succeed (direct vision→task)
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Direct Task".to_string(),
        parent_id: None, // No parent - should use NULL for both strategy and initiative
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let task_result = create_task.call_tool().await;
    assert!(task_result.is_ok(), "Task creation without parent should succeed in direct mode: {:?}", task_result);

    // 4. Vision and ADR should always work
    let create_adr = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "adr".to_string(),
        title: "Test ADR".to_string(),
        parent_id: None,
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("architect".to_string()),
    };
    
    let adr_result = create_adr.call_tool().await;
    assert!(adr_result.is_ok(), "ADR creation should always succeed: {:?}", adr_result);

    println!("✅ Direct configuration workflows validated");
    Ok(())
}

/// Test MCP server behavior with full configuration
/// Full: all document types enabled with proper hierarchy enforcement
#[tokio::test]
async fn test_full_configuration_workflows() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    // Set full configuration
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    config_repo.set("flight_levels", r#"{"strategies_enabled":true,"initiatives_enabled":true}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    println!("=== Test Full Configuration ===");

    // 1. Create strategy (should succeed)
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let strategy_result = create_strategy.call_tool().await;
    assert!(strategy_result.is_ok(), "Strategy creation should succeed in full mode: {:?}", strategy_result);
    let strategy_short_code = extract_short_code(&strategy_result.unwrap());

    // 2. Initiative creation should require strategy parent
    let create_initiative_no_parent = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "No Parent Initiative".to_string(),
        parent_id: None, // No parent provided
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
    };
    
    let no_parent_result = create_initiative_no_parent.call_tool().await;
    assert!(no_parent_result.is_err(), "Initiative without parent should fail in full mode");
    let error_msg = format!("{:?}", no_parent_result.unwrap_err());
    assert!(error_msg.contains("requires a parent strategy") || error_msg.contains("parent strategy ID"), "Should mention strategy requirement, got: {}", error_msg);

    // 3. Initiative with strategy parent should succeed
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: Some(strategy_short_code),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
    };
    
    let initiative_result = create_initiative.call_tool().await;
    assert!(initiative_result.is_ok(), "Initiative with strategy parent should succeed: {:?}", initiative_result);
    let initiative_short_code = extract_short_code(&initiative_result.unwrap());

    // 4. Task creation should require initiative parent
    let create_task_no_parent = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "No Parent Task".to_string(),
        parent_id: None,
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let no_parent_task_result = create_task_no_parent.call_tool().await;
    assert!(no_parent_task_result.is_err(), "Task without parent should fail in full mode");

    // 5. Task with initiative parent should succeed
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Test Task".to_string(),
        parent_id: Some(initiative_short_code),
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let task_result = create_task.call_tool().await;
    assert!(task_result.is_ok(), "Task with initiative parent should succeed: {:?}", task_result);

    println!("✅ Full configuration workflows validated");
    Ok(())
}

/// Test configuration error messages provide actionable guidance
#[tokio::test]
async fn test_configuration_error_messages() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    println!("=== Test Configuration Error Messages ===");

    // Test streamlined mode error messages (default)
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: None,
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let result = create_strategy.call_tool().await;
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());
    
    // Verify error message contains all expected elements
    assert!(error_msg.contains("strategy creation is disabled"), "Should explain what's disabled");
    assert!(error_msg.contains("streamlined mode"), "Should identify current mode");
    assert!(error_msg.contains("Available document types:"), "Should list available types");
    assert!(error_msg.contains("vision"), "Should include vision in available types");
    assert!(error_msg.contains("initiative"), "Should include initiative in available types");
    assert!(error_msg.contains("task"), "Should include task in available types");
    assert!(error_msg.contains("adr"), "Should include adr in available types");
    assert!(error_msg.contains("metis config set --preset full"), "Should provide remediation");
    assert!(error_msg.contains("--strategies true --initiatives true"), "Should provide alternative remediation");

    println!("✅ Error messages provide comprehensive guidance");
    Ok(())
}

/// Test configuration switching doesn't break existing documents
#[tokio::test]
async fn test_configuration_switching_compatibility() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    println!("=== Test Configuration Switching ===");

    // Start with full configuration and create documents
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    config_repo.set("flight_levels", r#"{"strategies_enabled":true,"initiatives_enabled":true}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    // Create full hierarchy
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    create_strategy.call_tool().await.map_err(|e| anyhow::anyhow!("Strategy creation failed: {:?}", e))?;

    // Switch to streamlined configuration
    config_repo.set("flight_levels", r#"{"strategies_enabled":false,"initiatives_enabled":true}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    // Should still be able to create initiatives (now they go under NULL strategy)
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Streamlined Initiative".to_string(),
        parent_id: None, // No parent in streamlined mode
        risk_level: None,
        complexity: Some("s".to_string()),
        stakeholders: None,
        decision_maker: None,
    };
    
    let result = create_initiative.call_tool().await;
    assert!(result.is_ok(), "Should be able to create initiatives after switching to streamlined: {:?}", result);

    // Should not be able to create new strategies
    let create_new_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "New Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };
    
    let strategy_result = create_new_strategy.call_tool().await;
    assert!(strategy_result.is_err(), "Should not be able to create strategies in streamlined mode");

    println!("✅ Configuration switching maintains compatibility");
    Ok(())
}