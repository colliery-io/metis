use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::documents::types::DocumentId,
    Document, Initiative, Phase, Tag,
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

    // 4. Find the strategy ID for the initiative
    let strategy_id = find_strategy_for_initiative(&metis_dir, initiative_id).await?;
    
    // 5. Create the task using the DocumentCreationService
    let result = creation_service
        .create_task(config, &strategy_id, initiative_id)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create task: {}", e))?;

    println!("✓ Created task: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Title: {}", title);
    println!("  Parent Initiative: {}", initiative_doc_id);

    Ok(())
}

/// Find an initiative by ID and return its DocumentId and file path
async fn find_initiative(
    workspace_dir: &Path,
    initiative_id: &str,
) -> Result<(DocumentId, std::path::PathBuf)> {
    let strategies_dir = workspace_dir.join("strategies");

    if !strategies_dir.exists() {
        anyhow::bail!("No strategies directory found. Create a strategy and initiative first.");
    }

    // Search through all strategy directories for the initiative
    for strategy_entry in std::fs::read_dir(&strategies_dir)? {
        let strategy_dir = strategy_entry?.path();
        if !strategy_dir.is_dir() {
            continue;
        }

        let initiatives_dir = strategy_dir.join("initiatives");
        if !initiatives_dir.exists() {
            continue;
        }

        // Look for the initiative directory
        let initiative_dir = initiatives_dir.join(initiative_id);
        if !initiative_dir.exists() || !initiative_dir.is_dir() {
            continue;
        }

        // Parse the initiative document to get its actual ID
        let initiative_path = initiative_dir.join("initiative.md");
        if !initiative_path.exists() {
            continue;
        }

        let initiative = Initiative::from_file(&initiative_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse initiative document: {}", e))?;

        let initiative_doc_id = initiative.id();

        // Verify the directory name matches the initiative ID
        if initiative_doc_id.to_string() == initiative_id {
            return Ok((initiative_doc_id, initiative_path));
        }
    }

    // If we get here, initiative wasn't found
    let available = list_available_initiatives(workspace_dir)?;
    if available.is_empty() {
        anyhow::bail!("No initiatives found. Create an initiative first.");
    } else {
        anyhow::bail!(
            "Initiative '{}' not found. Available initiatives: {}",
            initiative_id,
            available.join(", ")
        );
    }
}

/// List all available initiative IDs across all strategies
fn list_available_initiatives(workspace_dir: &Path) -> Result<Vec<String>> {
    let mut initiatives = Vec::new();
    let strategies_dir = workspace_dir.join("strategies");

    if !strategies_dir.exists() {
        return Ok(initiatives);
    }

    for strategy_entry in std::fs::read_dir(&strategies_dir)? {
        let strategy_dir = strategy_entry?.path();
        if !strategy_dir.is_dir() {
            continue;
        }

        let initiatives_dir = strategy_dir.join("initiatives");
        if !initiatives_dir.exists() {
            continue;
        }

        for initiative_entry in std::fs::read_dir(&initiatives_dir)? {
            let initiative_dir = initiative_entry?.path();
            if initiative_dir.is_dir() {
                if let Some(name) = initiative_dir.file_name().and_then(|n| n.to_str()) {
                    initiatives.push(name.to_string());
                }
            }
        }
    }

    initiatives.sort();
    Ok(initiatives)
}

/// Find the strategy ID that contains the given initiative
async fn find_strategy_for_initiative(workspace_dir: &Path, initiative_id: &str) -> Result<String> {
    let strategies_dir = workspace_dir.join("strategies");

    if !strategies_dir.exists() {
        anyhow::bail!("No strategies directory found");
    }

    for strategy_entry in std::fs::read_dir(&strategies_dir)? {
        let strategy_dir = strategy_entry?.path();
        if !strategy_dir.is_dir() {
            continue;
        }

        let initiatives_dir = strategy_dir.join("initiatives");
        if !initiatives_dir.exists() {
            continue;
        }

        // Check if this strategy contains the initiative
        let initiative_dir = initiatives_dir.join(initiative_id);
        if initiative_dir.exists() && initiative_dir.is_dir() {
            // Return the strategy directory name as the strategy ID
            if let Some(strategy_name) = strategy_dir.file_name().and_then(|n| n.to_str()) {
                return Ok(strategy_name.to_string());
            }
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
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_task("Test Task", "some-initiative").await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Not in a Metis workspace"));

            // Restore original directory
            let _ = std::env::set_current_dir(original_dir);
        }
    }

    #[tokio::test]
    async fn test_find_initiative_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Ensure we can change to temp directory
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            std::env::set_current_dir(original_dir).unwrap();
            return;
        }

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
        };
        if init_cmd.execute().await.is_err() {
            std::env::set_current_dir(original_dir).unwrap();
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
            .contains("No initiatives found"));

        // Always restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_list_available_initiatives() {
        let temp_dir = tempdir().unwrap();
        let strategies_dir = temp_dir.path().join("strategies");

        // Create some strategy and initiative directories
        fs::create_dir_all(&strategies_dir).unwrap();

        // Strategy 1 with initiatives
        let strategy1_dir = strategies_dir.join("strategy-1");
        let initiatives1_dir = strategy1_dir.join("initiatives");
        fs::create_dir_all(&initiatives1_dir).unwrap();
        fs::create_dir(initiatives1_dir.join("initiative-1")).unwrap();
        fs::create_dir(initiatives1_dir.join("initiative-2")).unwrap();

        // Strategy 2 with initiatives
        let strategy2_dir = strategies_dir.join("strategy-2");
        let initiatives2_dir = strategy2_dir.join("initiatives");
        fs::create_dir_all(&initiatives2_dir).unwrap();
        fs::create_dir(initiatives2_dir.join("another-initiative")).unwrap();

        let initiatives = list_available_initiatives(temp_dir.path()).unwrap();
        assert_eq!(initiatives.len(), 3);
        assert!(initiatives.contains(&"initiative-1".to_string()));
        assert!(initiatives.contains(&"initiative-2".to_string()));
        assert!(initiatives.contains(&"another-initiative".to_string()));
    }
}
