pub mod components;
pub mod dialogs;
pub mod rendering;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::*;

pub use dialogs::*;
pub use rendering::*;

// Helper functions for UI rendering

/// Determine if we should show messages based on app state and message presence
fn should_show_messages(app: &App) -> bool {
    let has_messages = app.ui_state.message_state.has_messages();
    let in_dialog = matches!(
        app.app_state(),
        AppState::CreatingDocument | AppState::CreatingChildDocument | 
        AppState::CreatingAdr | AppState::Confirming | AppState::SelectingBacklogCategory
    );
    has_messages && !in_dialog
}

/// Calculate message area height based on app state
fn calculate_message_height(app: &App) -> u16 {
    if should_show_messages(app) { 3 } else { 0 }
}

/// Render the appropriate main content based on app state
fn draw_main_content(f: &mut Frame, app: &App, area: Rect) {
    match app.app_state() {
        AppState::EditingContent => {
            draw_content_edit_form(f, app, area);
        }
        _ => {
            if !app.is_ready() {
                draw_loading(f, app, area);
            } else {
                draw_kanban_board(f, app, area);
            }
            
            // Render any modal dialogs on top
            draw_modal_dialogs(f, app, f.area());
        }
    }
}

/// Render modal dialogs based on current app state
fn draw_modal_dialogs(f: &mut Frame, app: &App, area: Rect) {
    match app.app_state() {
        AppState::CreatingDocument => draw_creation_dialog(f, app, area),
        AppState::CreatingChildDocument => draw_child_creation_dialog(f, app, area),
        AppState::CreatingAdr => draw_adr_creation_dialog(f, app, area),
        AppState::SelectingBacklogCategory => draw_backlog_category_selection_dialog(f, app, area),
        AppState::Confirming => draw_confirmation_dialog(f, app, area),
        _ => {} // No modal for other states
    }
}

/// Generate footer text based on current app state and board
fn get_footer_text(app: &App) -> &'static str {
    match app.app_state() {
        AppState::CreatingDocument => "Enter: Create | Escape: Cancel | Type to enter title",
        AppState::CreatingChildDocument => "Enter: Create | Escape: Cancel | Type to enter title",
        AppState::CreatingAdr => "Enter: Create ADR | Escape: Cancel | Type to enter title",
        AppState::SelectingBacklogCategory => "↑↓: Navigate | Enter: Select | Escape: Cancel",
        AppState::EditingContent => "Ctrl+S: Save | Esc: Cancel | Type to edit document content",
        AppState::Confirming => get_confirmation_footer_text(app),
        _ => get_normal_state_footer_text(app),
    }
}

/// Get footer text for confirmation dialogs
fn get_confirmation_footer_text(app: &App) -> &'static str {
    match app.ui_state.confirmation_type {
        Some(crate::app::state::ConfirmationType::Delete) => "Y: Yes, delete | N: Cancel",
        Some(crate::app::state::ConfirmationType::Transition) => "Y: Yes, transition | N: Cancel",
        None => "Y: Yes | N: Cancel",
    }
}

/// Get footer text for normal app state based on current board
fn get_normal_state_footer_text(app: &App) -> &'static str {
    if !app.is_ready() {
        return "q: Quit";
    }
    
    use crate::models::BoardType;
    match app.ui_state.current_board {
        BoardType::Strategy => "↑↓←→: Navigate | 1-5: Jump to board | v: Vision | Tab: Switch | Enter: Edit | Ctrl+n: Create Initiative | d: Del | r: Archive | t: Trans | y: Sync | q: Quit",
        BoardType::Initiative => "↑↓←→: Navigate | 1-5: Jump to board | v: Vision | Tab: Switch | Enter: Edit | Ctrl+n: Create Task | d: Del | r: Archive | t: Trans | y: Sync | q: Quit",
        BoardType::Task => "↑↓←→: Navigate | 1-5: Jump to board | v: Vision | Tab: Switch | Enter: Edit | d: Del | r: Archive | t: Trans | y: Sync | q: Quit",
        BoardType::Adr => "↑↓←→: Navigate | 1-5: Jump to board | v: Vision | Tab: Switch | Enter: Edit | Ctrl+n: Create ADR | d: Del | r: Archive | t: Trans | y: Sync | q: Quit",
        BoardType::Backlog => "↑↓←→: Navigate | 1-5: Jump to board | v: Vision | Tab: Switch | Enter: Edit | Ctrl+n: Create Backlog Item | d: Del | r: Archive | t: Trans | y: Sync | q: Quit",
    }
}

/// Get board title for header
fn get_board_title(app: &App) -> String {
    if app.is_ready() {
        format!(
            "{} Board",
            match app.ui_state.current_board {
                BoardType::Strategy => "Strategy",
                BoardType::Initiative => "Initiative",
                BoardType::Task => "Task",
                BoardType::Adr => "ADR",
                BoardType::Backlog => "Backlog",
            }
        )
    } else {
        "Initializing...".to_string()
    }
}

pub fn draw(f: &mut Frame, app: &App) {
    let message_height = calculate_message_height(app);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),          // Header
            Constraint::Min(0),             // Main content  
            Constraint::Length(message_height), // Messages
            Constraint::Length(3),          // Footer
        ])
        .split(f.area());

    // Header
    draw_header(f, app, chunks[0]);

    // Main content
    draw_main_content(f, app, chunks[1]);

    // Messages area - only show when appropriate
    if should_show_messages(app) {
        draw_message_area(f, &app.ui_state.message_state, chunks[2]);
    }

    // Footer  
    draw_footer(f, app, chunks[3]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = get_board_title(app);

    let header = Paragraph::new(title)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL))
        .alignment(Alignment::Center);
    f.render_widget(header, area);
}

fn draw_loading(f: &mut Frame, app: &App, area: Rect) {
    let mut status_lines = vec![];

    if app.core_state.workspace_valid {
        status_lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Workspace found"),
        ]));
    } else {
        status_lines.push(Line::from(vec![
            Span::styled("○ ", Style::default().fg(Color::Yellow)),
            Span::raw("Checking workspace..."),
        ]));
    }

    if app.core_state.sync_in_progress {
        status_lines.push(Line::from(vec![
            Span::styled("⟳ ", Style::default().fg(Color::Cyan)),
            Span::raw("Syncing database..."),
        ]));
    } else if app.core_state.sync_complete {
        status_lines.push(Line::from(vec![
            Span::styled("✓ ", Style::default().fg(Color::Green)),
            Span::raw("Database synchronized"),
        ]));
    } else if app.core_state.workspace_valid {
        status_lines.push(Line::from(vec![
            Span::styled("○ ", Style::default().fg(Color::Yellow)),
            Span::raw("Synchronizing database..."),
        ]));
    }

    let loading = Paragraph::new(status_lines)
        .style(Style::default().fg(Color::White))
        .block(Block::default().title("Loading").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(loading, area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect) {
    let footer_text = get_footer_text(app);

    let footer = Paragraph::new(footer_text)
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

fn draw_content_edit_form(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref textarea) = app.ui_state.strategy_editor {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Title
                Constraint::Min(1),    // Content editor
            ])
            .split(area);

        // Title
        let title = if app.ui_state.editing_vision_path.is_some() {
            "Editing Vision Document".to_string()
        } else if let Some(selected_item) = app.get_viewed_ticket() {
            format!(
                "Editing {}: {}",
                selected_item.doc_type(),
                selected_item.title()
            )
        } else {
            "Editing Document".to_string()
        };

        let title_widget = Paragraph::new(title)
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(title_widget, chunks[0]);

        // Content editor - render the TextArea
        f.render_widget(textarea, chunks[1]);
    }
}
