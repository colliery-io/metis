use crate::workspace;
use anyhow::Result;
use clap::{Args, Subcommand};
use metis_core::{
    domain::configuration::{ConfigFile, FlightLevelConfig},
    Database,
};

#[derive(Args)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub action: ConfigAction,
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Show current configuration
    Show,
    /// Set configuration using preset or custom values
    Set {
        /// Configuration preset (full, streamlined, direct)
        #[arg(short, long)]
        preset: Option<String>,
        /// Enable/disable strategies (true/false)
        #[arg(long)]
        strategies: Option<bool>,
        /// Enable/disable initiatives (true/false)
        #[arg(long)]
        initiatives: Option<bool>,
    },
    /// Get a specific configuration value
    Get {
        /// Configuration key to retrieve
        key: String,
    },
}

impl ConfigCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Connect to database
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Database connection failed: {}", e))?;
        let mut config_repo = db
            .configuration_repository()
            .map_err(|e| anyhow::anyhow!("Failed to create configuration repository: {}", e))?;

        // 3. Execute the requested action
        match &self.action {
            ConfigAction::Show => self.show_config(&mut config_repo).await,
            ConfigAction::Set {
                preset,
                strategies,
                initiatives,
            } => {
                self.set_config(&mut config_repo, &metis_dir, preset, *strategies, *initiatives)
                    .await
            }
            ConfigAction::Get { key } => self.get_config(&mut config_repo, key).await,
        }
    }

    async fn show_config(
        &self,
        config_repo: &mut metis_core::dal::database::configuration_repository::ConfigurationRepository,
    ) -> Result<()> {
        let flight_config = config_repo
            .get_flight_level_config()
            .map_err(|e| anyhow::anyhow!("Failed to get flight level configuration: {}", e))?;

        println!("Current Flight Level Configuration:");
        println!("  Preset: {}", flight_config.preset_name());
        println!("  Strategies enabled: {}", flight_config.strategies_enabled);
        println!(
            "  Initiatives enabled: {}",
            flight_config.initiatives_enabled
        );
        println!();
        println!("Hierarchy: {}", flight_config.hierarchy_display());
        println!();
        println!("Available document types:");
        for doc_type in flight_config.enabled_document_types() {
            println!("  - {}", doc_type);
        }

        Ok(())
    }

    async fn set_config(
        &self,
        config_repo: &mut metis_core::dal::database::configuration_repository::ConfigurationRepository,
        metis_dir: &std::path::Path,
        preset: &Option<String>,
        strategies: Option<bool>,
        initiatives: Option<bool>,
    ) -> Result<()> {
        let new_config = if let Some(preset_name) = preset {
            // Use preset configuration
            match preset_name.as_str() {
                "full" => FlightLevelConfig::full(),
                "streamlined" => FlightLevelConfig::streamlined(),
                "direct" => FlightLevelConfig::direct(),
                _ => {
                    anyhow::bail!(
                        "Invalid preset '{}'. Valid presets are: full, streamlined, direct",
                        preset_name
                    );
                }
            }
        } else if strategies.is_some() || initiatives.is_some() {
            // Use custom configuration
            let current_config = config_repo
                .get_flight_level_config()
                .map_err(|e| anyhow::anyhow!("Failed to get current configuration: {}", e))?;

            let strategies_enabled = strategies.unwrap_or(current_config.strategies_enabled);
            let initiatives_enabled = initiatives.unwrap_or(current_config.initiatives_enabled);

            FlightLevelConfig::new(strategies_enabled, initiatives_enabled)
                .map_err(|e| anyhow::anyhow!("Invalid configuration: {}", e))?
        } else {
            anyhow::bail!(
                "Must specify either --preset or at least one of --strategies/--initiatives"
            );
        };

        // Gate: strategies require upstream sync configuration
        if new_config.strategies_enabled {
            let config_file_path = metis_dir.join("config.toml");
            let has_sync = if config_file_path.exists() {
                let config_file = ConfigFile::load(&config_file_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load config.toml: {}", e))?;
                config_file.is_multi_workspace()
            } else {
                false
            };
            if !has_sync {
                anyhow::bail!(
                    "Strategies require multi-workspace sync.\n\
                     Configure upstream first with: metis init --upstream <url> --workspace-prefix <prefix>"
                );
            }
        }

        // Save the new configuration
        config_repo
            .set_flight_level_config(&new_config)
            .map_err(|e| anyhow::anyhow!("Failed to save configuration: {}", e))?;

        println!("Configuration updated successfully!");
        println!("New configuration:");
        println!("  Preset: {}", new_config.preset_name());
        println!("  Strategies enabled: {}", new_config.strategies_enabled);
        println!("  Initiatives enabled: {}", new_config.initiatives_enabled);
        println!("  Hierarchy: {}", new_config.hierarchy_display());

        Ok(())
    }

    async fn get_config(
        &self,
        config_repo: &mut metis_core::dal::database::configuration_repository::ConfigurationRepository,
        key: &str,
    ) -> Result<()> {
        let value = config_repo
            .get(key)
            .map_err(|e| anyhow::anyhow!("Failed to get configuration value: {}", e))?;

        match value {
            Some(v) => {
                println!("{}", v);
                Ok(())
            }
            None => {
                anyhow::bail!("Configuration key '{}' not found", key);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_config_show_default() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Initialize a new project first
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

        // Test config show
        let config_cmd = ConfigCommand {
            action: ConfigAction::Show,
        };

        let result = config_cmd.execute().await;
        assert!(result.is_ok());

        // Restore original directory
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_set_streamlined_preset() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Initialize a new project first
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

        // Test setting streamlined preset
        let config_cmd = ConfigCommand {
            action: ConfigAction::Set {
                preset: Some("streamlined".to_string()),
                strategies: None,
                initiatives: None,
            },
        };

        let result = config_cmd.execute().await;
        assert!(result.is_ok());

        // Restore original directory
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_set_invalid_preset() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Initialize a new project first
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

        // Test setting invalid preset
        let config_cmd = ConfigCommand {
            action: ConfigAction::Set {
                preset: Some("invalid".to_string()),
                strategies: None,
                initiatives: None,
            },
        };

        let result = config_cmd.execute().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid preset"));

        // Restore original directory
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_set_preset_full_without_sync_fails() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Initialize a new project (streamlined by default, no sync)
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

        // Try setting full preset without sync — should fail
        let config_cmd = ConfigCommand {
            action: ConfigAction::Set {
                preset: Some("full".to_string()),
                strategies: None,
                initiatives: None,
            },
        };

        let result = config_cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Strategies require multi-workspace sync"));

        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_set_strategies_true_without_sync_fails() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

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

        // Try setting --strategies true without sync — should fail
        let config_cmd = ConfigCommand {
            action: ConfigAction::Set {
                preset: None,
                strategies: Some(true),
                initiatives: None,
            },
        };

        let result = config_cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Strategies require multi-workspace sync"));

        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }

    #[tokio::test]
    async fn test_config_without_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without initializing a workspace
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Test config show without workspace
        let config_cmd = ConfigCommand {
            action: ConfigAction::Show,
        };

        let result = config_cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));

        // Restore original directory
        if let Some(dir) = original_dir {
            std::env::set_current_dir(dir).unwrap();
        }
    }
}
