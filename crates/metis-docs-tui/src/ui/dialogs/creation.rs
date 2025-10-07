use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::BoardType;
use metis_core::domain::documents::types::DocumentType;

pub fn draw_creation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 60;
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
            Constraint::Length(5), // Input field - make it taller
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Instructions
        ])
        .margin(2)
        .split(dialog_area);

    // Dialog title - dynamic based on current board
    let doc_type_name = match app.ui_state.current_board {
        BoardType::Strategy => "Strategy",
        BoardType::Initiative => "Initiative",
        BoardType::Task => "Task",
        BoardType::Adr => "ADR",
        BoardType::Backlog => "Backlog Item",
    };

    let title = Paragraph::new(format!("Create New {}", doc_type_name)).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
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
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "{} Title ({}/30)",
                    doc_type_name,
                    input_value.len()
                ))
                .style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(input_widget, dialog_chunks[2]);

    // Instructions
    let instructions = Paragraph::new(format!(
        "Enter a title for your new {}.\nPress Enter to create or Escape to cancel.",
        doc_type_name.to_lowercase()
    ))
    .style(Style::default().fg(Color::Gray))
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, dialog_chunks[4]);
}

pub fn draw_child_creation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 60;
    let dialog_height = 15;
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
            Constraint::Length(2), // Parent info
            Constraint::Length(1), // Spacer
            Constraint::Length(5), // Input field
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Instructions
        ])
        .margin(2)
        .split(dialog_area);

    // Determine child document type based on current selection
    let (child_doc_type, parent_type) = if let Some(selected_item) = app.get_selected_item() {
        match selected_item.doc_type() {
            DocumentType::Strategy => ("Initiative", "Strategy"),
            DocumentType::Initiative => ("Task", "Initiative"),
            _ => ("Document", "Parent"),
        }
    } else {
        ("Document", "Parent")
    };

    let title = Paragraph::new(format!("Create New {}", child_doc_type)).style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, dialog_chunks[0]);

    // Parent info
    let parent_info = if let Some(selected_item) = app.get_selected_item() {
        format!("Parent {}: {}", parent_type, selected_item.title())
    } else {
        "No parent selected".to_string()
    };

    let parent_widget = Paragraph::new(parent_info)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Parent Document")
                .style(Style::default().fg(Color::Yellow)),
        );
    f.render_widget(parent_widget, dialog_chunks[2]);

    // Input field with cursor
    let input_value = app.ui_state.input_title.value();
    let cursor_pos = app.ui_state.input_title.cursor();

    let displayed_text = if input_value.is_empty() {
        "█".to_string()
    } else {
        let mut chars: Vec<char> = input_value.chars().collect();
        if cursor_pos <= chars.len() {
            chars.insert(cursor_pos, '█');
        }
        chars.into_iter().collect()
    };

    let input_widget = Paragraph::new(displayed_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(
                    "{} Title ({}/30)",
                    child_doc_type,
                    input_value.len()
                ))
                .style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(input_widget, dialog_chunks[4]);

    // Instructions
    let instructions = Paragraph::new(
        format!("Enter a title for your new {}.\nThis will be created under the selected {}.\nPress Enter to create or Escape to cancel.", 
        child_doc_type.to_lowercase(), parent_type.to_lowercase()),
    )
    .style(Style::default().fg(Color::Gray))
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, dialog_chunks[6]);
}

pub fn draw_adr_creation_dialog(f: &mut Frame, app: &App, area: Rect) {
    // Create a centered dialog
    let dialog_width = 60;
    let dialog_height = 16;
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
            Constraint::Length(3), // Context info
            Constraint::Length(1), // Spacer
            Constraint::Length(5), // Input field
            Constraint::Length(1), // Spacer
            Constraint::Min(1),    // Instructions
        ])
        .margin(2)
        .split(dialog_area);

    let title = Paragraph::new("Create Architecture Decision Record").style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(title, dialog_chunks[0]);

    // Context info
    let context_info = if let Some(selected_item) = app.get_selected_item() {
        format!(
            "Context will be extracted from:\n{}: {}",
            selected_item.doc_type(),
            selected_item.title()
        )
    } else {
        "No context document selected.\nADR will be created without context.".to_string()
    };

    let context_widget = Paragraph::new(context_info)
        .style(Style::default().fg(Color::Yellow))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Context Source")
                .style(Style::default().fg(Color::Yellow)),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(context_widget, dialog_chunks[2]);

    // Input field with cursor
    let input_value = app.ui_state.input_title.value();
    let cursor_pos = app.ui_state.input_title.cursor();

    let displayed_text = if input_value.is_empty() {
        "█".to_string()
    } else {
        let mut chars: Vec<char> = input_value.chars().collect();
        if cursor_pos <= chars.len() {
            chars.insert(cursor_pos, '█');
        }
        chars.into_iter().collect()
    };

    let input_widget = Paragraph::new(displayed_text)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!("ADR Title ({}/50)", input_value.len()))
                .style(Style::default().fg(Color::Cyan)),
        );
    f.render_widget(input_widget, dialog_chunks[4]);

    // Instructions
    let instructions = Paragraph::new(
        "Enter a title for your new ADR.\nThe ADR will be automatically numbered.\nPress Enter to create or Escape to cancel."
    )
    .style(Style::default().fg(Color::Gray))
    .wrap(Wrap { trim: true });
    f.render_widget(instructions, dialog_chunks[6]);
}
