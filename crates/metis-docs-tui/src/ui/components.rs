// Reusable UI components for the TUI

use ratatui::{
    layout::Rect,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::KanbanItem;

/// Renders a ticket/kanban item as a bordered component
pub fn draw_ticket(f: &mut Frame, item: &KanbanItem, area: Rect, is_selected: bool) {
    let base_style = if is_selected {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::White)
    };
    
    let meta_style = if is_selected {
        Style::default().fg(Color::Black).bg(Color::Yellow)
    } else {
        Style::default().fg(Color::Gray)
    };
    
    let mut lines = Vec::new();
    
    // Line 1: Risk/Complexity + ID + Title
    let mut title_line = vec![
        Span::styled(format!(" [{}] ", item.id()), meta_style),
        Span::styled(format!("{}", item.title()), base_style),
    ];

    if let Some(ref risk_complexity) = item.risk_complexity {
        title_line.insert(0, Span::styled(format!("{} ", risk_complexity), meta_style));
    }

    lines.push(Line::from(title_line));
    
    // Line 2: Blocked By (if any)
    if !item.blocked_by().is_empty() {
        let info_line = vec![
            Span::styled(
                format!("🚫 {}", item.blocked_by().join(", ")),
                if is_selected { 
                    Style::default().fg(Color::Red).bg(Color::Yellow) 
                } else { 
                    Style::default().fg(Color::Red) 
                }
            )
        ];
        lines.push(Line::from(info_line));
    }
    
    // Line 3: Parent context (if any)
    if let Some(ref parent_title) = item.parent_title() {
        lines.push(Line::from(vec![
            Span::styled(format!("→ {}", parent_title), meta_style),
        ]));
    }
    
    // Line 4: Description prelude (if any)
    if !item.prelude.is_empty() {
        lines.push(Line::from(vec![
            Span::styled(&item.prelude, meta_style),
        ]));
    }
    
    // Create the ticket block with border
    let ticket_block = Block::default()
        .borders(Borders::ALL)
        .border_style(if is_selected { 
            Style::default().fg(Color::Yellow) 
        } else { 
            Style::default().fg(Color::Gray) 
        });
    
    let paragraph = Paragraph::new(lines)
        .block(ticket_block)
        .wrap(Wrap { trim: true });
    
    f.render_widget(paragraph, area);
}

/// Helper function to create styled spans with consistent coloring
pub fn styled_span(text: &str, is_selected: bool) -> Span {
    if is_selected {
        Span::styled(text, Style::default().fg(Color::Black).bg(Color::Yellow))
    } else {
        Span::styled(text, Style::default().fg(Color::White))
    }
}

/// Helper function to create metadata styled spans
pub fn meta_span(text: &str, is_selected: bool) -> Span {
    if is_selected {
        Span::styled(text, Style::default().fg(Color::Black).bg(Color::Yellow))
    } else {
        Span::styled(text, Style::default().fg(Color::Gray))
    }
}

/// Helper function to create error/warning styled spans
pub fn error_span(text: &str, is_selected: bool) -> Span {
    if is_selected {
        Span::styled(text, Style::default().fg(Color::Red).bg(Color::Yellow))
    } else {
        Span::styled(text, Style::default().fg(Color::Red))
    }
}