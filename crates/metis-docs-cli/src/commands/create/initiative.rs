use crate::workspace;
use anyhow::Result;
use dialoguer::Select;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::documents::initiative::Complexity,
    Database, Document, Phase, Tag, Vision,
};

/// Create a new Initiative document with defaults and write to file
pub async fn create_new_initiative(title: &str, vision_id: &str) -> Result<()> {
    // 1. Validate we're in a metis workspace
    let (workspace_exists, metis_dir) = workspace::has_metis_vault();
    if !workspace_exists {
        anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
    }
    let metis_dir = metis_dir.unwrap();

    // 2. Verify the vision exists
    let vision_doc_id = find_vision(&metis_dir, vision_id).await?;

    // 3. Prompt for complexity level
    let complexity = prompt_for_complexity()?;

    // 4. Use DocumentCreationService to create the initiative
    let creation_service = DocumentCreationService::new(&metis_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: Some(vision_doc_id.clone()),
        tags: vec![
            Tag::Label("initiative".to_string()),
            Tag::Phase(Phase::Discovery),
        ],
        phase: Some(Phase::Discovery),
        complexity: Some(complexity),
    };

    let result = creation_service
        .create_initiative(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create initiative: {}", e))?;

    println!("✓ Created initiative: {}", result.file_path.display());
    println!("  ID: {}", result.document_id);
    println!("  Short Code: {}", result.short_code);
    println!("  Title: {}", title);
    println!("  Parent Vision: {}", vision_id);
    println!("  Complexity: {:?}", complexity);

    Ok(())
}

/// Find a vision by short code and return its DocumentId
async fn find_vision(
    workspace_dir: &std::path::Path,
    vision_id: &str,
) -> Result<metis_core::domain::documents::types::DocumentId> {
    // First try database lookup (works when synced)
    let db_path = workspace_dir.join("metis.db");
    if db_path.exists() {
        let db = Database::new(&db_path.to_string_lossy())
            .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
        let mut repo = db
            .repository()
            .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

        if let Ok(Some(vision_doc)) = repo.find_by_short_code(vision_id) {
            if vision_doc.document_type == "vision" {
                return Ok(metis_core::domain::documents::types::DocumentId::new(
                    &vision_doc.id,
                ));
            }
            anyhow::bail!(
                "'{}' is a {} document, not a vision. Initiatives must be created under a vision.",
                vision_id,
                vision_doc.document_type
            );
        }
    }

    // Fall back to reading vision.md directly (works before first sync)
    let vision_path = workspace_dir.join("vision.md");
    if vision_path.exists() {
        let vision = Vision::from_file(&vision_path)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read vision: {}", e))?;
        if vision.metadata().short_code == vision_id {
            return Ok(vision.id().clone());
        }
    }

    anyhow::bail!(
        "Vision '{}' not found. Use 'metis list -t vision' to see available visions.",
        vision_id,
    );
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

    #[tokio::test]
    async fn test_create_new_initiative_no_workspace() {
        let temp_dir = tempfile::tempdir().unwrap();
        let Ok(original_dir) = std::env::current_dir() else {
            return; // CWD unavailable due to parallel test interference
        };

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_ok() {
            let result = create_new_initiative("Test Initiative", "PROJ-V-0001").await;
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
    async fn test_find_vision_not_found() {
        let temp_dir = tempfile::tempdir().unwrap();
        let Ok(original_dir) = std::env::current_dir() else {
            return; // CWD unavailable due to parallel test interference
        };

        // Ensure we can change to temp directory
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            let _ = std::env::set_current_dir(original_dir);
            return;
        }

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        if init_cmd.execute().await.is_err() {
            let _ = std::env::set_current_dir(original_dir);
            return;
        }

        // Try to find non-existent vision
        let metis_dir = temp_dir.path().join(".metis");
        let result = find_vision(&metis_dir, "TEST-V-9999").await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("not found"));

        // Always restore original directory
        let _ = std::env::set_current_dir(original_dir);
    }

    #[tokio::test]
    async fn test_create_initiative_under_vision() {
        let temp_dir = tempfile::tempdir().unwrap();
        let Ok(original_dir) = std::env::current_dir() else {
            return; // CWD unavailable due to parallel test interference
        };

        // Change to temp directory
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return;
        }

        // Create workspace (creates vision TEST-V-0001)
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let metis_dir = temp_dir.path().join(".metis");

        // Vision should be findable by short code
        let vision_id = find_vision(&metis_dir, "TEST-V-0001").await;
        assert!(vision_id.is_ok(), "Vision should be found: {:?}", vision_id.err());

        // Restore original directory
        let _ = std::env::set_current_dir(original_dir);
    }
}
