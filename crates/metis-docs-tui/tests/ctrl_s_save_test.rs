mod common;

use anyhow::Result;
use common::TestHelper;
use metis_core::{domain::configuration::FlightLevelConfig, Document};
use metis_docs_tui::models::AppState;
use tui_input::backend::crossterm::EventHandler;

#[tokio::test]
async fn test_ctrl_s_saves_strategy_document() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Set up full configuration to enable strategies
    if let Some(workspace_dir) = &app.core_state.workspace_dir {
        let db_path = workspace_dir.join("metis.db");
        let database = metis_core::Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = database.configuration_repository().unwrap();

        // Set full preset to enable strategies
        let full_config = FlightLevelConfig::full();
        config_repo.set_flight_level_config(&full_config).unwrap();

        // Load the new configuration
        app.load_flight_config().await?;
    }

    // First create a strategy
    app.start_document_creation();

    // Type the title
    let title = "Test Strategy for Ctrl+S";
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

    // Enter edit mode
    app.view_selected_ticket();
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // The editor should have content loaded
    assert!(app.ui_state.strategy_editor.is_some());

    // Clear and add new content
    if let Some(ref mut editor) = app.ui_state.strategy_editor {
        editor.select_all();
        editor.delete_str(1); // Delete selected content
        editor.insert_str("# Test Strategy\n\nThis content was added and saved via Ctrl+S shortcut.\n\n## Test Goals\n- Verify Ctrl+S functionality\n- Ensure content persistence");
    }

    // Before save - we should be in editing mode
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // Simulate Ctrl+S key press by calling the save and cancel operations
    app.save_content_edit().await?;
    app.cancel_content_editing();

    // After save - we should be back to normal mode
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the strategy was updated correctly by reloading it
    let strategies_dir = helper.metis_dir().join("strategies");
    let strategy_dirs: Vec<_> = std::fs::read_dir(&strategies_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let is_dir = entry.path().is_dir();
            let is_null = entry.file_name().to_string_lossy() == "NULL";
            is_dir && !is_null // Exclude NULL directory for strategy tests
        })
        .collect();

    assert!(
        !strategy_dirs.is_empty(),
        "Should have at least one strategy directory"
    );

    let strategy_file = strategy_dirs[0].path().join("strategy.md");
    let reloaded_strategy = metis_core::Strategy::from_file(&strategy_file).await?;

    assert_eq!(
        reloaded_strategy.content().body,
        "# Test Strategy\n\nThis content was added and saved via Ctrl+S shortcut.\n\n## Test Goals\n- Verify Ctrl+S functionality\n- Ensure content persistence"
    );

    // Verify the file content directly
    let content = std::fs::read_to_string(&strategy_file)?;
    assert!(
        content.contains("This content was added and saved via Ctrl+S shortcut"),
        "Should contain Ctrl+S specific content"
    );
    assert!(
        content.contains("Verify Ctrl+S functionality"),
        "Should contain test goal"
    );
    assert!(
        content.contains("Ensure content persistence"),
        "Should contain persistence test"
    );

    // Verify frontmatter is still intact
    assert!(
        content.contains("level: strategy"),
        "Should still have strategy level"
    );
    assert!(
        content.contains("title: \"Test Strategy for Ctrl+S\""),
        "Should still have original title"
    );

    println!("✅ Ctrl+S save functionality verified successfully!");

    Ok(())
}

#[tokio::test]
async fn test_ctrl_s_with_vision_document() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Load flight config first
    app.load_flight_config().await?;

    // The vision should be created and we should be able to edit it
    // For this test, we'll simulate editing a vision document
    let vision_file = helper.metis_dir().join("vision.md");

    // Manually set up vision editing state
    app.ui_state.editing_vision_path = Some(vision_file.clone());
    app.ui_state.set_app_state(AppState::EditingContent);

    // Create textarea with vision content
    let mut textarea = tui_textarea::TextArea::default();
    textarea.set_block(
        ratatui::widgets::Block::default()
            .borders(ratatui::widgets::Borders::ALL)
            .title("Vision Editor"),
    );

    textarea.insert_str("# Updated Vision\n\nThis vision was updated via Ctrl+S functionality.\n\n## Vision Statement\n- Clear direction\n- Improved outcomes");
    app.ui_state.strategy_editor = Some(textarea);

    // Test Ctrl+S for vision document
    app.save_content_edit().await?;
    app.cancel_content_editing();

    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify vision content was saved
    let vision_content = std::fs::read_to_string(&vision_file)?;
    assert!(
        vision_content.contains("This vision was updated via Ctrl+S functionality"),
        "Vision should contain Ctrl+S updated content"
    );
    assert!(
        vision_content.contains("Clear direction"),
        "Should contain vision statement"
    );

    println!("✅ Ctrl+S save functionality for vision documents verified!");

    Ok(())
}

#[tokio::test]
async fn test_ctrl_s_save_in_streamlined_configuration() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Set up streamlined configuration
    if let Some(workspace_dir) = &app.core_state.workspace_dir {
        let db_path = workspace_dir.join("metis.db");
        let database = metis_core::Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = database.configuration_repository().unwrap();

        // Set streamlined preset
        let streamlined_config = FlightLevelConfig::streamlined();
        config_repo
            .set_flight_level_config(&streamlined_config)
            .unwrap();

        // Load the new configuration
        app.load_flight_config().await?;
    }

    // Jump to initiative board (since strategies are disabled in streamlined)
    app.jump_to_initiative_board();

    // Create an initiative document in streamlined mode
    app.start_document_creation();

    let title = "Streamlined Initiative for Ctrl+S Test";
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

    // Enter edit mode for the initiative
    app.view_selected_ticket();
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // Add content to the editor
    if let Some(ref mut editor) = app.ui_state.strategy_editor {
        editor.select_all();
        editor.delete_str(1);
        editor.insert_str("# Streamlined Initiative\n\nThis initiative was saved in streamlined configuration.\n\n## Objectives\n- Test streamlined save functionality\n- Verify configuration independence");
    }

    // Save using Ctrl+S simulation
    app.save_content_edit().await?;
    app.cancel_content_editing();

    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the initiative was saved in the NULL strategy directory
    let initiatives_dir = helper
        .metis_dir()
        .join("strategies")
        .join("NULL")
        .join("initiatives");
    assert!(
        initiatives_dir.exists(),
        "NULL strategy initiatives directory should exist"
    );

    let initiative_dirs: Vec<_> = std::fs::read_dir(&initiatives_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();

    assert!(
        !initiative_dirs.is_empty(),
        "Should have at least one initiative directory"
    );

    let initiative_file = initiative_dirs[0].path().join("initiative.md");
    let content = std::fs::read_to_string(&initiative_file)?;

    assert!(
        content.contains("This initiative was saved in streamlined configuration"),
        "Should contain streamlined-specific content"
    );
    assert!(
        content.contains("Test streamlined save functionality"),
        "Should contain test objective"
    );

    println!("✅ Ctrl+S save functionality verified in streamlined configuration!");

    Ok(())
}

#[tokio::test]
async fn test_ctrl_s_save_in_direct_configuration() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Set up direct configuration
    if let Some(workspace_dir) = &app.core_state.workspace_dir {
        let db_path = workspace_dir.join("metis.db");
        let database = metis_core::Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = database.configuration_repository().unwrap();

        // Set direct preset
        let direct_config = FlightLevelConfig::direct();
        config_repo.set_flight_level_config(&direct_config).unwrap();

        // Load the new configuration
        app.load_flight_config().await?;
    }

    // Jump to task board (since that's what's available in direct mode)
    app.jump_to_task_board();

    // Create a task document in direct mode
    app.start_document_creation();

    let title = "Direct Task for Ctrl+S Test";
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

    // Enter edit mode for the task
    app.view_selected_ticket();
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // Add content to the editor
    if let Some(ref mut editor) = app.ui_state.strategy_editor {
        editor.select_all();
        editor.delete_str(1);
        editor.insert_str("# Direct Task\n\nThis task was saved in direct configuration.\n\n## Actions\n- Test direct save functionality\n- Verify minimal hierarchy");
    }

    // Save using Ctrl+S simulation
    app.save_content_edit().await?;
    app.cancel_content_editing();

    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the task was saved in the direct structure (NULL/NULL)
    let tasks_dir = helper
        .metis_dir()
        .join("strategies")
        .join("NULL")
        .join("initiatives")
        .join("NULL")
        .join("tasks");
    assert!(tasks_dir.exists(), "Direct tasks directory should exist");

    let task_files: Vec<_> = std::fs::read_dir(&tasks_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "md")
        })
        .collect();

    assert!(!task_files.is_empty(), "Should have at least one task file");

    let task_file = &task_files[0].path();
    let content = std::fs::read_to_string(task_file)?;

    assert!(
        content.contains("This task was saved in direct configuration"),
        "Should contain direct-specific content"
    );
    assert!(
        content.contains("Test direct save functionality"),
        "Should contain test action"
    );

    println!("✅ Ctrl+S save functionality verified in direct configuration!");

    Ok(())
}
