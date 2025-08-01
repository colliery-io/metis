use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::models::{MessageState, MessageType};

/// Draw the message area at the bottom of the screen
pub fn draw_message_area(f: &mut Frame, message_state: &MessageState, area: Rect) {
    let current_message = match message_state.get_current_message() {
        Some(msg) => msg,
        None => return, // Don't render anything if no message
    };
    
    // Fixed height for single message display
    let required_height = 3; // 1 line content + 2 for borders
    
    if area.height < required_height {
        return; // Not enough space
    }

    // Create message area at the bottom of the provided area
    let message_area = Rect {
        x: area.x,
        y: area.y.saturating_sub(required_height),
        width: area.width,
        height: required_height,
    };

    // Create styled line for the message
    let style = get_message_style(current_message.message_type);
    let message_text = current_message.display_text();
    let content = Line::from(vec![
        Span::styled(message_text, style)
    ]);

    let block = Block::default()
        .borders(Borders::ALL)
        .title("Message")
        .border_style(Style::default().fg(Color::Yellow));

    let paragraph = Paragraph::new(vec![content])
        .block(block)
        .wrap(Wrap { trim: true });

    f.render_widget(paragraph, message_area);
}

fn get_message_style(message_type: MessageType) -> Style {
    match message_type {
        MessageType::Error => Style::default()
            .fg(Color::White)
            .bg(Color::Red)
            .add_modifier(Modifier::BOLD),
        MessageType::Success => Style::default()
            .fg(Color::Black)
            .bg(Color::Green),
        MessageType::Warning => Style::default()
            .fg(Color::Black)
            .bg(Color::Yellow),
        MessageType::Info => Style::default()
            .fg(Color::White)
            .bg(Color::Blue),
    }
}