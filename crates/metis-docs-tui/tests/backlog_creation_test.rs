use anyhow::Result;
use metis_docs_tui::models::{AppState, BoardType};
use tui_input::backend::crossterm::EventHandler;

mod common;
use common::TestHelper;

#[tokio::test]
async fn test_create_backlog_item_via_tui() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Load flight configuration from database
    app.load_flight_config().await?;

    // Jump to backlog board
    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    // Start document creation (simulates pressing 'n' key)
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Simulate user typing a title
    let title = "Fix login bug";
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

    // Verify title was entered
    assert_eq!(app.ui_state.input_title.value(), "Fix login bug");

    // Create the document (simulates pressing Enter)
    app.create_new_document().await?;

    // Should be back to normal state
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the backlog item was created and is visible
    let backlog_board = &app.ui_state.backlog_board;

    // Count total items in all columns
    let total_items: usize = backlog_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(
        total_items, 1,
        "Should have exactly 1 item in backlog board"
    );

    // Find the created item
    let mut found_item = false;
    for column in &backlog_board.columns {
        for item in &column.items {
            if item.title() == "Fix login bug" {
                found_item = true;
                println!(
                    "Found backlog item '{}' in column '{}'",
                    item.title(),
                    column.title
                );
                break;
            }
        }
    }
    assert!(
        found_item,
        "Should find the created backlog item in one of the columns"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_backlog_items() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Load flight configuration from database
    app.load_flight_config().await?;

    // Jump to backlog board
    app.jump_to_backlog_board();

    // Create first backlog item
    app.start_document_creation();
    let title1 = "Bug fix task";
    for ch in title1.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_new_document().await?;

    // Create second backlog item
    app.start_document_creation();
    app.ui_state.input_title.reset(); // Clear previous input
    let title2 = "Feature request";
    for ch in title2.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_new_document().await?;

    // Verify both items exist
    let backlog_board = &app.ui_state.backlog_board;
    let total_items: usize = backlog_board.columns.iter().map(|c| c.items.len()).sum();
    assert_eq!(
        total_items, 2,
        "Should have exactly 2 items in backlog board"
    );

    Ok(())
}

#[tokio::test]
async fn test_backlog_board_columns_exist() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to backlog board and verify structure
    app.jump_to_backlog_board();
    let backlog_board = &app.ui_state.backlog_board;

    // Verify we have 4 columns with correct names
    assert_eq!(backlog_board.columns.len(), 4);

    let expected_columns = vec!["backlog", "bugs", "features", "tech-debt"];
    let actual_columns: Vec<&str> = backlog_board
        .columns
        .iter()
        .map(|c| c.title.as_str())
        .collect();

    assert_eq!(
        actual_columns, expected_columns,
        "Backlog board should have correct column structure"
    );

    // All columns should start empty
    for (i, column) in backlog_board.columns.iter().enumerate() {
        assert_eq!(
            column.items.len(),
            0,
            "Column {} '{}' should start empty",
            i,
            column.title
        );
    }

    Ok(())
}
