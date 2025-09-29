use crate::app::App;
use crate::error::AppError;
use crate::models::{kanban::DocumentObject, AppState};
use anyhow::Result;
use metis_core::Document;

impl App {
    pub fn start_content_editing(&mut self) {
        self.ui_state.set_app_state(AppState::EditingContent);

        // Initialize text editor with current document content
        if let Some(selected_item) = self.get_viewed_ticket() {
            let content = match &selected_item.document {
                DocumentObject::Strategy(strategy) => {
                    strategy.content().full_content()
                }
                DocumentObject::Initiative(initiative) => {
                    initiative.content().full_content()
                }
                DocumentObject::Task(task) => {
                    task.content().full_content()
                }
                DocumentObject::Adr(adr) => {
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
}