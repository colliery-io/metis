use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;

pub fn draw_confirmation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 50;
    let dialog_height = 8;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;

    let dialog_area = Rect {
        x,
        y,
        width: dialog_width,
        height: dialog_height,
    };

    // Clear the dialog area
    f.render_widget(
        Block::default()
            .style(Style::default().bg(Color::Black))
            .borders(Borders::ALL),
        dialog_area,
    );

    // Create layout for the dialog content
    let dialog_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Spacer
            Constraint::Min(2),    // Message
            Constraint::Length(1), // Spacer
            Constraint::Length(1), // Instructions
        ])
        .margin(2)
        .split(dialog_area);

    // Get confirmation details based on type
    let (title_text, title_color, message_text, instructions_text) =
        match app.ui_state.confirmation_type {
            Some(crate::app::state::ConfirmationType::Delete) => {
                let message = if let Some(selected_item) = &app.get_selected_item() {
                    format!(
                        "Are you sure you want to delete:\n\n\"{}\"",
                        selected_item.title()
                    )
                } else {
                    "Are you sure you want to delete this document?".to_string()
                };
                (
                    "Delete Document",
                    Color::Red,
                    message,
                    "Y: Yes, delete | N: Cancel",
                )
            }
            Some(crate::app::state::ConfirmationType::Transition) => {
                let message = if let Some(selected_item) = &app.get_selected_item() {
                    format!(
                        "Are you sure you want to transition:\n\n\"{}\" to the next phase?",
                        selected_item.title()
                    )
                } else {
                    "Are you sure you want to transition this document?".to_string()
                };
                (
                    "Transition Document",
                    Color::Yellow,
                    message,
                    "Y: Yes, transition | N: Cancel",
                )
            }
            None => (
                "Confirm Action",
                Color::White,
                "Are you sure?".to_string(),
                "Y: Yes | N: Cancel",
            ),
        };

    // Dialog title
    let title = Paragraph::new(title_text).style(
        Style::default()
            .fg(title_color)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, dialog_chunks[0]);

    // Confirmation message
    let message_widget = Paragraph::new(message_text)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(message_widget, dialog_chunks[2]);

    // Instructions
    let instructions = Paragraph::new(instructions_text).style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, dialog_chunks[4]);
}
