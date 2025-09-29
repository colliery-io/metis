use anyhow::Result;
use metis_docs_tui::models::BoardType;
use tui_input::backend::crossterm::EventHandler;

mod common;
use common::TestHelper;

#[tokio::test]
async fn test_backlog_board_navigation() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Verify initial state is Strategy board
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Test Tab navigation through all boards including backlog
    app.next_board(); // Strategy -> Initiative
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.next_board(); // Initiative -> Task
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.next_board(); // Task -> Adr
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.next_board(); // Adr -> Backlog
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.next_board(); // Backlog -> Strategy (full cycle)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Test Shift+Tab (previous) navigation
    app.previous_board(); // Strategy -> Backlog
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.previous_board(); // Backlog -> Adr
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    // Test direct jump to backlog board
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    Ok(())
}

#[tokio::test]
async fn test_backlog_board_structure() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to backlog board
    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    // Verify backlog board has correct columns
    let backlog_board = &app.ui_state.backlog_board;
    assert_eq!(backlog_board.columns.len(), 4);
    
    let column_titles: Vec<&str> = backlog_board.columns.iter().map(|c| c.title.as_str()).collect();
    assert_eq!(column_titles, vec!["backlog", "bugs", "features", "tech-debt"]);

    // Verify all columns start empty
    for column in &backlog_board.columns {
        assert_eq!(column.items.len(), 0, "Column '{}' should start empty", column.title);
    }

    Ok(())
}

#[tokio::test]
async fn test_backlog_item_detection_and_placement() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    // Project is already initialized by TestHelper::new()

    // Create a backlog task with bug tag using the document service
    let mut app = helper.create_app();
    app.jump_to_backlog_board();
    
    // Set up the document creation with bug tag
    app.start_document_creation();
    
    // Add title 
    let title = "Test Backlog Task";
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
    
    // Create the backlog item
    app.create_new_document().await?;
    
    // Now manually add the bug tag to the created document
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        let backlog_task = docs.iter().find(|d| d.title == "Test Backlog Task").unwrap();
        
        // Read the file and add bug tag
        let task_content = std::fs::read_to_string(&backlog_task.filepath)?;
        let updated_content = task_content.replace(
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"",
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"\n  - \"#bug\""
        );
        std::fs::write(&backlog_task.filepath, updated_content)?;
    }

    // Reload documents to pick up the tag change
    app.load_documents().await?;

    // Jump to backlog board
    app.jump_to_backlog_board();
    
    // Verify the backlog task was detected and placed in the bugs column
    let backlog_board = &app.ui_state.backlog_board;
    
    // Should be in bugs column (index 1) because "- [x] Bug" is checked
    assert_eq!(backlog_board.columns[1].items.len(), 1, "Bugs column should have 1 item");
    assert_eq!(backlog_board.columns[1].items[0].title(), "Test Backlog Task");

    Ok(())
}

#[tokio::test]
async fn test_backlog_item_feature_categorization() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    // Project is already initialized by TestHelper::new()

    // Create app and create backlog task using TUI, then add feature tag
    let mut app = helper.create_app();
    
    // Use TUI to create backlog task
    app.jump_to_backlog_board();
    app.start_document_creation();
    
    // Add title 
    let title = "Feature Request Task";
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
    
    // Create the backlog item
    app.create_new_document().await?;
    
    // Now manually add the feature tag to the created document
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        let backlog_task = docs.iter().find(|d| d.title == "Feature Request Task").unwrap();
        
        // Read the file and add feature tag
        let task_content = std::fs::read_to_string(&backlog_task.filepath)?;
        let updated_content = task_content.replace(
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"",
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"\n  - \"#feature\""
        );
        std::fs::write(&backlog_task.filepath, updated_content)?;
    }

    // Reload documents to pick up the tag change
    app.load_documents().await?;

    // Jump to backlog board and verify placement in features column
    app.jump_to_backlog_board();
    let backlog_board = &app.ui_state.backlog_board;
    
    // Should be in features column (index 2)
    assert_eq!(backlog_board.columns[2].items.len(), 1, "Features column should have 1 item");
    assert_eq!(backlog_board.columns[2].items[0].title(), "Feature Request Task");

    Ok(())
}

#[tokio::test]
async fn test_backlog_item_tech_debt_categorization() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    // Project is already initialized by TestHelper::new()

    // Create app and create backlog task using TUI, then add tech-debt tag
    let mut app = helper.create_app();
    
    // Use TUI to create backlog task
    app.jump_to_backlog_board();
    app.start_document_creation();
    
    // Add title 
    let title = "Tech Debt Refactor";
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
    
    // Create the backlog item
    app.create_new_document().await?;
    
    // Now manually add the tech-debt tag to the created document
    if let Some(ref doc_service) = app.document_service {
        let docs = doc_service.load_documents_from_database().await?;
        let backlog_task = docs.iter().find(|d| d.title == "Tech Debt Refactor").unwrap();
        
        // Read the file and add tech-debt tag
        let task_content = std::fs::read_to_string(&backlog_task.filepath)?;
        let updated_content = task_content.replace(
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"",
            "tags:\n  - \"#task\"\n  - \"#phase/backlog\"\n  - \"#tech-debt\""
        );
        std::fs::write(&backlog_task.filepath, updated_content)?;
    }

    // Reload documents to pick up the tag change
    app.load_documents().await?;

    // Jump to backlog board and verify placement in tech-debt column
    app.jump_to_backlog_board();
    let backlog_board = &app.ui_state.backlog_board;
    
    // Should be in tech-debt column (index 3)
    assert_eq!(backlog_board.columns[3].items.len(), 1, "Tech-debt column should have 1 item");
    assert_eq!(backlog_board.columns[3].items[0].title(), "Tech Debt Refactor");

    Ok(())
}

#[tokio::test]
async fn test_backlog_item_default_categorization() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    // Project is already initialized by TestHelper::new()

    // Create app and create backlog task using TUI (no additional tags)
    let mut app = helper.create_app();
    
    // Use TUI to create backlog task
    app.jump_to_backlog_board();
    app.start_document_creation();
    
    // Add title 
    let title = "General Backlog Item";
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
    
    // Create the backlog item (no additional tags)
    app.create_new_document().await?;
    
    // Load documents
    app.load_documents().await?;

    // Jump to backlog board and verify placement in default backlog column
    app.jump_to_backlog_board();
    let backlog_board = &app.ui_state.backlog_board;
    
    // Should be in backlog column (index 0) as default
    assert_eq!(backlog_board.columns[0].items.len(), 1, "Backlog column should have 1 item");
    assert_eq!(backlog_board.columns[0].items[0].title(), "General Backlog Item");

    Ok(())
}

#[tokio::test]
async fn test_backlog_selection_state() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to backlog board
    app.jump_to_backlog_board();
    
    // Test selection state is properly initialized for backlog
    let selection = app.selection_state.get_current_selection(BoardType::Backlog);
    assert_eq!(selection, (0, 0), "Backlog selection should start at (0,0)");

    // Test selection movement within backlog board
    app.move_selection_right();
    let selection = app.selection_state.get_current_selection(BoardType::Backlog);
    assert_eq!(selection, (1, 0), "After moving right, should be at (1,0)");

    app.move_selection_right();
    let selection = app.selection_state.get_current_selection(BoardType::Backlog);
    assert_eq!(selection, (2, 0), "After moving right again, should be at (2,0)");

    app.move_selection_left();
    let selection = app.selection_state.get_current_selection(BoardType::Backlog);
    assert_eq!(selection, (1, 0), "After moving left, should be back at (1,0)");

    Ok(())
}

#[tokio::test]
async fn test_keyboard_shortcut_five_for_backlog() -> Result<()> {
    let mut helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Start at strategy board
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Simulate pressing '5' key to jump to backlog
    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    // Test other number keys still work
    app.jump_to_strategy_board(); // Simulate '1'
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    app.jump_to_initiative_board(); // Simulate '2'
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.jump_to_task_board(); // Simulate '3'
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.jump_to_adr_board(); // Simulate '4'
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.jump_to_backlog_board(); // Simulate '5'
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    Ok(())
}