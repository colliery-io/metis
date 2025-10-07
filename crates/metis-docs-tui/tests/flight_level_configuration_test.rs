use anyhow::Result;
use metis_core::domain::configuration::FlightLevelConfig;
use metis_docs_tui::app::App;
use metis_docs_tui::models::BoardType;
use tempfile::TempDir;

/// Test helper to create an app with a specific flight level configuration
async fn create_app_with_config(config: FlightLevelConfig) -> Result<App> {
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

    // Set the flight level configuration
    app.core_state.set_flight_config(config);

    // Initialize services
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(metis_dir));

    // Ensure the current board is valid for the configuration
    app.ui_state
        .ensure_valid_board(&app.core_state.flight_config);

    Ok(app)
}

#[tokio::test]
async fn test_full_configuration_navigation() -> Result<()> {
    let mut app = create_app_with_config(FlightLevelConfig::full()).await?;

    // In full configuration, should start with strategy board
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Test complete navigation cycle: Strategy → Initiative → Task → ADR → Backlog → Strategy
    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);

    // Test reverse navigation
    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    // Test direct board jumping - all should work
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

    Ok(())
}

#[tokio::test]
async fn test_streamlined_configuration_navigation() -> Result<()> {
    let mut app = create_app_with_config(FlightLevelConfig::streamlined()).await?;

    // In streamlined configuration, should start with initiative board (not strategy)
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    // Test navigation cycle: Initiative → Task → ADR → Backlog → Initiative (no strategy)
    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative); // Should skip strategy

    // Test reverse navigation
    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative); // Should skip strategy

    // Test direct board jumping
    app.jump_to_strategy_board(); // Should be ignored
    assert_eq!(app.ui_state.current_board, BoardType::Initiative); // Should stay on initiative

    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    Ok(())
}

#[tokio::test]
async fn test_direct_configuration_navigation() -> Result<()> {
    let mut app = create_app_with_config(FlightLevelConfig::direct()).await?;

    // In direct configuration, should start with task board (no strategy or initiative)
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    // Test navigation cycle: Task → ADR → Backlog → Task (no strategy or initiative)
    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.next_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should skip strategy and initiative

    // Test reverse navigation
    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.previous_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should skip strategy and initiative

    // Test direct board jumping
    app.jump_to_strategy_board(); // Should be ignored
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should stay on task

    app.jump_to_initiative_board(); // Should be ignored
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should stay on task

    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);

    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    Ok(())
}

#[tokio::test]
async fn test_enabled_boards_lists() -> Result<()> {
    use metis_docs_tui::app::state::UiState;

    // Test full configuration enabled boards
    let full_boards = UiState::get_enabled_boards(&FlightLevelConfig::full());
    assert_eq!(
        full_boards,
        vec![
            BoardType::Strategy,
            BoardType::Initiative,
            BoardType::Task,
            BoardType::Adr,
            BoardType::Backlog
        ]
    );

    // Test streamlined configuration enabled boards
    let streamlined_boards = UiState::get_enabled_boards(&FlightLevelConfig::streamlined());
    assert_eq!(
        streamlined_boards,
        vec![
            BoardType::Initiative,
            BoardType::Task,
            BoardType::Adr,
            BoardType::Backlog
        ]
    );

    // Test direct configuration enabled boards
    let direct_boards = UiState::get_enabled_boards(&FlightLevelConfig::direct());
    assert_eq!(
        direct_boards,
        vec![BoardType::Task, BoardType::Adr, BoardType::Backlog]
    );

    Ok(())
}

#[tokio::test]
async fn test_board_enabled_checks() -> Result<()> {
    use metis_docs_tui::app::state::UiState;

    let full_config = FlightLevelConfig::full();
    let streamlined_config = FlightLevelConfig::streamlined();
    let direct_config = FlightLevelConfig::direct();

    // Full configuration - all boards enabled
    assert!(UiState::is_board_enabled(BoardType::Strategy, &full_config));
    assert!(UiState::is_board_enabled(
        BoardType::Initiative,
        &full_config
    ));
    assert!(UiState::is_board_enabled(BoardType::Task, &full_config));
    assert!(UiState::is_board_enabled(BoardType::Adr, &full_config));
    assert!(UiState::is_board_enabled(BoardType::Backlog, &full_config));

    // Streamlined configuration - no strategy
    assert!(!UiState::is_board_enabled(
        BoardType::Strategy,
        &streamlined_config
    ));
    assert!(UiState::is_board_enabled(
        BoardType::Initiative,
        &streamlined_config
    ));
    assert!(UiState::is_board_enabled(
        BoardType::Task,
        &streamlined_config
    ));
    assert!(UiState::is_board_enabled(
        BoardType::Adr,
        &streamlined_config
    ));
    assert!(UiState::is_board_enabled(
        BoardType::Backlog,
        &streamlined_config
    ));

    // Direct configuration - no strategy or initiative
    assert!(!UiState::is_board_enabled(
        BoardType::Strategy,
        &direct_config
    ));
    assert!(!UiState::is_board_enabled(
        BoardType::Initiative,
        &direct_config
    ));
    assert!(UiState::is_board_enabled(BoardType::Task, &direct_config));
    assert!(UiState::is_board_enabled(BoardType::Adr, &direct_config));
    assert!(UiState::is_board_enabled(
        BoardType::Backlog,
        &direct_config
    ));

    Ok(())
}

#[tokio::test]
async fn test_ensure_valid_board_function() -> Result<()> {
    // Test starting with strategy board but using streamlined config
    let mut app = App::new();
    app.ui_state.current_board = BoardType::Strategy; // Start with strategy

    let streamlined_config = FlightLevelConfig::streamlined();
    app.ui_state.ensure_valid_board(&streamlined_config);

    // Should switch to initiative (first enabled board in streamlined)
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    // Test starting with initiative board but using direct config
    app.ui_state.current_board = BoardType::Initiative; // Start with initiative

    let direct_config = FlightLevelConfig::direct();
    app.ui_state.ensure_valid_board(&direct_config);

    // Should switch to task (first enabled board in direct)
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    // Test with already valid board
    app.ui_state.current_board = BoardType::Task;
    let full_config = FlightLevelConfig::full();
    app.ui_state.ensure_valid_board(&full_config);

    // Should stay on task (it's valid in full config)
    assert_eq!(app.ui_state.current_board, BoardType::Task);

    Ok(())
}
