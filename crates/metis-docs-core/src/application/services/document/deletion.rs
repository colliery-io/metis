use crate::Result;
use std::path::Path;

/// Service for recursive document deletion
///
/// Handles the complete deletion of a document and all its children:
/// 1. Identifies document type from path
/// 2. For strategies/initiatives: rm -r the folder
/// 3. For tasks: delete the file
/// 4. Caller can sync to update database
pub struct DeletionService {}

impl Default for DeletionService {
    fn default() -> Self {
        Self::new()
    }
}

impl DeletionService {
    pub fn new() -> Self {
        Self {}
    }

    /// Delete a document and all its children recursively
    pub async fn delete_document_recursive(&self, filepath: &str) -> Result<DeletionResult> {
        let file_path = Path::new(filepath);

        if !file_path.exists() {
            return Ok(DeletionResult {
                deleted_files: vec![],
                cleaned_directories: vec![],
            });
        }

        let mut deleted_files = Vec::new();
        let mut cleaned_directories = Vec::new();

        // For documents structured as "parent-dir/document.md",
        // we need to delete the entire parent directory
        if let Some(parent_dir) = file_path.parent() {
            // Check if parent is not the workspace root and is a directory
            if parent_dir != Path::new(".") && parent_dir != Path::new("") && parent_dir.is_dir() {
                // For strategy/initiative documents, delete the entire parent directory
                // This handles cases like "strategy-id/strategy.md" -> delete "strategy-id/"
                if file_path.file_name() == Some(std::ffi::OsStr::new("strategy.md"))
                    || file_path.file_name() == Some(std::ffi::OsStr::new("initiative.md"))
                {
                    Self::remove_directory_recursive(
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
        }

        // For other files (like tasks or documents at root), just delete the file
        if file_path.is_file() {
            if let Err(e) = std::fs::remove_file(file_path) {
                eprintln!(
                    "Warning: Could not delete file {}: {}",
                    file_path.display(),
                    e
                );
            } else {
                deleted_files.push(file_path.display().to_string());
            }
        }

        Ok(DeletionResult {
            deleted_files,
            cleaned_directories,
        })
    }

    /// Recursively remove a directory and all its contents
    fn remove_directory_recursive(
        dir_path: &Path,
        deleted_files: &mut Vec<String>,
        cleaned_directories: &mut Vec<String>,
    ) -> Result<()> {
        if !dir_path.exists() || !dir_path.is_dir() {
            return Ok(());
        }

        // First, collect all files in this directory and subdirectories
        let entries = std::fs::read_dir(dir_path).map_err(|e| {
            crate::MetisError::FileSystem(format!(
                "Failed to read directory {}: {}",
                dir_path.display(),
                e
            ))
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| crate::MetisError::FileSystem(e.to_string()))?;
            let path = entry.path();

            if path.is_file() {
                // Delete the file
                if let Err(e) = std::fs::remove_file(&path) {
                    eprintln!("Warning: Could not delete file {}: {}", path.display(), e);
                } else {
                    deleted_files.push(path.display().to_string());
                }
            } else if path.is_dir() {
                // Recursively remove subdirectory
                Self::remove_directory_recursive(&path, deleted_files, cleaned_directories)?;
            }
        }

        // Now remove the empty directory
        if let Err(e) = std::fs::remove_dir(dir_path) {
            eprintln!(
                "Warning: Could not remove directory {}: {}",
                dir_path.display(),
                e
            );
        } else {
            cleaned_directories.push(dir_path.display().to_string());
        }

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
            risk_level: None,
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
    async fn test_delete_strategy_with_folder() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create strategy using creation service
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let strategy_config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: Some("Test strategy description".to_string()),
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

        // Sync the strategy to database so it can be found by the initiative creation
        let db_path = workspace_dir.join("metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&strategy_result.file_path)
            .await
            .unwrap();

        // Create initiative under strategy
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("Test initiative description".to_string()),
            parent_id: Some(strategy_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let initiative_result = creation_service
            .create_initiative(initiative_config, &strategy_result.short_code)
            .await
            .unwrap();

        // Sync the initiative to database so it can be found by the task creation
        sync_service
            .import_from_file(&initiative_result.file_path)
            .await
            .unwrap();

        // Create task under initiative
        let task_config = DocumentCreationConfig {
            title: "Test Task".to_string(),
            description: Some("Test task description".to_string()),
            parent_id: Some(initiative_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let task_result = creation_service
            .create_task(
                task_config,
                &strategy_result.short_code,
                &initiative_result.short_code,
            )
            .await
            .unwrap();

        // Verify files exist before deletion
        assert!(strategy_result.file_path.exists());
        assert!(initiative_result.file_path.exists());
        assert!(task_result.file_path.exists());

        // Delete the strategy
        let deletion_service = DeletionService::new();
        let result = deletion_service
            .delete_document_recursive(&strategy_result.file_path.to_string_lossy())
            .await
            .unwrap();

        // Verify entire strategy folder was deleted
        let strategy_path = &strategy_result.file_path;
        let strategy_folder = strategy_path.parent().unwrap();
        assert!(!strategy_folder.exists());
        assert!(!initiative_result.file_path.exists());
        assert!(!task_result.file_path.exists());

        // Should have deleted all files and directories
        assert!(result.deleted_files.len() >= 3); // at least strategy.md + initiative.md + task.md
        assert!(!result.cleaned_directories.is_empty()); // at least the strategy folder
    }

    #[tokio::test]
    async fn test_delete_initiative_with_folder() {
        let (_temp_dir, workspace_dir) = setup_test_workspace().await;

        // Create strategy first (required parent)
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let strategy_config = DocumentCreationConfig {
            title: "Parent Strategy".to_string(),
            description: Some("Parent strategy".to_string()),
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

        // Sync the strategy to database so it can be found by the initiative creation
        let db_path = workspace_dir.join("metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&strategy_result.file_path)
            .await
            .unwrap();

        // Create initiative
        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("Test initiative".to_string()),
            parent_id: Some(strategy_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let initiative_result = creation_service
            .create_initiative(initiative_config, &strategy_result.short_code)
            .await
            .unwrap();

        // Sync the initiative to database so it can be found by the task creation
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
            risk_level: None,
        };
        let task1_result = creation_service
            .create_task(
                task1_config,
                &strategy_result.short_code,
                &initiative_result.short_code,
            )
            .await
            .unwrap();

        let task2_config = DocumentCreationConfig {
            title: "Task Two".to_string(),
            description: Some("Second task".to_string()),
            parent_id: Some(initiative_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let task2_result = creation_service
            .create_task(
                task2_config,
                &strategy_result.short_code,
                &initiative_result.short_code,
            )
            .await
            .unwrap();

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

        // Verify strategy still exists
        assert!(strategy_result.file_path.exists());

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

        // Create full hierarchy up to task
        let creation_service = DocumentCreationService::new(&workspace_dir);
        let strategy_config = DocumentCreationConfig {
            title: "Test Strategy".to_string(),
            description: Some("Test strategy".to_string()),
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

        // Sync the strategy to database so it can be found by the initiative creation
        let db_path = workspace_dir.join("metis.db");
        let db = crate::Database::new(&db_path.to_string_lossy()).unwrap();
        let mut db_service =
            crate::application::services::DatabaseService::new(db.repository().unwrap());
        let mut sync_service = crate::application::services::SyncService::new(&mut db_service);
        sync_service
            .import_from_file(&strategy_result.file_path)
            .await
            .unwrap();

        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: Some("Test initiative".to_string()),
            parent_id: Some(strategy_result.document_id.clone()),
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        let initiative_result = creation_service
            .create_initiative(initiative_config, &strategy_result.short_code)
            .await
            .unwrap();

        // Sync the initiative to database so it can be found by the task creation
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
            risk_level: None,
        };
        let task_result = creation_service
            .create_task(
                task_config,
                &strategy_result.short_code,
                &initiative_result.short_code,
            )
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

        // Parent documents should still exist
        assert!(initiative_result.file_path.exists());
        assert!(strategy_result.file_path.exists());

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
