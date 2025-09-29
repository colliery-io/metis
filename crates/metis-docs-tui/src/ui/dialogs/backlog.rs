use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::app::state::BacklogCategory;

pub fn draw_backlog_category_selection_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 30;
    let dialog_height = 12;
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
            Constraint::Length(4), // Category options (4 categories, 1 line each)
            Constraint::Length(1), // Spacer
            Constraint::Length(2), // Instructions (allow 2 lines for wrapping)
        ])
        .margin(2)
        .split(dialog_area);

    // Dialog title
    let title = Paragraph::new("Select Category").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, dialog_chunks[0]);

    // Category options
    let categories = [
        BacklogCategory::General,
        BacklogCategory::Bug,
        BacklogCategory::Feature,
        BacklogCategory::TechDebt,
    ];

    for (i, category) in categories.iter().enumerate() {
        let is_selected = i == app.ui_state.backlog_category_selection;
        
        let style = if is_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::White)
        };

        let option_area = Rect {
            x: dialog_chunks[2].x,
            y: dialog_chunks[2].y + i as u16,
            width: dialog_chunks[2].width,
            height: 1,
        };

        // Category name only
        let name_paragraph = Paragraph::new(format!("  {}", category.display_name()))
            .style(style);
        f.render_widget(name_paragraph, option_area);
    }

    // Instructions
    let instructions = Paragraph::new(
        "↑↓: Navigate | Enter: Select | Esc: Cancel",
    )
    .style(Style::default().fg(Color::Gray))
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, dialog_chunks[4]);
}