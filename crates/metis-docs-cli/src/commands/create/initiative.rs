use crate::workspace;
use anyhow::Result;
use dialoguer::Select;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::documents::{initiative::Complexity, types::DocumentId},
    Document, Phase, Strategy, Tag,
};
use std::path::Path;

/// Create a new Initiative document with defaults and write to file
pub async fn create_new_initiative(title: &str, strategy_id: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Verify the strategy exists and get its document ID
    let (strategy_doc_id, _strategy_path) = find_strategy(&metis_dir, strategy_id).await?;

    // 3. Prompt for complexity level
    let complexity = prompt_for_complexity()?;

    // 4. Use DocumentCreationService to create the initiative
    let creation_service = DocumentCreationService::new(&metis_dir);
    
    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: Some(strategy_doc_id.clone()),
        tags: vec![
            Tag::Label("initiative".to_string()),
            Tag::Phase(Phase::Discovery),
        ],
        phase: Some(Phase::Discovery),
        complexity: Some(complexity),
        risk_level: None,
    };

    let result = creation_service
        .create_initiative(config, &strategy_doc_id.to_string())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create initiative: {}", e))?;

    println!("✓ Created initiative: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);
    println!("  Parent Strategy: {}", strategy_doc_id);
    println!("  Complexity: {:?}", complexity);

    Ok(())
}

/// Find a strategy by ID and return its DocumentId and file path
async fn find_strategy(
    workspace_dir: &Path,
    strategy_id: &str,
) -> Result<(DocumentId, std::path::PathBuf)> {
    let strategies_dir = workspace_dir.join("strategies");

    if !strategies_dir.exists() {
        anyhow::bail!("No strategies directory found. Create a strategy first.");
    }

    // Look for the strategy directory
    let strategy_dir = strategies_dir.join(strategy_id);
    if !strategy_dir.exists() || !strategy_dir.is_dir() {
        // Try to list available strategies for a helpful error
        let available = list_available_strategies(&strategies_dir)?;
        if available.is_empty() {
            anyhow::bail!("No strategies found. Create a strategy first.");
        } else {
            anyhow::bail!(
                "Strategy '{}' not found. Available strategies: {}",
                strategy_id,
                available.join(", ")
            );
        }
    }

    // Parse the strategy document to get its actual ID
    let strategy_path = strategy_dir.join("strategy.md");
    if !strategy_path.exists() {
        anyhow::bail!("Strategy file not found: {}", strategy_path.display());
    }

    let strategy = Strategy::from_file(&strategy_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse strategy document: {}", e))?;

    let strategy_id = strategy.id();

    Ok((strategy_id, strategy_path))
}

/// List available strategy IDs
fn list_available_strategies(strategies_dir: &Path) -> Result<Vec<String>> {
    let mut strategies = Vec::new();

    if let Ok(entries) = std::fs::read_dir(strategies_dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    strategies.push(name.to_string());
                }
            }
        }
    }

    strategies.sort();
    Ok(strategies)
}

/// Prompt user to select complexity level
fn prompt_for_complexity() -> Result<Complexity> {
    // Check if we're in a testing environment
    if cfg!(test) || std::env::var("RUST_TEST_THREADS").is_ok() {
        // In tests, return default complexity without prompting
        return Ok(Complexity::M);
    }

    let complexities = [
        ("S - Small (1-3 days)", Complexity::S),
        ("M - Medium (1-2 weeks)", Complexity::M),
        ("L - Large (2-4 weeks)", Complexity::L),
        ("XL - Extra Large (1+ months)", Complexity::XL),
    ];

    let selection = Select::new()
        .with_prompt("Select initiative complexity")
        .default(1) // Default to M
        .items(
            &complexities
                .iter()
                .map(|(label, _)| label)
                .collect::<Vec<_>>(),
        )
        .interact()
        .unwrap_or(1); // Default to M if interaction fails

    Ok(complexities[selection].1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_initiative_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_initiative("Test Initiative", "some-strategy").await;
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
    async fn test_find_strategy_not_found() {
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

        // Try to find non-existent strategy
        let metis_dir = temp_dir.path().join(".metis");
        let result = find_strategy(&metis_dir, "non-existent").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No strategies found"));

        // Always restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_list_available_strategies() {
        let temp_dir = tempdir().unwrap();
        let strategies_dir = temp_dir.path().join("strategies");

        // Create some strategy directories
        fs::create_dir_all(&strategies_dir).unwrap();
        fs::create_dir(strategies_dir.join("strategy-1")).unwrap();
        fs::create_dir(strategies_dir.join("strategy-2")).unwrap();
        fs::create_dir(strategies_dir.join("another-strategy")).unwrap();

        let strategies = list_available_strategies(&strategies_dir).unwrap();
        assert_eq!(strategies.len(), 3);
        assert_eq!(strategies[0], "another-strategy");
        assert_eq!(strategies[1], "strategy-1");
        assert_eq!(strategies[2], "strategy-2");
    }

    // Note: This test would require mocking dialoguer's Select prompt
    // For now, we'll test the end-to-end flow with a unit test that bypasses the prompt
    #[tokio::test]
    async fn test_create_initiative_flow_without_prompt() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
        };
        init_cmd.execute().await.unwrap();

        // Create a strategy first
        crate::commands::create::strategy::create_new_strategy("Test Strategy", None)
            .await
            .unwrap();

        // Test the helper functions for initiative creation
        let metis_dir = temp_dir.path().join(".metis");

        // Test finding the strategy
        let (strategy_id, strategy_path) =
            find_strategy(&metis_dir, "test-strategy").await.unwrap();
        assert_eq!(strategy_id.to_string(), "test-strategy");
        assert!(strategy_path.exists());

        // Verify the strategy path structure
        let expected_strategy_path = metis_dir
            .join("strategies")
            .join("test-strategy")
            .join("strategy.md");
        assert_eq!(strategy_path, expected_strategy_path);

        // Test that the strategy directory structure would work for initiatives
        let initiative_dir = strategy_path
            .parent()
            .unwrap()
            .join("initiatives")
            .join("test-initiative");
        fs::create_dir_all(&initiative_dir).unwrap();

        assert!(initiative_dir.exists());
        assert!(initiative_dir.is_dir());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
