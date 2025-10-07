use anyhow::Result;
use metis_core::{
    application::services::document::{
        creation::DocumentCreationConfig, DeletionService, DocumentCreationService,
    },
    dal::Database,
    domain::documents::types::DocumentType,
};
use std::path::PathBuf;
use std::str::FromStr;

/// Service for document operations
pub struct DocumentService {
    workspace_dir: PathBuf,
}

impl DocumentService {
    pub fn new(workspace_dir: PathBuf) -> Self {
        Self { workspace_dir }
    }

    pub async fn create_document(
        &self,
        document_type: DocumentType,
        title: String,
        description: Option<String>,
        parent_id: Option<String>,
    ) -> Result<String> {
        let creation_service = DocumentCreationService::new(&self.workspace_dir);

        let config = DocumentCreationConfig {
            title,
            description,
            parent_id: parent_id
                .as_ref()
                .map(|id| metis_core::domain::documents::types::DocumentId::from(id.clone())),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        let result = match document_type {
            DocumentType::Vision => creation_service.create_vision(config).await?,
            DocumentType::Strategy => creation_service.create_strategy(config).await?,
            DocumentType::Initiative => {
                if let Some(parent_id) = &parent_id {
                    creation_service
                        .create_initiative(config, parent_id)
                        .await?
                } else {
                    // Create initiative without parent (streamlined configuration)
                    // Use "NULL" as strategy_id which the core service handles correctly
                    creation_service
                        .create_initiative_with_config(
                            config,
                            "NULL",
                            &metis_core::domain::configuration::FlightLevelConfig::streamlined(),
                        )
                        .await?
                }
            }
            DocumentType::Task => {
                if let Some(initiative_id) = &parent_id {
                    // For tasks with parent, we need both strategy and initiative IDs
                    // Try to create the task and let the creation service handle validation
                    match creation_service
                        .create_task(config, initiative_id, initiative_id)
                        .await
                    {
                        Ok(result) => result,
                        Err(e) => return Err(anyhow::anyhow!("Failed to create task: {}", e)),
                    }
                } else {
                    // For tasks without parent, create as backlog item
                    creation_service.create_backlog_item(config).await?
                }
            }
            DocumentType::Adr => creation_service.create_adr(config).await?,
        };

        Ok(result.document_id.to_string())
    }

    pub async fn delete_document(&self, file_path: &str) -> Result<()> {
        let deletion_service = DeletionService::new();
        deletion_service
            .delete_document_recursive(file_path)
            .await?;
        Ok(())
    }

    pub async fn load_documents_from_database(
        &self,
    ) -> Result<Vec<crate::models::DatabaseDocument>> {
        let db_path = self.workspace_dir.join("metis.db");
        let db = Database::new(&db_path.to_string_lossy())
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut repository = db.into_repository();
        let mut documents = Vec::new();

        // Collect all document types
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            if let Ok(mut docs) = repository.find_by_type(doc_type) {
                documents.append(&mut docs);
            }
        }

        Ok(documents
            .into_iter()
            .filter(|doc| !doc.archived) // Filter out archived documents
            .map(|doc| crate::models::DatabaseDocument {
                id: doc.id,
                title: doc.title,
                document_type: DocumentType::from_str(&doc.document_type).unwrap(),
                filepath: doc.filepath,
                archived: doc.archived,
            })
            .collect())
    }

    pub async fn save_document_content(&self, file_path: &str, new_content: &str) -> Result<()> {
        use metis_core::{Adr, Document, Initiative, Strategy, Task};

        // Load the document from file, update its content, then save it back
        let path = std::path::Path::new(file_path);

        // Try to determine document type from file path or load and inspect
        if file_path.contains("/strategies/") && file_path.ends_with("/strategy.md") {
            let mut strategy = Strategy::from_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load strategy: {}", e))?;
            strategy
                .update_content_body(new_content.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to update strategy content: {}", e))?;
            strategy
                .to_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save strategy: {}", e))?;
        } else if file_path.contains("/initiatives/") && file_path.ends_with("/initiative.md") {
            let mut initiative = Initiative::from_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load initiative: {}", e))?;
            initiative
                .update_content_body(new_content.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to update initiative content: {}", e))?;
            initiative
                .to_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save initiative: {}", e))?;
        } else if file_path.contains("/tasks/") {
            let mut task = Task::from_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load task: {}", e))?;
            task.update_content_body(new_content.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to update task content: {}", e))?;
            task.to_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save task: {}", e))?;
        } else if file_path.contains("/adrs/") {
            let mut adr = Adr::from_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load adr: {}", e))?;
            adr.update_content_body(new_content.to_string())
                .map_err(|e| anyhow::anyhow!("Failed to update adr content: {}", e))?;
            adr.to_file(path)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save adr: {}", e))?;
        } else {
            return Err(anyhow::anyhow!(
                "Unable to determine document type from path: {}",
                file_path
            ));
        }

        Ok(())
    }

    pub async fn create_child_task(
        &self,
        title: String,
        strategy_id: String,
        initiative_id: String,
    ) -> Result<String> {
        let creation_service = DocumentCreationService::new(&self.workspace_dir);

        let config = DocumentCreationConfig {
            title,
            description: None,
            parent_id: Some(metis_core::domain::documents::types::DocumentId::from(
                initiative_id.clone(),
            )),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };

        // Determine flight configuration based on strategy_id
        let flight_config = if strategy_id == "NULL" {
            metis_core::domain::configuration::FlightLevelConfig::streamlined()
        } else {
            metis_core::domain::configuration::FlightLevelConfig::full()
        };

        let result = creation_service
            .create_task_with_config(config, &strategy_id, &initiative_id, &flight_config)
            .await?;
        Ok(result.document_id.to_string())
    }

    pub async fn create_adr(&self, title: String, context: Option<String>) -> Result<PathBuf> {
        let creation_service = DocumentCreationService::new(&self.workspace_dir);

        let config = DocumentCreationConfig {
            title,
            description: context,
            parent_id: None, // ADRs are top-level documents
            tags: vec![],
            phase: None, // Will default to draft
            complexity: None,
            risk_level: None,
        };

        let result = creation_service.create_adr(config).await?;
        Ok(result.file_path)
    }
}
