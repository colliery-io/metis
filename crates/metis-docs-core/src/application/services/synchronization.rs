use crate::application::services::{DatabaseService, FilesystemService};
use crate::dal::database::models::{Document, NewDocument};
use crate::domain::documents::{factory::DocumentFactory, traits::Document as DocumentTrait};
use crate::{MetisError, Result};
use serde_json;
use std::path::Path;

/// Synchronization service - bridges filesystem and database
pub struct SyncService<'a> {
    db_service: &'a mut DatabaseService,
}

impl<'a> SyncService<'a> {
    pub fn new(db_service: &'a mut DatabaseService) -> Self {
        Self { db_service }
    }

    /// Direction 1: File → DocumentObject → Database
    /// Load a document from filesystem and store in database
    pub async fn import_from_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<Document> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();

        // Use DocumentFactory to parse file into domain object
        let document_obj = DocumentFactory::from_file(&file_path).await.map_err(|e| {
            MetisError::ValidationFailed {
                message: format!("Failed to parse document: {}", e),
            }
        })?;

        // Get file metadata
        let file_hash = FilesystemService::compute_file_hash(&file_path)?;
        let updated_at = FilesystemService::get_file_mtime(&file_path)?;
        let content = FilesystemService::read_file(&file_path)?;

        // Convert domain object to database model
        let new_doc = self.domain_to_database_model(
            document_obj.as_ref(),
            &path_str,
            file_hash,
            updated_at,
            content,
        )?;

        // Store in database
        self.db_service.create_document(new_doc)
    }

    /// Direction 2: Database → DocumentObject → File  
    /// Export a document from database to filesystem
    pub async fn export_to_file(&mut self, filepath: &str) -> Result<()> {
        // Get document from database
        let db_doc = self.db_service.find_by_filepath(filepath)?.ok_or_else(|| {
            MetisError::DocumentNotFound {
                id: filepath.to_string(),
            }
        })?;

        // Get content from database
        let content = db_doc.content.ok_or_else(|| MetisError::ValidationFailed {
            message: "Document has no content".to_string(),
        })?;

        // Write to filesystem
        FilesystemService::write_file(filepath, &content)?;

        Ok(())
    }

    /// Convert domain object to database model
    fn domain_to_database_model(
        &self,
        document_obj: &dyn DocumentTrait,
        filepath: &str,
        file_hash: String,
        updated_at: f64,
        content: String,
    ) -> Result<NewDocument> {
        let core = document_obj.core();
        let phase = document_obj
            .phase()
            .map_err(|e| MetisError::ValidationFailed {
                message: format!("Failed to get document phase: {}", e),
            })?
            .to_string();

        Ok(NewDocument {
            filepath: filepath.to_string(),
            id: document_obj.id().to_string(),
            title: core.title.clone(),
            document_type: document_obj.document_type().to_string(),
            created_at: core.metadata.created_at.timestamp() as f64,
            updated_at,
            archived: core.archived,
            exit_criteria_met: document_obj.exit_criteria_met(),
            file_hash,
            frontmatter_json: serde_json::to_string(&core.metadata).map_err(MetisError::Json)?,
            content: Some(content),
            phase,
        })
    }

    /// Synchronize a single file between filesystem and database using directional methods
    pub async fn sync_file<P: AsRef<Path>>(&mut self, file_path: P) -> Result<SyncResult> {
        let path_str = file_path.as_ref().to_string_lossy().to_string();

        // Check if file exists on filesystem
        let file_exists = FilesystemService::file_exists(&file_path);

        // Check if document exists in database
        let db_doc = self.db_service.find_by_filepath(&path_str)?;

        match (file_exists, db_doc) {
            // File exists, not in database - import it
            (true, None) => {
                self.import_from_file(&file_path).await?;
                Ok(SyncResult::Imported { filepath: path_str })
            }

            // File doesn't exist, but in database - remove from database
            (false, Some(_)) => {
                self.db_service.delete_document(&path_str)?;
                Ok(SyncResult::Deleted { filepath: path_str })
            }

            // Both exist - check if file changed
            (true, Some(db_doc)) => {
                let current_hash = FilesystemService::compute_file_hash(&file_path)?;

                if db_doc.file_hash != current_hash {
                    // File changed, reimport (file is source of truth)
                    self.db_service.delete_document(&path_str)?;
                    self.import_from_file(&file_path).await?;
                    Ok(SyncResult::Updated { filepath: path_str })
                } else {
                    Ok(SyncResult::UpToDate { filepath: path_str })
                }
            }

            // Neither exists
            (false, None) => Ok(SyncResult::NotFound { filepath: path_str }),
        }
    }

    /// Sync all markdown files in a directory
    pub async fn sync_directory<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<Vec<SyncResult>> {
        let mut results = Vec::new();

        // Find all markdown files
        let files = FilesystemService::find_markdown_files(&dir_path)?;

        // Sync each file
        for file_path in files {
            match self.sync_file(&file_path).await {
                Ok(result) => results.push(result),
                Err(e) => results.push(SyncResult::Error {
                    filepath: file_path,
                    error: e.to_string(),
                }),
            }
        }

        // Check for orphaned database entries (files that were deleted)
        let db_pairs = self.db_service.get_all_id_filepath_pairs()?;
        for (_, filepath) in db_pairs {
            if !FilesystemService::file_exists(&filepath) {
                // File no longer exists, delete from database
                match self.db_service.delete_document(&filepath) {
                    Ok(_) => results.push(SyncResult::Deleted { filepath }),
                    Err(e) => results.push(SyncResult::Error {
                        filepath,
                        error: e.to_string(),
                    }),
                }
            }
        }

        Ok(results)
    }

    /// Verify database and filesystem are in sync
    pub fn verify_sync<P: AsRef<Path>>(&mut self, dir_path: P) -> Result<Vec<SyncIssue>> {
        let mut issues = Vec::new();

        // Find all markdown files
        let files = FilesystemService::find_markdown_files(&dir_path)?;

        // Check each file
        for file_path in &files {
            let path_str = file_path.to_string();

            if let Some(db_doc) = self.db_service.find_by_filepath(&path_str)? {
                let current_hash = FilesystemService::compute_file_hash(file_path)?;
                if db_doc.file_hash != current_hash {
                    issues.push(SyncIssue::OutOfSync {
                        filepath: path_str,
                        reason: "File hash mismatch".to_string(),
                    });
                }
            } else {
                issues.push(SyncIssue::MissingFromDatabase { filepath: path_str });
            }
        }

        // Check for orphaned database entries
        let db_pairs = self.db_service.get_all_id_filepath_pairs()?;
        for (_, filepath) in db_pairs {
            if !files.contains(&filepath) && !FilesystemService::file_exists(&filepath) {
                issues.push(SyncIssue::MissingFromFilesystem {
                    filepath: filepath.clone(),
                });
            }
        }

        Ok(issues)
    }
}

/// Result of synchronizing a single document
#[derive(Debug, Clone, PartialEq)]
pub enum SyncResult {
    Imported { filepath: String },
    Updated { filepath: String },
    Deleted { filepath: String },
    UpToDate { filepath: String },
    NotFound { filepath: String },
    Error { filepath: String, error: String },
}

impl SyncResult {
    /// Get the filepath for this result
    pub fn filepath(&self) -> &str {
        match self {
            SyncResult::Imported { filepath }
            | SyncResult::Updated { filepath }
            | SyncResult::Deleted { filepath }
            | SyncResult::UpToDate { filepath }
            | SyncResult::NotFound { filepath }
            | SyncResult::Error { filepath, .. } => filepath,
        }
    }

    /// Check if this result represents a change
    pub fn is_change(&self) -> bool {
        matches!(
            self,
            SyncResult::Imported { .. } | SyncResult::Updated { .. } | SyncResult::Deleted { .. }
        )
    }

    /// Check if this result represents an error
    pub fn is_error(&self) -> bool {
        matches!(self, SyncResult::Error { .. })
    }
}

/// Issues found during sync verification
#[derive(Debug, Clone)]
pub enum SyncIssue {
    MissingFromDatabase { filepath: String },
    MissingFromFilesystem { filepath: String },
    OutOfSync { filepath: String, reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::Database;
    use tempfile::tempdir;

    fn setup_services() -> (tempfile::TempDir, DatabaseService) {
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let db = Database::new(":memory:").expect("Failed to create test database");
        let db_service = DatabaseService::new(db.into_repository());
        (temp_dir, db_service)
    }

    fn create_test_document_content() -> String {
        "---\n".to_string()
            + "title: Test Document\n"
            + "level: vision\n"
            + "created_at: \"2021-01-01T00:00:00Z\"\n"
            + "updated_at: \"2021-01-01T00:00:00Z\"\n"
            + "archived: false\n"
            + "exit_criteria_met: false\n"
            + "tags:\n"
            + "  - \"#phase/draft\"\n"
            + "---\n\n"
            + "# Test Document\n\n"
            + "Test content.\n"
    }

    #[tokio::test]
    async fn test_import_from_file() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service);

        let file_path = temp_dir.path().join("test.md");
        FilesystemService::write_file(&file_path, &create_test_document_content())
            .expect("Failed to write file");

        let doc = sync_service
            .import_from_file(&file_path)
            .await
            .expect("Failed to import");
        assert_eq!(doc.title, "Test Document");
        assert_eq!(doc.document_type, "vision");

        // Verify it's in the database
        assert!(db_service
            .document_exists(&file_path.to_string_lossy())
            .expect("Failed to check"));
    }

    #[tokio::test]
    async fn test_sync_file_operations() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service);

        let file_path = temp_dir.path().join("test.md");
        let path_str = file_path.to_string_lossy().to_string();

        // Test sync when file doesn't exist
        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::NotFound {
                filepath: path_str.clone()
            }
        );

        // Create file and sync
        FilesystemService::write_file(&file_path, &create_test_document_content())
            .expect("Failed to write file");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Imported {
                filepath: path_str.clone()
            }
        );

        // Sync again - should be up to date
        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::UpToDate {
                filepath: path_str.clone()
            }
        );

        // Modify file
        let modified_content =
            &create_test_document_content().replace("Test content.", "Modified content.");
        FilesystemService::write_file(&file_path, modified_content).expect("Failed to write");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Updated {
                filepath: path_str.clone()
            }
        );

        // Delete file
        FilesystemService::delete_file(&file_path).expect("Failed to delete");

        let result = sync_service
            .sync_file(&file_path)
            .await
            .expect("Failed to sync");
        assert_eq!(
            result,
            SyncResult::Deleted {
                filepath: path_str.clone()
            }
        );

        // Verify it's gone from database
        assert!(!db_service
            .document_exists(&path_str)
            .expect("Failed to check"));
    }

    #[tokio::test]
    async fn test_sync_directory() {
        let (temp_dir, mut db_service) = setup_services();
        let mut sync_service = SyncService::new(&mut db_service);

        // Create multiple files
        let files = vec![
            ("doc1.md", "test-1"),
            ("subdir/doc2.md", "test-2"),
            ("subdir/nested/doc3.md", "test-3"),
        ];

        for (file_path, id) in &files {
            let full_path = temp_dir.path().join(file_path);
            let content = &create_test_document_content()
                .replace("Test Document", &format!("Test Document {}", id))
                .replace("test-doc-1", id);
            FilesystemService::write_file(&full_path, content).expect("Failed to write");
        }

        // Sync directory
        let results = sync_service
            .sync_directory(temp_dir.path())
            .await
            .expect("Failed to sync directory");

        // Should have 3 imports
        let imports = results
            .iter()
            .filter(|r| matches!(r, SyncResult::Imported { .. }))
            .count();
        assert_eq!(imports, 3);

        // Sync again - all should be up to date
        let results = sync_service
            .sync_directory(temp_dir.path())
            .await
            .expect("Failed to sync directory");
        let up_to_date = results
            .iter()
            .filter(|r| matches!(r, SyncResult::UpToDate { .. }))
            .count();
        assert_eq!(up_to_date, 3);

        // Check that we have results for all files
        for (file_path, _) in &files {
            let full_path = temp_dir
                .path()
                .join(file_path)
                .to_string_lossy()
                .to_string();
            assert!(results.iter().any(|r| r.filepath() == full_path));
        }
    }
}
