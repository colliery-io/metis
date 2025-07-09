use anyhow::Result;
use std::path::PathBuf;
use std::str::FromStr;
use metis_core::{
    application::services::document::{
        DocumentCreationService, 
        DeletionService,
        creation::DocumentCreationConfig,
    },
    dal::Database,
    domain::documents::types::DocumentType,
};
use crate::models::KanbanItem;

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
            parent_id: parent_id.as_ref().map(|id| metis_core::domain::documents::types::DocumentId::from(id.clone())),
            tags: vec![],
            phase: None,
        };

        let result = match document_type {
            DocumentType::Vision => creation_service.create_vision(config).await?,
            DocumentType::Strategy => creation_service.create_strategy(config).await?,
            DocumentType::Initiative => {
                if let Some(parent_id) = &parent_id {
                    creation_service.create_initiative(config, parent_id).await?
                } else {
                    return Err(anyhow::anyhow!("Initiative requires a parent strategy"));
                }
            }
            DocumentType::Task => {
                if let Some(initiative_id) = &parent_id {
                    // For tasks, we need both strategy and initiative IDs
                    // Try to create the task and let the creation service handle validation
                    match creation_service.create_task(config, initiative_id, initiative_id).await {
                        Ok(result) => result,
                        Err(e) => return Err(anyhow::anyhow!("Failed to create task: {}", e)),
                    }
                } else {
                    return Err(anyhow::anyhow!("Task requires a parent initiative"));
                }
            }
            DocumentType::Adr => {
                creation_service.create_adr(config, "", vec![]).await?
            }
        };

        Ok(result.document_id.to_string())
    }

    pub async fn delete_document(&self, file_path: &str) -> Result<()> {
        let deletion_service = DeletionService::new(self.workspace_dir.clone());
        deletion_service.delete_document_recursive(file_path).await?;
        Ok(())
    }

    pub async fn load_documents_from_database(&self) -> Result<Vec<crate::models::DatabaseDocument>> {
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
        
        Ok(documents.into_iter().map(|doc| {
            // Parse parent_id from frontmatter_json
            let parent_id = if let Ok(frontmatter) = serde_json::from_str::<serde_json::Value>(&doc.frontmatter_json) {
                frontmatter.get("parent_id")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
            } else {
                None
            };
            
            crate::models::DatabaseDocument {
                id: doc.id,
                title: doc.title,
                description: None,
                document_type: DocumentType::from_str(&doc.document_type).unwrap(),
                phase: doc.phase,
                filepath: doc.filepath,
                parent_id,
                created_at: doc.created_at.to_string(),
                updated_at: doc.updated_at.to_string(),
            }
        }).collect())
    }

    pub async fn save_document_content(&self, file_path: &str, content: &str) -> Result<()> {
        tokio::fs::write(file_path, content).await?;
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
            parent_id: Some(metis_core::domain::documents::types::DocumentId::from(initiative_id.clone())),
            tags: vec![],
            phase: None,
        };

        let result = creation_service.create_task(config, &strategy_id, &initiative_id).await?;
        Ok(result.document_id.to_string())
    }
}