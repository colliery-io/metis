use crate::application::services::FilesystemService;
use crate::Result;
use std::path::Path;

/// Service for recursive document deletion
///
/// Handles the complete deletion of a document and all its children:
/// 1. Identifies document type from path
/// 2. For initiatives: rm -r the folder
/// 3. For tasks: delete the file
/// 4. Caller can sync to update database
pub struct DeletionService {
    fs: FilesystemService,
}

impl DeletionService {
    pub fn new() -> Self {
        Self {
            fs: FilesystemService::local(),
        }
    }

    pub fn new_for_workspace<P: AsRef<Path>>(workspace_dir: P) -> Self {
        Self {
            fs: FilesystemService::new(workspace_dir),
        }
    }

    /// Delete a document and all its children recursively
    pub async fn delete_document_recursive(&self, filepath: &str) -> Result<DeletionResult> {
        let file_path = Path::new(filepath);

        if !self.fs.file_exists(file_path) {
            return Ok(DeletionResult {
                deleted_files: vec![],
                cleaned_directories: vec![],
            });
        }

        let mut deleted_files = Vec::new();
        let mut cleaned_directories = Vec::new();

        // For initiative documents, delete via overlay tombstones for all files
        if file_path.file_name() == Some(std::ffi::OsStr::new("initiative.md")) {
            if let Some(parent_dir) = file_path.parent() {
                self.remove_directory_recursive(
                    parent_dir,
                    &mut deleted_files,
                    &mut cleaned_directories,
                )?;
                return Ok(DeletionResult {
                    deleted_files,
                    cleaned_directories,
                });
            }
        }

        // For other files (like tasks or documents at root), just delete the file
        if let Err(e) = self.fs.delete_file(file_path) {
            eprintln!(
                "Warning: Could not delete file {}: {}",
                file_path.display(),
                e
            );
        } else {
            deleted_files.push(file_path.display().to_string());
        }

        Ok(DeletionResult {
            deleted_files,
            cleaned_directories,
        })
    }

    /// Recursively remove a directory and all its contents
    fn remove_directory_recursive(
        &self,
        dir_path: &Path,
        deleted_files: &mut Vec<String>,
        cleaned_directories: &mut Vec<String>,
    ) -> Result<()> {
        // Find all markdown files in this directory via overlay-aware search
        let files = match self.fs.find_markdown_files(dir_path) {
            Ok(f) => f,
            Err(_) => return Ok(()),
        };

        for file_path in &files {
            let path = Path::new(file_path);
            if let Err(e) = self.fs.delete_file(path) {
                eprintln!("Warning: Could not delete file {}: {}", file_path, e);
            } else {
                deleted_files.push(file_path.clone());
            }
        }

        // On Local backend, also remove the physical directory
        if !self.fs.is_git_overlay() && dir_path.exists() {
            if let Err(e) = std::fs::remove_dir_all(dir_path) {
                eprintln!(
                    "Warning: Could not remove directory {}: {}",
                    dir_path.display(),
                    e
                );
            }
        }

        cleaned_directories.push(dir_path.display().to_string());
        Ok(())
    }
}

/// Result of a document deletion operation
#[derive(Debug)]
pub struct DeletionResult {
    pub deleted_files: Vec<String>,
    pub cleaned_directories: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::services::document::{
        creation::DocumentCreationConfig, DocumentCreationService,
    };
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    use crate::application::Application;
    use crate::dal::Database;
    use diesel::Connection;

    async fn setup_test_workspace() -> (tempfile::TempDir, PathBuf) {
        let temp_dir = tempdir().unwrap();
        let workspace_dir = temp_dir.path().to_path_buf();

        // Create .metis directory structure
        let metis_dir = workspace_dir.join(".metis");
        fs::create_dir_all(&metis_dir).unwrap();

        // Initialize database with configuration
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(&db_path.to_string_lossy()).unwrap();
        let mut config_repo =
            crate::dal::database::configuration_repository::ConfigurationRepository::new(
                diesel::sqlite::SqliteConnection::establish(&db_path.to_string_lossy()).unwrap(),
            );
        config_repo.set_project_prefix("TEST").unwrap();
        let app = Application::new(db);

        // Create vision (required as root)
        let creation_service = DocumentCreationService::new(&metis_dir);
        let vision_config = DocumentCreationConfig {
            title: "Test Vision".to_string(),
            description: Some("Root vision for testing".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
        };
        creation_service.create_vision(vision_config).await.unwrap();

        // Sync to database
        app.sync_directory(&metis_dir).await.unwrap();

        (temp_dir, metis_dir)
    }

    #[tokio::test]
    async fn test_delete_single_document_no_children() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;
        let service = DeletionService::new();

        // Create a test document (task - just a file)
        let doc_path = workspace_dir.join("test.md");
        fs::write(&doc_path, "# Test Document\nContent here").unwrap();

        // Delete the document
        let result = service
            .delete_document_recursive(&doc_path.display().to_string())
            .await
            .unwrap();

        // Verify results
        assert_eq!(result.deleted_files.len(), 1);
        assert!(!doc_path.exists());
    }

    #[tokio::test]
    async fn test_delete_initiative_with_folder() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create initiative (parent is vision)
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("Test initiative description".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
        };
        let initiative_result = creation_service
            .create_initiative(initiative_config)
            .await
            .unwrap();

        // Sync the initiative to database so it can be found by the task creation
        let db_path = workspace_dir.join("metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&initiative_result.file_path)
            .await
            .unwrap();

        // Create tasks under initiative
        let task1_config = DocumentCreationConfig {
            title: "Task One".to_string(),
            description: Some("First task".to_string()),
            parent_id: Some(initiative_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
        };
        let task1_result = creation_service
            .create_task(task1_config, &initiative_result.short_code)
            .await
            .unwrap();

        let task2_config = DocumentCreationConfig {
            title: "Task Two".to_string(),
            description: Some("Second task".to_string()),
            parent_id: Some(initiative_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
        };
        let task2_result = creation_service
            .create_task(task2_config, &initiative_result.short_code)
            .await
            .unwrap();

        // Verify files exist before deletion
        assert!(initiative_result.file_path.exists());
        assert!(task1_result.file_path.exists());
        assert!(task2_result.file_path.exists());

        // Delete the initiative
        let deletion_service = DeletionService::new();
        let result = deletion_service
            .delete_document_recursive(&initiative_result.file_path.to_string_lossy())
            .await
            .unwrap();

        // Verify initiative folder was deleted
        let initiative_path = &initiative_result.file_path;
        let initiative_folder = initiative_path.parent().unwrap();
        assert!(!initiative_folder.exists());
        assert!(!task1_result.file_path.exists());
        assert!(!task2_result.file_path.exists());

        // Should have deleted all files in the initiative folder
        assert!(result.deleted_files.len() >= 3); // at least initiative.md + task1.md + task2.md
        assert!(!result.cleaned_directories.is_empty()); // at least the initiative folder
    }

    #[tokio::test]
    async fn test_delete_nonexistent_document() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;
        let service = DeletionService::new();

        let nonexistent_path = workspace_dir.join("nonexistent.md");

        // Should handle gracefully
        let result = service
            .delete_document_recursive(&nonexistent_path.display().to_string())
            .await
            .unwrap();

        assert_eq!(result.deleted_files.len(), 0);
        assert_eq!(result.cleaned_directories.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_task_file_only() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create initiative first (required parent for tasks)
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("Test initiative".to_string()),
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
        };
        let initiative_result = creation_service
            .create_initiative(initiative_config)
            .await
            .unwrap();

        // Sync the initiative to database so it can be found by the task creation
        let db_path = workspace_dir.join("metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&initiative_result.file_path)
            .await
            .unwrap();

        let task_config = DocumentCreationConfig {
            title: "Test Task".to_string(),
            description: Some("Test task".to_string()),
            parent_id: Some(initiative_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
        };
        let task_result = creation_service
            .create_task(task_config, &initiative_result.short_code)
            .await
            .unwrap();

        // Delete just the task
        let deletion_service = DeletionService::new();
        let result = deletion_service
            .delete_document_recursive(&task_result.file_path.to_string_lossy())
            .await
            .unwrap();

        // Task should be deleted
        assert!(!task_result.file_path.exists());

        // Parent initiative should still exist
        assert!(initiative_result.file_path.exists());

        // Should only delete the task file
        assert_eq!(result.deleted_files.len(), 1);
        assert_eq!(result.cleaned_directories.len(), 0);
    }

    #[tokio::test]
    async fn test_delete_document_no_folder() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;
        let service = DeletionService::new();

        // Create a document without an associated folder
        let doc_path = workspace_dir.join("document.md");
        fs::write(&doc_path, "# Document").unwrap();

        // Delete the document
        let result = service
            .delete_document_recursive(&doc_path.display().to_string())
            .await
            .unwrap();

        // Should only delete the file
        assert!(!doc_path.exists());
        assert_eq!(result.deleted_files.len(), 1);
        assert_eq!(result.cleaned_directories.len(), 0);
    }
}
