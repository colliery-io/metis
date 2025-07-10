use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{Database, Phase, Tag, Vision};
use std::path::Path;

#[derive(Args)]
pub struct InitCommand {
    /// Project name for the vision document
    #[arg(short, long)]
    pub name: Option<String>,
}

impl InitCommand {
    pub async fn execute(&self) -> Result<()> {
        // Check if workspace already exists
        let (workspace_exists, _) = workspace::has_metis_vault();
        if workspace_exists {
            println!("Metis workspace already exists in this directory");
            return Ok(());
        }

        // Get current directory for workspace creation
        let current_dir = std::env::current_dir()?;
        let metis_dir = current_dir.join(".metis");

        // Create .metis directory
        std::fs::create_dir_all(&metis_dir)?;

        // Initialize database
        let db_path = metis_dir.join("metis.db");
        let _db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Database initialization failed: {}", e))?;

        // Create strategies directory
        let strategies_dir = metis_dir.join("strategies");
        std::fs::create_dir_all(&strategies_dir)?;

        // Create vision.md with defaults
        let project_name = self.name.as_deref().unwrap_or("Project Vision");
        create_default_vision(&metis_dir, project_name).await?;

        println!("✓ Initialized Metis workspace in {}", current_dir.display());
        println!("✓ Created vision.md with project template");

        Ok(())
    }
}

/// Create a new Vision document with defaults and write to file
async fn create_default_vision(workspace_dir: &Path, title: &str) -> Result<()> {
    // Create Vision with defaults
    let tags = vec![Tag::Label("vision".to_string()), Tag::Phase(Phase::Draft)];

    let vision = Vision::new(
        title.to_string(),
        tags,
        false, // not archived
    )
    .map_err(|e| anyhow::anyhow!("Failed to create vision: {}", e))?;

    // Write to vision.md at workspace root
    let vision_path = workspace_dir.join("vision.md");
    vision.to_file(&vision_path).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_init_command_creates_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify .metis directory was created
        let metis_dir = temp_dir.path().join(".metis");
        assert!(metis_dir.exists());
        assert!(metis_dir.is_dir());

        // Verify database was created
        let db_path = metis_dir.join("metis.db");
        assert!(db_path.exists());
        assert!(db_path.is_file());

        // Verify strategies directory was created
        let strategies_dir = metis_dir.join("strategies");
        assert!(strategies_dir.exists());
        assert!(strategies_dir.is_dir());

        // Verify vision.md was created
        let vision_path = metis_dir.join("vision.md");
        assert!(vision_path.exists());
        assert!(vision_path.is_file());

        // Verify vision.md content
        let vision_content = fs::read_to_string(&vision_path).unwrap();
        assert!(vision_content.contains("Test Project"));
        assert!(vision_content.contains("#vision"));
        assert!(vision_content.contains("#phase/draft"));
        assert!(vision_content.contains("archived: false"));

        // Verify template was rendered
        assert!(vision_content.contains("# Test Project Vision"));
        assert!(vision_content.contains("## Purpose"));
        assert!(vision_content.contains("## Current State"));
        assert!(vision_content.contains("## Future State"));
        assert!(vision_content.contains("## Success Criteria"));
        assert!(vision_content.contains("## Principles"));
        assert!(vision_content.contains("## Constraints"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_workspace_already_exists() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();
        let metis_dir = temp_dir.path().join(".metis");
        let db_path = metis_dir.join("metis.db");

        // Pre-create workspace
        fs::create_dir_all(&metis_dir).unwrap();
        fs::write(&db_path, "existing").unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify existing database wasn't overwritten
        let db_content = fs::read_to_string(&db_path).unwrap();
        assert_eq!(db_content, "existing");

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_init_command_default_name() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command without name
        let cmd = InitCommand { name: None };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify vision.md was created with default name
        let vision_path = temp_dir.path().join(".metis").join("vision.md");
        let vision_content = fs::read_to_string(&vision_path).unwrap();
        assert!(vision_content.contains("Project Vision"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
