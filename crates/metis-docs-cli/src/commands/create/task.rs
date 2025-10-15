use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::documents::types::DocumentId,
    Database, Document, Initiative, Phase, Tag,
};
use std::path::Path;

/// Create a new Task document with defaults and write to file
pub async fn create_new_task(title: &str, initiative_id: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Verify the initiative exists and get its document ID and file path
    let (initiative_doc_id, _initiative_path) = find_initiative(&metis_dir, initiative_id).await?;

    // 3. Use DocumentCreationService to create the task
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: Some(initiative_doc_id.clone()),
        tags: vec![Tag::Label("task".to_string()), Tag::Phase(Phase::Todo)],
        phase: Some(Phase::Todo),
        complexity: None,
        risk_level: None,
    };

    // 4. Find the strategy short code for the initiative
    let strategy_short_code = find_strategy_for_initiative(&metis_dir, initiative_id).await?;

    // 5. Create the task using the DocumentCreationService with short codes
    let result = creation_service
        .create_task(config, &strategy_short_code, initiative_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create task: {}", e))?;

    println!("✓ Created task: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Title: {}", title);
    println!("  Parent Initiative: {}", initiative_doc_id);

    Ok(())
}

/// Find an initiative by short code and return its DocumentId and file path
async fn find_initiative(
    workspace_dir: &Path,
    initiative_id: &str,
) -> Result<(DocumentId, std::path::PathBuf)> {
    let db_path = workspace_dir.join("metis.db");
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run 'metis sync' first.");
    }

    let db = Database::new(&db_path.to_string_lossy())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    // Find initiative by short code in database
    if let Ok(Some(initiative_doc)) = repo.find_by_short_code(initiative_id) {
        // Found in database, now find its strategy to build the correct path
        if let Some(strategy_id) = &initiative_doc.strategy_id {
            if let Ok(Some(strategy_doc)) = repo.find_by_short_code(strategy_id) {
                // Build path using short codes
                let strategies_dir = workspace_dir.join("strategies");
                let initiative_path = strategies_dir
                    .join(&strategy_doc.short_code)
                    .join("initiatives")
                    .join(&initiative_doc.short_code)
                    .join("initiative.md");

                if initiative_path.exists() {
                    let initiative = Initiative::from_file(&initiative_path).await?;
                    return Ok((initiative.id().clone(), initiative_path));
                }
            }
        }
    }

    anyhow::bail!("Initiative '{}' not found", initiative_id);
}

/// Find the strategy ID that contains the given initiative
async fn find_strategy_for_initiative(workspace_dir: &Path, initiative_id: &str) -> Result<String> {
    let db_path = workspace_dir.join("metis.db");
    if !db_path.exists() {
        anyhow::bail!("Database not found. Run 'metis sync' first.");
    }

    let db = Database::new(&db_path.to_string_lossy())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    // Find initiative by short code and get its strategy short code
    if let Ok(Some(initiative_doc)) = repo.find_by_short_code(initiative_id) {
        if let Some(strategy_id) = &initiative_doc.strategy_id {
            return Ok(strategy_id.clone());
        }
    }

    anyhow::bail!("Could not find strategy for initiative '{}'", initiative_id);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_task_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_task("Test Task", "some-initiative").await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Not in a Metis workspace"));

            // Restore original directory
            if let Some(original) = original_dir {
                let _ = std::env::set_current_dir(&original);
            }
        }
    }

    #[tokio::test]
    async fn test_find_initiative_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Ensure we can change to temp directory
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            if let Some(original) = original_dir {
                let _ = std::env::set_current_dir(&original);
            }
            return;
        }

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        if init_cmd.execute().await.is_err() {
            if let Some(original) = original_dir {
                let _ = std::env::set_current_dir(&original);
            }
            return;
        }

        // Try to find non-existent initiative
        let metis_dir = temp_dir.path().join(".metis");

        // Create strategies directory but no actual strategies
        fs::create_dir_all(metis_dir.join("strategies")).unwrap();

        let result = find_initiative(&metis_dir, "non-existent").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Initiative 'non-existent' not found"));

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
