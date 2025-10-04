use anyhow::Result;
use metis_core::domain::configuration::FlightLevelConfig;
use metis_docs_tui::app::App;
use metis_docs_tui::models::BoardType;
use tempfile::TempDir;

/// Test that simulates the full TUI initialization process including database persistence
#[tokio::test]
async fn test_streamlined_config_persistence_and_loading() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    let init_result = metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Step 1: Save streamlined configuration to database
    {
        let db = metis_core::Database::new(init_result.database_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        
        let streamlined_config = FlightLevelConfig::streamlined();
        config_repo.set_flight_level_config(&streamlined_config).unwrap();
    }

    // Step 2: Create app and manually initialize (avoid workspace detection which finds wrong directory)
    let mut app = App::new();
    
    // Check initial state - should start with Strategy (before config is loaded)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Set the workspace directory to the .metis directory (like real TUI does)
    app.core_state.set_workspace(init_result.metis_dir.clone());
    app.core_state.set_sync_complete();
    
    // Initialize services manually with the .metis directory
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        init_result.metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        init_result.metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(init_result.metis_dir));
    
    // Load flight level configuration (this is what we want to test)
    app.load_flight_config().await?;
    
    // After initialization, should be on Initiative board (streamlined config)
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);
    
    // Verify the configuration was loaded correctly
    assert!(!app.core_state.flight_config.strategies_enabled);
    assert!(app.core_state.flight_config.initiatives_enabled);

    Ok(())
}

#[tokio::test]
async fn test_direct_config_persistence_and_loading() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    let init_result = metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Step 1: Save direct configuration to database
    {
        let db = metis_core::Database::new(init_result.database_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        
        let direct_config = FlightLevelConfig::direct();
        config_repo.set_flight_level_config(&direct_config).unwrap();
    }

    // Step 2: Create app and manually initialize (avoid workspace detection which finds wrong directory)
    let mut app = App::new();
    
    // Check initial state - should start with Strategy (before config is loaded)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Set the workspace directory to the .metis directory (like real TUI does)
    app.core_state.set_workspace(init_result.metis_dir.clone());
    app.core_state.set_sync_complete();
    
    // Initialize services manually with the .metis directory
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        init_result.metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        init_result.metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(init_result.metis_dir));
    
    // Load flight level configuration (this is what we want to test)
    app.load_flight_config().await?;
    
    // After initialization, should be on Task board (direct config)
    assert_eq!(app.ui_state.current_board, BoardType::Task);
    
    // Verify the configuration was loaded correctly
    assert!(!app.core_state.flight_config.strategies_enabled);
    assert!(!app.core_state.flight_config.initiatives_enabled);

    Ok(())
}

#[tokio::test]
async fn test_full_config_persistence_and_loading() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    let init_result = metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Step 1: Save full configuration to database
    {
        let db = metis_core::Database::new(init_result.database_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        
        let full_config = FlightLevelConfig::full();
        config_repo.set_flight_level_config(&full_config).unwrap();
    }

    // Step 2: Create app and manually initialize (avoid workspace detection which finds wrong directory)
    let mut app = App::new();
    
    // Check initial state - should start with Strategy (before config is loaded)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Set the workspace directory to the .metis directory (like real TUI does)
    app.core_state.set_workspace(init_result.metis_dir.clone());
    app.core_state.set_sync_complete();
    
    // Initialize services manually with the .metis directory
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        init_result.metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        init_result.metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(init_result.metis_dir.clone()));
    
    // Load flight level configuration (this is what we want to test)
    app.load_flight_config().await?;
    
    // After initialization, should still be on Strategy board (full config)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Verify the configuration was loaded correctly
    assert!(app.core_state.flight_config.strategies_enabled);
    assert!(app.core_state.flight_config.initiatives_enabled);

    Ok(())
}

#[tokio::test]
async fn test_no_config_in_database_defaults_to_full() -> Result<()> {
    // Create a temporary directory and initialize a metis project
    let temp_dir = TempDir::new()?;
    let project_path = temp_dir.path().to_path_buf();

    // Initialize metis project
    let init_result = metis_core::application::services::workspace::WorkspaceInitializationService::initialize_workspace(
        &project_path,
        "Test Project"
    ).await?;

    // Step 1: Don't save any configuration to database (will use default)

    // Step 2: Create app and manually initialize (avoid workspace detection which finds wrong directory)
    let mut app = App::new();
    
    // Check initial state - should start with Strategy (before config is loaded)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Set the workspace directory to the .metis directory (like real TUI does)
    app.core_state.set_workspace(init_result.metis_dir.clone());
    app.core_state.set_sync_complete();
    
    // Initialize services manually with the .metis directory
    app.document_service = Some(metis_docs_tui::services::DocumentService::new(
        init_result.metis_dir.clone(),
    ));
    app.sync_service = Some(metis_docs_tui::services::SyncService::new(
        init_result.metis_dir.clone(),
    ));
    app.transition_service = Some(metis_docs_tui::services::TransitionService::new(init_result.metis_dir.clone()));
    
    // Load flight level configuration (this is what we want to test)
    app.load_flight_config().await?;
    
    // After initialization, should still be on Strategy board (defaults to full config)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Verify the configuration defaults to full
    assert!(app.core_state.flight_config.strategies_enabled);
    assert!(app.core_state.flight_config.initiatives_enabled);

    Ok(())
}