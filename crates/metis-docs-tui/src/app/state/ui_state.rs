use crate::models::{AppState, BoardType, KanbanBoard};
use tui_input::Input;
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub enum ConfirmationType {
    Delete,
    Transition,
}

/// UI-specific state that controls the user interface
#[derive(Debug)]
pub struct UiState {
    pub app_state: AppState,
    pub current_board: BoardType,
    pub strategy_board: KanbanBoard,
    pub initiative_board: KanbanBoard,
    pub task_board: KanbanBoard,
    pub adr_board: KanbanBoard,
    pub input_title: Input,
    pub input_description: String,
    pub viewing_ticket: Option<(BoardType, usize, usize)>,
    pub strategy_editor: Option<TextArea<'static>>,
    pub confirmation_type: Option<ConfirmationType>,
    pub editing_vision_path: Option<std::path::PathBuf>,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            app_state: AppState::Normal,
            current_board: BoardType::Strategy,
            strategy_board: KanbanBoard::create_strategy_board(),
            initiative_board: KanbanBoard::create_initiative_board(),
            task_board: KanbanBoard::create_task_board(),
            adr_board: KanbanBoard::create_adr_board(),
            input_title: Input::default(),
            input_description: String::new(),
            viewing_ticket: None,
            strategy_editor: None,
            confirmation_type: None,
            editing_vision_path: None,
        }
    }

    pub fn get_current_board(&self) -> &KanbanBoard {
        match self.current_board {
            BoardType::Strategy => &self.strategy_board,
            BoardType::Initiative => &self.initiative_board,
            BoardType::Task => &self.task_board,
            BoardType::Adr => &self.adr_board,
        }
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
            BoardType::Task => BoardType::Adr,
            BoardType::Adr => BoardType::Strategy,
        };
    }

    pub fn previous_board(&mut self) {
        self.current_board = match self.current_board {
            BoardType::Strategy => BoardType::Adr,
            BoardType::Initiative => BoardType::Strategy,
            BoardType::Task => BoardType::Initiative,
            BoardType::Adr => BoardType::Task,
        };
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}
