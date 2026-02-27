use anyhow::Result;
use clap::Args;
use metis_core::application::services::document::DocumentValidationService;
use std::path::PathBuf;

#[derive(Args)]
pub struct ValidateCommand {
    /// Path to the document file to validate
    pub file_path: PathBuf,
}

impl ValidateCommand {
    pub async fn execute(&self) -> Result<()> {
        // Use the validation service
        let service = DocumentValidationService::new();
        let result = service.validate_document(&self.file_path).await;

        match result {
            Ok(validation_result) => {
                if validation_result.is_valid {
                    println!(
                        "✓ Valid {} document: {}",
                        validation_result.document_type,
                        self.file_path.display()
                    );
                    Ok(())
                } else {
                    println!("✗ Invalid document: {}", self.file_path.display());
                    for error in &validation_result.errors {
                        println!("  - {}", error);
                    }
                    anyhow::bail!("Document validation failed")
                }
            }
            Err(e) => {
                println!("✗ Error validating document: {}", self.file_path.display());
                println!("  Error: {}", e);
                Err(e.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_validate_command_missing_file() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = ValidateCommand {
            file_path: PathBuf::from("nonexistent.md"),
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
            .contains("File does not exist"));
    }

    #[tokio::test]
    async fn test_validate_command_valid_vision() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace with vision document
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

        let vision_path = temp_dir.path().join(".metis/vision.md");

        let cmd = ValidateCommand {
            file_path: vision_path,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_validate_command_invalid_document() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create an invalid markdown file
        let invalid_path = temp_dir.path().join("invalid.md");
        fs::write(
            &invalid_path,
            "# Invalid Document\n\nThis has no frontmatter.",
        )
        .unwrap();

        let cmd = ValidateCommand {
            file_path: invalid_path,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
    }
}
