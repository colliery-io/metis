mod common;

use anyhow::Result;
use common::TestHelper;
use metis_docs_tui::models::{AppState, BoardType};
use tui_input::backend::crossterm::EventHandler;

#[tokio::test]
async fn test_create_strategy() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    // Load initial documents (should be empty except vision)
    app.load_documents().await?;

    // Verify we're on strategy board
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Verify board is empty initially
    let board = &app.ui_state.strategy_board;
    assert_eq!(
        board.columns[0].items.len(),
        0,
        "Strategy board should be empty initially"
    );

    // Start creating a new strategy
    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    // Type the title character by character
    let title = "Test Strategy";
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

    // Verify the input contains our title
    assert_eq!(app.ui_state.input_title.value(), title);

    // Create the document
    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Reload documents to see the new strategy
    app.load_documents().await?;

    // Verify the strategy exists in the UI
    let board = &app.ui_state.strategy_board;
    assert_eq!(
        board.columns[0].items.len(),
        1,
        "Should have 1 strategy in Shaping column"
    );
    assert_eq!(board.columns[0].items[0].prelude, "Test Strategy");

    // Verify the file exists on disk
    let strategies_dir = helper.metis_dir().join("strategies");
    assert!(strategies_dir.exists(), "Strategies directory should exist");

    // Find the strategy directory (it will have a generated ID)
    let strategy_dirs: Vec<_> = std::fs::read_dir(&strategies_dir)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect();

    assert_eq!(
        strategy_dirs.len(),
        1,
        "Should have exactly one strategy directory"
    );

    let strategy_file = strategy_dirs[0].path().join("strategy.md");
    assert!(strategy_file.exists(), "Strategy file should exist");

    // Read and verify the file content
    let content = std::fs::read_to_string(&strategy_file)?;

    assert!(
        content.contains("level: strategy") || content.contains("document_type: strategy"),
        "Should have strategy document type"
    );
    assert!(
        content.contains("title: \"Test Strategy\""),
        "Should have correct title"
    );
    assert!(
        content.contains("#phase/shaping"),
        "Should be in shaping phase"
    );

    // Verify database entry
    let db = metis_core::dal::Database::new(helper.metis_dir().join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    assert_eq!(db_strategies.len(), 1, "Should have 1 strategy in database");
    assert_eq!(db_strategies[0].title, "Test Strategy");
    assert_eq!(db_strategies[0].phase, "shaping");
    assert!(!db_strategies[0].archived);

    println!("✅ Strategy created successfully!");
    println!("   - UI shows strategy in Shaping column");
    println!("   - File exists at: {:?}", strategy_file);
    println!("   - Database contains strategy record");

    Ok(())
}
