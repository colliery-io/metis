use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::application::services::workspace::ArchiveService;

#[derive(Args)]
pub struct ArchiveCommand {
    /// Document ID to archive
    pub document_id: String,

    /// Document type (vision, strategy, initiative, task, adr) - auto-detected if not provided
    #[arg(short = 't', long)]
    pub document_type: Option<String>,
}

impl ArchiveCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Create the archive service
        let archive_service = ArchiveService::new(&metis_dir);

        // 3. Archive the document and its children
        let archive_result = archive_service.archive_document(&self.document_id).await?;

        // 4. Report results
        println!("✓ Archived {} documents:", archive_result.total_archived);
        for doc in archive_result.archived_documents {
            println!("  - {} ({})", doc.document_id, doc.document_type);
        }

        // 5. Auto-sync to update database
        println!("Archive completed. Run 'metis sync' to update the database.");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_archive_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return; // Skip test if we can't change directory
        }

        let cmd = ArchiveCommand {
            document_id: "test-doc".to_string(),
            document_type: None,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));
    }

    #[tokio::test]
    async fn test_archive_document_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
        };
        init_cmd.execute().await.unwrap();

        let cmd = ArchiveCommand {
            document_id: "non-existent-doc".to_string(),
            document_type: None,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_archive_vision_document() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
        };
        init_cmd.execute().await.unwrap();

        let metis_dir = temp_dir.path().join(".metis");
        let vision_path = metis_dir.join("vision.md");
        let archived_vision_path = metis_dir.join("archived").join("vision.md");

        // Verify vision exists
        assert!(vision_path.exists());
        assert!(!archived_vision_path.exists());

        let cmd = ArchiveCommand {
            document_id: "test-project".to_string(),
            document_type: Some("vision".to_string()),
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_ok());

        // Verify file was moved and marked as archived
        assert!(!vision_path.exists());
        assert!(archived_vision_path.exists());

        // Verify archived flag was set
        let archived_content = std::fs::read_to_string(&archived_vision_path).unwrap();
        assert!(archived_content.contains("archived: true"));
    }
}
