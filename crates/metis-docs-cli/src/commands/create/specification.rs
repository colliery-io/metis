use crate::workspace;
use anyhow::Result;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    Phase, Tag,
};

/// Create a new Specification document with defaults and write to file
pub async fn create_new_specification(title: &str, parent: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Use DocumentCreationService to create the Specification
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: Some(parent.into()),
        tags: vec![
            Tag::Label("specification".to_string()),
            Tag::Phase(Phase::Discovery),
        ],
        phase: Some(Phase::Discovery),
        complexity: None,
    };

    let result = creation_service
        .create_specification(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create specification: {}", e))?;

    println!("✓ Created Specification: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);
    println!("  Parent: {}", parent);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use metis_core::{Document, Specification};
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_create_new_specification_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_specification("Test Spec", "TEST-V-0001").await;
            assert!(result.is_err());
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("Not in a Metis workspace"));

            let _ = std::env::set_current_dir(original_dir);
        }
    }

    #[tokio::test]
    async fn test_create_new_specification_with_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().unwrap();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create specification
        let result = create_new_specification("System Design Spec", "TEST-V-0001").await;
        assert!(
            result.is_ok(),
            "Failed to create specification: {:?}",
            result.err()
        );

        // Verify path was created: /specifications/{SHORT_CODE}/specification.md
        let specs_base = temp_dir.path().join(".metis/specifications");
        assert!(specs_base.exists());

        // Find the specification directory
        let spec_dirs: Vec<_> = fs::read_dir(&specs_base)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .collect();

        assert_eq!(spec_dirs.len(), 1, "Expected exactly one specification dir");

        let spec_dir = spec_dirs[0].path();
        let dirname = spec_dir.file_name().unwrap().to_str().unwrap();
        assert!(
            dirname.starts_with("TEST-S-"),
            "Specification dir should be in short code format"
        );

        let spec_file = spec_dir.join("specification.md");
        assert!(spec_file.exists(), "specification.md should exist");

        // Verify file content
        let content = fs::read_to_string(&spec_file).unwrap();
        assert!(content.contains("level: specification"));
        assert!(content.contains("title: \"System Design Spec\""));
        assert!(content.contains("#specification"));
        assert!(content.contains("#phase/discovery"));

        // Test roundtrip
        let parsed = Specification::from_file(&spec_file).await;
        assert!(
            parsed.is_ok(),
            "Failed to parse specification: {:?}",
            parsed.err()
        );

        let spec = parsed.unwrap();
        assert_eq!(spec.title(), "System Design Spec");

        std::env::set_current_dir(original_dir).unwrap();
    }
}
