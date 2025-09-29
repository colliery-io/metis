use crate::app::state::ConfirmationType;
use crate::app::App;
use crate::models::{AppState, BoardType};

impl App {
    // Message handling methods
    pub fn add_success_message(&mut self, message: String) {
        self.ui_state.message_state.add_success(message);
    }

    pub fn add_error_message(&mut self, message: String) {
        self.ui_state.message_state.add_error(message);
    }

    pub fn add_warning_message(&mut self, message: String) {
        self.ui_state.message_state.add_warning(message);
    }

    pub fn add_info_message(&mut self, message: String) {
        self.ui_state.message_state.add_info(message);
    }

    pub fn clear_messages(&mut self) {
        self.ui_state.message_state.clear_message();
    }

    pub fn clear_expired_messages(&mut self) {
        self.ui_state.message_state.clear_expired_messages();
    }

    // Document creation state management methods
    pub fn start_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingDocument);
        self.ui_state.reset_input();
    }

    pub fn start_smart_document_creation(&mut self) {
        match self.ui_state.current_board {
            // Strategy board: Create initiative under selected strategy
            BoardType::Strategy => {
                self.ui_state.set_app_state(AppState::CreatingChildDocument);
                self.ui_state.reset_input();
            }
            // Initiative board: Create task under selected initiative
            BoardType::Initiative => {
                self.ui_state.set_app_state(AppState::CreatingChildDocument);
                self.ui_state.reset_input();
            }
            // Task board: No creation allowed - tasks created from Initiative or Backlog boards
            BoardType::Task => {
                self.add_error_message("Tasks are created from the Initiative board (with parent) or Backlog board (standalone)".to_string());
            }
            // ADR board: Create new ADR (standalone)
            BoardType::Adr => {
                self.ui_state.set_app_state(AppState::CreatingAdr);
                self.ui_state.reset_input();
            }
            // Backlog board: Create backlog item (standalone)
            BoardType::Backlog => {
                self.ui_state.set_app_state(AppState::SelectingBacklogCategory);
                self.ui_state.reset_input();
            }
        }
    }

    pub fn start_child_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingChildDocument);
        self.ui_state.reset_input();
    }

    pub fn start_adr_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingAdr);
        self.ui_state.reset_input();
    }

    pub fn cancel_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::Normal);
        self.ui_state.reset_input();
    }

    // Confirmation dialog state management methods
    pub fn start_delete_confirmation(&mut self) {
        self.ui_state.confirmation_type = Some(ConfirmationType::Delete);
        self.ui_state.set_app_state(AppState::Confirming);
    }

    pub fn start_transition_confirmation(&mut self) {
        self.ui_state.confirmation_type = Some(ConfirmationType::Transition);
        self.ui_state.set_app_state(AppState::Confirming);
    }

    pub fn cancel_confirmation(&mut self) {
        self.ui_state.confirmation_type = None;
        self.ui_state.set_app_state(AppState::Normal);
    }
}