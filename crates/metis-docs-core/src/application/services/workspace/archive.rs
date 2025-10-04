use crate::application::services::document::DocumentDiscoveryService;
use crate::application::services::DatabaseService;
use crate::domain::documents::traits::Document;
use crate::domain::documents::types::DocumentType;
use crate::Result;
use crate::{Adr, Initiative, MetisError, Strategy, Task, Vision};
use std::fs;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
    // Helper methods to reduce duplication

    /// Common helper for loading and marking a document as archived
    async fn mark_as_archived_helper(&self, file_path: &Path, doc_type: DocumentType) -> Result<()> {
        match doc_type {
            DocumentType::Vision => {
                let mut vision = Vision::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                vision.core_mut().archived = true;
                vision.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Strategy => {
                let mut strategy = Strategy::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                strategy.core_mut().archived = true;
                strategy.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Initiative => {
                let mut initiative = Initiative::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                initiative.core_mut().archived = true;
                initiative.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Task => {
                let mut task = Task::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                task.core_mut().archived = true;
                task.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
            DocumentType::Adr => {
                let mut adr = Adr::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                adr.core_mut().archived = true;
                adr.to_file(file_path).await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
            }
        }
        Ok(())
    }

    /// Helper to safely read directory contents
    fn read_dir_safe(&self, path: &Path) -> Result<fs::ReadDir> {
        fs::read_dir(path).map_err(|e| MetisError::FileSystem(e.to_string()))
    }

    /// Helper to check if a path is a markdown file
    fn is_markdown_file(&self, path: &Path) -> bool {
        path.is_file() && path.extension().is_some_and(|ext| ext == "md")
    }

    /// Helper to mark tasks in a directory as archived
    async fn mark_tasks_in_directory_as_archived(&self, dir_path: &Path) -> Result<()> {
        for entry in self.read_dir_safe(dir_path)? {
            let entry_path = entry
                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                .path();

            if self.is_markdown_file(&entry_path) {
                // Skip initiative.md itself
                if entry_path
                    .file_name()
                    .is_some_and(|name| name == "initiative.md")
                {
                    continue;
                }

                if Task::from_file(&entry_path).await.is_ok() {
                    self.mark_as_archived_helper(&entry_path, DocumentType::Task)
                        .await?;
                }
            }
            // Also check subdirectories for task files
            else if entry_path.is_dir() {
                if let Ok(subdir_entries) = fs::read_dir(&entry_path) {
                    for subentry in subdir_entries.flatten() {
                        let task_file_path = subentry.path();
                        if self.is_markdown_file(&task_file_path) {
                            if Task::from_file(&task_file_path).await.is_ok() {
                                self.mark_as_archived_helper(
                                    &task_file_path,
                                    DocumentType::Task,
                                )
                                .await?;
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Helper to archive initiatives and their tasks within a strategy
    async fn archive_strategy_initiatives(&self, initiatives_dir: &Path) -> Result<()> {
        if !initiatives_dir.exists() {
            return Ok(());
        }

        for entry in self.read_dir_safe(initiatives_dir)? {
            let initiative_dir = entry
                .map_err(|e| MetisError::FileSystem(e.to_string()))?
                .path();
            
            if initiative_dir.is_dir() {
                let initiative_file = initiative_dir.join("initiative.md");
                if initiative_file.exists() && Initiative::from_file(&initiative_file).await.is_ok() {
                    // Mark initiative as archived
                    self.mark_as_archived_helper(&initiative_file, DocumentType::Initiative)
                        .await?;
                    
                    // Mark all tasks in this initiative as archived
                    self.mark_tasks_in_directory_as_archived(&initiative_dir).await?;
                }
            }
        }
        Ok(())
    }

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
        let discovery_result = self
            .discovery_service
            .find_document_by_id(document_id)
            .await?;

        // Archive the document tree
        let archived_documents = self
            .archive_document_tree(&discovery_result.file_path, discovery_result.document_type)
            .await?;

        Ok(ArchiveResult {
            total_archived: archived_documents.len(),
            archived_documents,
        })
    }

    /// Archive a document using database lineage queries for efficient hierarchy discovery
    /// This method can be used when a database service is available for optimization
    pub async fn archive_document_with_database(
        &self, 
        document_id: &str, 
        db_service: &mut DatabaseService
    ) -> Result<ArchiveResult> {
        // Find the document in the database
        let doc = db_service.find_by_id(document_id)?.ok_or_else(|| {
            MetisError::DocumentNotFound {
                id: document_id.to_string(),
            }
        })?;

        let doc_type = DocumentType::from_str(&doc.document_type)
            .map_err(|e| MetisError::ValidationFailed { message: format!("Invalid document type: {}", e) })?;
        let mut archived_documents = Vec::new();

        match doc_type {
            DocumentType::Vision | DocumentType::Task | DocumentType::Adr => {
                // These document types don't have children, just archive the file
                let archived_doc = self.archive_single_file(&PathBuf::from(&doc.filepath), doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Strategy => {
                // Use database query to find all documents in strategy hierarchy
                let hierarchy_docs = db_service.find_strategy_hierarchy(document_id)?;
                
                // Mark all documents as archived first
                for db_doc in &hierarchy_docs {
                    let path = PathBuf::from(&db_doc.filepath);
                    let dt = DocumentType::from_str(&db_doc.document_type)
                        .map_err(|e| MetisError::ValidationFailed { message: format!("Invalid document type: {}", e) })?;
                    self.mark_as_archived_helper(&path, dt).await?;
                }

                // Archive the strategy directory (which moves everything intact)
                let strategy_path = PathBuf::from(&doc.filepath);
                let strategy_dir = strategy_path.parent().unwrap();
                let archived_doc = self.archive_directory(strategy_dir, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Initiative => {
                // Use database query to find all documents in initiative hierarchy
                let hierarchy_docs = db_service.find_initiative_hierarchy(document_id)?;
                
                // Mark all documents as archived first
                for db_doc in &hierarchy_docs {
                    let path = PathBuf::from(&db_doc.filepath);
                    let dt = DocumentType::from_str(&db_doc.document_type)
                        .map_err(|e| MetisError::ValidationFailed { message: format!("Invalid document type: {}", e) })?;
                    self.mark_as_archived_helper(&path, dt).await?;
                }

                // Archive the initiative directory (which moves everything intact)
                let initiative_path = PathBuf::from(&doc.filepath);
                let initiative_dir = initiative_path.parent().unwrap();
                let archived_doc = self.archive_directory(initiative_dir, doc_type).await?;
                archived_documents.push(archived_doc);
            }
        }

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
            DocumentType::Vision | DocumentType::Task | DocumentType::Adr => {
                // These document types don't have children, just archive the file
                let archived_doc = self.archive_single_file(file_path, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Strategy => {
                // Archive strategy and all its initiatives/tasks
                let strategy_dir = file_path.parent().unwrap();
                let initiatives_dir = strategy_dir.join("initiatives");

                // Mark all initiatives and their tasks as archived
                self.archive_strategy_initiatives(&initiatives_dir).await?;

                // Then archive the strategy directory (which moves everything intact)
                let archived_doc = self.archive_directory(strategy_dir, doc_type).await?;
                archived_documents.push(archived_doc);
            }

            DocumentType::Initiative => {
                // Archive initiative and all its tasks
                let initiative_dir = file_path.parent().unwrap();

                // Mark all tasks in this initiative as archived
                self.mark_tasks_in_directory_as_archived(initiative_dir).await?;

                // Then archive the initiative directory (which moves everything intact)
                let archived_doc = self.archive_directory(initiative_dir, doc_type).await?;
                archived_documents.push(archived_doc);
            }
        }

        Ok(archived_documents)
    }

    /// Archive a single file
    async fn archive_single_file(
        &self,
        file_path: &Path,
        doc_type: DocumentType,
    ) -> Result<ArchivedDocument> {
        // Calculate relative path from workspace
        let relative_path = file_path
            .strip_prefix(&self.workspace_dir)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Create archived path
        let archived_path = self.workspace_dir.join("archived").join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = archived_path.parent() {
            fs::create_dir_all(parent).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Mark as archived in frontmatter before moving
        self.mark_as_archived_helper(file_path, doc_type).await?;

        // Get document ID before moving
        let document_id = self.get_document_id(file_path, doc_type).await?;

        // Move the file
        fs::rename(file_path, &archived_path).map_err(|e| MetisError::FileSystem(e.to_string()))?;

        Ok(ArchivedDocument {
            document_id,
            document_type: doc_type,
            original_path: file_path.to_path_buf(),
            archived_path,
        })
    }

    /// Archive a directory (for strategies and initiatives)
    async fn archive_directory(
        &self,
        dir_path: &Path,
        doc_type: DocumentType,
    ) -> Result<ArchivedDocument> {
        // Calculate relative path from workspace
        let relative_path = dir_path
            .strip_prefix(&self.workspace_dir)
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;

        // Create archived path
        let archived_path = self.workspace_dir.join("archived").join(relative_path);

        // Ensure parent directory exists
        if let Some(parent) = archived_path.parent() {
            fs::create_dir_all(parent).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        // Get document ID before moving
        let main_file = match doc_type {
            DocumentType::Strategy => dir_path.join("strategy.md"),
            DocumentType::Initiative => dir_path.join("initiative.md"),
            _ => {
                return Err(MetisError::InvalidDocument(
                    "Invalid document type for directory archive".to_string(),
                ))
            }
        };

        // Mark as archived in frontmatter before moving
        self.mark_as_archived_helper(&main_file, doc_type).await?;

        let document_id = self.get_document_id(&main_file, doc_type).await?;

        // Move the entire directory (including the main document file)
        // Children should already be archived and their frontmatter marked as archived
        // Handle case where archived directory already exists by merging contents
        if archived_path.exists() {
            // Target exists, merge by moving individual files/subdirs
            self.merge_directory_contents(dir_path, &archived_path)
                .await?;
            // Remove the now-empty source directory
            fs::remove_dir_all(dir_path).map_err(|e| MetisError::FileSystem(e.to_string()))?;
        } else {
            // Target doesn't exist, can use simple rename
            fs::rename(dir_path, &archived_path)
                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        }

        Ok(ArchivedDocument {
            document_id,
            document_type: doc_type,
            original_path: dir_path.to_path_buf(),
            archived_path,
        })
    }


    /// Merge directory contents by moving files/subdirs from source to target
    /// Handles conflicts by overwriting (source takes precedence)
    async fn merge_directory_contents(&self, source_dir: &Path, target_dir: &Path) -> Result<()> {
        for entry in fs::read_dir(source_dir).map_err(|e| MetisError::FileSystem(e.to_string()))? {
            let entry = entry.map_err(|e| MetisError::FileSystem(e.to_string()))?;
            let source_path = entry.path();
            let file_name = source_path.file_name().unwrap();
            let target_path = target_dir.join(file_name);

            if source_path.is_dir() {
                // Recursively merge subdirectories
                if target_path.exists() {
                    // Target subdir exists, merge recursively
                    Box::pin(self.merge_directory_contents(&source_path, &target_path)).await?;
                    // Remove now-empty source subdir
                    if let Ok(entries) = fs::read_dir(&source_path) {
                        if entries.count() == 0 {
                            fs::remove_dir(&source_path)
                                .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                        }
                    }
                } else {
                    // Target subdir doesn't exist, can move entire directory
                    fs::rename(&source_path, &target_path)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                }
            } else {
                // Move file (overwrite if exists)
                if target_path.exists() {
                    fs::remove_file(&target_path)
                        .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                }
                fs::rename(&source_path, &target_path)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?;
            }
        }
        Ok(())
    }



    /// Get document ID from a file
    async fn get_document_id(&self, file_path: &Path, doc_type: DocumentType) -> Result<String> {
        match doc_type {
            DocumentType::Vision => {
                let vision = Vision::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(vision.id().to_string())
            }
            DocumentType::Strategy => {
                let strategy = Strategy::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(strategy.id().to_string())
            }
            DocumentType::Initiative => {
                let initiative = Initiative::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(initiative.id().to_string())
            }
            DocumentType::Task => {
                let task = Task::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(task.id().to_string())
            }
            DocumentType::Adr => {
                let adr = Adr::from_file(file_path)
                    .await
                    .map_err(|e| MetisError::InvalidDocument(e.to_string()))?;
                Ok(adr.id().to_string())
            }
        }
    }

    /// Check if a document is already archived
    pub async fn is_document_archived(&self, document_id: &str) -> Result<bool> {
        // First check if the document exists in the regular workspace
        match self
            .discovery_service
            .find_document_by_id(document_id)
            .await
        {
            Ok(result) => {
                // Check if the file is in the archived directory
                let relative_path = result
                    .file_path
                    .strip_prefix(&self.workspace_dir)
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?;
                Ok(relative_path.starts_with("archived"))
            }
            Err(MetisError::NotFound(_)) => {
                // If not found in regular workspace, check in archived directory
                let archived_docs = self.get_archived_documents().await?;
                Ok(archived_docs
                    .iter()
                    .any(|doc| doc.document_id == document_id))
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
        self.scan_archived_directory(&archived_dir, &mut archived_docs)
            .await?;
        Ok(archived_docs)
    }

    /// Recursively scan archived directory for documents
    async fn scan_archived_directory(
        &self,
        dir: &Path,
        results: &mut Vec<ArchivedDocument>,
    ) -> Result<()> {
        for entry in fs::read_dir(dir).map_err(|e| MetisError::FileSystem(e.to_string()))? {
            let entry = entry.map_err(|e| MetisError::FileSystem(e.to_string()))?;
            let path = entry.path();

            if path.is_dir() {
                Box::pin(self.scan_archived_directory(&path, results)).await?;
            } else if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
                // Try to determine document type and extract info
                if let Ok(doc_type) = self.determine_document_type(&path).await {
                    if let Ok(document_id) = self.get_document_id(&path, doc_type).await {
                        // Calculate original path
                        let archived_relative = path
                            .strip_prefix(self.workspace_dir.join("archived"))
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

        Err(MetisError::InvalidDocument(
            "Could not determine document type".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::document::creation::DocumentCreationConfig;
    use crate::application::services::document::DocumentCreationService;
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
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();

        // Archive the vision
        let archive_service = ArchiveService::new(&workspace_dir);
        let archive_result = archive_service
            .archive_document(&creation_result.document_id.to_string())
            .await
            .unwrap();

        assert_eq!(archive_result.total_archived, 1);
        assert_eq!(
            archive_result.archived_documents[0].document_type,
            DocumentType::Vision
        );
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
            complexity: None,
            risk_level: None,
        };
        let strategy_result = creation_service
            .create_strategy(strategy_config)
            .await
            .unwrap();

        // Create an initiative under the strategy
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("A test initiative".to_string()),
            parent_id: Some(strategy_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let _initiative_result = creation_service
            .create_initiative(initiative_config, &strategy_result.document_id.to_string())
            .await
            .unwrap();

        // Archive the strategy (should archive the initiative too)
        let archive_service = ArchiveService::new(&workspace_dir);
        let archive_result = archive_service
            .archive_document(&strategy_result.document_id.to_string())
            .await
            .unwrap();

        // Should have archived the strategy directory (which contains initiative and tasks)
        assert_eq!(archive_result.total_archived, 1);
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
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();
        archive_service
            .archive_document(&creation_result.document_id.to_string())
            .await
            .unwrap();

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
            complexity: None,
            risk_level: None,
        };
        let creation_result = creation_service.create_vision(config).await.unwrap();
        let document_id = creation_result.document_id.to_string();

        // Should not be archived initially
        assert!(!archive_service
            .is_document_archived(&document_id)
            .await
            .unwrap());

        // Archive the document
        archive_service
            .archive_document(&document_id)
            .await
            .unwrap();

        // Should be archived now
        assert!(archive_service
            .is_document_archived(&document_id)
            .await
            .unwrap());
    }
}
