use crate::workspace;
use anyhow::Result;
use metis_core::{Adr, Document, Phase, Tag};
use std::path::Path;

/// Create a new ADR document with defaults and write to file
pub async fn create_new_adr(title: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Get the next ADR number
    let next_number = get_next_adr_number(&metis_dir)?;

    // 3. Create ADR with defaults
    let tags = vec![Tag::Label("adr".to_string()), Tag::Phase(Phase::Draft)];

    let adr = Adr::new(
        next_number,
        title.to_string(),
        String::new(), // Empty decision maker initially
        None,          // No decision date until transition to "decided"
        None,          // No parent by default
        tags,
        false, // not archived
    )
    .unwrap();

    // 4. Determine file path: /adrs/{number}-{slug}.md
    let adrs_dir = metis_dir.join("adrs");
    std::fs::create_dir_all(&adrs_dir)?;

    let doc_id = adr.id();
    let file_path = adrs_dir.join(format!("{}.md", doc_id));

    // Check if file already exists
    if file_path.exists() {
        anyhow::bail!("ADR document already exists: {}", file_path.display());
    }

    // 5. Write to file
    adr.to_file(&file_path).await?;

    println!("✓ Created ADR: {}", file_path.display());
    println!("  ID: {}", doc_id);
    println!("  Number: {}", next_number);
    println!("  Title: {}", title);

    Ok(())
}

/// Get the next available ADR number by checking existing ADRs
fn get_next_adr_number(workspace_dir: &Path) -> Result<u32> {
    let adrs_dir = workspace_dir.join("adrs");

    if !adrs_dir.exists() {
        return Ok(1); // First ADR
    }

    let mut max_number = 0;

    for entry in std::fs::read_dir(&adrs_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            if let Some(filename) = path.file_stem().and_then(|name| name.to_str()) {
                // Parse number from filename like "0001-title" or "001-title"
                if let Some(dash_pos) = filename.find('-') {
                    let number_part = &filename[..dash_pos];
                    if let Ok(num) = number_part.parse::<u32>() {
                        max_number = max_number.max(num);
                    }
                }
            }
        }
    }

    Ok(max_number + 1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_get_next_adr_number_empty_directory() {
        let temp_dir = tempdir().unwrap();
        let adrs_dir = temp_dir.path().join("adrs");

        // Test with non-existent directory
        let next_number = get_next_adr_number(temp_dir.path()).unwrap();
        assert_eq!(next_number, 1);

        // Test with empty directory
        fs::create_dir(&adrs_dir).unwrap();
        let next_number = get_next_adr_number(temp_dir.path()).unwrap();
        assert_eq!(next_number, 1);
    }

    #[test]
    fn test_get_next_adr_number_with_existing_adrs() {
        let temp_dir = tempdir().unwrap();
        let adrs_dir = temp_dir.path().join("adrs");
        fs::create_dir(&adrs_dir).unwrap();

        // Create some mock ADR files
        fs::write(adrs_dir.join("0001-first-decision.md"), "content").unwrap();
        fs::write(adrs_dir.join("0003-third-decision.md"), "content").unwrap();
        fs::write(adrs_dir.join("0005-fifth-decision.md"), "content").unwrap();
        fs::write(adrs_dir.join("not-an-adr.md"), "content").unwrap(); // Should be ignored

        let next_number = get_next_adr_number(temp_dir.path()).unwrap();
        assert_eq!(next_number, 6); // Should be max + 1
    }

    #[test]
    fn test_get_next_adr_number_different_formats() {
        let temp_dir = tempdir().unwrap();
        let adrs_dir = temp_dir.path().join("adrs");
        fs::create_dir(&adrs_dir).unwrap();

        // Test different number formats
        fs::write(adrs_dir.join("1-simple.md"), "content").unwrap();
        fs::write(adrs_dir.join("002-padded.md"), "content").unwrap();
        fs::write(adrs_dir.join("0010-four-digits.md"), "content").unwrap();
        fs::write(adrs_dir.join("invalid-no-number.md"), "content").unwrap(); // Should be ignored

        let next_number = get_next_adr_number(temp_dir.path()).unwrap();
        assert_eq!(next_number, 11); // Should be 10 + 1
    }

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
            filename.starts_with("001-"),
            "ADR filename should start with 001-"
        );
        assert!(
            filename.contains("test-adr"),
            "ADR filename should contain the slug"
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
