use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use metis_core::domain::documents::types::DocumentType;

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

pub fn draw_creation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 60;
    let dialog_height = 12;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    
    let dialog_area = Rect {
        x: x,
        y: y,
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
            Constraint::Length(5), // Input field - make it taller
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Instructions
        ])
        .margin(2)
        .split(dialog_area);
    
    // Dialog title
    let title = Paragraph::new("Create New Strategy")
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD));
    f.render_widget(title, dialog_chunks[0]);
    
    // Input field with cursor
    let input_value = app.ui_state.input_title.value();
    let cursor_pos = app.ui_state.input_title.cursor();
    
    let displayed_text = if input_value.is_empty() {
        "█".to_string() // Show cursor when empty
    } else {
        let mut chars: Vec<char> = input_value.chars().collect();
        if cursor_pos <= chars.len() {
            chars.insert(cursor_pos, '█'); // Insert block cursor at cursor position
        }
        chars.into_iter().collect()
    };
    
    let input_widget = Paragraph::new(displayed_text)
        .style(Style::default().fg(Color::White))
        .block(Block::default()
            .borders(Borders::ALL)
            .title(format!("Strategy Title ({}/30)", input_value.len()))
            .style(Style::default().fg(Color::Cyan)));
    f.render_widget(input_widget, dialog_chunks[2]);
    
    // Instructions
    let instructions = Paragraph::new("Enter a title for your new strategy.\nPress Enter to create or Escape to cancel.")
        .style(Style::default().fg(Color::Gray))
        .wrap(Wrap { trim: true });
    f.render_widget(instructions, dialog_chunks[4]);
}

pub fn draw_delete_confirmation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 50;
    let dialog_height = 8;
    let x = (area.width.saturating_sub(dialog_width)) / 2;
    let y = (area.height.saturating_sub(dialog_height)) / 2;
    
    let dialog_area = Rect {
        x: x,
        y: y,
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
    
    // Dialog title
    let title = Paragraph::new("Delete Document")
        .style(Style::default().fg(Color::Red).add_modifier(Modifier::BOLD));
    f.render_widget(title, dialog_chunks[0]);
    
    // Confirmation message
    let message = if let Some(selected_item) = &app.get_selected_item() {
        format!("Are you sure you want to delete:\n\n\"{}\"", selected_item.title())
    } else {
        "Are you sure you want to delete this document?".to_string()
    };
    
    let message_widget = Paragraph::new(message)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true });
    f.render_widget(message_widget, dialog_chunks[2]);
    
    // Instructions
    let instructions = Paragraph::new("Y: Yes, delete | N: Cancel")
        .style(Style::default().fg(Color::Gray));
    f.render_widget(instructions, dialog_chunks[4]);
}

pub fn draw_child_creation_dialog(f: &mut Frame, app: &App, area: Rect) {
    let popup_area = centered_rect(50, 30, area);
    f.render_widget(Clear, popup_area);
    
    // Determine what type of child document we're creating
    let (child_type, parent_type) = if let Some(selected_item) = app.get_selected_item() {
        match selected_item.doc_type() {
            DocumentType::Strategy => ("Initiative", "Strategy"),
            DocumentType::Initiative => ("Task", "Initiative"),
            _ => ("Document", "Document"),
        }
    } else {
        ("Document", "Document")
    };
    
    let parent_title = if let Some(selected_item) = app.get_selected_item() {
        selected_item.title()
    } else {
        "Unknown"
    };
    
    let title = format!("Create {} for {}", child_type, parent_type);
    
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Length(1), // Parent info
            Constraint::Length(5), // Input field
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Instructions
        ])
        .split(popup_area.inner(Margin { horizontal: 1, vertical: 1 }));

    // Title
    let title_widget = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .alignment(Alignment::Center);
    f.render_widget(title_widget, chunks[0]);

    // Parent info
    let parent_info = format!("Parent: {}", parent_title);
    let parent_widget = Paragraph::new(parent_info)
        .style(Style::default().fg(Color::Gray));
    f.render_widget(parent_widget, chunks[1]);

    // Input field - make sure it's tall enough to show text
    let input_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!("{} Title", child_type));
    
    let input_widget = Paragraph::new(app.ui_state.input_title.value())
        .block(input_block)
        .style(Style::default().fg(Color::White));
    f.render_widget(input_widget, chunks[2]);

    // Show cursor
    let cursor_x = chunks[2].x + app.ui_state.input_title.visual_cursor() as u16 + 1;
    let cursor_y = chunks[2].y + 1;
    f.set_cursor_position((cursor_x, cursor_y));

    // Instructions
    let instructions = format!("Enter: Create {} | Esc: Cancel | Type to enter title", child_type);
    let instructions_widget = Paragraph::new(instructions)
        .style(Style::default().fg(Color::Gray))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(instructions_widget, chunks[4]);

    // Border
    let border = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan));
    f.render_widget(border, popup_area);
}