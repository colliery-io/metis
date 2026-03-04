//! Comprehensive functional tests for MCP server covering all configurations and workflows
//! These tests mirror real user workflows through MCP tool calls

use metis_core::domain::configuration::FlightLevelConfig;
use metis_core::Database;
use metis_mcp_server::tools::*;
use tempfile::TempDir;

/// Helper to setup project with specific flight configuration
async fn setup_project_with_config(config: FlightLevelConfig) -> (TempDir, String, String) {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: None,
    };
    let result = init_tool.call_tool().await;
    assert!(result.is_ok(), "Project initialization should succeed");

    // Set flight configuration in both DB and config.toml (required for new sync behavior)
    let db_path = format!("{}/.metis/metis.db", project_path);
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();

    // Get current prefix from DB
    let prefix = config_repo.get_project_prefix().unwrap().unwrap_or_else(|| "PROJ".to_string());

    // Update DB
    config_repo.set_flight_level_config(&config).unwrap();

    // Update config.toml to match
    use metis_core::domain::configuration::ConfigFile;
    let config_file = ConfigFile::new(prefix, config).unwrap();
    let config_file_path = format!("{}/.metis/config.toml", project_path);
    config_file.save(&config_file_path).unwrap();

    (temp_dir, project_path, metis_path)
}

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

/// Helper to get vision short code from list results (parses markdown table format)
async fn get_vision_short_code(metis_path: &str) -> String {
    let list_tool = ListDocumentsTool {
        project_path: metis_path.to_string(),
        include_archived: None,
    };
    let result = list_tool.call_tool().await.unwrap();

    if let Some(text) = extract_text_from_result(&result) {
        // Match pattern for vision row in unified table: "| vision | PROJ-V-0001 | ..."
        let re = regex::Regex::new(r"\|\s*vision\s*\|\s*([A-Z]+-V-\d{4})\s*\|").unwrap();
        if let Some(captures) = re.captures(&text) {
            if let Some(m) = captures.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    panic!("Could not find vision document")
}

/// Helper to extract short code from MCP response (parses markdown format)
fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    if let Some(text) = extract_text_from_result(result) {
        // Match pattern like "PROJ-X-0001" (any document type)
        let re = regex::Regex::new(r"([A-Z]+-[VITA]-\d{4})").unwrap();
        if let Some(captures) = re.captures(&text) {
            if let Some(m) = captures.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    panic!("Could not extract short_code from result")
}

#[tokio::test]
async fn test_streamlined_configuration_workflow() {
    println!("=== Testing Streamlined Configuration Workflow ===");

    let (_temp_dir, _project_path, metis_path) =
        setup_project_with_config(FlightLevelConfig::streamlined()).await;

    // Step 1: Init (done in setup)
    println!("✅ Init complete");

    // Step 2: Get actual vision short code and edit vision doc
    let vision_short_code = get_vision_short_code(&metis_path).await;
    let edit_vision = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_short_code.clone(),
        search: "{Why this vision exists and what it aims to achieve}".to_string(),
        replace: "Build efficient mobile apps with streamlined workflow".to_string(),
        replace_all: None,
    };
    let result = edit_vision.call_tool().await;
    assert!(result.is_ok(), "Edit vision should succeed");
    println!("✅ Vision doc edited");

    // Step 3: Create initiative linked to vision
    let create_initiative = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Mobile App Performance Improvements".to_string(),
        parent_id: Some(vision_short_code),
        complexity: Some("m".to_string()),
        stakeholders: Some(vec!["mobile_team".to_string()]),
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_initiative.call_tool().await;
    assert!(result.is_ok(), "Create initiative should succeed");
    let initiative_short_code = extract_short_code(&result.unwrap());
    println!("✅ Initiative created: {}", initiative_short_code);

    // Step 4: Move initiative
    let transition_initiative = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: initiative_short_code.clone(),
        phase: Some("design".to_string()),
        force: None,
    };
    let result = transition_initiative.call_tool().await;
    assert!(result.is_ok(), "Initiative phase transition should succeed");
    println!("✅ Initiative moved to design phase");

    // Step 5: Create task linked to initiative
    let create_task = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Optimize image loading".to_string(),
        parent_id: Some(initiative_short_code),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_task.call_tool().await;
    assert!(result.is_ok(), "Create task should succeed");
    let task_short_code = extract_short_code(&result.unwrap());
    println!("✅ Task created: {}", task_short_code);

    // Step 6: Move task
    let transition_task = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: task_short_code,
        phase: None, // Auto-transition to next phase
        force: None,
    };
    let result = transition_task.call_tool().await;
    assert!(result.is_ok(), "Task phase transition should succeed");
    println!("✅ Task moved to active phase");

    // Step 7: Create ADR
    let create_adr = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "adr".to_string(),
        title: "Use WebP format for images".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("mobile_lead".to_string()),
        backlog_category: None,
    };
    let result = create_adr.call_tool().await;
    assert!(result.is_ok(), "Create ADR should succeed");
    let adr_short_code = extract_short_code(&result.unwrap());
    println!("✅ ADR created: {}", adr_short_code);

    // Step 8: Move ADR
    let transition_adr = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: adr_short_code,
        phase: Some("discussion".to_string()),
        force: None,
    };
    let result = transition_adr.call_tool().await;
    assert!(result.is_ok(), "ADR phase transition should succeed");
    println!("✅ ADR moved to discussion phase");

    // Final verification
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    };
    let final_list = list_tool.call_tool().await;
    assert!(final_list.is_ok(), "Final document listing should succeed");

    println!("✅ Streamlined configuration workflow complete!");
}

#[tokio::test]
async fn test_direct_configuration_workflow() {
    println!("=== Testing Direct Configuration Workflow ===");

    let (_temp_dir, _project_path, metis_path) =
        setup_project_with_config(FlightLevelConfig::direct()).await;

    // Step 1: Init (done in setup)
    println!("✅ Init complete");

    // Step 2: Get actual vision short code and edit vision doc
    let vision_short_code = get_vision_short_code(&metis_path).await;
    let edit_vision = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_short_code.clone(),
        search: "{Why this vision exists and what it aims to achieve}".to_string(),
        replace: "Simple task management for direct execution".to_string(),
        replace_all: None,
    };
    let result = edit_vision.call_tool().await;
    assert!(result.is_ok(), "Edit vision should succeed");
    println!("✅ Vision doc edited");

    // Step 3: Create task (in direct config, no initiatives needed)
    let create_task1 = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Fix login bug".to_string(),
        parent_id: None, // No parent in direct mode
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_task1.call_tool().await;
    assert!(result.is_ok(), "Create task should succeed");
    let task1_short_code = extract_short_code(&result.unwrap());
    println!("✅ Task 1 created: {}", task1_short_code);

    // Step 4: Move task
    let transition_task1 = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: task1_short_code,
        phase: None, // Auto-transition to next phase
        force: None,
    };
    let result = transition_task1.call_tool().await;
    assert!(result.is_ok(), "Task 1 phase transition should succeed");
    println!("✅ Task 1 moved to active phase");

    // Step 5: Create another task
    let create_task2 = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Update documentation".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_task2.call_tool().await;
    assert!(result.is_ok(), "Create second task should succeed");
    let task2_short_code = extract_short_code(&result.unwrap());
    println!("✅ Task 2 created: {}", task2_short_code);

    // Step 6: Move second task
    let transition_task2 = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: task2_short_code,
        phase: None, // Auto-transition to next phase
        force: None,
    };
    let result = transition_task2.call_tool().await;
    assert!(result.is_ok(), "Task 2 phase transition should succeed");
    println!("✅ Task 2 moved to active phase");

    // Step 7: Create ADR
    let create_adr = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "adr".to_string(),
        title: "Use SQLite for local storage".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("developer".to_string()),
        backlog_category: None,
    };
    let result = create_adr.call_tool().await;
    assert!(result.is_ok(), "Create ADR should succeed");
    let adr_short_code = extract_short_code(&result.unwrap());
    println!("✅ ADR created: {}", adr_short_code);

    // Step 8: Move ADR
    let transition_adr = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: adr_short_code,
        phase: Some("discussion".to_string()),
        force: None,
    };
    let result = transition_adr.call_tool().await;
    assert!(result.is_ok(), "ADR phase transition should succeed");
    println!("✅ ADR moved to discussion phase");

    // Final verification
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    };
    let final_list = list_tool.call_tool().await;
    assert!(final_list.is_ok(), "Final document listing should succeed");

    println!("✅ Direct configuration workflow complete!");
}
