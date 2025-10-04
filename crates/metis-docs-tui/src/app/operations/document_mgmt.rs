use crate::app::App;
use crate::error::AppError;
use anyhow::Result;
use metis_core::application::services::workspace::ArchiveService;

impl App {
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
                // Create archive service with database optimization
                let db = match metis_core::dal::Database::new(&workspace_dir.join("metis.db").to_string_lossy()) {
                    Ok(db) => db,
                    Err(e) => {
                        self.add_error_message(format!("Database error: {}", e));
                        return Ok(());
                    }
                };
                let mut db_service = metis_core::application::services::DatabaseService::new(db.into_repository());
                let archive_service = ArchiveService::new(workspace_dir);
                match archive_service.archive_document(&selected_item.id(), &mut db_service).await {
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
                        let error_msg = format!("Failed to transition document: {}", e);
                        self.add_error_message(error_msg.clone());
                        self.error_handler.handle_error(AppError::from(e));
                    }
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
}