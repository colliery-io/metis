use crate::domain::documents::types::DocumentType;
use crate::domain::documents::traits::Document;
use crate::application::services::document::DocumentDiscoveryService;
use crate::{Vision, Strategy, Initiative, Task, Adr, MetisError};
use crate::Result;
use std::path::{Path, PathBuf};
use std::fs;

/// Service for archiving documents and managing the archived folder structure
pub struct ArchiveService {
    workspace_dir: PathBuf,
    discovery_service: DocumentDiscoveryService,
}

/// Result of archive operation
#[derive(Debug)]
pub struct ArchiveResult {
    pub archived_documents: Vec<ArchivedDocument>,
    pub total_archived: usize,
}

/// Information about an archived document
#[derive(Debug)]
pub struct ArchivedDocument {
    pub document_id: String,
    pub document_type: DocumentType,
    pub original_path: PathBuf,
    pub archived_path: PathBuf,
}

impl ArchiveService {
    /// Create a new archive service for a workspace
    pub fn new<P: AsRef<Path>>(workspace_dir: P) -> Self {
        let workspace_dir = workspace_dir.as_ref().to_path_buf();
        let discovery_service = DocumentDiscoveryService::new(&workspace_dir);
        
        Self {
            workspace_dir,
            discovery_service,
        }
    }

    /// Archive a document and all its children
    pub async fn archive_document(&self, document_id: &str) -> Result<ArchiveResult> {
        // Find the document
        let discovery_result = self.discovery_service.find_document_by_id(document_id).await?;
        
        // Archive the document tree
        let archived_documents = self.archive_document_tree(
            &discovery_result.file_path,
            discovery_result.document_type,
        ).await?;

        Ok(ArchiveResult {
            total_archived: archived_documents.len(),
            archived_documents,
        })
    }

    /// Archive a document tree starting from a specific file
    async fn archive_document_tree(
        &self,
        file_path: &Path,
        doc_type: DocumentType,
    ) -> Result<Vec<ArchivedDocument>> {
        let mut archived_documents = Vec::new();

        match doc_type {
            DocumentType::Vision => {
                // Archive the vision document
                let archived_doc = self.archive_single_file(file_path, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Strategy => {
                // Archive strategy and all its initiatives (which will archive their tasks)
                let strategy_dir = file_path.parent().unwrap();
                let initiatives_dir = strategy_dir.join("initiatives");

                // First archive all initiatives (and their tasks)
                if initiatives_dir.exists() {
                    for entry in fs::read_dir(&initiatives_dir)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                    {
                        let initiative_dir = entry
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?
                            .path();
                        if initiative_dir.is_dir() {
                            let initiative_file = initiative_dir.join("initiative.md");
                            if initiative_file.exists() {
                                if let Ok(_initiative) = Initiative::from_file(&initiative_file).await {
                                    let mut child_archived = Box::pin(self.archive_document_tree(
                                        &initiative_file,
                                        DocumentType::Initiative,
                                    )).await?;
                                    archived_documents.append(&mut child_archived);
                                }
                            }
                        }
                    }
                }

                // Then archive the strategy by just marking the file as archived and moving it
                let archived_doc = self.archive_single_file(file_path, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Initiative => {
                // Archive initiative and all its tasks
                let initiative_dir = file_path.parent().unwrap();

                // First archive all tasks in this initiative
                for entry in fs::read_dir(initiative_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?
                {
                    let task_path = entry
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?
                        .path();
                    if task_path.is_file() && task_path.extension().map_or(false, |ext| ext == "md") {
                        // Skip initiative.md itself
                        if task_path.file_name().map_or(false, |name| name == "initiative.md") {
                            continue;
                        }

                        if let Ok(_task) = Task::from_file(&task_path).await {
                            let archived_doc = self.archive_single_file(&task_path, DocumentType::Task).await?;
                            archived_documents.push(archived_doc);
                        }
                    }
                }

                // Then archive the initiative directory
                let archived_doc = self.archive_directory(initiative_dir, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Task => {
                // Tasks don't have children, just archive the file
                let archived_doc = self.archive_single_file(file_path, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Adr => {
                // ADRs don't have children, just archive the file
                let archived_doc = self.archive_single_file(file_path, doc_type).await?;
                archived_documents.push(archived_doc);
            }
        }

        Ok(archived_documents)
    }

    /// Archive a single file
    async fn archive_single_file(&self, file_path: &Path, doc_type: DocumentType) -> Result<ArchivedDocument> {
        // Calculate relative path from workspace
        let relative_path = file_path.strip_prefix(&self.workspace_dir)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Create archived path
        let archived_path = self.workspace_dir.join("archived").join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = archived_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Mark as archived in frontmatter before moving
        self.mark_document_as_archived(file_path, doc_type).await?;

        // Get document ID before moving
        let document_id = self.get_document_id(file_path, doc_type).await?;

        // Move the file
        fs::rename(file_path, &archived_path)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        Ok(ArchivedDocument {
            document_id,
            document_type: doc_type,
            original_path: file_path.to_path_buf(),
            archived_path,
        })
    }

    /// Archive a directory (for strategies and initiatives)
    async fn archive_directory(&self, dir_path: &Path, doc_type: DocumentType) -> Result<ArchivedDocument> {
        // Calculate relative path from workspace
        let relative_path = dir_path.strip_prefix(&self.workspace_dir)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Create archived path
        let archived_path = self.workspace_dir.join("archived").join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = archived_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Get document ID before moving
        let main_file = match doc_type {
            DocumentType::Strategy => dir_path.join("strategy.md"),
            DocumentType::Initiative => dir_path.join("initiative.md"),
            _ => return Err(MetisError::InvalidDocument("Invalid document type for directory archive".to_string())),
        };

        let document_id = self.get_document_id(&main_file, doc_type).await?;

        // Move the entire directory
        fs::rename(dir_path, &archived_path)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        Ok(ArchivedDocument {
            document_id,
            document_type: doc_type,
            original_path: dir_path.to_path_buf(),
            archived_path,
        })
    }

    /// Mark a document as archived by updating its frontmatter
    async fn mark_document_as_archived(&self, file_path: &Path, doc_type: DocumentType) -> Result<()> {
        match doc_type {
            DocumentType::Vision => {
                let mut vision = Vision::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                vision.core_mut().archived = true;
                vision.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Strategy => {
                let mut strategy = Strategy::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                strategy.core_mut().archived = true;
                strategy.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Initiative => {
                let mut initiative = Initiative::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                initiative.core_mut().archived = true;
                initiative.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Task => {
                let mut task = Task::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                task.core_mut().archived = true;
                task.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Adr => {
                let mut adr = Adr::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                adr.core_mut().archived = true;
                adr.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Get document ID from a file
    async fn get_document_id(&self, file_path: &Path, doc_type: DocumentType) -> Result<String> {
        match doc_type {
            DocumentType::Vision => {
                let vision = Vision::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(vision.id().to_string())
            }
            DocumentType::Strategy => {
                let strategy = Strategy::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(strategy.id().to_string())
            }
            DocumentType::Initiative => {
                let initiative = Initiative::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(initiative.id().to_string())
            }
            DocumentType::Task => {
                let task = Task::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(task.id().to_string())
            }
            DocumentType::Adr => {
                let adr = Adr::from_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(adr.id().to_string())
            }
        }
    }

    /// Check if a document is already archived
    pub async fn is_document_archived(&self, document_id: &str) -> Result<bool> {
        // First check if the document exists in the regular workspace
        match self.discovery_service.find_document_by_id(document_id).await {
            Ok(result) => {
                // Check if the file is in the archived directory
                let relative_path = result.file_path.strip_prefix(&self.workspace_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                Ok(relative_path.starts_with("archived"))
            }
            Err(MetisError::NotFound(_)) => {
                // If not found in regular workspace, check in archived directory
                let archived_docs = self.get_archived_documents().await?;
                Ok(archived_docs.iter().any(|doc| doc.document_id == document_id))
            }
            Err(e) => Err(e),
        }
    }

    /// Get all archived documents
    pub async fn get_archived_documents(&self) -> Result<Vec<ArchivedDocument>> {
        let archived_dir = self.workspace_dir.join("archived");
        if !archived_dir.exists() {
            return Ok(Vec::new());
        }

        let mut archived_docs = Vec::new();
        self.scan_archived_directory(&archived_dir, &mut archived_docs).await?;
        Ok(archived_docs)
    }

    /// Recursively scan archived directory for documents
    async fn scan_archived_directory(&self, dir: &Path, results: &mut Vec<ArchivedDocument>) -> Result<()> {
        for entry in fs::read_dir(dir)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?
        {
            let entry = entry.map_err(|e| MetisError::FileSystem(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                Box::pin(self.scan_archived_directory(&path, results)).await?;
            } else if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                // Try to determine document type and extract info
                if let Ok(doc_type) = self.determine_document_type(&path).await {
                    if let Ok(document_id) = self.get_document_id(&path, doc_type).await {
                        // Calculate original path
                        let archived_relative = path.strip_prefix(&self.workspace_dir.join("archived"))
                            .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                        let original_path = self.workspace_dir.join(archived_relative);

                        results.push(ArchivedDocument {
                            document_id,
                            document_type: doc_type,
                            original_path,
                            archived_path: path,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// Determine document type from file path and content
    async fn determine_document_type(&self, file_path: &Path) -> Result<DocumentType> {
        // Try each document type
        if Vision::from_file(file_path).await.is_ok() {
            return Ok(DocumentType::Vision);
        }
        if Strategy::from_file(file_path).await.is_ok() {
            return Ok(DocumentType::Strategy);
        }
        if Initiative::from_file(file_path).await.is_ok() {
            return Ok(DocumentType::Initiative);
        }
        if Task::from_file(file_path).await.is_ok() {
            return Ok(DocumentType::Task);
        }
        if Adr::from_file(file_path).await.is_ok() {
            return Ok(DocumentType::Adr);
        }

        Err(MetisError::InvalidDocument("Could not determine document type".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::document::DocumentCreationService;
    use crate::application::services::document::creation::DocumentCreationConfig;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_archive_vision_document() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        // Create a vision document
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();

        // Archive the vision
        let archive_service = ArchiveService::new(&workspace_dir);
        let archive_result = archive_service.archive_document(&creation_result.document_id.to_string()).await.unwrap();

        assert_eq!(archive_result.total_archived, 1);
        assert_eq!(archive_result.archived_documents[0].document_type, DocumentType::Vision);
        assert!(archive_result.archived_documents[0].archived_path.exists());
        assert!(!creation_result.file_path.exists());
    }

    #[tokio::test]
    async fn test_archive_strategy_with_initiatives() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        let creation_service = DocumentCreationService::new(&workspace_dir);

        // Create a strategy
        let strategy_config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: Some("A test strategy".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
        };
        let strategy_result = creation_service.create_strategy(strategy_config).await.unwrap();

        // Create an initiative under the strategy
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("A test initiative".to_string()),
            parent_id: Some(strategy_result.document_id.clone()),
            tags: vec![],
            phase: None,
        };
        let _initiative_result = creation_service.create_initiative(
            initiative_config, 
            &strategy_result.document_id.to_string()
        ).await.unwrap();

        // Archive the strategy (should archive the initiative too)
        let archive_service = ArchiveService::new(&workspace_dir);
        let archive_result = archive_service.archive_document(&strategy_result.document_id.to_string()).await.unwrap();

        // Should have archived both the initiative and the strategy
        assert_eq!(archive_result.total_archived, 2);
        assert!(!strategy_result.file_path.exists());
    }

    #[tokio::test]
    async fn test_get_archived_documents() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        let creation_service = DocumentCreationService::new(&workspace_dir);
        let archive_service = ArchiveService::new(&workspace_dir);

        // Create and archive a vision document
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();
        archive_service.archive_document(&creation_result.document_id.to_string()).await.unwrap();

        // Get all archived documents
        let archived_docs = archive_service.get_archived_documents().await.unwrap();
        assert_eq!(archived_docs.len(), 1);
        assert_eq!(archived_docs[0].document_type, DocumentType::Vision);
    }

    #[tokio::test]
    async fn test_is_document_archived() {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().join(".metis");
        fs::create_dir_all(&workspace_dir).unwrap();

        let creation_service = DocumentCreationService::new(&workspace_dir);
        let archive_service = ArchiveService::new(&workspace_dir);

        // Create a vision document
        let config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("A test vision".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();
        let document_id = creation_result.document_id.to_string();

        // Should not be archived initially
        assert!(!archive_service.is_document_archived(&document_id).await.unwrap());

        // Archive the document
        archive_service.archive_document(&document_id).await.unwrap();

        // Should be archived now
        assert!(archive_service.is_document_archived(&document_id).await.unwrap());
    }
}