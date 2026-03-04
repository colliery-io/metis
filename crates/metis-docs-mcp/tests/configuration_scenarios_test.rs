//! Comprehensive tests for MCP server behavior across different flight level configurations
//! These tests validate real-world usage scenarios for each configuration preset

use crate::common::McpTestHelper;
use anyhow::Result;
use metis_core::domain::configuration::FlightLevelConfig;
use metis_mcp_server::tools::*;
use regex::Regex;

mod common;

/// Helper to extract text content from MCP response (handles EmbeddedResource)
fn extract_text_from_result(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String> {
    match result.content.first() {
        Some(rust_mcp_sdk::schema::ContentBlock::TextContent(text_content)) => {
            Some(text_content.text.clone())
        }
        Some(rust_mcp_sdk::schema::ContentBlock::EmbeddedResource(embedded)) => {
            match &embedded.resource {
                rust_mcp_sdk::schema::EmbeddedResourceResource::TextResourceContents(text_resource) => {
                    Some(text_resource.text.clone())
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Helper to extract short code from MCP response (parses markdown format)
fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    if let Some(text) = extract_text_from_result(result) {
        // Match pattern like "PROJ-X-0001" (any document type: V, I, T, A)
        let re = Regex::new(r"([A-Z]+-[VITA]-\d{4})").unwrap();
        if let Some(captures) = re.captures(&text) {
            if let Some(m) = captures.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    panic!("Failed to extract short_code from MCP response");
}

/// Test MCP server behavior with default streamlined configuration
/// Streamlined: initiatives enabled (Vision → Initiative → Task)
#[tokio::test]
async fn test_streamlined_configuration_workflows() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    // Verify default is streamlined (no need to set config)
    let db = helper.get_database()?;
    let mut config_repo = db
        .configuration_repository()
        .map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    let flight_config = config_repo
        .get_flight_level_config()
        .map_err(|e| anyhow::anyhow!("Failed to get config: {}", e))?;
    assert!(
        flight_config.initiatives_enabled,
        "Initiatives should be enabled in streamlined"
    );

    println!("=== Test Streamlined Configuration (Default) ===");

    // 1. Initiative creation should succeed WITHOUT parent_id
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None, // No parent provided
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let initiative_result = create_initiative.call_tool().await;
    assert!(
        initiative_result.is_ok(),
        "Initiative creation should succeed in streamlined mode without parent: {:?}",
        initiative_result
    );
    let initiative_short_code = extract_short_code(&initiative_result.unwrap());

    // 2. Task creation with initiative parent should succeed
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Test Task".to_string(),
        parent_id: Some(initiative_short_code), // Reference the initiative by short code
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let task_result = create_task.call_tool().await;
    assert!(
        task_result.is_ok(),
        "Task creation with initiative parent should succeed: {:?}",
        task_result
    );

    // 3. Task creation without parent should fail (initiatives are enabled)
    let create_orphan_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Orphan Task".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let orphan_result = create_orphan_task.call_tool().await;
    assert!(
        orphan_result.is_err(),
        "Task without parent should fail when initiatives enabled"
    );
    let error_msg = format!("{:?}", orphan_result.unwrap_err());
    assert!(
        error_msg.contains("requires a parent initiative"),
        "Should mention initiative requirement"
    );

    println!("✅ Streamlined configuration workflows validated");
    Ok(())
}

/// Test MCP server behavior with direct configuration
/// Direct: initiatives disabled (Vision → Task only)
#[tokio::test]
async fn test_direct_configuration_workflows() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    // Set direct configuration via config.toml (filesystem is source of truth)
    helper.set_flight_level_config(FlightLevelConfig::direct())?;

    println!("=== Test Direct Configuration ===");

    // 1. Initiative creation should fail
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let initiative_result = create_initiative.call_tool().await;
    assert!(
        initiative_result.is_err(),
        "Initiative creation should fail in direct mode"
    );

    // 2. Task creation without parent should succeed (direct vision→task)
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Direct Task".to_string(),
        parent_id: None, // No parent - direct mode
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let task_result = create_task.call_tool().await;
    assert!(
        task_result.is_ok(),
        "Task creation without parent should succeed in direct mode: {:?}",
        task_result
    );

    // 3. Vision and ADR should always work
    let create_adr = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "adr".to_string(),
        title: "Test ADR".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("architect".to_string()),
        backlog_category: None,
    };

    let adr_result = create_adr.call_tool().await;
    assert!(
        adr_result.is_ok(),
        "ADR creation should always succeed: {:?}",
        adr_result
    );

    println!("✅ Direct configuration workflows validated");
    Ok(())
}

/// Test configuration error messages provide actionable guidance
#[tokio::test]
async fn test_configuration_error_messages() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    // Set direct mode to test initiative-disabled error messages
    helper.set_flight_level_config(FlightLevelConfig::direct())?;

    println!("=== Test Configuration Error Messages ===");

    // Test direct mode error messages - initiative creation should fail
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_initiative.call_tool().await;
    assert!(result.is_err());
    let error_msg = format!("{:?}", result.unwrap_err());

    // Verify error message contains expected elements
    assert!(
        error_msg.contains("creation is disabled"),
        "Should explain what's disabled"
    );
    assert!(
        error_msg.contains("direct mode"),
        "Should identify current mode"
    );
    assert!(
        error_msg.contains("Available document types"),
        "Should list available types"
    );
    assert!(
        error_msg.contains("vision"),
        "Should include vision in available types"
    );
    assert!(
        error_msg.contains("task"),
        "Should include task in available types"
    );
    assert!(
        error_msg.contains("adr"),
        "Should include adr in available types"
    );

    println!("✅ Error messages provide comprehensive guidance");
    Ok(())
}

/// Test configuration switching doesn't break existing documents
#[tokio::test]
async fn test_configuration_switching_compatibility() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== Test Configuration Switching ===");

    // Start with streamlined configuration (default)
    // Create an initiative
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: None,
        complexity: Some("s".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    create_initiative
        .call_tool()
        .await
        .map_err(|e| anyhow::anyhow!("Initiative creation failed: {:?}", e))?;

    // Switch to direct configuration via config.toml
    helper.set_flight_level_config(FlightLevelConfig::direct())?;

    // Should be able to create tasks without parents in direct mode
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Direct Task".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_task.call_tool().await;
    assert!(
        result.is_ok(),
        "Should be able to create tasks after switching to direct: {:?}",
        result
    );

    // Should not be able to create new initiatives
    let create_new_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "New Initiative".to_string(),
        parent_id: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let initiative_result = create_new_initiative.call_tool().await;
    assert!(
        initiative_result.is_err(),
        "Should not be able to create initiatives in direct mode"
    );

    println!("✅ Configuration switching maintains compatibility");
    Ok(())
}
