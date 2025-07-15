pub mod state;

use crate::app::state::ConfirmationType;
use crate::error::*;
use crate::models::*;
use crate::services::*;
use anyhow::Result;
use metis_core::{
    application::services::workspace::ArchiveService,
    domain::documents::types::DocumentType, Adr, Document, Initiative, Strategy, Task,
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
        self.error_handler.get_user_friendly_message()
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
            self.error_handler.handle_error(AppError::UserInputError(
                "Title cannot be empty".to_string(),
            ));
            return Ok(());
        }

        if let Some(document_service) = &self.document_service {
            let doc_type = match self.ui_state.current_board {
                BoardType::Strategy => DocumentType::Strategy,
                BoardType::Initiative => DocumentType::Initiative,
                BoardType::Task => DocumentType::Task,
                BoardType::Adr => DocumentType::Adr,
            };

            match document_service
                .create_document(
                    doc_type, title, None, // description
                    None, // parent_id
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
                    self.error_handler.handle_error(AppError::from(e));
                }
            }
        }

        Ok(())
    }

    pub async fn create_child_document(&mut self) -> Result<()> {
        let title = self.ui_state.input_title.value().to_string();
        if title.trim().is_empty() {
            self.error_handler.handle_error(AppError::UserInputError(
                "Title cannot be empty".to_string(),
            ));
            return Ok(());
        }

        if let Some(parent_item) = self.get_selected_item() {
            if let Some(document_service) = &self.document_service {
                // Determine child document type based on parent
                let child_doc_type = match parent_item.doc_type() {
                    DocumentType::Strategy => DocumentType::Initiative,
                    DocumentType::Initiative => DocumentType::Task,
                    _ => {
                        self.error_handler.handle_error(AppError::ValidationError(
                            "Cannot create child for this document type".to_string(),
                        ));
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
                                        self.error_handler.handle_error(AppError::from(e));
                                    }
                                }
                            } else {
                                self.error_handler.handle_error(AppError::ValidationError(
                                    "Initiative has no parent strategy".to_string(),
                                ));
                            }
                        } else {
                            self.error_handler.handle_error(AppError::ValidationError(
                                "Selected item is not an initiative".to_string(),
                            ));
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
            self.error_handler.handle_error(AppError::UserInputError(
                "ADR title cannot be empty".to_string(),
            ));
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
}
