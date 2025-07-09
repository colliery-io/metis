use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Margin},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame,
};

use crate::app::App;
use crate::models::*;
use crate::ui::components::draw_ticket;

pub fn draw_kanban_board(f: &mut Frame, app: &App, area: Rect) {
    let board = app.get_current_board();
    let current_board = app.ui_state.current_board;
    let selection = app.selection_state.get_current_selection(current_board);
    
    // Create horizontal layout for columns
    let column_count = board.columns.len();
    let constraints: Vec<Constraint> = (0..column_count)
        .map(|_| Constraint::Percentage(100 / column_count as u16))
        .collect();
    
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    // Draw each column
    for (i, column) in board.columns.iter().enumerate() {
        if i < chunks.len() {
            let is_selected_column = i == selection.0;
            let selected_item = if is_selected_column { Some(selection.1) } else { None };
            draw_kanban_column(f, column, chunks[i], selected_item);
        }
    }
}

fn draw_kanban_column(f: &mut Frame, column: &KanbanColumn, area: Rect, selected_item: Option<usize>) {
    // Draw column header
    let item_count = column.items.len();
    let title = format!("{} ({})", column.title, item_count);
    
    let column_block = Block::default()
        .title(title)
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White));
    
    f.render_widget(column_block, area);
    
    // Calculate inner area for items
    let inner_area = area.inner(Margin { horizontal: 1, vertical: 1 });
    
    // Calculate space needed for each item (rough estimate)
    let item_height = 6; // Approximate height per item including box
    let available_height = inner_area.height as usize;
    let max_visible_items = if item_height > 0 { available_height / item_height } else { 0 };
    
    // Calculate scroll offset based on selection
    let scroll_offset = if let Some(selected) = selected_item {
        if selected >= max_visible_items {
            selected.saturating_sub(max_visible_items - 1)
        } else {
            0
        }
    } else {
        0
    };
    
    // Draw visible items
    let mut current_y = inner_area.y;
    for (i, item) in column.items.iter().enumerate().skip(scroll_offset) {
        if current_y >= inner_area.bottom() {
            break; // No more space
        }
        
        let is_selected = selected_item == Some(i);
        
        // Calculate item area
        let item_area = Rect {
            x: inner_area.x,
            y: current_y,
            width: inner_area.width,
            height: (item_height as u16).min(inner_area.bottom() - current_y),
        };
        
        draw_ticket(f, item, item_area, is_selected);
        
        current_y += item_height as u16;
        if current_y >= inner_area.bottom() {
            break;
        }
    }
}