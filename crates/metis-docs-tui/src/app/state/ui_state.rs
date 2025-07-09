use crate::models::{AppState, BoardType, KanbanBoard};
use tui_input::Input;
use tui_textarea::TextArea;

/// UI-specific state that controls the user interface
#[derive(Debug)]
pub struct UiState {
    pub app_state: AppState,
    pub current_board: BoardType,
    pub strategy_board: KanbanBoard,
    pub initiative_board: KanbanBoard,
    pub task_board: KanbanBoard,
    pub error_message: Option<String>,
    pub input_title: Input,
    pub input_description: String,
    pub viewing_ticket: Option<(BoardType, usize, usize)>,
    pub strategy_editor: Option<TextArea<'static>>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Normal,
            current_board: BoardType::Strategy,
            strategy_board: KanbanBoard::create_strategy_board(),
            initiative_board: KanbanBoard::create_initiative_board(),
            task_board: KanbanBoard::create_task_board(),
            error_message: None,
            input_title: Input::default(),
            input_description: String::new(),
            viewing_ticket: None,
            strategy_editor: None,
        }
    }

    pub fn get_current_board(&self) -> &KanbanBoard {
        match self.current_board {
            BoardType::Strategy => &self.strategy_board,
            BoardType::Initiative => &self.initiative_board,
            BoardType::Task => &self.task_board,
        }
    }

    pub fn get_current_board_mut(&mut self) -> &mut KanbanBoard {
        match self.current_board {
            BoardType::Strategy => &mut self.strategy_board,
            BoardType::Initiative => &mut self.initiative_board,
            BoardType::Task => &mut self.task_board,
        }
    }

    pub fn set_error(&mut self, message: String) {
        self.error_message = Some(message);
    }

    pub fn clear_error(&mut self) {
        self.error_message = None;
    }

    pub fn reset_input(&mut self) {
        self.input_title = Input::default();
        self.input_description = String::new();
    }

    pub fn set_app_state(&mut self, state: AppState) {
        self.app_state = state;
    }

    pub fn next_board(&mut self) {
        self.current_board = match self.current_board {
            BoardType::Strategy => BoardType::Initiative,
            BoardType::Initiative => BoardType::Task,
            BoardType::Task => BoardType::Strategy,
        };
    }

    pub fn previous_board(&mut self) {
        self.current_board = match self.current_board {
            BoardType::Strategy => BoardType::Task,
            BoardType::Initiative => BoardType::Strategy,
            BoardType::Task => BoardType::Initiative,
        };
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}