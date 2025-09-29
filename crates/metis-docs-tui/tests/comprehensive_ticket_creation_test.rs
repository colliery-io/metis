use anyhow::Result;
use metis_docs_tui::models::{BoardType, AppState};
use tui_input::backend::crossterm::EventHandler;

mod common;
use common::TestHelper;

/// Helper function to input a title to the TUI
fn input_title(app: &mut metis_docs_tui::app::App, title: &str) {
    for ch in title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
}

#[tokio::test]
async fn test_create_strategy_ticket() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Start on strategy board
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Start document creation
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Input title
    input_title(&mut app, "Test Strategy");

    // Create the strategy
    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the strategy was created
    let strategy_board = &app.ui_state.strategy_board;
    let total_items: usize = strategy_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 1, "Should have exactly 1 strategy");

    // Find the created strategy
    let mut found_strategy = false;
    for column in &strategy_board.columns {
        for item in &column.items {
            if item.title() == "Test Strategy" {
                found_strategy = true;
                println!("Found strategy '{}' in column '{}'", item.title(), column.title);
                break;
            }
        }
    }
    assert!(found_strategy, "Should find the created strategy in one of the columns");

    Ok(())
}

#[tokio::test]
async fn test_create_initiative_ticket() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // First create a strategy to be the parent
    app.jump_to_strategy_board();
    app.start_document_creation();
    input_title(&mut app, "Parent Strategy");
    app.create_new_document().await?;

    // Switch to initiative board
    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    // Start document creation
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Input title
    input_title(&mut app, "Test Initiative");

    // Create the initiative
    println!("=== Creating Initiative ===");
    match app.create_new_document().await {
        Ok(_) => println!("Initiative creation returned Ok"),
        Err(e) => {
            println!("Initiative creation failed: {:?}", e);
            return Err(e);
        }
    }
    assert_eq!(*app.app_state(), AppState::Normal);

    // Check database after creation
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        println!("Documents in database after initiative creation: {}", docs.len());
        for doc in &docs {
            println!("  - '{}' (type: {:?}, path: {})", doc.title, doc.document_type, doc.filepath);
        }
    }

    // Verify the initiative was created
    let initiative_board = &app.ui_state.initiative_board;
    let total_items: usize = initiative_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 1, "Should have exactly 1 initiative");

    // Find the created initiative
    let mut found_initiative = false;
    for column in &initiative_board.columns {
        for item in &column.items {
            if item.title() == "Test Initiative" {
                found_initiative = true;
                println!("Found initiative '{}' in column '{}'", item.title(), column.title);
                break;
            }
        }
    }
    assert!(found_initiative, "Should find the created initiative in one of the columns");

    Ok(())
}

#[tokio::test]
async fn test_create_task_ticket() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // First create a strategy and initiative to be parents
    app.jump_to_strategy_board();
    app.start_document_creation();
    input_title(&mut app, "Parent Strategy");
    app.create_new_document().await?;

    app.jump_to_initiative_board();
    app.start_document_creation();
    input_title(&mut app, "Parent Initiative");
    app.create_new_document().await?;

    // Switch to task board
    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    // Start document creation
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Input title
    input_title(&mut app, "Test Task");

    // Create the task
    println!("=== Creating Task ===");
    println!("Initiative board items before task creation:");
    for (i, column) in app.ui_state.initiative_board.columns.iter().enumerate() {
        println!("  Column {}: '{}' has {} items", i, column.title, column.items.len());
        for item in &column.items {
            println!("    - '{}'", item.title());
        }
    }
    
    match app.create_new_document().await {
        Ok(_) => println!("Task creation returned Ok"),
        Err(e) => {
            println!("Task creation failed: {:?}", e);
            return Err(e);
        }
    }
    assert_eq!(*app.app_state(), AppState::Normal);

    // Check database after task creation
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        println!("Documents in database after task creation: {}", docs.len());
        for doc in &docs {
            println!("  - '{}' (type: {:?}, path: {})", doc.title, doc.document_type, doc.filepath);
        }
    }

    // Verify the task was created
    let task_board = &app.ui_state.task_board;
    let total_items: usize = task_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 1, "Should have exactly 1 task");

    // Find the created task
    let mut found_task = false;
    for column in &task_board.columns {
        for item in &column.items {
            if item.title() == "Test Task" {
                found_task = true;
                println!("Found task '{}' in column '{}'", item.title(), column.title);
                break;
            }
        }
    }
    assert!(found_task, "Should find the created task in one of the columns");

    Ok(())
}

#[tokio::test]
async fn test_create_adr_ticket() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Switch to ADR board
    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    // Start document creation
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Input title
    input_title(&mut app, "Test ADR");

    // Create the ADR
    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the ADR was created
    let adr_board = &app.ui_state.adr_board;
    let total_items: usize = adr_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 1, "Should have exactly 1 ADR");

    // Find the created ADR
    let mut found_adr = false;
    for column in &adr_board.columns {
        for item in &column.items {
            if item.title() == "Test ADR" {
                found_adr = true;
                println!("Found ADR '{}' in column '{}'", item.title(), column.title);
                break;
            }
        }
    }
    assert!(found_adr, "Should find the created ADR in one of the columns");

    Ok(())
}

#[tokio::test]
async fn test_create_backlog_ticket() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Switch to backlog board
    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    // Start document creation
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Input title
    input_title(&mut app, "Test Backlog Item");

    // Create the backlog item
    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the backlog item was created
    let backlog_board = &app.ui_state.backlog_board;
    let total_items: usize = backlog_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 1, "Should have exactly 1 backlog item");

    // Find the created backlog item
    let mut found_backlog = false;
    for column in &backlog_board.columns {
        for item in &column.items {
            if item.title() == "Test Backlog Item" {
                found_backlog = true;
                println!("Found backlog item '{}' in column '{}'", item.title(), column.title);
                break;
            }
        }
    }
    assert!(found_backlog, "Should find the created backlog item in one of the columns");

    Ok(())
}

#[tokio::test]
async fn test_create_multiple_ticket_types() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Create one of each ticket type
    
    // 1. Strategy
    app.jump_to_strategy_board();
    app.start_document_creation();
    input_title(&mut app, "Multi Test Strategy");
    app.create_new_document().await?;

    // 2. Initiative (need to reload documents first)
    app.load_documents().await?;
    app.jump_to_initiative_board();
    app.start_document_creation();
    input_title(&mut app, "Multi Test Initiative");
    app.create_new_document().await?;

    // 3. Task (need to reload documents to pick up the initiative)
    app.load_documents().await?;
    app.jump_to_task_board();
    app.start_document_creation();
    input_title(&mut app, "Multi Test Task");
    app.create_new_document().await?;

    // 4. ADR
    app.jump_to_adr_board();
    app.start_document_creation();
    input_title(&mut app, "Multi Test ADR");
    app.create_new_document().await?;

    // 5. Backlog
    app.jump_to_backlog_board();
    app.start_document_creation();
    input_title(&mut app, "Multi Test Backlog");
    app.create_new_document().await?;

    // Verify all tickets were created by checking database
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        
        println!("Total documents found: {}", docs.len());
        for doc in &docs {
            println!("  - '{}' (type: {:?}, path: {})", doc.title, doc.document_type, doc.filepath);
        }
        
        // Should have 6 documents: 1 vision (created by test helper) + 5 we created
        assert_eq!(docs.len(), 6, "Should have 6 documents total");

        // Verify each type was created
        let strategy_count = docs.iter().filter(|d| d.title.contains("Multi Test Strategy")).count();
        let initiative_count = docs.iter().filter(|d| d.title.contains("Multi Test Initiative")).count();
        let task_count = docs.iter().filter(|d| d.title.contains("Multi Test Task")).count();
        let adr_count = docs.iter().filter(|d| d.title.contains("Multi Test ADR")).count();
        let backlog_count = docs.iter().filter(|d| d.title.contains("Multi Test Backlog")).count();

        assert_eq!(strategy_count, 1, "Should have 1 strategy");
        assert_eq!(initiative_count, 1, "Should have 1 initiative");
        assert_eq!(task_count, 1, "Should have 1 task");
        assert_eq!(adr_count, 1, "Should have 1 ADR");
        assert_eq!(backlog_count, 1, "Should have 1 backlog item");

        println!("Successfully created all 5 document types!");
        for doc in &docs {
            if doc.title.contains("Multi Test") {
                println!("  - '{}' ({})", doc.title, doc.document_type);
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_empty_title_validation() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Try to create a document with empty title
    app.jump_to_strategy_board();
    app.start_document_creation();
    // Don't input any title

    // Try to create the document
    app.create_new_document().await?;

    // Should still be in creating state or back to normal with error
    // (empty titles should be rejected)
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify no document was created
    let strategy_board = &app.ui_state.strategy_board;
    let total_items: usize = strategy_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(total_items, 0, "Should have no items with empty title");

    Ok(())
}

#[tokio::test]
async fn test_create_child_documents() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Create a full hierarchy: Strategy -> Initiative -> Task

    // 1. Create Strategy
    app.jump_to_strategy_board();
    app.start_document_creation();
    input_title(&mut app, "Parent Strategy");
    app.create_new_document().await?;

    // 2. Create Initiative under Strategy
    app.jump_to_initiative_board();
    app.start_document_creation();
    input_title(&mut app, "Child Initiative");
    app.create_new_document().await?;

    // 3. Create Task under Initiative
    app.jump_to_task_board();
    app.start_document_creation();
    input_title(&mut app, "Child Task");
    app.create_new_document().await?;

    // Verify the hierarchy was created correctly
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        
        // Should have 4 documents: 1 vision + 3 we created
        assert_eq!(docs.len(), 4, "Should have 4 documents total");

        // Check that the documents exist
        let strategy = docs.iter().find(|d| d.title == "Parent Strategy");
        let initiative = docs.iter().find(|d| d.title == "Child Initiative"); 
        let task = docs.iter().find(|d| d.title == "Child Task");

        assert!(strategy.is_some(), "Should have created strategy");
        assert!(initiative.is_some(), "Should have created initiative");
        assert!(task.is_some(), "Should have created task");

        println!("Successfully created document hierarchy:");
        println!("  Strategy: '{}'", strategy.unwrap().title);
        println!("  Initiative: '{}'", initiative.unwrap().title);
        println!("  Task: '{}'", task.unwrap().title);
    }

    Ok(())
}