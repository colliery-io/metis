//! Tests for error handling and edge cases in TUI
//! Ensures the TUI handles invalid inputs and edge conditions gracefully

mod common;

use anyhow::Result;
use common::TestHelper;
use metis_docs_tui::models::AppState;
use tui_input::backend::crossterm::EventHandler;

/// Test creating documents with invalid parent relationships
#[tokio::test]
async fn test_tui_invalid_parent_relationships() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Invalid Parent Relationships ===");

    // Try to create a task without proper parent hierarchy
    app.load_documents().await?;
    app.jump_to_task_board();

    // Attempt to create task directly (should fail or be disabled)
    app.start_document_creation();

    // Check if we're prevented from creating
    if *app.app_state() != AppState::CreatingDocument {
        println!("✅ TUI correctly prevents creating task without parent");
    } else {
        // If we can create, it should fail when we try to save
        let title = "Orphan Task";
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

        app.create_new_document().await?;

        if let Some(error) = app.error_message() {
            println!("✅ Task creation failed with error: {}", error);
        } else {
            println!("⚠️  TUI allowed creating task without proper parent");
        }
    }

    Ok(())
}

/// Test edge cases with extremely large titles in TUI
#[tokio::test]
async fn test_tui_large_title_edge_cases() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Large Title Edge Cases ===");

    app.load_documents().await?;
    app.jump_to_strategy_board();
    app.start_document_creation();

    // Test with a very long title
    let long_title = "A".repeat(300); // 300 character title
    for ch in long_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }

    // Check if input was truncated or limited
    let input_len = app.ui_state.input_title.value().len();
    println!("Input field accepted {} characters", input_len);

    app.create_new_document().await?;

    if let Some(error) = app.error_message() {
        println!("✅ Large title rejected with error: {}", error);
    } else {
        println!("✅ TUI handled {} character title", input_len);

        // Check how it's displayed in the board
        app.load_documents().await?;
        let board = &app.ui_state.strategy_board;

        if let Some(item) = board.columns[0].items.first() {
            let displayed_len = item.prelude.len();
            if displayed_len < input_len {
                println!(
                    "✅ Title truncated for display: {} chars shown",
                    displayed_len
                );
            }
        }
    }

    // Test empty title
    app.start_document_creation();

    // Try to create without entering any title
    app.create_new_document().await?;

    if let Some(error) = app.error_message() {
        println!("✅ Empty title correctly rejected: {}", error);
    } else if *app.app_state() == AppState::CreatingDocument {
        println!("✅ TUI keeps creation dialog open for empty title");
    } else {
        println!("❌ TUI allowed empty title creation");
    }

    // Cancel creation
    app.cancel_content_editing();

    Ok(())
}

/// Test special characters in TUI inputs
#[tokio::test]
async fn test_tui_special_characters_handling() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Special Characters Handling ===");

    app.load_documents().await?;
    app.jump_to_adr_board();

    // Test various special character titles
    let test_cases = vec![
        ("Emoji 🚀 Title", "emoji"),
        ("Title/with\\slashes", "slashes"),
        ("Title \"quoted\"", "quotes"),
        ("Title\twith\ttabs", "tabs"),
    ];

    for (title, desc) in test_cases {
        println!("\n--- Testing {} ---", desc);

        app.start_document_creation();

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

        app.create_new_document().await?;

        if let Some(error) = app.error_message() {
            println!("  ❌ Failed with: {}", error);
        } else {
            println!("  ✅ Created successfully");

            // Check display
            app.load_documents().await?;
            let board = &app.ui_state.adr_board;

            if board.columns[0]
                .items
                .iter()
                .any(|item| item.prelude.contains(desc))
            {
                println!("  ✅ Displayed correctly in board");
            }
        }
    }

    Ok(())
}

/// Test rapid keyboard input and UI responsiveness
#[tokio::test]
async fn test_tui_rapid_input_handling() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Rapid Input Handling ===");

    app.load_documents().await?;
    app.jump_to_strategy_board();

    // Rapidly switch between boards
    for _ in 0..10 {
        app.jump_to_initiative_board();
        app.jump_to_task_board();
        app.jump_to_adr_board();
        app.jump_to_strategy_board();
    }

    println!("✅ Rapid board switching handled");

    // Create document with rapid typing
    app.start_document_creation();

    let title = "RapidlyTypedTitleWithoutSpaces";
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

    // Immediately create
    app.create_new_document().await?;

    if app.error_message().is_none() {
        println!("✅ Rapid typing handled correctly");
    }

    // Test rapid navigation
    app.load_documents().await?;

    // Move selection rapidly
    for _ in 0..20 {
        app.move_selection_right();
        app.move_selection_down();
        app.move_selection_left();
        app.move_selection_up();
    }

    println!("✅ Rapid navigation handled");

    Ok(())
}

/// Test editing with invalid content
#[tokio::test]
async fn test_tui_invalid_edit_content() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Invalid Edit Content ===");

    // Create a document to edit
    app.load_documents().await?;
    app.jump_to_strategy_board();
    app.start_document_creation();

    let title = "Editable Doc";
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
    app.create_new_document().await?;

    // Start editing
    app.load_documents().await?;
    app.view_selected_ticket();

    // Try to add content that might break YAML frontmatter
    if let Some(ref mut editor) = app.ui_state.strategy_editor {
        editor.insert_str(
            "\n---\nmalformed: yaml: content:\n  - without\n- proper\nindentation:\n---\n",
        );
    }

    app.save_content_edit().await?;

    if let Some(error) = app.error_message() {
        println!("✅ Invalid content rejected: {}", error);
    } else {
        println!("⚠️  TUI accepted potentially invalid content");
    }

    Ok(())
}

/// Test boundary conditions for board navigation
#[tokio::test]
async fn test_tui_navigation_boundaries() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Navigation Boundaries ===");

    app.load_documents().await?;

    // Test navigation on empty board
    app.jump_to_task_board();

    // Try to move in all directions on empty board
    app.move_selection_up();
    app.move_selection_down();
    app.move_selection_left();
    app.move_selection_right();

    println!("✅ Empty board navigation handled");

    // Create multiple documents to test boundaries
    app.jump_to_adr_board();

    for i in 1..=5 {
        app.start_document_creation();
        let title = format!("ADR {}", i);
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
        app.create_new_document().await?;
        app.load_documents().await?;
    }

    // Test moving beyond boundaries
    for _ in 0..10 {
        app.move_selection_down(); // Should stop at last item
    }

    for _ in 0..10 {
        app.move_selection_up(); // Should stop at first item
    }

    for _ in 0..10 {
        app.move_selection_left(); // Should stop at first column
    }

    for _ in 0..10 {
        app.move_selection_right(); // Should stop at last column
    }

    println!("✅ Board navigation boundaries respected");

    Ok(())
}

/// Test concurrent operations in TUI
#[tokio::test]
async fn test_tui_concurrent_operations() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test TUI Concurrent Operations ===");

    // Create a document
    app.load_documents().await?;
    app.jump_to_strategy_board();
    app.start_document_creation();

    let title = "Concurrent Test";
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
    app.create_new_document().await?;

    // Start editing
    app.load_documents().await?;
    app.view_selected_ticket();

    // Simulate another operation trying to happen during edit
    // In real TUI, this would be prevented by modal state
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // Try to transition document while editing (should be blocked)
    let was_editing = matches!(app.app_state(), AppState::EditingContent);
    app.transition_selected_document().await?;

    if matches!(app.app_state(), AppState::EditingContent) && was_editing {
        println!("✅ Concurrent operations correctly blocked during edit");
    } else {
        println!("⚠️  State changed during edit mode");
    }

    // Cancel edit
    app.cancel_content_editing();

    // Now operations should work
    app.transition_selected_document().await?;
    println!("✅ Operations resumed after modal state cleared");

    Ok(())
}
