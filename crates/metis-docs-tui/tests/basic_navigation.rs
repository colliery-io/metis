use anyhow::Result;
use metis_docs_tui::app::App;
use metis_docs_tui::models::BoardType;
use tempfile::TempDir;

#[tokio::test]
async fn test_basic_board_navigation() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Create app and set the workspace
    let mut app = App::new();
    let metis_dir = project_path.join(".metis");
    app.core_state.set_workspace(metis_dir.clone());
    app.core_state.set_sync_complete();

    // Initialize services
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(metis_dir));

    // Test board navigation with tab
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Test board navigation with shift-tab
    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    // Test direct board jumping
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    Ok(())
}

#[tokio::test]
async fn test_selection_movement() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Create app and set the workspace
    let mut app = App::new();
    let metis_dir = project_path.join(".metis");
    app.core_state.set_workspace(metis_dir);
    app.core_state.set_sync_complete();

    // Test basic selection movement
    let (col, row) = app
        .selection_state
        .get_current_selection(BoardType::Strategy);
    assert_eq!(col, 0);
    assert_eq!(row, 0);

    // Move right
    app.move_selection_right();
    let (col, row) = app
        .selection_state
        .get_current_selection(BoardType::Strategy);
    assert_eq!(col, 1);
    assert_eq!(row, 0);

    // Move left
    app.move_selection_left();
    let (col, row) = app
        .selection_state
        .get_current_selection(BoardType::Strategy);
    assert_eq!(col, 0);
    assert_eq!(row, 0);

    // Can't move left from column 0
    app.move_selection_left();
    let (col, row) = app
        .selection_state
        .get_current_selection(BoardType::Strategy);
    assert_eq!(col, 0);
    assert_eq!(row, 0);

    Ok(())
}
