use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::documents::{strategy::RiskLevel, types::DocumentId},
    Document, Phase, Tag, Vision,
};
use std::path::Path;

/// Create a new Strategy document with defaults and write to file
pub async fn create_new_strategy(title: &str, vision_slug: Option<&str>) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Get parent vision ID by parsing the vision document (if provided)
    let parent_id = if let Some(vision_slug) = vision_slug {
        Some(get_vision_document_id(&metis_dir, vision_slug).await?)
    } else {
        None
    };

    // 3. Use DocumentCreationService to create the strategy
    let creation_service = DocumentCreationService::new(&metis_dir);
    
    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: parent_id.clone(),
        tags: vec![
            Tag::Label("strategy".to_string()),
            Tag::Phase(Phase::Shaping),
        ],
        phase: Some(Phase::Shaping),
        complexity: None,
        risk_level: Some(RiskLevel::Medium),
    };

    let result = creation_service
        .create_strategy(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create strategy: {}", e))?;

    println!("✓ Created strategy: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);
    if let Some(parent) = parent_id {
        println!("  Parent Vision: {}", parent);
    }

    Ok(())
}

/// Get the actual DocumentId by parsing the vision document
async fn get_vision_document_id(workspace_dir: &Path, vision_slug: &str) -> Result<DocumentId> {
    // Try to find and parse the vision document
    let vision_path = workspace_dir.join("vision.md");

    if !vision_path.exists() {
        anyhow::bail!(
            "Vision document not found: {}. Expected vision.md in workspace root.",
            vision_path.display()
        );
    }

    // Parse the vision document to get its actual ID
    let vision = Vision::from_file(&vision_path)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse vision document: {}", e))?;

    let vision_id = vision.id();

    // Verify the provided slug matches the vision ID
    if vision_id.to_string() != vision_slug {
        anyhow::bail!(
            "Vision slug mismatch. Found vision with ID '{}', but you specified '{}'",
            vision_id.to_string(),
            vision_slug
        );
    }

    Ok(vision_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use metis_core::{Document, DocumentType, Strategy};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_strategy_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory without workspace
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let result = create_new_strategy("Test Strategy", None).await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }

    #[tokio::test]
    async fn test_create_new_strategy_with_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace first
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
        };
        init_cmd.execute().await.unwrap();

        // Create strategy without parent vision
        let result = create_new_strategy("Test Strategy", None).await;
        assert!(result.is_ok());

        // Verify hierarchical path was created: /strategies/{id}/strategy.md
        let strategies_base = temp_dir.path().join(".metis/strategies");
        assert!(strategies_base.exists());

        // Find the strategy directory
        let strategy_dirs: Vec<_> = fs::read_dir(&strategies_base)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .collect();

        assert_eq!(
            strategy_dirs.len(),
            1,
            "Expected exactly one strategy directory"
        );

        // Check that strategy.md exists in the strategy directory
        let strategy_dir = strategy_dirs[0].path();
        let strategy_file = strategy_dir.join("strategy.md");
        assert!(
            strategy_file.exists(),
            "strategy.md not found in strategy directory"
        );

        // Verify file content has proper structure
        let content = fs::read_to_string(&strategy_file).unwrap();
        assert!(content.contains("level: strategy"));
        assert!(content.contains("title: \"Test Strategy\""));
        assert!(content.contains("#strategy"));
        assert!(content.contains("#phase/shaping"));
        assert!(content.contains("risk_level: medium"));

        // Verify the template was rendered
        assert!(content.contains("# Test Strategy Strategy"));
        assert!(content.contains("## Problem Statement"));
        assert!(content.contains("## Success Metrics"));
        assert!(content.contains("## Solution Approach"));
        assert!(content.contains("## Scope"));
        assert!(content.contains("## Risks & Unknowns"));
        assert!(content.contains("## Implementation Dependencies"));
        assert!(content.contains("## Change Log"));

        // Test that the created file can be read back with Strategy::from_file
        let parsed_strategy = Strategy::from_file(&strategy_file).await;
        assert!(
            parsed_strategy.is_ok(),
            "Failed to parse strategy file: {:?}",
            parsed_strategy.err()
        );

        let strategy = parsed_strategy.unwrap();
        assert_eq!(strategy.title(), "Test Strategy");
        assert_eq!(strategy.document_type(), DocumentType::Strategy);
        assert!(!strategy.archived());
        assert_eq!(strategy.risk_level(), RiskLevel::Medium);

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
