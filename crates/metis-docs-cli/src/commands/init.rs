use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{
    application::services::document::creation::{DocumentCreationConfig, DocumentCreationService},
    domain::configuration::FlightLevelConfig,
    Database, Phase, Tag,
};
use std::path::Path;

#[derive(Args)]
pub struct InitCommand {
    /// Project name for the vision document
    #[arg(short, long)]
    pub name: Option<String>,
    /// Project prefix for document short codes, up to 6 characters (e.g., PROJ, ACME, TEST)
    #[arg(short = 'P', long)]
    pub prefix: Option<String>,
    /// Configuration preset (full, streamlined, direct). Default: streamlined
    #[arg(short, long)]
    pub preset: Option<String>,
    /// Enable/disable strategies (true/false)
    #[arg(long)]
    pub strategies: Option<bool>,
    /// Enable/disable initiatives (true/false)
    #[arg(long)]
    pub initiatives: Option<bool>,
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
        let db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Database initialization failed: {}", e))?;

        // Set up flight level configuration
        let flight_config = self.determine_flight_config()?;
        let mut config_repo = db
            .configuration_repository()
            .map_err(|e| anyhow::anyhow!("Failed to create configuration repository: {}", e))?;
        config_repo
            .set_flight_level_config(&flight_config)
            .map_err(|e| anyhow::anyhow!("Failed to set flight level configuration: {}", e))?;

        // Set project prefix for document creation
        let project_prefix = if let Some(prefix) = &self.prefix {
            // Use explicitly provided prefix, but limit to 6 characters
            let truncated = prefix.to_uppercase();
            if truncated.len() > 6 {
                truncated.chars().take(6).collect()
            } else {
                truncated
            }
        } else if cfg!(test) {
            // Use "TEST" in test mode
            "TEST".to_string()
        } else {
            // Extract first 6 uppercase letters from project name, or use "PROJ" as fallback
            let project_name = self.name.as_deref().unwrap_or("Project Vision");
            project_name
                .chars()
                .filter(|c| c.is_alphabetic())
                .map(|c| c.to_uppercase().collect::<String>())
                .collect::<String>()
                .get(0..6.min(project_name.len()))
                .unwrap_or("PROJ")
                .to_string()
        };
        config_repo
            .set_project_prefix(&project_prefix)
            .map_err(|e| anyhow::anyhow!("Failed to set project prefix: {}", e))?;

        // Create strategies directory
        let strategies_dir = metis_dir.join("strategies");
        std::fs::create_dir_all(&strategies_dir)?;

        // Create vision.md with defaults
        let project_name = self.name.as_deref().unwrap_or("Project Vision");
        create_default_vision(&metis_dir, project_name).await?;

        println!("✓ Initialized Metis workspace in {}", current_dir.display());
        println!("✓ Created vision.md with project template");
        println!("✓ Set project prefix: {}", project_prefix);
        println!(
            "✓ Set flight level configuration: {}",
            flight_config.preset_name()
        );

        Ok(())
    }

    /// Determine the flight level configuration based on command arguments
    fn determine_flight_config(&self) -> Result<FlightLevelConfig> {
        if let Some(preset_name) = &self.preset {
            // Use specified preset
            match preset_name.as_str() {
                "full" => Ok(FlightLevelConfig::full()),
                "streamlined" => Ok(FlightLevelConfig::streamlined()),
                "direct" => Ok(FlightLevelConfig::direct()),
                _ => {
                    anyhow::bail!(
                        "Invalid preset '{}'. Valid presets are: full, streamlined, direct",
                        preset_name
                    );
                }
            }
        } else if self.strategies.is_some() || self.initiatives.is_some() {
            // Use custom configuration, with streamlined as default base
            let default_config = FlightLevelConfig::streamlined();
            let strategies_enabled = self.strategies.unwrap_or(default_config.strategies_enabled);
            let initiatives_enabled = self
                .initiatives
                .unwrap_or(default_config.initiatives_enabled);

            FlightLevelConfig::new(strategies_enabled, initiatives_enabled)
                .map_err(|e| anyhow::anyhow!("Invalid configuration: {}", e))
        } else {
            // Default to streamlined preset
            Ok(FlightLevelConfig::streamlined())
        }
    }
}

/// Create a new Vision document with defaults and write to file
async fn create_default_vision(workspace_dir: &Path, title: &str) -> Result<()> {
    // Use DocumentCreationService to create the vision
    let creation_service = DocumentCreationService::new(workspace_dir);

    let config = DocumentCreationConfig {
        title: title.to_string(),
        description: None,
        parent_id: None,
        tags: vec![Tag::Label("vision".to_string()), Tag::Phase(Phase::Draft)],
        phase: Some(Phase::Draft),
        complexity: None,
        risk_level: None,
    };

    let _result = creation_service
        .create_vision(config)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create vision: {}", e))?;

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
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
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
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_workspace_already_exists() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
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
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify existing database wasn't overwritten
        let db_content = fs::read_to_string(&db_path).unwrap();
        assert_eq!(db_content, "existing");

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_default_name() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command without name
        let cmd = InitCommand {
            name: None,
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify vision.md was created with default name
        let vision_path = temp_dir.path().join(".metis").join("vision.md");
        let vision_content = fs::read_to_string(&vision_path).unwrap();
        assert!(vision_content.contains("Project Vision"));

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_with_preset() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command with full preset
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: Some("full".to_string()),
            strategies: None,
            initiatives: None,
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify workspace was created
        let metis_dir = temp_dir.path().join(".metis");
        assert!(metis_dir.exists());

        // Verify configuration was set
        use metis_core::Database;
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        let config = config_repo.get_flight_level_config().unwrap();

        assert_eq!(
            config,
            metis_core::domain::configuration::FlightLevelConfig::full()
        );

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_with_custom_flags() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command with custom flags (strategies disabled, initiatives enabled)
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: Some(false),
            initiatives: Some(true),
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify configuration was set
        use metis_core::Database;
        let metis_dir = temp_dir.path().join(".metis");
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        let config = config_repo.get_flight_level_config().unwrap();

        assert!(!config.strategies_enabled);
        assert!(config.initiatives_enabled);

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_default_streamlined() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command with no preset specified (should default to streamlined)
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_ok());

        // Verify configuration defaults to streamlined
        use metis_core::Database;
        let metis_dir = temp_dir.path().join(".metis");
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = db.configuration_repository().unwrap();
        let config = config_repo.get_flight_level_config().unwrap();

        assert_eq!(
            config,
            metis_core::domain::configuration::FlightLevelConfig::streamlined()
        );

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_invalid_preset() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Run init command with invalid preset
        let cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: Some("invalid".to_string()),
            strategies: None,
            initiatives: None,
            prefix: None,
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid preset"));

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
