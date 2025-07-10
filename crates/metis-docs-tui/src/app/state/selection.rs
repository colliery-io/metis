use crate::models::BoardType;

/// Selection state management for navigation within boards
#[derive(Debug, Clone)]
pub struct SelectionState {
    pub strategy_selection: (usize, usize),
    pub initiative_selection: (usize, usize),
    pub task_selection: (usize, usize),
    pub adr_selection: (usize, usize),
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            strategy_selection: (0, 0),
            initiative_selection: (0, 0),
            task_selection: (0, 0),
            adr_selection: (0, 0),
        }
    }

    pub fn get_current_selection(&self, board_type: BoardType) -> (usize, usize) {
        match board_type {
            BoardType::Strategy => self.strategy_selection,
            BoardType::Initiative => self.initiative_selection,
            BoardType::Task => self.task_selection,
            BoardType::Adr => self.adr_selection,
        }
    }

    pub fn get_current_selection_mut(&mut self, board_type: BoardType) -> &mut (usize, usize) {
        match board_type {
            BoardType::Strategy => &mut self.strategy_selection,
            BoardType::Initiative => &mut self.initiative_selection,
            BoardType::Task => &mut self.task_selection,
            BoardType::Adr => &mut self.adr_selection,
        }
    }

    pub fn move_selection_up(&mut self, board_type: BoardType) {
        let selection = self.get_current_selection_mut(board_type);
        if selection.1 > 0 {
            selection.1 -= 1;
        }
    }

    pub fn move_selection_down(&mut self, board_type: BoardType, max_items: usize) {
        let selection = self.get_current_selection_mut(board_type);
        if selection.1 < max_items.saturating_sub(1) {
            selection.1 += 1;
        }
    }

    pub fn move_selection_left(&mut self, board_type: BoardType) {
        let selection = self.get_current_selection_mut(board_type);
        if selection.0 > 0 {
            selection.0 -= 1;
            selection.1 = 0; // Reset item selection when changing columns
        }
    }

    pub fn move_selection_right(&mut self, board_type: BoardType, max_columns: usize) {
        let selection = self.get_current_selection_mut(board_type);
        if selection.0 < max_columns.saturating_sub(1) {
            selection.0 += 1;
            selection.1 = 0; // Reset item selection when changing columns
        }
    }
}

impl Default for SelectionState {
    fn default() -> Self {
        Self::new()
    }
}
