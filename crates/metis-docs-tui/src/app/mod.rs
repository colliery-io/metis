pub mod actions;
pub mod mapping;
pub mod operations;
pub mod state;

use crate::app::mapping::*;
use crate::error::*;
use crate::models::*;
use crate::services::*;
use anyhow::Result;
use metis_core::{
    domain::documents::types::DocumentType,
    Initiative, Strategy, Task,
};


pub struct App {
    // Core application state
    pub core_state: state::CoreAppState,
    // UI state
    pub ui_state: state::UiState,
    // Selection state
    pub selection_state: state::SelectionState,
    // Error handler
    pub error_handler: ErrorHandler,
    // Services
    pub workspace_service: WorkspaceService,
    pub document_service: Option<DocumentService>,
    pub sync_service: Option<SyncService>,
    pub transition_service: Option<TransitionService>,
}

impl App {
    pub fn new() -> Self {
        Self {
            core_state: state::CoreAppState::new(),
            ui_state: state::UiState::new(),
            selection_state: state::SelectionState::new(),
            error_handler: ErrorHandler::new(),
            workspace_service: WorkspaceService::new(),
            document_service: None,
            sync_service: None,
            transition_service: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // 1. Check if we're in a metis workspace
        match self.workspace_service.check_workspace().await {
            Ok(Some(workspace_dir)) => {
                self.core_state.set_workspace(workspace_dir.clone());

                // Initialize services
                self.document_service = Some(DocumentService::new(workspace_dir.clone()));
                self.sync_service = Some(SyncService::new(workspace_dir.clone()));
                self.transition_service = Some(TransitionService::new(workspace_dir));

                // 2. Perform database synchronization
                if let Some(sync_service) = &self.sync_service {
                    match sync_service.sync_database().await {
                        Ok(_) => {
                            self.core_state.set_sync_complete();

                            // 3. Load documents into boards
                            self.load_documents().await?;
                        }
                        Err(e) => {
                            self.error_handler.handle_error(AppError::from(e));
                        }
                    }
                }
            }
            Ok(None) => {
                self.error_handler
                    .handle_error(AppError::WorkspaceError("No workspace found".to_string()));
            }
            Err(e) => {
                self.error_handler.handle_error(AppError::from(e));
            }
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        self.core_state.is_ready()
    }

    pub fn get_current_board(&self) -> &KanbanBoard {
        self.ui_state.get_current_board()
    }

    // Convenience methods for accessing state
    pub fn app_state(&self) -> &AppState {
        &self.ui_state.app_state
    }

    pub fn error_message(&self) -> Option<String> {
        self.ui_state.message_state.get_current_message().map(|msg| msg.content.clone())
    }


    pub fn get_selected_item(&self) -> Option<&KanbanItem> {
        let current_board = self.ui_state.current_board;
        let (col_idx, item_idx) = self.selection_state.get_current_selection(current_board);
        let board = self.ui_state.get_current_board();

        if col_idx < board.columns.len() && item_idx < board.columns[col_idx].items.len() {
            Some(&board.columns[col_idx].items[item_idx])
        } else {
            None
        }
    }

    pub fn view_selected_ticket(&mut self) {
        let current_board = self.ui_state.current_board;
        let selection = self.selection_state.get_current_selection(current_board);
        self.ui_state.viewing_ticket = Some((current_board, selection.0, selection.1));
        // Go directly to edit mode instead of view mode
        self.start_content_editing();
    }

    pub fn get_viewed_ticket(&self) -> Option<&KanbanItem> {
        if let Some((board_type, col_idx, item_idx)) = self.ui_state.viewing_ticket {
            let board = match board_type {
                BoardType::Strategy => &self.ui_state.strategy_board,
                BoardType::Initiative => &self.ui_state.initiative_board,
                BoardType::Task => &self.ui_state.task_board,
                BoardType::Adr => &self.ui_state.adr_board,
                BoardType::Backlog => &self.ui_state.backlog_board,
            };

            if col_idx < board.columns.len() && item_idx < board.columns[col_idx].items.len() {
                Some(&board.columns[col_idx].items[item_idx])
            } else {
                None
            }
        } else {
            None
        }
    }

    // Input handling
    pub fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) {
        use tui_input::backend::crossterm::EventHandler;
        self.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(key));
    }




    pub async fn load_documents(&mut self) -> Result<()> {
        if let Some(document_service) = &self.document_service {
            // Clear all boards before loading new documents
            for column in &mut self.ui_state.strategy_board.columns {
                column.items.clear();
            }
            for column in &mut self.ui_state.initiative_board.columns {
                column.items.clear();
            }
            for column in &mut self.ui_state.task_board.columns {
                column.items.clear();
            }
            for column in &mut self.ui_state.adr_board.columns {
                column.items.clear();
            }
            for column in &mut self.ui_state.backlog_board.columns {
                column.items.clear();
            }
            
            // Reset selection state to avoid referencing non-existent items
            self.selection_state.strategy_selection = (0, 0);
            self.selection_state.initiative_selection = (0, 0);
            self.selection_state.task_selection = (0, 0);
            self.selection_state.adr_selection = (0, 0);
            self.selection_state.backlog_selection = (0, 0);
            
            let mut documents = document_service.load_documents_from_database().await?;

            // Sort documents by type first, then by appropriate criteria
            documents.sort_by(|a, b| {
                use std::cmp::Ordering;

                // Helper function to get document type order
                let type_order = |doc_type: &DocumentType| -> u8 {
                    match doc_type {
                        DocumentType::Vision => 0,
                        DocumentType::Strategy => 1,
                        DocumentType::Initiative => 2,
                        DocumentType::Task => 3,
                        DocumentType::Adr => 4,
                    }
                };

                // First compare by document type
                let a_type_order = type_order(&a.document_type);
                let b_type_order = type_order(&b.document_type);

                match a_type_order.cmp(&b_type_order) {
                    Ordering::Equal => {
                        // Same document type, use type-specific sorting
                        match (&a.document_type, &b.document_type) {
                            (DocumentType::Adr, DocumentType::Adr) => {
                                // For ADRs, extract number from ID and sort numerically
                                let a_num = extract_adr_number(&a.id);
                                let b_num = extract_adr_number(&b.id);
                                a_num.cmp(&b_num)
                            }
                            _ => a.title.cmp(&b.title), // Other documents sort by title
                        }
                    }
                    other => other, // Different types, use type ordering
                }
            });

            // Clear existing boards
            self.ui_state.strategy_board = KanbanBoard::create_strategy_board();
            self.ui_state.initiative_board = KanbanBoard::create_initiative_board();
            self.ui_state.task_board = KanbanBoard::create_task_board();
            self.ui_state.adr_board = KanbanBoard::create_adr_board();
            self.ui_state.backlog_board = KanbanBoard::create_backlog_board();

            // Load documents into appropriate boards
            for doc in documents {
                match doc.document_type {
                    DocumentType::Strategy => {
                        if let Ok(strategy) =
                            Strategy::from_file(std::path::Path::new(&doc.filepath)).await
                        {
                            let column_index = get_strategy_column_index(&strategy);
                            let item = KanbanItem {
                                document: DocumentObject::Strategy(strategy),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            if column_index < self.ui_state.strategy_board.columns.len() {
                                self.ui_state.strategy_board.columns[column_index]
                                    .items
                                    .push(item);
                            }
                        }
                    }
                    DocumentType::Initiative => {
                        if let Ok(initiative) =
                            Initiative::from_file(std::path::Path::new(&doc.filepath)).await
                        {
                            let column_index = get_initiative_column_index(&initiative);
                            let item = KanbanItem {
                                document: DocumentObject::Initiative(initiative),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            if column_index < self.ui_state.initiative_board.columns.len() {
                                self.ui_state.initiative_board.columns[column_index]
                                    .items
                                    .push(item);
                            }
                        }
                    }
                    DocumentType::Task => {
                        if let Ok(task) = Task::from_file(std::path::Path::new(&doc.filepath)).await
                        {
                            use metis_core::{domain::documents::types::Phase, Document};
                            
                            // Check if this is a backlog item (Phase::Backlog or no parent)
                            let is_backlog = task.phase() == Ok(Phase::Backlog) || task.parent_id().is_none();
                            
                            if is_backlog {
                                // Place in backlog board
                                let column_index = get_backlog_column_index(&task);
                                let item = KanbanItem {
                                    document: DocumentObject::Task(task),
                                    prelude: doc.title.clone(),
                                    risk_complexity: None,
                                    file_path: doc.filepath,
                                };
                                if column_index < self.ui_state.backlog_board.columns.len() {
                                    self.ui_state.backlog_board.columns[column_index]
                                        .items
                                        .push(item);
                                }
                            } else {
                                // Place in regular task board
                                let column_index = get_task_column_index(&task);
                                let item = KanbanItem {
                                    document: DocumentObject::Task(task),
                                    prelude: doc.title.clone(),
                                    risk_complexity: None,
                                    file_path: doc.filepath,
                                };
                                if column_index < self.ui_state.task_board.columns.len() {
                                    self.ui_state.task_board.columns[column_index]
                                        .items
                                        .push(item);
                                }
                            }
                        }
                    }
                    DocumentType::Adr => {
                        if let Ok(adr) =
                            metis_core::Adr::from_file(std::path::Path::new(&doc.filepath)).await
                        {
                            let column_index = get_adr_column_index(&adr);
                            let item = KanbanItem {
                                document: DocumentObject::Adr(adr),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            if column_index < self.ui_state.adr_board.columns.len() {
                                self.ui_state.adr_board.columns[column_index]
                                    .items
                                    .push(item);
                            }
                        }
                    }
                    _ => {
                        // Skip other document types for now
                    }
                }
            }
        }

        Ok(())
    }




}
