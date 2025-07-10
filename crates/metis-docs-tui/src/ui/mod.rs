pub mod board;
pub mod components;
pub mod dialog;

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::*;

pub use board::*;
pub use dialog::*;

pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Main content
            Constraint::Length(3), // Footer
        ])
        .split(f.area());

    // Header
    draw_header(f, app, chunks[0]);

    // Main content
    match app.app_state() {
        AppState::EditingContent => {
            draw_content_edit_form(f, app, chunks[1]);
        }
        _ => {
            if let Some(error) = app.error_message() {
                draw_error(f, &error, chunks[1]);
            } else if !app.is_ready() {
                draw_loading(f, app, chunks[1]);
            } else {
                draw_kanban_board(f, app, chunks[1]);
            }

            // Overlays for modals
            if app.app_state() == &AppState::CreatingDocument {
                draw_creation_dialog(f, app, f.area());
            } else if app.app_state() == &AppState::CreatingChildDocument {
                draw_child_creation_dialog(f, app, f.area());
            } else if app.app_state() == &AppState::CreatingAdr {
                draw_adr_creation_dialog(f, app, f.area());
            } else if app.app_state() == &AppState::Confirming {
                draw_confirmation_dialog(f, app, f.area());
            }
        }
    }

    // Footer
    draw_footer(f, app, chunks[2]);
}

fn draw_header(f: &mut Frame, app: &App, area: Rect) {
    let title = if app.is_ready() {
        format!(
            "{} Board",
            match app.ui_state.current_board {
                BoardType::Strategy => "Strategy",
                BoardType::Initiative => "Initiative",
                BoardType::Task => "Task",
                BoardType::Adr => "ADR",
            }
        )
    } else {
        "Initializing...".to_string()
    };

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

fn draw_error(f: &mut Frame, error: &str, area: Rect) {
    let error_widget = Paragraph::new(error)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().title("Error").borders(Borders::ALL))
        .wrap(Wrap { trim: true });
    f.render_widget(error_widget, area);
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

    if app.core_state.sync_complete {
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
    let footer_text = match app.app_state() {
        AppState::CreatingDocument => "Enter: Create | Escape: Cancel | Type to enter title",
        AppState::CreatingChildDocument => "Enter: Create | Escape: Cancel | Type to enter title",
        AppState::CreatingAdr => "Enter: Create ADR | Escape: Cancel | Type to enter title",
        AppState::EditingContent => "Ctrl+S: Save | Esc: Cancel | Type to edit document content",
        AppState::Confirming => match app.ui_state.confirmation_type {
            Some(crate::app::state::ConfirmationType::Delete) => "Y: Yes, delete | N: Cancel",
            Some(crate::app::state::ConfirmationType::Transition) => {
                "Y: Yes, transition | N: Cancel"
            }
            None => "Y: Yes | N: Cancel",
        },
        _ => {
            if app.is_ready() {
                "↑↓←→: Navigate | 1-4: Jump to board | v: Vision | Tab: Switch | Enter: Edit | n: New | Ctrl+n: Child | Ctrl+a: ADR | d: Del | t: Trans | q: Quit"
            } else {
                "q: Quit"
            }
        }
    };

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
