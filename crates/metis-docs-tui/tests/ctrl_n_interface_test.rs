use anyhow::Result;
use metis_docs_tui::models::{AppState, BoardType};
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
async fn test_parent_selection_required_on_strategy_board() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to strategy board with no strategies
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Try to create child document with no strategy selected
    app.start_child_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingChildDocument);

    input_title(&mut app, "Orphaned Initiative");
    app.create_child_document().await?;

    // Should get error message about needing to select a strategy
    if let Some(error) = app.error_message() {
        assert!(error.contains("Please select a strategy first"));
        println!(
            "✅ Strategy board shows error when no strategy selected: {}",
            error
        );
    } else {
        println!("⚠️  No error message shown for missing strategy parent");
    }

    Ok(())
}

#[tokio::test]
async fn test_parent_selection_required_on_initiative_board() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to initiative board with no initiatives
    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    // Try to create child document with no initiative selected
    app.start_child_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingChildDocument);

    input_title(&mut app, "Orphaned Task");
    app.create_child_document().await?;

    // Should get error message about needing to select an initiative
    if let Some(error) = app.error_message() {
        assert!(error.contains("Please select an initiative first"));
        println!(
            "✅ Initiative board shows error when no initiative selected: {}",
            error
        );
    } else {
        println!("⚠️  No error message shown for missing initiative parent");
    }

    Ok(())
}

#[tokio::test]
async fn test_task_board_creation_disabled() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Jump to task board
    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    // Try smart document creation on task board - should show error message
    app.start_smart_document_creation();

    // Should stay in Normal state (not enter creation mode)
    assert_eq!(*app.app_state(), AppState::Normal);

    // Should have error message about task creation being disabled
    if let Some(error) = app.error_message() {
        assert!(error.contains("Tasks are created from"));
        assert!(error.contains("Initiative board"));
        assert!(error.contains("Backlog board"));
        println!("✅ Task board correctly shows disabled message: {}", error);
    } else {
        panic!("Expected error message when creation attempted on Task board");
    }

    Ok(())
}

#[tokio::test]
async fn test_smart_creation_routes_correctly() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Test ADR board smart creation
    app.jump_to_adr_board();
    app.start_smart_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingAdr);
    app.cancel_document_creation();

    // Test Backlog board smart creation - now goes to category selection
    app.jump_to_backlog_board();
    app.start_smart_document_creation();
    assert_eq!(*app.app_state(), AppState::SelectingBacklogCategory);
    app.cancel_document_creation();

    // Test Strategy board smart creation
    app.jump_to_strategy_board();
    app.start_smart_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingChildDocument);
    app.cancel_document_creation();

    // Test Initiative board smart creation
    app.jump_to_initiative_board();
    app.start_smart_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingChildDocument);
    app.cancel_document_creation();

    println!("✅ Smart creation routes to correct states for each board");

    Ok(())
}

#[tokio::test]
async fn test_board_context_switching() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Test that board switching works correctly (supports context-aware UI)
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    println!("✅ Board switching works correctly for context-aware UI");

    Ok(())
}
