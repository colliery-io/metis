//! File synchronization engine for maintaining consistency between filesystem and database

use crate::{DocumentStore, MetisError, Result};
use gray_matter;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Result of a sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    pub files_processed: usize,
    pub files_updated: usize,
    pub files_deleted: usize,
    pub errors: Vec<SyncError>,
}

/// Error during sync operation
#[derive(Debug, Clone)]
pub struct SyncError {
    pub file_path: PathBuf,
    pub error: String,
}

/// File sync engine for vault operations
#[derive(Clone)]
pub struct SyncEngine {
    store: DocumentStore,
}

impl SyncEngine {
    /// Create a new sync engine with the given document store
    pub fn new(store: DocumentStore) -> Self {
        Self { store }
    }

    /// Sync all markdown files from the filesystem into the database
    pub async fn sync_from_filesystem(&self, vault_path: &Path) -> Result<SyncResult> {
        let mut result = SyncResult {
            files_processed: 0,
            files_updated: 0,
            files_deleted: 0,
            errors: Vec::new(),
        };

        // Find all markdown files in the vault
        let markdown_files = self.find_markdown_files(vault_path)?;

        // Process each file
        for file_path in &markdown_files {
            result.files_processed += 1;

            match self.process_file(file_path).await {
                Ok(was_updated) => {
                    if was_updated {
                        result.files_updated += 1;
                    }
                }
                Err(e) => {
                    result.errors.push(SyncError {
                        file_path: file_path.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        // Handle orphan cleanup (files deleted from filesystem but still in database)
        let orphaned = self.find_orphaned_documents(&markdown_files).await?;
        for orphan_id in orphaned {
            // Document deletion event
            println!("SYNC EVENT: Document deleted - {}", orphan_id);

            match self.store.delete_document(&orphan_id).await {
                Ok(true) => result.files_deleted += 1,
                Ok(false) => {} // Document didn't exist
                Err(e) => result.errors.push(SyncError {
                    file_path: PathBuf::from(format!("orphan:{}", orphan_id)),
                    error: e.to_string(),
                }),
            }
        }

        Ok(result)
    }

    /// Process a single markdown file, updating database if needed
    async fn process_file(&self, file_path: &Path) -> Result<bool> {
        // Get current file metadata
        let metadata = fs::metadata(file_path).map_err(MetisError::Io)?;

        let current_size = metadata.len();
        let current_mtime = metadata.modified().map_err(MetisError::Io)?;
        let current_mtime_secs = current_mtime
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Check if document exists in database by filepath
        let existing_doc = self.get_existing_document_by_path(file_path).await?;

        let file_changed = match &existing_doc {
            None => {
                // New file - always process
                true
            }
            Some(doc) => {
                // Compare with stored metadata
                let stored_size = doc.file_size.unwrap_or(0) as u64;
                let stored_mtime = doc.file_modified_at.unwrap_or(0.0);

                current_size != stored_size || current_mtime_secs != stored_mtime
            }
        };

        if !file_changed {
            return Ok(false); // No changes detected
        }

        // File has changed or is new - process it
        let is_new_document = existing_doc.is_none();

        // Store/update the document (this will update file metadata too)
        self.store.store_document(file_path).await?;

        // Handle the three basic events
        if is_new_document {
            // Document creation event
            println!("SYNC EVENT: Document created - {}", file_path.display());
        } else {
            // Document modification event
            println!("SYNC EVENT: Document changed - {}", file_path.display());
        }

        Ok(true)
    }

    /// Find all markdown files in the vault directory
    fn find_markdown_files(&self, vault_path: &Path) -> Result<Vec<PathBuf>> {
        let mut markdown_files = Vec::new();
        self.find_markdown_files_recursive(vault_path, &mut markdown_files)?;
        Ok(markdown_files)
    }

    /// Recursively find markdown files
    #[allow(clippy::only_used_in_recursion)]
    fn find_markdown_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        let entries = fs::read_dir(dir).map_err(MetisError::Io)?;

        for entry in entries {
            let entry = entry.map_err(MetisError::Io)?;
            let path = entry.path();

            if path.is_dir() {
                // Skip hidden directories
                if let Some(dir_name) = path.file_name() {
                    if dir_name.to_string_lossy().starts_with('.') {
                        continue;
                    }
                }
                self.find_markdown_files_recursive(&path, files)?;
            } else if path.extension().is_some_and(|ext| ext == "md") {
                files.push(path);
            }
        }

        Ok(())
    }

    /// Get existing document from database by filepath
    async fn get_existing_document_by_path(
        &self,
        file_path: &Path,
    ) -> Result<Option<crate::Document>> {
        // Convert path to string for database query
        let filepath_str = file_path.to_string_lossy().to_string();

        // Query database by filepath
        let record = sqlx::query!(
            "SELECT id, filepath, document_type, level, status, parent_id, 
                    created_at, updated_at, content_hash, frontmatter_json, 
                    exit_criteria_met, content, file_size, file_modified_at
             FROM documents WHERE filepath = ?",
            filepath_str
        )
        .fetch_optional(self.store.pool())
        .await?;

        if let Some(row) = record {
            let document_type: crate::DocumentType = row.document_type.parse().unwrap_or_default();
            let level: crate::DocumentType = row.level.parse().unwrap_or_default();
            let frontmatter: serde_json::Value =
                serde_json::from_str(&row.frontmatter_json).unwrap_or_default();
            let created_at =
                chrono::DateTime::from_timestamp(row.created_at as i64, 0).unwrap_or_default();
            let updated_at =
                chrono::DateTime::from_timestamp(row.updated_at as i64, 0).unwrap_or_default();

            Ok(Some(crate::Document {
                id: row.id.unwrap_or_default(),
                filepath: row.filepath,
                document_type,
                level,
                status: row.status,
                parent_id: row.parent_id,
                created_at,
                updated_at,
                content_hash: row.content_hash,
                frontmatter,
                exit_criteria_met: row.exit_criteria_met.unwrap_or(false),
                content: row.content,
                file_size: row.file_size,
                file_modified_at: row.file_modified_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Check if file has changed based on size, mtime, or content hash
    #[allow(dead_code)]
    async fn file_has_changed(
        &self,
        existing_doc: &crate::Document,
        file_size: u64,
        modified_time: SystemTime,
    ) -> Result<bool> {
        // First check file size - if different, definitely changed
        let existing_metadata = fs::metadata(&existing_doc.filepath).map_err(MetisError::Io)?;

        if file_size != existing_metadata.len() {
            return Ok(true);
        }

        // Then check modification time - if newer, likely changed
        let existing_modified = existing_metadata.modified().map_err(MetisError::Io)?;

        if modified_time > existing_modified {
            return Ok(true);
        }

        // If size and time are the same, assume no change
        Ok(false)
    }

    /// Find documents in database that no longer have corresponding files
    async fn find_orphaned_documents(&self, existing_files: &[PathBuf]) -> Result<Vec<String>> {
        // Get all documents from database
        let query_service = self.store.query_service();

        // For simplicity, we'll check all document types
        // In practice, we might want a more efficient "get all documents" query
        let mut all_docs = Vec::new();

        use crate::DocumentType;
        for doc_type in [
            DocumentType::Vision,
            DocumentType::Strategy,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ] {
            let docs = query_service.find_documents_by_type(doc_type).await?;
            all_docs.extend(docs);
        }

        let mut orphaned = Vec::new();

        for doc in all_docs {
            let doc_path = PathBuf::from(&doc.filepath);
            if !existing_files.contains(&doc_path) && !doc_path.exists() {
                orphaned.push(doc.id);
            }
        }

        Ok(orphaned)
    }

    /// Validate consistency between filesystem and database
    pub async fn validate_consistency(&self, _vault_path: &Path) -> Result<Vec<String>> {
        let issues = Vec::new();

        // TODO: Implement consistency validation
        // - Check all database documents have corresponding files
        // - Check all markdown files are in database
        // - Validate frontmatter schema compliance

        Ok(issues)
    }

    /// Calculate content hash for a file
    #[allow(dead_code)]
    fn calculate_content_hash(&self, content: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Extract document ID from frontmatter
    #[allow(dead_code)]
    fn extract_document_id(&self, content: &str) -> Option<String> {
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(content);
        if let Some(gray_matter::Pod::Hash(map)) = parsed.data {
            if let Some(gray_matter::Pod::String(id)) = map.get("id") {
                return Some(id.clone());
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    async fn create_test_sync_engine() -> (SyncEngine, TempDir) {
        use sqlx::SqlitePool;
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let store = DocumentStore::from_pool(pool).await.unwrap();
        let sync_engine = SyncEngine::new(store);
        let temp_dir = TempDir::new().unwrap();
        (sync_engine, temp_dir)
    }

    fn create_test_markdown_file(dir: &Path, filename: &str, content: &str) -> PathBuf {
        let file_path = dir.join(filename);
        fs::write(&file_path, content).unwrap();
        file_path
    }

    #[tokio::test]
    async fn test_find_markdown_files() {
        let (sync_engine, temp_dir) = create_test_sync_engine().await;

        // Create test markdown files
        create_test_markdown_file(temp_dir.path(), "doc1.md", "# Test 1");
        create_test_markdown_file(temp_dir.path(), "doc2.md", "# Test 2");
        create_test_markdown_file(temp_dir.path(), "readme.txt", "Not markdown"); // Should be ignored

        // Create subdirectory with more files
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();
        create_test_markdown_file(&subdir, "doc3.md", "# Test 3");

        let files = sync_engine.find_markdown_files(temp_dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().any(|p| p.file_name().unwrap() == "doc1.md"));
        assert!(files.iter().any(|p| p.file_name().unwrap() == "doc2.md"));
        assert!(files.iter().any(|p| p.file_name().unwrap() == "doc3.md"));
    }

    #[tokio::test]
    async fn test_sync_from_filesystem() {
        let (sync_engine, temp_dir) = create_test_sync_engine().await;

        // Create test documents with proper frontmatter
        let doc1_content = r#"---
id: test-doc-1
level: vision
status: draft
phase: shaping
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Document 1
"#;

        let doc2_content = r#"---
id: test-doc-2
level: strategy
status: active
phase: design
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Document 2
"#;

        create_test_markdown_file(temp_dir.path(), "doc1.md", doc1_content);
        create_test_markdown_file(temp_dir.path(), "doc2.md", doc2_content);

        let result = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();

        assert_eq!(result.files_processed, 2);
        assert_eq!(result.files_updated, 2);
        assert_eq!(result.files_deleted, 0);
        assert!(result.errors.is_empty());

        // Verify documents were stored
        let doc1 = sync_engine.store.get_document("test-doc-1").await.unwrap();
        assert!(doc1.is_some());
        assert_eq!(doc1.unwrap().id, "test-doc-1");

        let doc2 = sync_engine.store.get_document("test-doc-2").await.unwrap();
        assert!(doc2.is_some());
        assert_eq!(doc2.unwrap().id, "test-doc-2");
    }

    #[tokio::test]
    async fn test_orphan_cleanup() {
        let (sync_engine, temp_dir) = create_test_sync_engine().await;

        // Create initial document
        let doc_content = r#"---
id: test-orphan
level: task
status: todo
phase: todo
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Orphan Document
"#;

        let doc_path = create_test_markdown_file(temp_dir.path(), "orphan.md", doc_content);

        // First sync - document should be added
        let result1 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result1.files_processed, 1);
        assert_eq!(result1.files_updated, 1);
        assert_eq!(result1.files_deleted, 0);

        // Verify document exists
        let doc = sync_engine.store.get_document("test-orphan").await.unwrap();
        assert!(doc.is_some());

        // Delete the file from filesystem
        fs::remove_file(&doc_path).unwrap();

        // Second sync - document should be removed from database
        let result2 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result2.files_processed, 0); // No files to process
        assert_eq!(result2.files_updated, 0);
        assert_eq!(result2.files_deleted, 1); // One orphan removed

        // Verify document no longer exists
        let doc_after = sync_engine.store.get_document("test-orphan").await.unwrap();
        assert!(doc_after.is_none());
    }

    #[tokio::test]
    async fn test_incremental_sync() {
        let (sync_engine, temp_dir) = create_test_sync_engine().await;

        let doc_content = r#"---
id: test-incremental
level: vision
status: draft
phase: shaping
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Incremental Document
"#;

        create_test_markdown_file(temp_dir.path(), "incremental.md", doc_content);

        // First sync
        let result1 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result1.files_updated, 1);

        // Second sync with no changes - sync engine should detect no changes
        let result2 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result2.files_processed, 1);
        assert_eq!(result2.files_updated, 0); // No changes detected, file not updated
    }

    #[tokio::test]
    async fn test_sync_event_detection() {
        let (sync_engine, temp_dir) = create_test_sync_engine().await;

        let doc_content = r#"---
id: test-sync-events
level: task
status: todo
phase: todo
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Sync Events Document
"#;

        let doc_path = create_test_markdown_file(temp_dir.path(), "events.md", doc_content);

        // First sync - should detect creation event
        let result1 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result1.files_updated, 1);
        assert_eq!(result1.files_deleted, 0);

        // Modify the document content
        let updated_content = r#"---
id: test-sync-events
level: task
status: doing
phase: doing
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Sync Events Document

This content has been updated!
"#;

        fs::write(&doc_path, updated_content).unwrap();

        // Second sync - should detect modification event
        let result2 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result2.files_updated, 1);
        assert_eq!(result2.files_deleted, 0);

        // Delete the file
        fs::remove_file(&doc_path).unwrap();

        // Third sync - should detect deletion event
        let result3 = sync_engine
            .sync_from_filesystem(temp_dir.path())
            .await
            .unwrap();
        assert_eq!(result3.files_processed, 0); // No files to process
        assert_eq!(result3.files_updated, 0);
        assert_eq!(result3.files_deleted, 1); // One file deleted
    }
}
