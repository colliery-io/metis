use crate::app::App;
use crate::error::AppError;
use crate::models::AppState;

impl App {
    // Board navigation methods
    pub fn next_board(&mut self) {
        self.ui_state
            .next_board_with_config(&self.core_state.flight_config);
    }

    pub fn previous_board(&mut self) {
        self.ui_state
            .previous_board_with_config(&self.core_state.flight_config);
    }

    pub fn jump_to_strategy_board(&mut self) {
        use crate::app::state::UiState;
        if UiState::is_board_enabled(
            crate::models::BoardType::Strategy,
            &self.core_state.flight_config,
        ) {
            self.ui_state.current_board = crate::models::BoardType::Strategy;
        }
    }

    pub fn jump_to_initiative_board(&mut self) {
        use crate::app::state::UiState;
        if UiState::is_board_enabled(
            crate::models::BoardType::Initiative,
            &self.core_state.flight_config,
        ) {
            self.ui_state.current_board = crate::models::BoardType::Initiative;
        }
    }

    pub fn jump_to_task_board(&mut self) {
        // Task board is always enabled
        self.ui_state.current_board = crate::models::BoardType::Task;
    }

    pub fn jump_to_adr_board(&mut self) {
        // ADR board is always enabled
        self.ui_state.current_board = crate::models::BoardType::Adr;
    }

    pub fn jump_to_backlog_board(&mut self) {
        // Backlog board is always enabled
        self.ui_state.current_board = crate::models::BoardType::Backlog;
    }

    // Selection movement methods
    pub fn move_selection_left(&mut self) {
        let current_board = self.ui_state.current_board;
        self.selection_state.move_selection_left(current_board);
    }

    pub fn move_selection_right(&mut self) {
        let current_board = self.ui_state.current_board;
        let board = self.ui_state.get_current_board();
        self.selection_state
            .move_selection_right(current_board, board.columns.len());
    }

    pub fn move_selection_up(&mut self) {
        let current_board = self.ui_state.current_board;
        self.selection_state.move_selection_up(current_board);
    }

    pub fn move_selection_down(&mut self) {
        let current_board = self.ui_state.current_board;
        let (col_idx, _) = self.selection_state.get_current_selection(current_board);
        let board = self.ui_state.get_current_board();
        let max_items = if col_idx < board.columns.len() {
            board.columns[col_idx].items.len()
        } else {
            0
        };
        self.selection_state
            .move_selection_down(current_board, max_items);
    }

    // Vision document navigation
    pub fn view_vision_document(&mut self) {
        // Look for vision.md in the workspace
        if let Some(workspace_dir) = &self.core_state.workspace_dir {
            let vision_path = workspace_dir.join("vision.md");
            if vision_path.exists() {
                // Create a temporary KanbanItem for the vision document
                match std::fs::read_to_string(&vision_path) {
                    Ok(_) => {
                        // Set viewing state to simulate selecting the vision document
                        // We'll use a special board type and position that doesn't exist
                        self.ui_state.viewing_ticket =
                            Some((crate::models::BoardType::Strategy, 999, 999));

                        // Go directly to edit mode
                        self.start_content_editing_for_vision(vision_path);
                    }
                    Err(e) => {
                        self.error_handler.handle_error(AppError::IoError(format!(
                            "Failed to read vision document: {}",
                            e
                        )));
                    }
                }
            } else {
                self.error_handler.handle_error(AppError::DocumentError(
                    "No vision document found. Create one with 'metis create vision' first."
                        .to_string(),
                ));
            }
        }
    }

    fn start_content_editing_for_vision(&mut self, vision_path: std::path::PathBuf) {
        self.ui_state.set_app_state(AppState::EditingContent);

        // Load vision content
        if let Ok(content) = std::fs::read_to_string(&vision_path) {
            // Create and initialize textarea
            let mut textarea = tui_textarea::TextArea::default();
            textarea.set_block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Vision Document Editor"),
            );

            // Set the content
            for line in content.lines() {
                textarea.insert_str(line);
                textarea.insert_newline();
            }

            self.ui_state.strategy_editor = Some(textarea);
            // Store the vision file path for saving
            self.ui_state.editing_vision_path = Some(vision_path);
        }
    }
}
