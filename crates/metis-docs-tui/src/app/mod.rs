pub mod state;
pub mod navigation;
pub mod document;

use anyhow::Result;
use crate::models::*;
use crate::services::*;
use crate::error::*;
use metis_core::{Strategy, Initiative, Task, Document, domain::documents::types::DocumentType};

pub struct App {
    // Core application state
    pub core_state: state::CoreAppState,
    // UI state
    pub ui_state: state::UiState,
    // Selection state
    pub selection_state: state::SelectionState,
    // Editing state
    pub editing_state: state::EditingState,
    // Error handler
    pub error_handler: ErrorHandler,
    // Services
    pub workspace_service: WorkspaceService,
    pub document_service: Option<DocumentService>,
    pub sync_service: Option<SyncService>,
}

impl App {
    pub fn new() -> Self {
        Self {
            core_state: state::CoreAppState::new(),
            ui_state: state::UiState::new(),
            selection_state: state::SelectionState::new(),
            editing_state: state::EditingState::new(),
            error_handler: ErrorHandler::new(),
            workspace_service: WorkspaceService::new(),
            document_service: None,
            sync_service: None,
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        // 1. Check if we're in a metis workspace
        match self.workspace_service.check_workspace().await {
            Ok(Some(workspace_dir)) => {
                self.core_state.set_workspace(workspace_dir.clone());
                
                // Initialize services
                self.document_service = Some(DocumentService::new(workspace_dir.clone()));
                self.sync_service = Some(SyncService::new(workspace_dir));
                
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
                self.error_handler.handle_error(AppError::WorkspaceError("No workspace found".to_string()));
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
        self.error_handler.get_user_friendly_message()
    }

    // Navigation methods
    pub fn next_board(&mut self) {
        self.ui_state.next_board();
    }

    pub fn previous_board(&mut self) {
        self.ui_state.previous_board();
    }

    pub fn move_selection_left(&mut self) {
        let current_board = self.ui_state.current_board;
        let board = self.ui_state.get_current_board();
        self.selection_state.move_selection_left(current_board, board.columns.len());
    }

    pub fn move_selection_right(&mut self) {
        let current_board = self.ui_state.current_board;
        let board = self.ui_state.get_current_board();
        self.selection_state.move_selection_right(current_board, board.columns.len());
    }

    pub fn move_selection_up(&mut self) {
        let current_board = self.ui_state.current_board;
        let (col_idx, _) = self.selection_state.get_current_selection(current_board);
        let board = self.ui_state.get_current_board();
        let max_items = if col_idx < board.columns.len() {
            board.columns[col_idx].items.len()
        } else {
            0
        };
        self.selection_state.move_selection_up(current_board, max_items);
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
        self.selection_state.move_selection_down(current_board, max_items);
    }

    // Document management methods
    pub fn start_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingDocument);
        self.ui_state.reset_input();
    }

    pub fn start_child_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingChildDocument);
        self.ui_state.reset_input();
    }

    pub fn cancel_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::Normal);
        self.ui_state.reset_input();
    }

    pub fn start_delete_confirmation(&mut self) {
        self.ui_state.set_app_state(AppState::ConfirmingDelete);
    }

    pub fn cancel_delete_confirmation(&mut self) {
        self.ui_state.set_app_state(AppState::Normal);
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

    pub fn close_ticket_view(&mut self) {
        self.ui_state.viewing_ticket = None;
        self.ui_state.set_app_state(AppState::Normal);
    }

    pub fn get_viewed_ticket(&self) -> Option<&KanbanItem> {
        if let Some((board_type, col_idx, item_idx)) = self.ui_state.viewing_ticket {
            let board = match board_type {
                BoardType::Strategy => &self.ui_state.strategy_board,
                BoardType::Initiative => &self.ui_state.initiative_board,
                BoardType::Task => &self.ui_state.task_board,
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
        self.ui_state.input_title.handle_event(&crossterm::event::Event::Key(key));
    }

    pub async fn create_new_document(&mut self) -> Result<()> {
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.error_handler.handle_error(AppError::UserInputError("Title cannot be empty".to_string()));
            return Ok(());
        }
        
        if let Some(document_service) = &self.document_service {
            let doc_type = match self.ui_state.current_board {
                BoardType::Strategy => DocumentType::Strategy,
                BoardType::Initiative => DocumentType::Initiative,
                BoardType::Task => DocumentType::Task,
            };
            
            match document_service.create_document(
                doc_type,
                title,
                None, // description
                None, // parent_id
            ).await {
                Ok(_) => {
                    self.ui_state.set_app_state(AppState::Normal);
                    self.ui_state.reset_input();
                    // Sync database and reload documents
                    if let Some(sync_service) = &self.sync_service {
                        let _ = sync_service.sync_database().await;
                    }
                    self.load_documents().await?;
                }
                Err(e) => {
                    self.error_handler.handle_error(AppError::from(e));
                }
            }
        }
        
        Ok(())
    }

    pub async fn create_child_document(&mut self) -> Result<()> {
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.error_handler.handle_error(AppError::UserInputError("Title cannot be empty".to_string()));
            return Ok(());
        }
        
        if let Some(parent_item) = self.get_selected_item() {
            if let Some(document_service) = &self.document_service {
                // Determine child document type based on parent
                let child_doc_type = match parent_item.doc_type() {
                    DocumentType::Strategy => DocumentType::Initiative,
                    DocumentType::Initiative => DocumentType::Task,
                    _ => {
                        self.error_handler.handle_error(AppError::ValidationError("Cannot create child for this document type".to_string()));
                        return Ok(());
                    }
                };
                
                let parent_id = parent_item.id();
                
                // For tasks, we need to call create_child_task with special handling
                match child_doc_type {
                    DocumentType::Task => {
                        // For tasks, we need both strategy_id and initiative_id
                        // Get strategy_id from the initiative's parent
                        if let DocumentObject::Initiative(ref initiative) = &parent_item.document {
                            if let Some(strategy_id) = initiative.parent_id() {
                                match document_service.create_child_task(
                                    title,
                                    strategy_id.to_string(),
                                    parent_id.to_string(),
                                ).await {
                                    Ok(_) => {
                                        self.ui_state.set_app_state(AppState::Normal);
                                        self.ui_state.reset_input();
                                        // Sync database and reload documents
                                        if let Some(sync_service) = &self.sync_service {
                                            let _ = sync_service.sync_database().await;
                                        }
                                        self.load_documents().await?;
                                    }
                                    Err(e) => {
                                        self.error_handler.handle_error(AppError::from(e));
                                    }
                                }
                            } else {
                                self.error_handler.handle_error(AppError::ValidationError("Initiative has no parent strategy".to_string()));
                            }
                        } else {
                            self.error_handler.handle_error(AppError::ValidationError("Selected item is not an initiative".to_string()));
                        }
                    }
                    _ => {
                        // For other document types, use regular creation
                        match document_service.create_document(
                            child_doc_type,
                            title,
                            None, // description
                            Some(parent_id.to_string()),
                        ).await {
                            Ok(_) => {
                                self.ui_state.set_app_state(AppState::Normal);
                                self.ui_state.reset_input();
                                // Sync database and reload documents
                                if let Some(sync_service) = &self.sync_service {
                                    let _ = sync_service.sync_database().await;
                                }
                                self.load_documents().await?;
                            }
                            Err(e) => {
                                self.error_handler.handle_error(AppError::from(e));
                            }
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn delete_selected_document(&mut self) -> Result<()> {
        if let Some(selected_item) = self.get_selected_item() {
            if let Some(document_service) = &self.document_service {
                match document_service.delete_document(&selected_item.file_path).await {
                    Ok(_) => {
                        // Sync database and reload documents
                        if let Some(sync_service) = &self.sync_service {
                            let _ = sync_service.sync_database().await;
                        }
                        self.load_documents().await?;
                    }
                    Err(e) => {
                        self.error_handler.handle_error(AppError::from(e));
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn load_documents(&mut self) -> Result<()> {
        if let Some(document_service) = &self.document_service {
            let documents = document_service.load_documents_from_database().await?;
            
            // Clear existing boards
            self.ui_state.strategy_board = KanbanBoard::create_strategy_board();
            self.ui_state.initiative_board = KanbanBoard::create_initiative_board();
            self.ui_state.task_board = KanbanBoard::create_task_board();
            
            // Load documents into appropriate boards
            for doc in documents {
                match doc.document_type {
                    DocumentType::Strategy => {
                        // For now, just add to draft column (index 0)
                        // In a real implementation, we'd check the phase
                        // Load the actual strategy from file
                        if let Ok(strategy) = Strategy::from_file(std::path::Path::new(&doc.filepath)).await {
                            let item = KanbanItem {
                                document: DocumentObject::Strategy(strategy),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            self.ui_state.strategy_board.columns[0].items.push(item);
                        }
                    }
                    DocumentType::Initiative => {
                        if let Ok(initiative) = Initiative::from_file(std::path::Path::new(&doc.filepath)).await {
                            let item = KanbanItem {
                                document: DocumentObject::Initiative(initiative),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            self.ui_state.initiative_board.columns[0].items.push(item);
                        }
                    }
                    DocumentType::Task => {
                        if let Ok(task) = Task::from_file(std::path::Path::new(&doc.filepath)).await {
                            let item = KanbanItem {
                                document: DocumentObject::Task(task),
                                prelude: doc.title.clone(),
                                risk_complexity: None,
                                file_path: doc.filepath,
                            };
                            self.ui_state.task_board.columns[0].items.push(item);
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

    pub fn start_content_editing(&mut self) {
        self.ui_state.set_app_state(AppState::EditingStrategy);
        
        // Initialize text editor with current document content
        if let Some(selected_item) = self.get_viewed_ticket() {
            let content = match &selected_item.document {
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
            
            // Create and initialize textarea
            let mut textarea = tui_textarea::TextArea::default();
            textarea.set_block(
                ratatui::widgets::Block::default()
                    .borders(ratatui::widgets::Borders::ALL)
                    .title("Content Editor")
            );
            
            // Set the content
            for line in content.lines() {
                textarea.insert_str(line);
                textarea.insert_newline();
            }
            
            self.ui_state.strategy_editor = Some(textarea);
        }
    }

    pub fn cancel_content_editing(&mut self) {
        self.ui_state.set_app_state(AppState::Normal);
        self.ui_state.strategy_editor = None;
        self.ui_state.viewing_ticket = None;
    }

    pub async fn save_content_edit(&mut self) -> Result<()> {
        if let Some(selected_item) = self.get_viewed_ticket() {
            if let Some(ref textarea) = self.ui_state.strategy_editor {
                // Get current content from textarea
                let new_content = textarea.lines().join("\n");
                
                // Save the content using document service
                if let Some(document_service) = &self.document_service {
                    match document_service.save_document_content(&selected_item.file_path, &new_content).await {
                        Ok(_) => {
                            // Sync database and reload documents
                            if let Some(sync_service) = &self.sync_service {
                                let _ = sync_service.sync_database().await;
                            }
                            self.load_documents().await?;
                        }
                        Err(e) => {
                            self.error_handler.handle_error(AppError::from(e));
                        }
                    }
                }
            }
        }
        
        Ok(())
    }

    pub fn cancel_editing(&mut self) {
        self.editing_state.stop_editing();
        self.ui_state.set_app_state(AppState::Normal);
    }

    pub fn edit_next_field(&mut self) {
        self.editing_state.next_field();
    }

    pub fn edit_previous_field(&mut self) {
        self.editing_state.previous_field();
    }

    pub fn save_edit(&mut self) {
        // TODO: Implement save logic
    }

    pub fn edit_handle_backspace(&mut self) {
        // TODO: Implement backspace handling
    }

    pub fn edit_handle_input(&mut self, _c: char) {
        // TODO: Implement input handling
    }
}