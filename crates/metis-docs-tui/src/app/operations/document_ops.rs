use crate::app::state::BacklogCategory;
use crate::app::App;
use crate::models::{kanban::DocumentObject, AppState, BoardType};
use anyhow::Result;
use metis_core::{domain::documents::types::DocumentType, Document};

impl App {
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

            match result {
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
                    Some(format!(
                        "Context from strategy '{}': {}",
                        strategy.title(),
                        strategy.content().full_content()
                    ))
                }
                DocumentObject::Initiative(initiative) => {
                    Some(format!(
                        "Context from initiative '{}': {}",
                        initiative.title(),
                        initiative.content().full_content()
                    ))
                }
                DocumentObject::Task(task) => {
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