use anyhow::Result;
use metis_core::domain::configuration::FlightLevelConfig;
use metis_docs_tui::app::App;
use metis_docs_tui::models::{AppState, BoardType};
use tempfile::TempDir;
use tui_input::backend::crossterm::EventHandler;

/// Helper function to input a title to the TUI
fn input_title(app: &mut App, title: &str) {
    for ch in title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::NONE,
                ),
            ));
    }
}

/// Test helper to create an app with a specific flight level configuration and some test documents
async fn create_app_with_config_and_docs(config: FlightLevelConfig) -> Result<(App, TempDir)> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    let init_result = metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Save configuration to database
    {
        let db = metis_core::Database::new(init_result.database_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        config_repo.set_flight_level_config(&config).unwrap();
    }

    // Create app and manually initialize
    let mut app = App::new();
    
    // Set the workspace directory to the .metis directory (like real TUI does)
    app.core_state.set_workspace(init_result.metis_dir.clone());
    app.core_state.set_sync_complete();
    app.core_state.set_flight_config(config);
    
    // Initialize services manually with the .metis directory
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        init_result.metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        init_result.metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(init_result.metis_dir));
    
    // Load flight level configuration and ensure valid board
    app.load_flight_config().await?;

    // Load any existing documents
    app.load_documents().await?;

    Ok((app, temp_dir))
}

#[tokio::test]
async fn test_full_configuration_creation_workflow() -> Result<()> {
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::full()).await?;
    
    // Should start on Strategy board
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Test 1: Create Strategy from Strategy board
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument);
    
    // Set title and create
    input_title(&mut app, "Test Strategy");
    app.create_new_document().await?;
    
    // Should return to normal state and reload documents
    assert_eq!(app.ui_state.app_state, AppState::Normal);
    
    // Load documents to see the created strategy
    app.load_documents().await?;
    
    // Should have one strategy in the strategy board
    let strategy_count: usize = app.ui_state.strategy_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(strategy_count, 1);
    
    // Test 2: Navigate to Initiative board and create Initiative
    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);
    
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingChildDocument);  // Should be child creation
    
    input_title(&mut app, "Test Initiative");
    app.create_child_document().await?;
    
    app.load_documents().await?;
    
    // Should have one initiative
    let initiative_count: usize = app.ui_state.initiative_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(initiative_count, 1);
    
    // Test 3: Navigate to Task board and create Task
    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);
    
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingChildDocument);  // Should be child creation
    
    input_title(&mut app, "Test Task");
    app.create_child_document().await?;
    
    app.load_documents().await?;
    
    // Should have one task
    let task_count: usize = app.ui_state.task_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(task_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_streamlined_configuration_creation_workflow() -> Result<()> {
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::streamlined()).await?;
    
    // Should start on Initiative board (not Strategy)
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);
    
    // Test 1: Create Initiative from Initiative board (should be root document creation, not child)
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument);  // Should be root creation since no strategies
    
    input_title(&mut app, "Test Initiative");
    app.create_new_document().await?;
    
    app.load_documents().await?;
    
    // Should have one initiative
    let initiative_count: usize = app.ui_state.initiative_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(initiative_count, 1);
    
    // Test 2: Strategy board should not be accessible
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative); // Should stay on Initiative
    
    // Test 3: Create Task by selecting initiative and creating child
    // Stay on Initiative board and select the first initiative
    app.selection_state.initiative_selection = (0, 0); // Select first initiative in first column
    
    // Start child document creation (Ctrl+N equivalent)
    app.start_child_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingChildDocument);  // Should be child creation
    
    input_title(&mut app, "Test Task");
    app.create_child_document().await?;
    
    app.load_documents().await?;
    
    // Navigate to Task board to check if task was created
    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);
    
    // Should have one task
    let task_count: usize = app.ui_state.task_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(task_count, 1);

    Ok(())
}

#[tokio::test]
async fn test_direct_configuration_creation_workflow() -> Result<()> {
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::direct()).await?;
    
    // Should start on Task board (not Strategy or Initiative)
    assert_eq!(app.ui_state.current_board, BoardType::Task);
    
    // Test 1: Create Task from Task board (should be root document creation)
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument);  // Should be root creation since no strategies/initiatives
    
    input_title(&mut app, "Test Task");
    app.create_new_document().await?;
    
    app.load_documents().await?;
    
    // Should have one task
    let task_count: usize = app.ui_state.task_board.columns.iter()
        .map(|col| col.items.len())
        .sum();
    assert_eq!(task_count, 1);
    
    // Test 2: Strategy and Initiative boards should not be accessible
    app.jump_to_strategy_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should stay on Task
    
    app.jump_to_initiative_board(); 
    assert_eq!(app.ui_state.current_board, BoardType::Task); // Should stay on Task
    
    // Test 3: Should be able to access ADR and Backlog boards
    app.jump_to_adr_board();
    assert_eq!(app.ui_state.current_board, BoardType::Adr);
    
    app.jump_to_backlog_board();
    assert_eq!(app.ui_state.current_board, BoardType::Backlog);

    Ok(())
}

#[tokio::test] 
async fn test_adr_creation_works_in_all_configurations() -> Result<()> {
    // Test that ADR creation works the same in all configurations
    let configs = vec![
        ("full", FlightLevelConfig::full()),
        ("streamlined", FlightLevelConfig::streamlined()),
        ("direct", FlightLevelConfig::direct()),
    ];
    
    for (config_name, config) in configs {
        let (mut app, _temp_dir) = create_app_with_config_and_docs(config).await?;
        
        // Navigate to ADR board
        app.jump_to_adr_board();
        assert_eq!(app.ui_state.current_board, BoardType::Adr);
        
        // Create ADR
        app.start_smart_document_creation();
        assert_eq!(app.ui_state.app_state, AppState::CreatingAdr);
        
        input_title(&mut app, &format!("Test ADR for {}", config_name));
        app.create_adr_from_ticket().await?;
        
        app.load_documents().await?;
        
        // Should have one ADR
        let adr_count: usize = app.ui_state.adr_board.columns.iter()
            .map(|col| col.items.len())
            .sum();
        assert_eq!(adr_count, 1, "ADR creation failed for {} configuration", config_name);
    }

    Ok(())
}

#[tokio::test]
async fn test_backlog_creation_works_in_all_configurations() -> Result<()> {
    // Test that Backlog creation works the same in all configurations
    let configs = vec![
        ("full", FlightLevelConfig::full()),
        ("streamlined", FlightLevelConfig::streamlined()),
        ("direct", FlightLevelConfig::direct()),
    ];
    
    for (config_name, config) in configs {
        let (mut app, _temp_dir) = create_app_with_config_and_docs(config).await?;
        
        // Navigate to Backlog board
        app.jump_to_backlog_board();
        assert_eq!(app.ui_state.current_board, BoardType::Backlog);
        
        // Create Backlog item
        app.start_smart_document_creation();
        assert_eq!(app.ui_state.app_state, AppState::SelectingBacklogCategory);
        
        // Select category and create
        app.confirm_category_selection(); // Should use default General category
        assert_eq!(app.ui_state.app_state, AppState::CreatingDocument);
        
        input_title(&mut app, &format!("Test Backlog Item for {}", config_name));
        app.create_new_document().await?;
        
        app.load_documents().await?;
        
        // Should have one backlog item
        let backlog_count: usize = app.ui_state.backlog_board.columns.iter()
            .map(|col| col.items.len())
            .sum();
        assert_eq!(backlog_count, 1, "Backlog creation failed for {} configuration", config_name);
    }

    Ok(())
}

#[tokio::test]
async fn test_smart_creation_logic_per_configuration() -> Result<()> {
    // Test that smart creation chooses the right creation mode based on configuration and current board
    
    // Full config: Strategy board -> CreatingDocument, Initiative board -> CreatingChildDocument  
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::full()).await?;
    app.jump_to_strategy_board();
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument);
    app.cancel_document_creation(); // Reset state
    
    app.jump_to_initiative_board();
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingChildDocument);
    app.cancel_document_creation();
    
    // Streamlined config: Initiative board -> CreatingDocument (no parent strategies)
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::streamlined()).await?;
    app.jump_to_initiative_board();
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument); // Root creation since no strategies
    app.cancel_document_creation();
    
    // Direct config: Task board -> CreatingDocument (no parent strategies/initiatives)
    let (mut app, _temp_dir) = create_app_with_config_and_docs(FlightLevelConfig::direct()).await?;
    app.jump_to_task_board();
    app.start_smart_document_creation();
    assert_eq!(app.ui_state.app_state, AppState::CreatingDocument); // Root creation since no parents
    app.cancel_document_creation();

    Ok(())
}