use crate::models::{AppState, BoardType, KanbanBoard, MessageState};
use metis_core::domain::configuration::FlightLevelConfig;
use tui_input::Input;
use tui_textarea::TextArea;

#[derive(Debug, Clone)]
pub enum ConfirmationType {
    Delete,
    Transition,
}

/// Backlog category types for user selection
#[derive(Debug, Clone, PartialEq)]
pub enum BacklogCategory {
    General,
    Bug,
    Feature,
    TechDebt,
}

impl BacklogCategory {
    pub fn as_tag(&self) -> Option<&'static str> {
        match self {
            BacklogCategory::General => None,
            BacklogCategory::Bug => Some("#bug"),
            BacklogCategory::Feature => Some("#feature"),
            BacklogCategory::TechDebt => Some("#tech-debt"),
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            BacklogCategory::General => "General",
            BacklogCategory::Bug => "Bug",
            BacklogCategory::Feature => "Feature",
            BacklogCategory::TechDebt => "Tech Debt",
        }
    }
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
    pub backlog_board: KanbanBoard,
    pub input_title: Input,
    pub input_description: String,
    pub viewing_ticket: Option<(BoardType, usize, usize)>,
    pub strategy_editor: Option<TextArea<'static>>,
    pub confirmation_type: Option<ConfirmationType>,
    pub editing_vision_path: Option<std::path::PathBuf>,
    pub message_state: MessageState,
    pub selected_backlog_category: BacklogCategory,
    pub backlog_category_selection: usize,
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
            backlog_board: KanbanBoard::create_backlog_board(),
            input_title: Input::default(),
            input_description: String::new(),
            viewing_ticket: None,
            strategy_editor: None,
            confirmation_type: None,
            editing_vision_path: None,
            message_state: MessageState::new(),
            selected_backlog_category: BacklogCategory::General,
            backlog_category_selection: 0,
        }
    }

    pub fn get_current_board(&self) -> &KanbanBoard {
        match self.current_board {
            BoardType::Strategy => &self.strategy_board,
            BoardType::Initiative => &self.initiative_board,
            BoardType::Task => &self.task_board,
            BoardType::Adr => &self.adr_board,
            BoardType::Backlog => &self.backlog_board,
        }
    }

    pub fn reset_input(&mut self) {
        self.input_title = Input::default();
        self.input_description = String::new();
        self.selected_backlog_category = BacklogCategory::General;
        self.backlog_category_selection = 0;
    }

    pub fn set_app_state(&mut self, state: AppState) {
        self.app_state = state;
    }

    pub fn next_board(&mut self) {
        self.current_board = match self.current_board {
            BoardType::Strategy => BoardType::Initiative,
            BoardType::Initiative => BoardType::Task,
            BoardType::Task => BoardType::Adr,
            BoardType::Adr => BoardType::Backlog,
            BoardType::Backlog => BoardType::Strategy,
        };
    }

    pub fn previous_board(&mut self) {
        self.current_board = match self.current_board {
            BoardType::Strategy => BoardType::Backlog,
            BoardType::Initiative => BoardType::Strategy,
            BoardType::Task => BoardType::Initiative,
            BoardType::Adr => BoardType::Task,
            BoardType::Backlog => BoardType::Adr,
        };
    }

    /// Get enabled board types based on flight level configuration
    pub fn get_enabled_boards(flight_config: &FlightLevelConfig) -> Vec<BoardType> {
        let mut boards = vec![];

        if flight_config.strategies_enabled {
            boards.push(BoardType::Strategy);
        }

        if flight_config.initiatives_enabled {
            boards.push(BoardType::Initiative);
        }

        // Task board is always available
        boards.push(BoardType::Task);

        // ADR and Backlog are always available
        boards.push(BoardType::Adr);
        boards.push(BoardType::Backlog);

        boards
    }

    /// Navigate to next enabled board based on configuration
    pub fn next_board_with_config(&mut self, flight_config: &FlightLevelConfig) {
        let enabled_boards = Self::get_enabled_boards(flight_config);

        if let Some(current_index) = enabled_boards
            .iter()
            .position(|&board| board == self.current_board)
        {
            let next_index = (current_index + 1) % enabled_boards.len();
            self.current_board = enabled_boards[next_index];
        } else {
            // If current board is not enabled, jump to first enabled board
            if let Some(&first_board) = enabled_boards.first() {
                self.current_board = first_board;
            }
        }
    }

    /// Navigate to previous enabled board based on configuration
    pub fn previous_board_with_config(&mut self, flight_config: &FlightLevelConfig) {
        let enabled_boards = Self::get_enabled_boards(flight_config);

        if let Some(current_index) = enabled_boards
            .iter()
            .position(|&board| board == self.current_board)
        {
            let previous_index = if current_index == 0 {
                enabled_boards.len() - 1
            } else {
                current_index - 1
            };
            self.current_board = enabled_boards[previous_index];
        } else {
            // If current board is not enabled, jump to first enabled board
            if let Some(&first_board) = enabled_boards.first() {
                self.current_board = first_board;
            }
        }
    }

    /// Check if a board type is enabled based on configuration
    pub fn is_board_enabled(board_type: BoardType, flight_config: &FlightLevelConfig) -> bool {
        match board_type {
            BoardType::Strategy => flight_config.strategies_enabled,
            BoardType::Initiative => flight_config.initiatives_enabled,
            BoardType::Task | BoardType::Adr | BoardType::Backlog => true, // Always enabled
        }
    }

    /// Set the current board to the first enabled board if current board is disabled
    pub fn ensure_valid_board(&mut self, flight_config: &FlightLevelConfig) {
        if !Self::is_board_enabled(self.current_board, flight_config) {
            let enabled_boards = Self::get_enabled_boards(flight_config);
            if let Some(&first_board) = enabled_boards.first() {
                self.current_board = first_board;
            }
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::BoardType;
    use metis_core::domain::configuration::FlightLevelConfig;

    #[test]
    fn test_get_enabled_boards_full_config() {
        let config = FlightLevelConfig::full();
        let enabled_boards = UiState::get_enabled_boards(&config);

        assert_eq!(
            enabled_boards,
            vec![
                BoardType::Strategy,
                BoardType::Initiative,
                BoardType::Task,
                BoardType::Adr,
                BoardType::Backlog
            ]
        );
    }

    #[test]
    fn test_get_enabled_boards_streamlined_config() {
        let config = FlightLevelConfig::streamlined();
        let enabled_boards = UiState::get_enabled_boards(&config);

        assert_eq!(
            enabled_boards,
            vec![
                BoardType::Initiative,
                BoardType::Task,
                BoardType::Adr,
                BoardType::Backlog
            ]
        );
    }

    #[test]
    fn test_get_enabled_boards_direct_config() {
        let config = FlightLevelConfig::direct();
        let enabled_boards = UiState::get_enabled_boards(&config);

        assert_eq!(
            enabled_boards,
            vec![BoardType::Task, BoardType::Adr, BoardType::Backlog]
        );
    }

    #[test]
    fn test_is_board_enabled() {
        let full_config = FlightLevelConfig::full();
        let streamlined_config = FlightLevelConfig::streamlined();
        let direct_config = FlightLevelConfig::direct();

        // Full configuration
        assert!(UiState::is_board_enabled(BoardType::Strategy, &full_config));
        assert!(UiState::is_board_enabled(
            BoardType::Initiative,
            &full_config
        ));
        assert!(UiState::is_board_enabled(BoardType::Task, &full_config));
        assert!(UiState::is_board_enabled(BoardType::Adr, &full_config));
        assert!(UiState::is_board_enabled(BoardType::Backlog, &full_config));

        // Streamlined configuration
        assert!(!UiState::is_board_enabled(
            BoardType::Strategy,
            &streamlined_config
        ));
        assert!(UiState::is_board_enabled(
            BoardType::Initiative,
            &streamlined_config
        ));
        assert!(UiState::is_board_enabled(
            BoardType::Task,
            &streamlined_config
        ));
        assert!(UiState::is_board_enabled(
            BoardType::Adr,
            &streamlined_config
        ));
        assert!(UiState::is_board_enabled(
            BoardType::Backlog,
            &streamlined_config
        ));

        // Direct configuration
        assert!(!UiState::is_board_enabled(
            BoardType::Strategy,
            &direct_config
        ));
        assert!(!UiState::is_board_enabled(
            BoardType::Initiative,
            &direct_config
        ));
        assert!(UiState::is_board_enabled(BoardType::Task, &direct_config));
        assert!(UiState::is_board_enabled(BoardType::Adr, &direct_config));
        assert!(UiState::is_board_enabled(
            BoardType::Backlog,
            &direct_config
        ));
    }

    #[test]
    fn test_ensure_valid_board() {
        let mut ui_state = UiState::new();

        // Start with strategy board in full config (valid)
        ui_state.current_board = BoardType::Strategy;
        let full_config = FlightLevelConfig::full();
        ui_state.ensure_valid_board(&full_config);
        assert_eq!(ui_state.current_board, BoardType::Strategy); // Should stay the same

        // Switch to streamlined config where strategy is disabled
        let streamlined_config = FlightLevelConfig::streamlined();
        ui_state.ensure_valid_board(&streamlined_config);
        assert_eq!(ui_state.current_board, BoardType::Initiative); // Should move to first enabled

        // Switch to direct config where initiative is also disabled
        let direct_config = FlightLevelConfig::direct();
        ui_state.ensure_valid_board(&direct_config);
        assert_eq!(ui_state.current_board, BoardType::Task); // Should move to first enabled
    }
}
