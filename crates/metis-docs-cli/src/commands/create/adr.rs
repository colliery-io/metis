use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    Phase, Tag,
};

/// Create a new ADR document with defaults and write to file
pub async fn create_new_adr(title: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Use DocumentCreationService to create the ADR
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: None,
        tags: vec![Tag::Label("adr".to_string()), Tag::Phase(Phase::Draft)],
        phase: Some(Phase::Draft),
        complexity: None,
        risk_level: None,
    };

    let result = creation_service
        .create_adr(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create ADR: {}", e))?;

    println!("✓ Created ADR: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use metis_core::{Adr, Document};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_adr_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_adr("Test ADR").await;
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
    async fn test_create_new_adr_with_workspace() {
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
            upstream: None,
            workspace_prefix: None,
            team: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create ADR
        let result = create_new_adr("Test ADR").await;
        assert!(result.is_ok(), "Failed to create ADR: {:?}", result.err());

        // Verify hierarchical path was created: /adrs/{number}-{slug}.md
        let adrs_base = temp_dir.path().join(".metis/adrs");
        assert!(adrs_base.exists());

        // Find the ADR file
        let adr_files: Vec<_> = fs::read_dir(&adrs_base)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().is_file() && entry.path().extension().is_some_and(|ext| ext == "md")
            })
            .collect();

        assert_eq!(adr_files.len(), 1, "Expected exactly one ADR file");

        // Check that the ADR file has the correct format
        let adr_file = adr_files[0].path();
        let filename = adr_file.file_stem().unwrap().to_str().unwrap();
        assert!(
            filename.starts_with("TEST-A-"),
            "ADR filename should be in short code format"
        );

        // Verify file content has proper structure
        let content = fs::read_to_string(&adr_file).unwrap();
        println!("Generated ADR content:\n{}", content); // Debug output

        assert!(content.contains("level: adr"));
        assert!(content.contains("title: \"Test ADR\""));
        assert!(content.contains("#adr"));
        assert!(content.contains("#phase/draft"));
        assert!(content.contains("decision_maker:"));
        assert!(content.contains("decision_date:"));

        // Test that the created file can be read back with Adr::from_file
        let parsed_adr = Adr::from_file(&adr_file).await;
        assert!(
            parsed_adr.is_ok(),
            "Failed to parse ADR file: {:?}",
            parsed_adr.err()
        );

        let adr = parsed_adr.unwrap();
        assert_eq!(adr.title(), "Test ADR");
        assert_eq!(adr.number(), 1);
        assert_eq!(adr.decision_maker(), "");
        assert!(adr.decision_date().is_none());

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();
    }
}
