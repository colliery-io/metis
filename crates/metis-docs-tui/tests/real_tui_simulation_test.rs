use anyhow::Result;
use metis_docs_tui::app::App;
use metis_docs_tui::models::BoardType;

/// Test that simulates exactly what the real TUI does on startup
#[tokio::test]
async fn test_real_tui_initialization_sequence() -> Result<()> {
    // Create app exactly like the real TUI does
    let mut app = App::new();
    
    // Check initial state - should start with Strategy (before config is loaded)
    assert_eq!(app.ui_state.current_board, BoardType::Strategy);
    
    // Call initialize() exactly like the real TUI does
    // This should find the actual workspace and load the real configuration
    if let Err(e) = app.initialize().await {
        app.add_error_message("Failed to initialize application".to_string());
        app.error_handler
            .handle_with_context(metis_docs_tui::error::AppError::from(e), "Initialization");
    }
    
    // After initialization, print what configuration was loaded
    println!("Flight config loaded:");
    println!("  Strategies enabled: {}", app.core_state.flight_config.strategies_enabled);
    println!("  Initiatives enabled: {}", app.core_state.flight_config.initiatives_enabled);
    println!("  Current board: {:?}", app.ui_state.current_board);
    println!("  Workspace dir: {:?}", app.core_state.workspace_dir);
    println!("  App ready: {}", app.is_ready());
    
    // The test itself doesn't assert anything - it's just for debugging
    // We want to see what actually happens during real initialization
    
    Ok(())
}