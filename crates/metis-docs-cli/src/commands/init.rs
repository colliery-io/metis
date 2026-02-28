use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{
    application::services::workspace::WorkspaceInitializationService,
    domain::configuration::{validate_workspace_prefix, ConfigFile, FlightLevelConfig},
    Database,
};
use std::time::Instant;

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
    /// Upstream central repo URL for multi-workspace sync
    #[arg(short, long)]
    pub upstream: Option<String>,
    /// Workspace prefix — folder name in central repo (2-20 chars, lowercase alphanum + hyphens)
    #[arg(short = 'w', long)]
    pub workspace_prefix: Option<String>,
    /// Team label for multi-workspace views (optional)
    #[arg(short, long)]
    pub team: Option<String>,
}

impl InitCommand {
    pub async fn execute(&self) -> Result<()> {
        // If --upstream is provided, use the upstream configuration flow
        if self.upstream.is_some() {
            return self.execute_with_upstream().await;
        }

        // --- Standard single-workspace init (unchanged) ---

        let (workspace_exists, _) = workspace::has_metis_vault();
        if workspace_exists {
            println!("Metis workspace already exists in this directory");
            return Ok(());
        }

        let metis_dir = self.create_workspace().await?;

        // Create/update .gitignore in .metis directory to ignore database
        let gitignore_path = metis_dir.join(".gitignore");
        std::fs::write(&gitignore_path, "metis.db\nmetis-mcp-server.log\n")
            .map_err(|e| anyhow::anyhow!("Failed to create .gitignore: {}", e))?;

        let current_dir = std::env::current_dir()?;
        let project_prefix = self.determine_project_prefix(
            self.name.as_deref().unwrap_or("Project Vision"),
        );
        let flight_config = self.determine_flight_config()?;

        // Gate: strategies require upstream sync configuration
        if flight_config.strategies_enabled && self.upstream.is_none() {
            anyhow::bail!(
                "Strategies require multi-workspace sync.\n\
                 Use: metis init --upstream <url> --workspace-prefix <prefix> --preset full\n\
                 Or use --preset streamlined (default) for single-workspace mode."
            );
        }

        println!(
            "[+] Initialized Metis workspace in {}",
            current_dir.display()
        );
        println!("[+] Created vision.md with project template");
        println!("[+] Created config.toml with project settings");
        println!("[+] Set project prefix: {}", project_prefix);
        println!(
            "[+] Set flight level configuration: {}",
            flight_config.preset_name()
        );

        Ok(())
    }

    /// Execute the upstream configuration flow: create workspace (if needed),
    /// test connectivity, write upstream config, run initial sync.
    async fn execute_with_upstream(&self) -> Result<()> {
        let upstream_url = self.upstream.as_deref().unwrap();

        // Validate workspace prefix is provided
        let ws_prefix = self.workspace_prefix.as_deref().ok_or_else(|| {
            anyhow::anyhow!(
                "--workspace-prefix is required when using --upstream.\n\
                 Example: metis init --upstream <url> --workspace-prefix api"
            )
        })?;

        // Validate the workspace prefix format
        validate_workspace_prefix(ws_prefix)
            .map_err(|e| anyhow::anyhow!("{}", e))?;

        let (workspace_exists, existing_dir) = workspace::has_metis_vault();

        let metis_dir = if workspace_exists {
            let dir = existing_dir.unwrap();

            // Check for existing upstream configuration
            let config_path = dir.join("config.toml");
            if config_path.exists() {
                let config = ConfigFile::load(&config_path)
                    .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
                if config.is_multi_workspace() {
                    anyhow::bail!(
                        "Upstream already configured. Use `metis sync` to sync."
                    );
                }
            }
            println!("Adding upstream configuration to existing workspace...");
            dir
        } else {
            // Create workspace first
            let dir = self.create_workspace().await?;

            // Create .gitignore
            let gitignore_path = dir.join(".gitignore");
            std::fs::write(&gitignore_path, "metis.db\nmetis-mcp-server.log\n")
                .map_err(|e| anyhow::anyhow!("Failed to create .gitignore: {}", e))?;

            println!("[+] Initialized Metis workspace");
            dir
        };

        // Test connectivity before writing any config
        println!("\nConnecting to {}...", upstream_url);
        let mut ctx = metis_sync::SyncContext::new(upstream_url, ws_prefix)
            .map_err(|e| Self::format_connectivity_error(e, upstream_url))?;
        ctx.fetch()
            .map_err(|e| Self::format_connectivity_error(e, upstream_url))?;
        println!("  OK");

        // Write upstream config to config.toml
        let config_path = metis_dir.join("config.toml");
        let mut config = ConfigFile::load(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;
        config
            .set_workspace(ws_prefix.to_string(), self.team.clone())
            .map_err(|e| anyhow::anyhow!("Failed to set workspace config: {}", e))?;
        config
            .set_sync(upstream_url.to_string())
            .map_err(|e| anyhow::anyhow!("Failed to set sync config: {}", e))?;
        config
            .save(&config_path)
            .map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))?;

        println!("[+] Workspace prefix: {}", ws_prefix);
        if let Some(ref team) = self.team {
            println!("[+] Team label: {}", team);
        }

        // Run initial sync
        println!("\nRunning initial sync...");
        Self::run_initial_sync(&metis_dir, upstream_url, ws_prefix, config.last_synced_commit())?;

        println!("\nReady. Run `metis sync` to sync with central.");

        Ok(())
    }

    /// Create a new Metis workspace and return the .metis directory path.
    async fn create_workspace(&self) -> Result<std::path::PathBuf> {
        let current_dir = std::env::current_dir()?;
        let project_name = self.name.as_deref().unwrap_or("Project Vision");
        let project_prefix = self.determine_project_prefix(project_name);

        let result = WorkspaceInitializationService::initialize_workspace_with_prefix(
            &current_dir,
            project_name,
            Some(&project_prefix),
        )
        .await
        .map_err(|e| anyhow::anyhow!("Failed to initialize workspace: {}", e))?;

        // Set flight level config if specified
        let flight_config = self.determine_flight_config()?;

        // Gate: strategies require upstream sync configuration
        if flight_config.strategies_enabled && self.upstream.is_none() {
            anyhow::bail!(
                "Strategies require multi-workspace sync.\n\
                 Use: metis init --upstream <url> --workspace-prefix <prefix> --preset full\n\
                 Or use --preset streamlined (default) for single-workspace mode."
            );
        }

        let db = Database::new(result.database_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let mut config_repo = db
            .configuration_repository()
            .map_err(|e| anyhow::anyhow!("Failed to create configuration repository: {}", e))?;

        let current_config = config_repo
            .get_flight_level_config()
            .map_err(|e| anyhow::anyhow!("Failed to get flight level config: {}", e))?;
        if flight_config != current_config {
            config_repo
                .set_flight_level_config(&flight_config)
                .map_err(|e| anyhow::anyhow!("Failed to set flight level configuration: {}", e))?;

            let config_file_path = result.metis_dir.join("config.toml");
            let config_file = ConfigFile::new(project_prefix, flight_config)
                .map_err(|e| anyhow::anyhow!("Failed to create config file: {}", e))?;
            config_file
                .save(&config_file_path)
                .map_err(|e| anyhow::anyhow!("Failed to save config.toml: {}", e))?;
        }

        Ok(result.metis_dir)
    }

    /// Run the initial sync cycle (fetch → hydrate → dehydrate → push).
    fn run_initial_sync(
        metis_dir: &std::path::Path,
        upstream_url: &str,
        ws_prefix: &str,
        last_synced: Option<&str>,
    ) -> Result<()> {
        let start = Instant::now();

        // Flatten local workspace documents for pushing
        let flatten_result =
            metis_core::application::services::layout::flatten_workspace(metis_dir)
                .map_err(|e| anyhow::anyhow!("Failed to flatten workspace: {}", e))?;

        let local_documents: Vec<metis_sync::dehydration::FlatDoc> = flatten_result
            .documents
            .iter()
            .map(|d| metis_sync::dehydration::FlatDoc {
                short_code: d.short_code.clone(),
                filename: d.filename.clone(),
                content: d.content.clone(),
            })
            .collect();

        let sync_config = metis_sync::orchestration::SyncConfig {
            upstream_url: upstream_url.to_string(),
            workspace_prefix: ws_prefix.to_string(),
            last_synced_commit: last_synced.map(|s| s.to_string()),
        };

        let sync_options = metis_sync::orchestration::SyncOptions::new();

        let result = metis_sync::orchestration::sync(
            &sync_config,
            metis_dir,
            &local_documents,
            &sync_options,
        )
        .map_err(|e| Self::format_connectivity_error(e, upstream_url))?;

        let elapsed = start.elapsed();

        // Update last_synced_commit in config.toml
        if let Some(ref new_sha) = result.new_synced_commit {
            let config_path = metis_dir.join("config.toml");
            if let Ok(mut cfg) = ConfigFile::load(&config_path) {
                if cfg.update_last_synced_commit(new_sha).is_ok() {
                    let _ = cfg.save(&config_path);
                }
            }
        }

        // Print results
        if let Some(ref hydration) = result.hydration {
            if !hydration.hydrated_workspaces.is_empty() {
                for ws in &hydration.hydrated_workspaces {
                    let ws_dir = metis_dir.join(ws);
                    let count = std::fs::read_dir(&ws_dir)
                        .map(|entries| {
                            entries
                                .filter_map(|e| e.ok())
                                .filter(|e| {
                                    e.path().extension().is_some_and(|ext| ext == "md")
                                })
                                .count()
                        })
                        .unwrap_or(0);
                    println!("  Pulled: {}/ ({} docs)", ws, count);
                }
            }
        }

        if result.pushed() {
            println!(
                "  Registered: {}/ ({} docs pushed)",
                ws_prefix,
                result.files_pushed()
            );
        }

        println!("  Sync complete in {:.1}s", elapsed.as_secs_f64());

        Ok(())
    }

    /// Convert SyncError to a user-friendly error message for connectivity issues.
    fn format_connectivity_error(
        error: metis_sync::SyncError,
        upstream_url: &str,
    ) -> anyhow::Error {
        match &error {
            metis_sync::SyncError::Auth { message } => {
                anyhow::anyhow!(
                    "Authentication failed for {}: {}\nCheck your SSH keys or credentials.",
                    upstream_url,
                    message
                )
            }
            metis_sync::SyncError::FetchFailed { url, reason } => {
                anyhow::anyhow!(
                    "Cannot reach {}: {}\nCheck your network connection and URL.",
                    url,
                    reason
                )
            }
            metis_sync::SyncError::InvalidUrl { url } => {
                anyhow::anyhow!(
                    "Invalid upstream URL: {}\nExpected SSH (git@host:org/repo.git) or HTTPS (https://host/repo.git).",
                    url
                )
            }
            _ => anyhow::anyhow!("Connection failed: {}", error),
        }
    }

    /// Determine the project prefix from command arguments or project name
    fn determine_project_prefix(&self, project_name: &str) -> String {
        if let Some(prefix) = &self.prefix {
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
            project_name
                .chars()
                .filter(|c| c.is_alphabetic())
                .map(|c| c.to_uppercase().collect::<String>())
                .collect::<String>()
                .get(0..6.min(project_name.len()))
                .unwrap_or("PROJ")
                .to_string()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    /// Helper to create a basic InitCommand with no upstream config.
    fn basic_init(name: Option<&str>) -> InitCommand {
        InitCommand {
            name: name.map(|s| s.to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        }
    }

    #[tokio::test]
    async fn test_init_command_creates_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = basic_init(Some("Test Project"));
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

        // Verify config.toml was created
        let config_path = metis_dir.join("config.toml");
        assert!(config_path.exists(), "config.toml should be created");
        assert!(config_path.is_file());

        // Verify config.toml content
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(config_content.contains("[project]"));
        assert!(config_content.contains("prefix = \"TEST\""));
        assert!(config_content.contains("[flight_levels]"));

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

        let cmd = basic_init(Some("Test Project"));
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

        let cmd = basic_init(None);
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
    async fn test_init_command_with_preset_full_without_upstream_fails() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = InitCommand {
            preset: Some("full".to_string()),
            ..basic_init(Some("Test Project"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Strategies require multi-workspace sync"));

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_strategies_true_without_upstream_fails() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = InitCommand {
            strategies: Some(true),
            initiatives: Some(true),
            ..basic_init(Some("Test Project"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Strategies require multi-workspace sync"));

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_command_with_preset_full_and_upstream_succeeds() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Create bare git repo as "central"
        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let cmd = InitCommand {
            preset: Some("full".to_string()),
            upstream: Some(central_url),
            workspace_prefix: Some("api".to_string()),
            prefix: Some("TEST".to_string()),
            ..basic_init(Some("Test Project"))
        };

        let result = cmd.execute().await;
        assert!(result.is_ok(), "Init with --preset full and --upstream should succeed: {:?}", result);

        // Verify workspace was created
        let metis_dir = project_dir.join(".metis");
        assert!(metis_dir.exists());

        // Verify configuration was set to full
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

        let cmd = InitCommand {
            strategies: Some(false),
            initiatives: Some(true),
            ..basic_init(Some("Test Project"))
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

        let cmd = basic_init(Some("Test Project"));
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

        let cmd = InitCommand {
            preset: Some("invalid".to_string()),
            ..basic_init(Some("Test Project"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid preset"));

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    // --- Upstream configuration tests ---

    #[tokio::test]
    async fn test_init_upstream_missing_workspace_prefix() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = InitCommand {
            upstream: Some("git@github.com:org/repo.git".to_string()),
            ..basic_init(Some("Test"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("--workspace-prefix is required"));

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_invalid_workspace_prefix() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = InitCommand {
            upstream: Some("git@github.com:org/repo.git".to_string()),
            workspace_prefix: Some("INVALID".to_string()), // uppercase not allowed
            ..basic_init(Some("Test"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("invalid") || err_msg.contains("Invalid"),
            "Expected validation error, got: {}",
            err_msg
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_unreachable() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = InitCommand {
            upstream: Some("git@nonexistent.invalid:org/repo.git".to_string()),
            workspace_prefix: Some("api".to_string()),
            ..basic_init(Some("Test"))
        };

        let result = cmd.execute().await;
        assert!(result.is_err());

        // Config should NOT be written (connectivity test failed)
        let config_path = temp_dir.path().join(".metis/config.toml");
        if config_path.exists() {
            let config = ConfigFile::load(&config_path).unwrap();
            assert!(
                !config.is_multi_workspace(),
                "Config should not have upstream after connectivity failure"
            );
        }

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_already_configured() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace with upstream already configured
        let cmd = basic_init(Some("Test"));
        cmd.execute().await.unwrap();

        // Manually add upstream config
        let config_path = temp_dir.path().join(".metis/config.toml");
        let mut config = ConfigFile::load(&config_path).unwrap();
        config
            .set_workspace("existing".to_string(), None)
            .unwrap();
        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();
        config.save(&config_path).unwrap();

        // Try to add upstream again
        let cmd2 = InitCommand {
            upstream: Some("git@github.com:org/other.git".to_string()),
            workspace_prefix: Some("api".to_string()),
            ..basic_init(Some("Test"))
        };

        let result = cmd2.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Upstream already configured"));

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_e2e_new_project() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Create bare git repo as "central"
        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        // Work in a subdirectory
        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        // Init with upstream (new project)
        let cmd = InitCommand {
            upstream: Some(central_url.clone()),
            workspace_prefix: Some("api".to_string()),
            prefix: Some("API".to_string()),
            ..basic_init(Some("API Service"))
        };

        let result = cmd.execute().await;
        assert!(result.is_ok(), "Init with upstream failed: {:?}", result);

        // Verify workspace was created
        let metis_dir = project_dir.join(".metis");
        assert!(metis_dir.exists());

        // Verify config has upstream
        let config_path = metis_dir.join("config.toml");
        let config = ConfigFile::load(&config_path).unwrap();
        assert!(config.is_multi_workspace());
        assert_eq!(config.workspace_prefix(), Some("api"));
        assert_eq!(config.upstream_url(), Some(central_url.as_str()));

        // Verify last_synced_commit was set (initial sync ran)
        assert!(
            config.last_synced_commit().is_some(),
            "last_synced_commit should be set after initial sync"
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_e2e_existing_project() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Create bare git repo as "central"
        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        // Create existing workspace without upstream
        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let init_cmd = basic_init(Some("Existing Project"));
        init_cmd.execute().await.unwrap();

        // Verify no upstream
        let config_path = project_dir.join(".metis/config.toml");
        let config = ConfigFile::load(&config_path).unwrap();
        assert!(!config.is_multi_workspace());

        // Add upstream to existing project
        let cmd = InitCommand {
            upstream: Some(central_url.clone()),
            workspace_prefix: Some("existing".to_string()),
            ..basic_init(Some("Existing Project"))
        };

        let result = cmd.execute().await;
        assert!(
            result.is_ok(),
            "Adding upstream to existing project failed: {:?}",
            result
        );

        // Verify upstream was configured
        let config2 = ConfigFile::load(&config_path).unwrap();
        assert!(config2.is_multi_workspace());
        assert_eq!(config2.workspace_prefix(), Some("existing"));
        assert!(config2.last_synced_commit().is_some());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_upstream_with_team_label() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let cmd = InitCommand {
            upstream: Some(central_url),
            workspace_prefix: Some("api".to_string()),
            team: Some("platform".to_string()),
            prefix: Some("API".to_string()),
            ..basic_init(Some("API Service"))
        };

        let result = cmd.execute().await;
        assert!(result.is_ok(), "Init with team failed: {:?}", result);

        // Verify team label was written to config
        let config_path = project_dir.join(".metis/config.toml");
        let config_content = fs::read_to_string(&config_path).unwrap();
        assert!(
            config_content.contains("team = \"platform\""),
            "Config should contain team label"
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_init_then_sync() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        // Init with upstream
        let cmd = InitCommand {
            upstream: Some(central_url),
            workspace_prefix: Some("api".to_string()),
            prefix: Some("API".to_string()),
            ..basic_init(Some("API Service"))
        };
        cmd.execute().await.unwrap();

        // Subsequent sync should work (using SyncCommand)
        use crate::commands::SyncCommand;
        let sync_cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = sync_cmd.execute().await;
        assert!(
            result.is_ok(),
            "Sync after init failed: {:?}",
            result
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
