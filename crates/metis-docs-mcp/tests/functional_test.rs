//! Clean functional tests for MCP tools using short codes

use metis_mcp_server::tools::*;
use regex::Regex;
use tempfile::tempdir;

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
        // Match pattern like "PROJ-X-0001" (any document type: V, S, I, T, A)
        let re = Regex::new(r"([A-Z]+-[VSITA]-\d{4})").unwrap();
        if let Some(captures) = re.captures(&text) {
            if let Some(m) = captures.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    panic!("Could not extract short_code from result")
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
        let re = Regex::new(r"\|\s*vision\s*\|\s*([A-Z]+-V-\d{4})\s*\|").unwrap();
        if let Some(captures) = re.captures(&text) {
            if let Some(m) = captures.get(1) {
                return m.as_str().to_string();
            }
        }
    }
    panic!("Could not find vision document")
}

#[tokio::test]
async fn test_initialize_and_create_documents() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // 1. Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: None,
    };
    let result = init_tool.call_tool().await;
    assert!(result.is_ok(), "Initialize should succeed");

    // 2. Enable all document types by updating config.toml file
    let config_path = format!("{}/config.toml", metis_path);
    let config_content = r#"
[project]
name = "Test Project"
prefix = "PROJ"

[flight_levels]
strategies_enabled = true
initiatives_enabled = true
"#;
    std::fs::write(&config_path, config_content).unwrap();

    // 3. Get vision short code
    let vision_short_code = get_vision_short_code(&metis_path).await;
    println!("Vision short code: {}", vision_short_code);

    // 4. Create strategy using vision short code as parent
    let create_strategy = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Test Strategy".to_string(),
        parent_id: Some(vision_short_code),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["dev_team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(result.is_ok(), "Create strategy should succeed");
    let strategy_short_code = extract_short_code(&result.unwrap());
    println!("Strategy short code: {}", strategy_short_code);

    // 5. Create initiative using strategy short code as parent
    let create_initiative = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative".to_string(),
        parent_id: Some(strategy_short_code.clone()),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: Some(vec!["product_team".to_string()]),
        decision_maker: None,
    };

    let result = create_initiative.call_tool().await;
    if let Err(ref e) = result {
        println!("Initiative creation error: {:?}", e);
    }
    assert!(result.is_ok(), "Create initiative should succeed");
    let initiative_short_code = extract_short_code(&result.unwrap());
    println!("Initiative short code: {}", initiative_short_code);

    // 6. Test read document with short code
    let read_tool = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: strategy_short_code.clone(),
    };

    let result = read_tool.call_tool().await;
    assert!(result.is_ok(), "Read document should succeed");

    // Let's see what the strategy content looks like
    if let Ok(ref read_result) = result {
        if let Some(rust_mcp_sdk::schema::ContentBlock::TextContent(text_content)) =
            read_result.content.first()
        {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text_content.text) {
                if let Some(content) = json["content"].as_str() {
                    println!("Strategy content:\n{}", content);
                }
            }
        }
    }

    // 7. Test edit document with short code
    let edit_tool = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: strategy_short_code.clone(),
        search: "{Describe the problem and why it matters - 1-2 paragraphs}".to_string(),
        replace:
            "This strategy addresses the need for better short code interfaces in our MCP server."
                .to_string(),
        replace_all: None,
    };

    let result = edit_tool.call_tool().await;
    if let Err(ref e) = result {
        println!("Edit document error: {:?}", e);
    }
    assert!(result.is_ok(), "Edit document should succeed");

    // 8. Test phase transition with short code (from shaping to design)
    let transition_tool = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: strategy_short_code,
        phase: Some("design".to_string()),
        force: None,
    };

    let result = transition_tool.call_tool().await;
    if let Err(ref e) = result {
        println!("Phase transition error: {:?}", e);
    }
    assert!(result.is_ok(), "Phase transition should succeed");
}

#[tokio::test]
async fn test_archive_with_short_codes() {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // Initialize and set up
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: None,
    };
    init_tool.call_tool().await.unwrap();

    // Enable full configuration to allow strategies by updating config.toml
    let config_path = format!("{}/config.toml", metis_path);
    let config_content = r#"
[project]
name = "Test Project"
prefix = "PROJ"

[flight_levels]
strategies_enabled = true
initiatives_enabled = true
"#;
    std::fs::write(&config_path, config_content).unwrap();

    let vision_short_code = get_vision_short_code(&metis_path).await;

    // Create strategy
    let create_strategy = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Archive Test Strategy".to_string(),
        parent_id: Some(vision_short_code),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await.unwrap();
    let strategy_short_code = extract_short_code(&result);

    // Archive using short code
    let archive_tool = ArchiveDocumentTool {
        project_path: metis_path.clone(),
        short_code: strategy_short_code,
    };

    let result = archive_tool.call_tool().await;
    assert!(result.is_ok(), "Archive should succeed");
}

#[tokio::test]
async fn test_search_with_short_code_hyphen() {
    // This tests the FTS special character fix - hyphens in queries like "PROJ-I-0001"
    // were being interpreted as the NOT operator causing "no such column: I" errors
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: None,
    };
    init_tool.call_tool().await.unwrap();

    // Use streamlined configuration - initiative links directly to vision
    let config_path = format!("{}/config.toml", metis_path);
    let config_content = r#"
[project]
name = "Test Project"
prefix = "PROJ"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true
"#;
    std::fs::write(&config_path, config_content).unwrap();

    // Get vision and create an initiative (streamlined: vision -> initiative)
    let vision_short_code = get_vision_short_code(&metis_path).await;

    let create_initiative = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Search Test Initiative".to_string(),
        parent_id: Some(vision_short_code),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
    };

    let result = create_initiative.call_tool().await;
    if let Err(ref e) = result {
        println!("Initiative creation error: {:?}", e);
    }
    assert!(result.is_ok(), "Create initiative should succeed");
    let initiative_short_code = extract_short_code(&result.unwrap());
    println!("Created initiative: {}", initiative_short_code);

    // Now search using the short code with hyphens
    let search_tool = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: initiative_short_code.clone(),
        document_type: None,
        limit: None,
        include_archived: None,
    };

    let result = search_tool.call_tool().await;
    assert!(result.is_ok(), "Search with hyphenated short code should succeed (not fail with 'no such column' error)");

    if let Ok(ref search_result) = result {
        if let Some(text) = extract_text_from_result(search_result) {
            println!("Search result: {}", text);
            // Should find our initiative
            assert!(text.contains(&initiative_short_code), "Search should find the initiative by short code");
        }
    }
}

#[tokio::test]
async fn test_list_and_search_include_archived() {
    // Test that include_archived=true shows archived documents
    // and include_archived=false (default) hides them
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // Initialize project
    let init_tool = InitializeProjectTool {
        project_path: project_path.clone(),
        prefix: None,
    };
    init_tool.call_tool().await.unwrap();

    // Enable full configuration
    let config_path = format!("{}/config.toml", metis_path);
    let config_content = r#"
[project]
name = "Test Project"
prefix = "PROJ"

[flight_levels]
strategies_enabled = true
initiatives_enabled = true
"#;
    std::fs::write(&config_path, config_content).unwrap();

    // Get vision and create a strategy
    let vision_short_code = get_vision_short_code(&metis_path).await;

    let create_strategy = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "strategy".to_string(),
        title: "Archivable Strategy".to_string(),
        parent_id: Some(vision_short_code),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await.unwrap();
    let strategy_short_code = extract_short_code(&result);
    println!("Created strategy: {}", strategy_short_code);

    // Verify strategy appears in list (default = unarchived only)
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    };
    let result = list_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(text.contains(&strategy_short_code), "Strategy should appear in default list");

    // Verify strategy appears in search
    let search_tool = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: "Archivable".to_string(),
        document_type: None,
        limit: None,
        include_archived: None,
    };
    let result = search_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(text.contains(&strategy_short_code), "Strategy should appear in default search");

    // Archive the strategy
    let archive_tool = ArchiveDocumentTool {
        project_path: metis_path.clone(),
        short_code: strategy_short_code.clone(),
    };
    archive_tool.call_tool().await.unwrap();

    // Verify strategy no longer appears in default list (unarchived only)
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    };
    let result = list_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(!text.contains(&strategy_short_code), "Archived strategy should NOT appear in default list");

    // Verify strategy DOES appear when include_archived=true
    let list_tool = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: Some(true),
    };
    let result = list_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(text.contains(&strategy_short_code), "Archived strategy SHOULD appear when include_archived=true");

    // Verify strategy no longer appears in default search (unarchived only)
    let search_tool = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: "Archivable".to_string(),
        document_type: None,
        limit: None,
        include_archived: None,
    };
    let result = search_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(!text.contains(&strategy_short_code), "Archived strategy should NOT appear in default search");

    // Verify strategy DOES appear in search when include_archived=true
    let search_tool = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: "Archivable".to_string(),
        document_type: None,
        limit: None,
        include_archived: Some(true),
    };
    let result = search_tool.call_tool().await.unwrap();
    let text = extract_text_from_result(&result).unwrap();
    assert!(text.contains(&strategy_short_code), "Archived strategy SHOULD appear in search when include_archived=true");
}
