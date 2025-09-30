use anyhow::Result;

mod common;
use common::TestHelper;

#[tokio::test]
async fn test_debug_backlog_item_loading() -> Result<()> {
    let helper = TestHelper::new().await?;

    // Create a proper backlog task with frontmatter and content
    let task_content = "---
id: debug-test-task
level: task
title: \"Debug Test Task\"
created_at: 2024-01-01T00:00:00Z
updated_at: 2024-01-01T00:00:00Z
parent:
blocked_by: []
archived: false

tags:
  - \"#phase/backlog\"

exit_criteria_met: false
---

# Debug Test Task

## Objective
Test backlog item detection

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

## Acceptance Criteria
- [ ] Bug is identified and reproduced
";

    // Write the task file to root metis directory first to test recognition
    let task_path = helper.metis_dir.join("debug-task.md");
    tokio::fs::write(&task_path, task_content).await?;
    
    println!("Created task file at: {:?}", task_path);
    println!("Task content: {}", task_content);

    // Create app and load documents
    let mut app = helper.create_app();
    
    // Check if document service is available
    println!("Document service available: {}", app.document_service.is_some());
    
    // Sync database first (this is the missing step!)
    if let Some(sync_service) = &app.sync_service {
        match sync_service.sync_database().await {
            Ok(_) => println!("Database synced successfully"),
            Err(e) => println!("Error syncing database: {:?}", e),
        }
    }
    
    // Then load documents 
    match app.load_documents().await {
        Ok(_) => println!("Documents loaded successfully"),
        Err(e) => println!("Error loading documents: {:?}", e),
    }

    // Check all boards for content
    println!("\n=== Board Contents ===");
    
    // Check strategy board
    println!("Strategy board columns: {}", app.ui_state.strategy_board.columns.len());
    for (i, column) in app.ui_state.strategy_board.columns.iter().enumerate() {
        println!("  Column {}: '{}' has {} items", i, column.title, column.items.len());
    }
    
    // Check task board
    println!("Task board columns: {}", app.ui_state.task_board.columns.len());
    for (i, column) in app.ui_state.task_board.columns.iter().enumerate() {
        println!("  Column {}: '{}' has {} items", i, column.title, column.items.len());
    }
    
    // Check backlog board
    println!("Backlog board columns: {}", app.ui_state.backlog_board.columns.len());
    for (i, column) in app.ui_state.backlog_board.columns.iter().enumerate() {
        println!("  Column {}: '{}' has {} items", i, column.title, column.items.len());
        for (j, item) in column.items.iter().enumerate() {
            println!("    Item {}: '{}'", j, item.title());
        }
    }

    // Check what the document service finds
    if let Some(ref doc_service) = app.document_service {
        match doc_service.load_documents_from_database().await {
            Ok(docs) => {
                println!("\n=== Documents found by service ===");
                for doc in docs {
                    println!("Document: '{}' type: {:?} path: {}", doc.title, doc.document_type, doc.filepath);
                }
            }
            Err(e) => println!("Error loading from database: {:?}", e),
        }
    }

    Ok(())
}