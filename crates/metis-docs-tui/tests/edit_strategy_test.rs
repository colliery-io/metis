mod common;

use anyhow::Result;
use common::TestHelper;
use metis_core::Document;
use metis_docs_tui::models::AppState;
use tui_input::backend::crossterm::EventHandler;

#[tokio::test]
async fn test_edit_and_save_strategy() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // First create a strategy
    app.start_document_creation();

    // Type the title
    let title = "My Strategy";
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

    // Now edit it - select the strategy and enter edit mode
    app.view_selected_ticket();
    assert_eq!(*app.app_state(), AppState::EditingContent);

    // The editor should have content loaded
    assert!(app.ui_state.strategy_editor.is_some());

    // Clear and add new content
    if let Some(ref mut editor) = app.ui_state.strategy_editor {
        // Clear existing content by selecting all and deleting
        editor.select_all();
        editor.delete_str(1); // Delete selected content

        // Insert new content
        editor.insert_str("# My Strategy\n\nThis is my edited content.\n\n## Goals\n- Increase performance\n- Improve user experience");
    }

    // Save the edit
    app.save_content_edit().await?;
    app.cancel_content_editing();
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify the strategy was updated correctly by reloading it
    let strategy_dirs: Vec<_> = std::fs::read_dir(helper.metis_dir.join("strategies"))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();
    let strategy_file = strategy_dirs[0].path().join("strategy.md");
    let reloaded_strategy = metis_core::Strategy::from_file(&strategy_file).await?;
    assert_eq!(reloaded_strategy.content().body, "# My Strategy\n\nThis is my edited content.\n\n## Goals\n- Increase performance\n- Improve user experience");

    // Verify the file content directly
    let content = std::fs::read_to_string(&strategy_file)?;
    assert!(
        content.contains("This is my edited content"),
        "Should contain edited content"
    );
    assert!(
        content.contains("Increase performance"),
        "Should contain first goal"
    );
    assert!(
        content.contains("Improve user experience"),
        "Should contain second goal"
    );

    // Verify frontmatter is still intact
    assert!(
        content.contains("level: strategy"),
        "Should still have strategy level"
    );
    assert!(
        content.contains("title: \"My Strategy\""),
        "Should still have title"
    );

    // Verify the old template content is NOT present
    assert!(
        !content.contains("Problem Statement"),
        "Should not contain template content"
    );
    assert!(
        !content.contains("Success Metrics"),
        "Should not contain template content"
    );

    println!("✅ Strategy edited and saved successfully!");

    Ok(())
}
