use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::app::App;
use crate::models::*;

pub fn draw_ticket_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(ticket) = app.get_viewed_ticket() {
        // Create main layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Header
                Constraint::Min(0),     // Content
            ])
            .split(area);

        draw_detail_header(f, ticket, chunks[0]);
        draw_detail_content(f, ticket, chunks[1]);
    }
}

fn draw_detail_header(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    let mut title_spans = vec![
        Span::styled(format!("[{}] ", ticket.id()), Style::default().fg(Color::Cyan)),
        Span::styled(ticket.title(), Style::default().fg(Color::White).add_modifier(Modifier::BOLD)),
    ];

    // Add risk/complexity indicator
    if let Some(ref risk_complexity) = ticket.risk_complexity {
        title_spans.insert(0, Span::styled(format!("{} ", risk_complexity), Style::default().fg(Color::Yellow)));
    }

    let header = Paragraph::new(Line::from(title_spans))
        .block(Block::default().borders(Borders::ALL).title("Document Details"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(header, area);
}

fn draw_detail_content(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    let inner_area = area.inner(Margin { horizontal: 1, vertical: 1 });
    
    // Split content area into sections
    let mut constraints = vec![
        Constraint::Length(3), // Metadata section
    ];
    
    // Add constraints for optional sections
    if !ticket.blocked_by().is_empty() {
        constraints.push(Constraint::Length(3)); // Blocked by section
    }
    
    if ticket.parent_title().is_some() {
        constraints.push(Constraint::Length(3)); // Parent section
    }
    
    // Always show content section
    constraints.push(Constraint::Min(5)); // Content section

    let content_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner_area);

    let mut chunk_index = 0;

    // Metadata section
    draw_metadata_section(f, ticket, content_chunks[chunk_index]);
    chunk_index += 1;

    // Blocked by section (if applicable)
    if !ticket.blocked_by().is_empty() {
        draw_blocked_section(f, ticket, content_chunks[chunk_index]);
        chunk_index += 1;
    }

    // Parent section (if applicable)
    if ticket.parent_title().is_some() {
        draw_parent_section(f, ticket, content_chunks[chunk_index]);
        chunk_index += 1;
    }

    // Content section
    if chunk_index < content_chunks.len() {
        draw_description_section(f, ticket, content_chunks[chunk_index]);
    }
}

fn draw_metadata_section(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    let metadata_lines = vec![
        Line::from(vec![
            Span::styled("Type: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{:?}", ticket.doc_type()), Style::default().fg(Color::White)),
            Span::raw("  "),
            Span::styled("Phase: ", Style::default().fg(Color::Gray)),
            Span::styled(ticket.phase(), Style::default().fg(Color::White)),
        ]),
    ];

    let metadata = Paragraph::new(metadata_lines)
        .block(Block::default().borders(Borders::ALL).title("Metadata"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(metadata, area);
}

fn draw_blocked_section(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    let blocked_text = format!("🚫 Blocked by: {}", ticket.blocked_by().join(", "));
    
    let blocked = Paragraph::new(blocked_text)
        .style(Style::default().fg(Color::Red))
        .block(Block::default().borders(Borders::ALL).title("Dependencies"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(blocked, area);
}

fn draw_parent_section(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    if let Some(ref parent_title) = ticket.parent_title() {
        let parent_text = format!("→ {}", parent_title);
        
        let parent = Paragraph::new(parent_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("Parent Document"))
            .wrap(Wrap { trim: true });
        
        f.render_widget(parent, area);
    }
}

fn draw_description_section(f: &mut Frame, ticket: &KanbanItem, area: Rect) {
    // Get the full content from the document instead of just the prelude
    let content = match &ticket.document {
        DocumentObject::Strategy(strategy) => {
            use metis_core::Document;
            strategy.content().full_content()
        }
        DocumentObject::Initiative(initiative) => {
            use metis_core::Document;
            initiative.content().full_content()
        }
        DocumentObject::Task(task) => {
            use metis_core::Document;
            task.content().full_content()
        }
        DocumentObject::Adr(adr) => {
            use metis_core::Document;
            adr.content().full_content()
        }
    };
    
    let description = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .block(Block::default().borders(Borders::ALL).title("Content"))
        .wrap(Wrap { trim: true });
    
    f.render_widget(description, area);
}

