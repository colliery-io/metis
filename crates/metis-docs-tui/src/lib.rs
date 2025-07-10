use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

mod models;
mod app;
mod ui;
mod services;
mod error;

use app::App;
use models::AppState;
use app::state::ConfirmationType;
use error::AppError;

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

async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    let mut initialization_done = false;

    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle initialization
        if !initialization_done {
            if let Err(e) = app.initialize().await {
                app.error_handler.handle_with_context(AppError::from(e), "Initialization");
            }
            initialization_done = true;
        }

        // Handle input events with timeout
        if crossterm::event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                match app.app_state() {
                    AppState::Normal => {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
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
                                if app.is_ready() {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        // Ctrl+N: Create child document for selected item
                                        app.start_child_document_creation();
                                    } else {
                                        // N: Create new document
                                        app.start_document_creation();
                                    }
                                }
                            }
                            KeyCode::Char('a') => {
                                if app.is_ready() && key.modifiers.contains(KeyModifiers::CONTROL) {
                                    // Ctrl+A: Start ADR creation dialog
                                    app.start_adr_creation();
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
                            KeyCode::Char('v') | KeyCode::Char('V') => {
                                if app.is_ready() {
                                    app.view_vision_document();
                                }
                            }
                            KeyCode::Enter => {
                                if app.is_ready() {
                                    app.view_selected_ticket();
                                }
                            }
                            _ => {}
                        }
                    }
                    AppState::CreatingDocument => {
                        match key.code {
                            KeyCode::Esc => {
                                app.cancel_document_creation();
                            }
                            KeyCode::Enter => {
                                if let Err(e) = app.create_new_document().await {
                                    app.error_handler.handle_with_context(AppError::from(e), "Document creation");
                                }
                            }
                            _ => {
                                // Let tui-input handle all other key events
                                app.handle_key_event(key);
                            }
                        }
                    }
                    AppState::CreatingChildDocument => {
                        match key.code {
                            KeyCode::Esc => {
                                app.cancel_document_creation();
                            }
                            KeyCode::Enter => {
                                if let Err(e) = app.create_child_document().await {
                                    app.error_handler.handle_with_context(AppError::from(e), "Child document creation");
                                }
                            }
                            _ => {
                                // Let tui-input handle all other key events
                                app.handle_key_event(key);
                            }
                        }
                    }
                    AppState::CreatingAdr => {
                        match key.code {
                            KeyCode::Esc => {
                                app.cancel_document_creation();
                            }
                            KeyCode::Enter => {
                                if let Err(e) = app.create_adr_from_ticket().await {
                                    app.error_handler.handle_with_context(AppError::from(e), "ADR creation");
                                }
                            }
                            _ => {
                                // Let tui-input handle all other key events
                                app.handle_key_event(key);
                            }
                        }
                    }
                    AppState::Confirming => {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                match app.ui_state.confirmation_type {
                                    Some(ConfirmationType::Delete) => {
                                        if let Err(e) = app.delete_selected_document().await {
                                            app.error_handler.handle_with_context(AppError::from(e), "Document deletion");
                                        }
                                    }
                                    Some(ConfirmationType::Transition) => {
                                        if let Err(e) = app.transition_selected_document().await {
                                            app.error_handler.handle_with_context(AppError::from(e), "Document transition");
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
                    }
                    AppState::EditingContent => {
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
                                // Pass the key event to the textarea
                                if let Some(ref mut textarea) = app.ui_state.strategy_editor {
                                    use tui_textarea::Input;
                                    let input = Input::from(key);
                                    textarea.input(input);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}