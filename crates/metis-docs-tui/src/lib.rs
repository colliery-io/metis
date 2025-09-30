use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;

pub mod app;
pub mod error;
pub mod models;
pub mod services;
mod ui;

use app::state::ConfirmationType;
use app::App;
use error::AppError;
use models::AppState;

/// Run the TUI application
pub async fn run() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create and initialize app
    let mut app = App::new();

    // Run the app with initialization
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

// Helper functions for keyboard input handling

/// Handle keyboard input for normal state
async fn handle_normal_state_input(app: &mut App, key: crossterm::event::KeyEvent) -> Result<bool> {
    // Clear message on any meaningful keystroke (except space which explicitly dismisses)
    if !matches!(key.code, KeyCode::Char(' ')) {
        app.clear_messages();
    }
    
    match key.code {
        KeyCode::Char('q') => return Ok(true), // Signal to quit
        KeyCode::Tab => {
            if app.is_ready() {
                app.next_board();
            }
        }
        KeyCode::BackTab => {
            if app.is_ready() {
                app.previous_board();
            }
        }
        KeyCode::Left => {
            if app.is_ready() {
                app.move_selection_left();
            }
        }
        KeyCode::Right => {
            if app.is_ready() {
                app.move_selection_right();
            }
        }
        KeyCode::Up => {
            if app.is_ready() {
                app.move_selection_up();
            }
        }
        KeyCode::Down => {
            if app.is_ready() {
                app.move_selection_down();
            }
        }
        KeyCode::Char('n') => {
            if app.is_ready() && key.modifiers.contains(KeyModifiers::CONTROL) {
                app.start_smart_document_creation();
            }
        }
        KeyCode::Char('d') | KeyCode::Char('D') => {
            if app.is_ready() && app.get_selected_item().is_some() {
                app.start_delete_confirmation();
            }
        }
        KeyCode::Char('t') | KeyCode::Char('T') => {
            if app.is_ready() && app.get_selected_item().is_some() {
                app.start_transition_confirmation();
            }
        }
        KeyCode::Char('1') => {
            if app.is_ready() {
                app.jump_to_strategy_board();
            }
        }
        KeyCode::Char('2') => {
            if app.is_ready() {
                app.jump_to_initiative_board();
            }
        }
        KeyCode::Char('3') => {
            if app.is_ready() {
                app.jump_to_task_board();
            }
        }
        KeyCode::Char('4') => {
            if app.is_ready() {
                app.jump_to_adr_board();
            }
        }
        KeyCode::Char('5') => {
            if app.is_ready() {
                app.jump_to_backlog_board();
            }
        }
        KeyCode::Char('v') | KeyCode::Char('V') => {
            if app.is_ready() {
                app.view_vision_document();
            }
        }
        KeyCode::Char(' ') => {
            app.ui_state.message_state.clear_message();
        }
        KeyCode::Char('r') | KeyCode::Char('R') => {
            if app.is_ready() && app.get_selected_item().is_some() {
                if let Err(e) = app.archive_selected_document().await {
                    app.error_handler.handle_with_context(
                        AppError::from(e),
                        "Archive operation",
                    );
                }
            }
        }
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            if app.is_ready() {
                if let Err(e) = app.sync_and_reload().await {
                    app.add_error_message(format!("Sync failed: {}", e));
                    app.error_handler.handle_with_context(
                        AppError::from(e),
                        "Sync operation",
                    );
                }
            }
        }
        KeyCode::Enter => {
            if app.is_ready() {
                app.view_selected_ticket();
            }
        }
        _ => {}
    }
    Ok(false) // Don't quit
}

/// Handle keyboard input for document creation states
async fn handle_creation_state_input(app: &mut App, key: crossterm::event::KeyEvent, state: AppState) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_document_creation();
        }
        KeyCode::Enter => {
            let result = match state {
                AppState::CreatingDocument => app.create_new_document().await,
                AppState::CreatingChildDocument => app.create_child_document().await,
                AppState::CreatingAdr => app.create_adr_from_ticket().await,
                _ => return Ok(()),
            };
            
            if let Err(e) = result {
                let context = match state {
                    AppState::CreatingDocument => "Document creation",
                    AppState::CreatingChildDocument => "Child document creation", 
                    AppState::CreatingAdr => "ADR creation",
                    _ => "Creation",
                };
                app.error_handler.handle_with_context(AppError::from(e), context);
            }
        }
        _ => {
            app.handle_key_event(key);
        }
    }
    Ok(())
}

/// Handle keyboard input for confirmation state
async fn handle_confirmation_input(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            match app.ui_state.confirmation_type {
                Some(ConfirmationType::Delete) => {
                    if let Err(e) = app.delete_selected_document().await {
                        app.error_handler.handle_with_context(
                            AppError::from(e),
                            "Document deletion",
                        );
                    }
                }
                Some(ConfirmationType::Transition) => {
                    if let Err(e) = app.transition_selected_document().await {
                        app.error_handler.handle_with_context(
                            AppError::from(e),
                            "Document transition",
                        );
                    }
                }
                None => {}
            }
            app.cancel_confirmation();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.cancel_confirmation();
        }
        _ => {}
    }
    Ok(())
}

/// Handle keyboard input for backlog category selection
fn handle_backlog_category_input(app: &mut App, key: crossterm::event::KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.cancel_document_creation();
        }
        KeyCode::Up => {
            app.move_category_selection_up();
        }
        KeyCode::Down => {
            app.move_category_selection_down();
        }
        KeyCode::Enter => {
            app.confirm_category_selection();
        }
        _ => {}
    }
}

/// Handle keyboard input for content editing
async fn handle_content_editing_input(app: &mut App, key: crossterm::event::KeyEvent) -> Result<()> {
    match key.code {
        KeyCode::Esc => {
            app.cancel_content_editing();
        }
        KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            if let Err(e) = app.save_content_edit().await {
                app.error_handler.handle_with_context(AppError::from(e), "Document save");
            }
            app.cancel_content_editing();
        }
        _ => {
            if let Some(ref mut textarea) = app.ui_state.strategy_editor {
                use tui_textarea::Input;
                let input = Input::from(key);
                textarea.input(input);
            }
        }
    }
    Ok(())
}

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut initialization_done = false;

    loop {
        // Clear expired messages on each loop iteration
        app.clear_expired_messages();

        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle initialization
        if !initialization_done {
            if let Err(e) = app.initialize().await {
                app.add_error_message("Failed to initialize application".to_string());
                app.error_handler
                    .handle_with_context(AppError::from(e), "Initialization");
            }
            initialization_done = true;
        }

        // Handle input events with timeout
        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                let current_state = app.app_state();
                
                match current_state {
                    AppState::Normal => {
                        if handle_normal_state_input(app, key).await? {
                            return Ok(()); // Quit requested
                        }
                    }
                    AppState::CreatingDocument => {
                        handle_creation_state_input(app, key, AppState::CreatingDocument).await?;
                    }
                    AppState::CreatingChildDocument => {
                        handle_creation_state_input(app, key, AppState::CreatingChildDocument).await?;
                    }
                    AppState::CreatingAdr => {
                        handle_creation_state_input(app, key, AppState::CreatingAdr).await?;
                    }
                    AppState::Confirming => {
                        handle_confirmation_input(app, key).await?;
                    }
                    AppState::SelectingBacklogCategory => {
                        handle_backlog_category_input(app, key);
                    }
                    AppState::EditingContent => {
                        handle_content_editing_input(app, key).await?;
                    }
                }
            }
        }
    }
}
