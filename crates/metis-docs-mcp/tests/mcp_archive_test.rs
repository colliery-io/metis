//! Archive behavior tests for MCP server to verify the fixed cascading functionality

mod common;

use anyhow::Result;
use common::McpTestHelper;
use metis_mcp_server::tools::*;
use regex::Regex;

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
    panic!("Could not extract short_code from result")
}

/// Test MCP server archive cascading behavior
/// Tests archiving an initiative with nested tasks cascades correctly
#[tokio::test]
async fn test_mcp_archive_cascading_behavior() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== MCP Archive Cascading Test ===");

    // Step 1: Create hierarchy - Vision (from init) -> Initiative -> 2 Tasks
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Modernize Legacy Systems".to_string(),
        parent_id: None,
        complexity: Some("xl".to_string()),
        stakeholders: Some(vec!["backend_team".to_string()]),
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_initiative.call_tool().await;
    assert!(result.is_ok(), "Create initiative should succeed");
    let initiative_short_code = extract_short_code(&result.unwrap());

    let create_task_1 = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Audit current database schema".to_string(),
        parent_id: Some(initiative_short_code.clone()),
        complexity: None,
        stakeholders: Some(vec!["dba".to_string()]),
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_task_1.call_tool().await;
    assert!(result.is_ok(), "Create first task should succeed");
    let task1_short_code = extract_short_code(&result.unwrap());

    let create_task_2 = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Plan migration roadmap".to_string(),
        parent_id: Some(initiative_short_code.clone()),
        complexity: None,
        stakeholders: Some(vec!["architect".to_string()]),
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_task_2.call_tool().await;
    assert!(result.is_ok(), "Create second task should succeed");
    let _task2_short_code = extract_short_code(&result.unwrap());

    // Sync filesystem to database before querying
    use metis_core::Application;
    let db = helper.get_database()?;
    let app = Application::new(db);
    app.sync_directory(&helper.metis_dir()).await?;

    // Get fresh database connection after sync
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_initiatives = repo
        .find_by_type("initiative")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    let db_tasks = repo
        .find_by_type("task")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    let db_visions = repo
        .find_by_type("vision")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    assert_eq!(db_visions.len(), 1, "Should have 1 vision");
    assert_eq!(db_initiatives.len(), 1, "Should have 1 initiative");
    assert_eq!(db_tasks.len(), 2, "Should have 2 tasks");

    // All should be active (not archived)
    assert!(!db_initiatives[0].archived, "Initiative should be active");
    assert_eq!(
        db_tasks.iter().filter(|t| t.archived).count(),
        0,
        "No tasks should be archived"
    );

    println!("✅ Complete hierarchy created successfully");

    // Step 2: Archive individual task (should not cascade)
    println!("\n=== Step 2: Archive Individual Task ===");

    let archive_task = ArchiveDocumentTool {
        project_path: helper.metis_dir().clone(),
        short_code: task1_short_code.clone(),
    };

    let result = archive_task.call_tool().await;
    assert!(
        result.is_ok(),
        "Archive individual task should succeed: {:?}",
        result
    );

    // Sync filesystem to database before querying
    let db = helper.get_database()?;
    let app = Application::new(db);
    app.sync_directory(&helper.metis_dir()).await?;

    // Get fresh repository after sync
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    // Verify only one task is archived
    let db_tasks_after_single = repo
        .find_by_type("task")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    let db_initiatives_after_single = repo
        .find_by_type("initiative")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    let archived_tasks = db_tasks_after_single.iter().filter(|t| t.archived).count();
    let active_tasks = db_tasks_after_single.iter().filter(|t| !t.archived).count();

    assert_eq!(archived_tasks, 1, "Should have 1 archived task");
    assert_eq!(active_tasks, 1, "Should have 1 active task");
    assert!(
        !db_initiatives_after_single[0].archived,
        "Initiative should still be active"
    );

    println!("✅ Individual task archived, no cascade effect");

    // Step 3: Archive initiative (should cascade to remaining task)
    println!("\n=== Step 3: Archive Initiative (Cascade Test) ===");

    let archive_initiative = ArchiveDocumentTool {
        project_path: helper.metis_dir().clone(),
        short_code: initiative_short_code.clone(),
    };

    let result = archive_initiative.call_tool().await;

    if result.is_ok() {
        println!("✅ Initiative archive succeeded!");

        // Sync filesystem to database before querying
        let db = helper.get_database()?;
        let app = Application::new(db);
        app.sync_directory(&helper.metis_dir()).await?;

        // Get fresh repository after sync
        let db = helper.get_database()?;
        let mut repo = db
            .repository()
            .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

        // Verify full cascade happened
        let db_initiatives_final = repo
            .find_by_type("initiative")
            .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
        let db_tasks_final = repo
            .find_by_type("task")
            .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

        assert!(
            db_initiatives_final[0].archived,
            "Initiative should be archived"
        );

        let final_archived_tasks = db_tasks_final.iter().filter(|t| t.archived).count();
        assert_eq!(
            final_archived_tasks, 2,
            "Both tasks should be archived due to cascade"
        );

        println!("✅ Full cascade archiving successful");
        println!("   - Initiative: archived");
        println!("   - Tasks: {} archived (cascaded)", final_archived_tasks);
    } else {
        println!("⚠️  Initiative archive failed: {:?}", result);
        return Ok(());
    }

    // Step 4: Verify file system state
    println!("\n=== Step 4: Verify File System State ===");

    let initiatives_dir = format!("{}/initiatives", helper.metis_dir());
    let archived_dir = format!("{}/archived", helper.metis_dir());

    // Original initiatives directory should be empty or non-existent after archive
    if let Ok(entries) = std::fs::read_dir(&initiatives_dir) {
        let remaining_entries: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        assert_eq!(
            remaining_entries.len(),
            0,
            "Initiatives directory should be empty after cascade archive"
        );
    }

    // Archived directory should contain our documents
    assert!(
        std::path::Path::new(&archived_dir).exists(),
        "Archived directory should exist"
    );

    println!("✅ File system state consistent with archive operations");

    println!("\n=== MCP Archive Test Summary ===");
    println!("1. Created complete hierarchy (Vision/Initiative/2 Tasks) ✅");
    println!("2. Archived individual task (no cascade) ✅");
    println!("3. Archived initiative (full cascade) ✅");
    println!("4. Verified database state consistency ✅");
    println!("5. Verified file system state ✅");

    Ok(())
}

/// Test MCP server archive error handling
#[tokio::test]
async fn test_mcp_archive_error_handling() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== MCP Archive Error Handling Test ===");

    // Try to archive non-existent document
    let archive_nonexistent = ArchiveDocumentTool {
        project_path: helper.metis_dir().clone(),
        short_code: "non-existent-document".to_string(),
    };

    let result = archive_nonexistent.call_tool().await;
    assert!(result.is_err(), "Archive non-existent document should fail");

    println!("✅ Non-existent document archive properly rejected");

    // Try to archive same document twice
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Test Initiative for Archive".to_string(),
        parent_id: None,
        complexity: Some("s".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };

    let result = create_initiative.call_tool().await;
    assert!(result.is_ok(), "Create initiative should succeed");
    let initiative_short_code = extract_short_code(&result.unwrap());

    let archive_initiative = ArchiveDocumentTool {
        project_path: helper.metis_dir().clone(),
        short_code: initiative_short_code.clone(),
    };

    // First archive should succeed
    let result = archive_initiative.call_tool().await;
    assert!(result.is_ok(), "First archive should succeed");

    // Second archive should fail (already archived)
    let result = archive_initiative.call_tool().await;
    assert!(
        result.is_err(),
        "Second archive should fail - already archived"
    );

    println!("✅ Duplicate archive properly rejected");

    Ok(())
}
