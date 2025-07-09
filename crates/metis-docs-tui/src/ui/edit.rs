use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::*;

pub fn draw_edit_form(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ref edit_state) = app.editing_state.edit_state {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Form content
                Constraint::Length(3),  // Footer
            ])
            .split(area);

        draw_edit_header(f, edit_state, chunks[0]);
        draw_edit_content(f, edit_state, chunks[1]);
        draw_edit_footer(f, chunks[2]);
    }
}

fn draw_edit_header(f: &mut Frame, edit_state: &EditState, area: Rect) {
    let title_spans = vec![
        Span::styled("Editing: ", Style::default().fg(Color::Cyan)),
        Span::styled(format!("[{}] ", edit_state.original_item_id), Style::default().fg(Color::Gray)),
        Span::styled(&edit_state.original_item_title, Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ];

    let header = Paragraph::new(Line::from(title_spans))
        .block(Block::default().borders(Borders::ALL).title("Edit Document"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(header, area);
}

fn draw_edit_content(f: &mut Frame, edit_state: &EditState, area: Rect) {
    let inner_area = area.inner(Margin { horizontal: 1, vertical: 1 });
    
    // Split content area into form fields
    let form_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Title field
            Constraint::Length(1), // Spacer
            Constraint::Min(6),    // Description field
        ])
        .split(inner_area);

    // Title field
    draw_title_field(f, edit_state, form_chunks[0]);
    
    // Description field
    draw_description_field(f, edit_state, form_chunks[2]);
}

fn draw_title_field(f: &mut Frame, edit_state: &EditState, area: Rect) {
    let is_selected = matches!(edit_state.current_field, EditField::Title);
    
    let border_style = if is_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let text_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let title_text = if edit_state.title.is_empty() {
        if is_selected {
            Span::styled("(enter title here)", Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC))
        } else {
            Span::styled("(empty)", Style::default().fg(Color::Gray))
        }
    } else {
        Span::styled(&edit_state.title, text_style)
    };

    let cursor = if is_selected {
        Span::styled("█", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };

    let content = vec![
        Line::from("Title:"),
        Line::from(vec![title_text, cursor]),
    ];

    let title_field = Paragraph::new(content)
        .block(Block::default().borders(Borders::ALL).border_style(border_style))
        .wrap(Wrap { trim: true });
    
    f.render_widget(title_field, area);
}

fn draw_description_field(f: &mut Frame, edit_state: &EditState, area: Rect) {
    let is_selected = matches!(edit_state.current_field, EditField::Description);
    
    let border_style = if is_selected {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };

    let text_style = if is_selected {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let description_text = if edit_state.description.is_empty() {
        if is_selected {
            "(enter description here)"
        } else {
            "(empty)"
        }
    } else {
        &edit_state.description
    };

    // Split description into lines and add cursor to the end of last line
    let mut lines: Vec<Line> = vec![Line::from("Description:")];
    
    if edit_state.description.is_empty() {
        let placeholder_style = if is_selected {
            Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC)
        } else {
            Style::default().fg(Color::Gray)
        };
        let cursor = if is_selected {
            Span::styled("█", Style::default().fg(Color::Yellow))
        } else {
            Span::raw("")
        };
        lines.push(Line::from(vec![
            Span::styled(description_text, placeholder_style),
            cursor,
        ]));
    } else {
        // Break description into lines for better display
        let desc_lines: Vec<&str> = description_text.lines().collect();
        for (i, line) in desc_lines.iter().enumerate() {
            if i == desc_lines.len() - 1 && is_selected {
                // Add cursor to last line
                let cursor = Span::styled("█", Style::default().fg(Color::Yellow));
                lines.push(Line::from(vec![
                    Span::styled(*line, text_style),
                    cursor,
                ]));
            } else {
                lines.push(Line::from(Span::styled(*line, text_style)));
            }
        }
        
        // If description doesn't end with newline and is selected, add cursor
        if is_selected && !description_text.ends_with('\n') && desc_lines.is_empty() {
            let cursor = Span::styled("█", Style::default().fg(Color::Yellow));
            lines.push(Line::from(cursor));
        }
    }

    let description_field = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).border_style(border_style))
        .wrap(Wrap { trim: true });
    
    f.render_widget(description_field, area);
}

fn draw_edit_footer(f: &mut Frame, area: Rect) {
    let footer = Paragraph::new("Tab/Shift+Tab: Switch fields | Ctrl+S: Save | Esc: Cancel | Type to edit")
        .style(Style::default().fg(Color::Gray))
        .block(Block::default().borders(Borders::ALL));
    
    f.render_widget(footer, area);
}