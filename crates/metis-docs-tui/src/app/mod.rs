pub mod state;

use crate::app::state::ConfirmationType;
use crate::error::*;
use crate::models::*;
use crate::services::*;
use anyhow::Result;
use metis_core::{
    application::services::workspace::ArchiveService, domain::documents::types::DocumentType, Adr,
    Document, Initiative, Strategy, Task,
};

/// Extract numerical part from ADR ID (e.g., "001-some-title" -> 1)
fn extract_adr_number(id: &str) -> u32 {
    if let Some(dash_pos) = id.find('-') {
        if let Ok(num) = id[..dash_pos].parse::<u32>() {
            return num;
        }
    }
    // Fallback: if parsing fails, return 0 to sort to beginning
    0
}

/// Get column index for strategy board based on phase
/// Columns: shaping(0), design(1), ready(2), active(3), completed(4)
fn get_strategy_column_index(strategy: &Strategy) -> usize {
    use metis_core::{domain::documents::types::Phase, Document};
    match strategy.phase() {
        Ok(Phase::Shaping) => 0,
        Ok(Phase::Design) => 1,
        Ok(Phase::Ready) => 2,
        Ok(Phase::Active) => 3,
        Ok(Phase::Completed) => 4,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for initiative board based on phase
/// Columns: discovery(0), design(1), ready(2), decompose(3), active(4), completed(5)
fn get_initiative_column_index(initiative: &Initiative) -> usize {
    use metis_core::{domain::documents::types::Phase, Document};
    match initiative.phase() {
        Ok(Phase::Discovery) => 0,
        Ok(Phase::Design) => 1,
        Ok(Phase::Ready) => 2,
        Ok(Phase::Decompose) => 3,
        Ok(Phase::Active) => 4,
        Ok(Phase::Completed) => 5,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for task board based on phase
/// Columns: todo(0), active(1), blocked(2), completed(3)
fn get_task_column_index(task: &Task) -> usize {
    use metis_core::{domain::documents::types::Phase, Document};
    match task.phase() {
        Ok(Phase::Todo) => 0,
        Ok(Phase::Active) => 1,
        Ok(Phase::Blocked) => 2,
        Ok(Phase::Completed) => 3,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for ADR board based on phase
/// Columns: draft(0), discussion(1), decided(2), superseded(3)
fn get_adr_column_index(adr: &Adr) -> usize {
    use metis_core::{domain::documents::types::Phase, Document};
    match adr.phase() {
        Ok(Phase::Draft) => 0,
        Ok(Phase::Discussion) => 1,
        Ok(Phase::Decided) => 2,
        Ok(Phase::Superseded) => 3,
        _ => 0, // Default to first column if phase is unknown
    }
}

/// Get column index for backlog board based on task type
/// Columns: backlog(0), bugs(1), features(2), tech-debt(3)
fn get_backlog_column_index(task: &Task) -> usize {
    use metis_core::{Document, domain::documents::types::Tag};
    
    // Check task tags to determine type
    for tag in &task.core().tags {
        if let Tag::Label(label) = tag {
            match label.as_str() {
                "bug" => return 1, // bugs column
                "feature" => return 2, // features column
                "tech-debt" => return 3, // tech-debt column
                _ => {}
            }
        }
    }
    
    // Default to backlog column (general items)
    0
}

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

    // Navigation methods
    pub fn next_board(&mut self) {
        self.ui_state.next_board();
    }

    pub fn previous_board(&mut self) {
        self.ui_state.previous_board();
    }

    pub fn jump_to_strategy_board(&mut self) {
        self.ui_state.current_board = BoardType::Strategy;
    }

    pub fn jump_to_initiative_board(&mut self) {
        self.ui_state.current_board = BoardType::Initiative;
    }

    pub fn jump_to_task_board(&mut self) {
        self.ui_state.current_board = BoardType::Task;
    }

    pub fn jump_to_adr_board(&mut self) {
        self.ui_state.current_board = BoardType::Adr;
    }

    pub fn jump_to_backlog_board(&mut self) {
        self.ui_state.current_board = BoardType::Backlog;
    }

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
                        self.ui_state.viewing_ticket = Some((BoardType::Strategy, 999, 999));

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

    // Document management methods
    pub fn start_document_creation(&mut self) {
        self.ui_state.set_app_state(AppState::CreatingDocument);
        self.ui_state.reset_input();
    }

    pub fn start_smart_document_creation(&mut self) {
        use crate::models::BoardType;
        
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

    pub async fn create_new_document(&mut self) -> Result<()> {
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.add_error_message("Title cannot be empty".to_string());
            self.ui_state.set_app_state(AppState::Normal);
            self.ui_state.reset_input();
            return Ok(());
        }

        if let Some(document_service) = &self.document_service {
            let doc_type = match self.ui_state.current_board {
                BoardType::Strategy => DocumentType::Strategy,
                BoardType::Initiative => DocumentType::Initiative,
                BoardType::Task => DocumentType::Task,
                BoardType::Adr => DocumentType::Adr,
                BoardType::Backlog => DocumentType::Task, // Backlog items are tasks
            };

            // For initiatives and tasks, we need to find an available parent
            let parent_id = match doc_type {
                DocumentType::Initiative => {
                    // Find the first available strategy to use as parent
                    let strategy_items: Vec<_> = self.ui_state.strategy_board.columns
                        .iter()
                        .flat_map(|col| &col.items)
                        .collect();
                    
                    if let Some(strategy) = strategy_items.first() {
                        Some(strategy.id())
                    } else {
                        self.add_error_message("Cannot create initiative: No strategy available as parent".to_string());
                        self.ui_state.set_app_state(AppState::Normal);
                        self.ui_state.reset_input();
                        return Ok(());
                    }
                }
                DocumentType::Task => {
                    // For tasks on task board, find the first available initiative as parent
                    if self.ui_state.current_board == BoardType::Task {
                        let initiative_items: Vec<_> = self.ui_state.initiative_board.columns
                            .iter()
                            .flat_map(|col| &col.items)
                            .collect();
                        
                        if let Some(initiative) = initiative_items.first() {
                            Some(initiative.id())
                        } else {
                            self.add_error_message("Cannot create task: No initiative available as parent".to_string());
                            self.ui_state.set_app_state(AppState::Normal);
                            self.ui_state.reset_input();
                            return Ok(());
                        }
                    } else {
                        // For backlog tasks, no parent needed
                        None
                    }
                }
                _ => None
            };

            // For tasks with parent initiative, use create_child_task
            let result = if doc_type == DocumentType::Task && parent_id.is_some() && self.ui_state.current_board == BoardType::Task {
                // Find the strategy that owns this initiative
                let initiative_items: Vec<_> = self.ui_state.initiative_board.columns
                    .iter()
                    .flat_map(|col| &col.items)
                    .collect();
                
                if let Some(initiative) = initiative_items.first() {
                    // Get the strategy ID from the initiative's parent
                    use metis_core::Document;
                    use crate::models::kanban::DocumentObject;
                    
                    let strategy_id = match &initiative.document {
                        DocumentObject::Initiative(init) => init.parent_id(),
                        _ => None,
                    };
                    
                    if let Some(strategy_id) = strategy_id {
                        match document_service
                            .create_child_task(
                                title,
                                strategy_id.to_string(),
                                initiative.id(),
                            )
                            .await
                        {
                            Ok(task_id) => Ok(task_id),
                            Err(e) => Err(e)
                        }
                    } else {
                        Err(anyhow::anyhow!("Initiative has no parent strategy"))
                    }
                } else {
                    Err(anyhow::anyhow!("No initiative found for task creation"))
                }
            } else {
                match document_service
                    .create_document(
                        doc_type, title, None, // description
                        parent_id,
                    )
                    .await
                {
                    Ok(doc_id) => Ok(doc_id),
                    Err(e) => Err(e)
                }
            };

            match result
            {
                Ok(doc_id) => {
                    // For backlog items, add the selected category tag
                    if self.ui_state.current_board == BoardType::Backlog {
                        if let Some(tag) = self.ui_state.selected_backlog_category.as_tag() {
                            // Force a sync first to ensure the document is in the database
                            if let Some(sync_service) = &self.sync_service {
                                let _ = sync_service.sync_database().await;
                            }
                            
                            // Add the tag to the document
                            if let Err(_e) = self.add_tag_to_document(&doc_id, tag).await {
                                // Silently handle tag addition errors
                            }
                        }
                    }

                    self.add_success_message(format!("{} created successfully", doc_type));
                    self.ui_state.set_app_state(AppState::Normal);
                    self.ui_state.reset_input();
                    // Sync database and reload documents
                    if let Some(sync_service) = &self.sync_service {
                        let _ = sync_service.sync_database().await;
                    }
                    self.load_documents().await?;
                }
                Err(e) => {
                    self.add_error_message(format!("Failed to create {}: {}", doc_type, e));
                    self.ui_state.set_app_state(AppState::Normal);
                    self.ui_state.reset_input();
                }
            }
        }

        Ok(())
    }

    pub async fn create_child_document(&mut self) -> Result<()> {
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.add_error_message("Title cannot be empty".to_string());
            self.ui_state.set_app_state(AppState::Normal);
            self.ui_state.reset_input();
            return Ok(());
        }

        // Check if a parent item is selected
        if let Some(parent_item) = self.get_selected_item() {
            if let Some(document_service) = &self.document_service {
                // Determine child document type based on parent
                let child_doc_type = match parent_item.doc_type() {
                    DocumentType::Strategy => DocumentType::Initiative,
                    DocumentType::Initiative => DocumentType::Task,
                    _ => {
                        self.add_error_message("Cannot create child for this document type".to_string());
                        self.ui_state.set_app_state(AppState::Normal);
                        self.ui_state.reset_input();
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
                                match document_service
                                    .create_child_task(
                                        title,
                                        strategy_id.to_string(),
                                        parent_id.to_string(),
                                    )
                                    .await
                                {
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
                                        self.add_error_message(format!("Failed to create task: {}", e));
                                        self.ui_state.set_app_state(AppState::Normal);
                                        self.ui_state.reset_input();
                                    }
                                }
                            } else {
                                self.add_error_message("Initiative has no parent strategy".to_string());
                                self.ui_state.set_app_state(AppState::Normal);
                                self.ui_state.reset_input();
                            }
                        } else {
                            self.add_error_message("Selected item is not an initiative".to_string());
                            self.ui_state.set_app_state(AppState::Normal);
                            self.ui_state.reset_input();
                        }
                    }
                    _ => {
                        // For other document types, use regular creation
                        match document_service
                            .create_document(
                                child_doc_type,
                                title,
                                None, // description
                                Some(parent_id.to_string()),
                            )
                            .await
                        {
                            Ok(_) => {
                                self.add_success_message(format!("{} created successfully", child_doc_type));
                                self.ui_state.set_app_state(AppState::Normal);
                                self.ui_state.reset_input();
                                // Sync database and reload documents
                                if let Some(sync_service) = &self.sync_service {
                                    let _ = sync_service.sync_database().await;
                                }
                                self.load_documents().await?;
                            }
                            Err(e) => {
                                self.add_error_message(format!("Failed to create {}: {}", child_doc_type, e));
                                self.ui_state.set_app_state(AppState::Normal);
                                self.ui_state.reset_input();
                            }
                        }
                    }
                }
            }
        } else {
            // No parent selected - show appropriate error message based on board
            use crate::models::BoardType;
            let error_msg = match self.ui_state.current_board {
                BoardType::Strategy => "Please select a strategy first to create an initiative under it",
                BoardType::Initiative => "Please select an initiative first to create a task under it",
                _ => "Please select a parent document first",
            };
            self.add_error_message(error_msg.to_string());
            self.ui_state.set_app_state(AppState::Normal);
            self.ui_state.reset_input();
        }

        Ok(())
    }

    pub async fn delete_selected_document(&mut self) -> Result<()> {
        if let Some(selected_item) = self.get_selected_item() {
            if let Some(document_service) = &self.document_service {
                match document_service
                    .delete_document(&selected_item.file_path)
                    .await
                {
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

    pub async fn archive_selected_document(&mut self) -> Result<()> {
        if let Some(selected_item) = self.get_selected_item() {
            if let Some(workspace_dir) = &self.core_state.workspace_dir {
                let archive_service = ArchiveService::new(workspace_dir);
                match archive_service.archive_document(&selected_item.id()).await {
                    Ok(_archive_result) => {
                        self.add_success_message(format!("Document '{}' archived successfully", selected_item.title()));
                        // Sync database and reload documents
                        if let Some(sync_service) = &self.sync_service {
                            let _ = sync_service.sync_database().await;
                        }
                        self.load_documents().await?;
                    }
                    Err(e) => {
                        self.add_error_message(format!("Failed to archive document: {}", e));
                    }
                }
            }
        }

        Ok(())
    }

    pub async fn transition_selected_document(&mut self) -> Result<()> {
        if let Some(selected_item) = self.get_selected_item() {
            if let Some(transition_service) = &self.transition_service {
                match transition_service
                    .transition_to_next_phase(selected_item.id())
                    .await
                {
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

    pub fn start_content_editing(&mut self) {
        self.ui_state.set_app_state(AppState::EditingContent);

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
                    .title("Content Editor"),
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
        self.ui_state.editing_vision_path = None;
    }

    pub async fn save_content_edit(&mut self) -> Result<()> {
        if let Some(ref textarea) = self.ui_state.strategy_editor {
            // Get current content from textarea
            let new_content = textarea.lines().join("\n");

            // Check if we're editing a vision document
            if let Some(ref vision_path) = self.ui_state.editing_vision_path {
                // Save vision document directly
                match tokio::fs::write(vision_path, &new_content).await {
                    Ok(_) => {
                        // Vision saved successfully
                    }
                    Err(e) => {
                        self.error_handler.handle_error(AppError::IoError(format!(
                            "Failed to save vision: {}",
                            e
                        )));
                    }
                }
            } else if let Some(selected_item) = self.get_viewed_ticket() {
                // Save regular document using document service
                if let Some(document_service) = &self.document_service {
                    match document_service
                        .save_document_content(&selected_item.file_path, &new_content)
                        .await
                    {
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

    pub async fn create_adr_from_ticket(&mut self) -> Result<()> {
        // Get the title from user input
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.add_error_message("ADR title cannot be empty".to_string());
            self.ui_state.set_app_state(AppState::Normal);
            self.ui_state.reset_input();
            return Ok(());
        }

        // Get the currently selected ticket for context
        let context = if let Some(selected_item) = self.get_selected_item() {
            // Allow ADR creation from strategies, initiatives, and tasks
            match &selected_item.document {
                DocumentObject::Strategy(strategy) => {
                    use metis_core::Document;
                    Some(format!(
                        "Context from strategy '{}': {}",
                        strategy.title(),
                        strategy.content().full_content()
                    ))
                }
                DocumentObject::Initiative(initiative) => {
                    use metis_core::Document;
                    Some(format!(
                        "Context from initiative '{}': {}",
                        initiative.title(),
                        initiative.content().full_content()
                    ))
                }
                DocumentObject::Task(task) => {
                    use metis_core::Document;
                    Some(format!(
                        "Context from task '{}': {}",
                        task.title(),
                        task.content().full_content()
                    ))
                }
                DocumentObject::Adr(_) => None, // Cannot create ADR from ADR
            }
        } else {
            None // No context if no ticket selected
        };

        if let Some(document_service) = &self.document_service {
            match document_service.create_adr(title, context).await {
                Ok(_file_path) => {
                    self.add_success_message("ADR created successfully".to_string());
                    self.ui_state.set_app_state(AppState::Normal);
                    self.ui_state.reset_input();
                    // Sync database and reload documents
                    if let Some(sync_service) = &self.sync_service {
                        let _ = sync_service.sync_database().await;
                    }
                    self.load_documents().await?;
                }
                Err(e) => {
                    self.add_error_message(format!("Failed to create ADR: {}", e));
                    self.ui_state.set_app_state(AppState::Normal);
                    self.ui_state.reset_input();
                }
            }
        }

        Ok(())
    }

    pub async fn sync_and_reload(&mut self) -> Result<()> {
        // Set sync in progress to show immediate feedback
        self.core_state.set_sync_in_progress();

        // Perform database synchronization
        if let Some(sync_service) = &self.sync_service {
            sync_service.sync_database().await?;
        }

        // Reload documents into boards
        self.load_documents().await?;

        // Mark sync as complete
        self.core_state.set_sync_complete();
        
        // Show success message
        self.add_success_message("Synchronization completed".to_string());

        Ok(())
    }

    // Backlog category selection methods
    pub fn move_category_selection_up(&mut self) {
        if self.ui_state.backlog_category_selection > 0 {
            self.ui_state.backlog_category_selection -= 1;
            self.update_selected_category();
        }
    }

    pub fn move_category_selection_down(&mut self) {
        if self.ui_state.backlog_category_selection < 3 { // 4 categories (0-3)
            self.ui_state.backlog_category_selection += 1;
            self.update_selected_category();
        }
    }

    pub fn confirm_category_selection(&mut self) {
        // Move to document creation with the selected category
        self.ui_state.set_app_state(AppState::CreatingDocument);
    }

    fn update_selected_category(&mut self) {
        use crate::app::state::BacklogCategory;
        self.ui_state.selected_backlog_category = match self.ui_state.backlog_category_selection {
            0 => BacklogCategory::General,
            1 => BacklogCategory::Bug,
            2 => BacklogCategory::Feature,
            3 => BacklogCategory::TechDebt,
            _ => BacklogCategory::General,
        };
    }

    async fn add_tag_to_document(&self, doc_id: &str, tag: &str) -> Result<()> {
        if let Some(document_service) = &self.document_service {
            // Get document from database to find file path
            let docs = document_service.load_documents_from_database().await?;
            
            if let Some(doc) = docs.iter().find(|d| d.id == doc_id) {
                // Read the file content
                let content = std::fs::read_to_string(&doc.filepath)?;
                
                // Add the tag to the frontmatter - look for the actual format used
                let updated_content = if content.contains("tags:") {
                    // Look for the pattern: find the last tag line and insert after it
                    let lines: Vec<&str> = content.lines().collect();
                    let mut new_lines = Vec::new();
                    let mut in_tags_section = false;
                    let mut tags_section_ended = false;
                    let tag_line = format!("  - \"{}\"", tag);
                    
                    for line in lines {
                        if line.trim() == "tags:" {
                            in_tags_section = true;
                            new_lines.push(line.to_string());
                        } else if in_tags_section && !tags_section_ended {
                            if line.trim().starts_with("- \"#") {
                                // This is a tag line, keep it
                                new_lines.push(line.to_string());
                            } else if line.trim().is_empty() {
                                // Found empty line after tags, insert our tag before it
                                new_lines.push(tag_line.clone());
                                new_lines.push(line.to_string());
                                tags_section_ended = true;
                            } else {
                                // Non-tag line found, insert our tag before it
                                new_lines.push(tag_line.clone());
                                new_lines.push(line.to_string());
                                tags_section_ended = true;
                            }
                        } else {
                            new_lines.push(line.to_string());
                        }
                    }
                    
                    new_lines.join("\n")
                } else {
                    // This shouldn't happen for properly created documents, but handle it
                    content
                };
                
                // Write the updated content back
                std::fs::write(&doc.filepath, updated_content)?;
            }
        }
        Ok(())
    }
}
