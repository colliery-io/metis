use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::domain::configuration::ConfigFile;
use metis_core::{Application, Database};
use std::time::Instant;

#[derive(Args)]
pub struct SyncCommand {
    /// Show what would be synced without actually pushing
    #[arg(long)]
    pub dry_run: bool,

    /// Suppress output (for git hook usage)
    #[arg(short, long)]
    pub quiet: bool,

    /// Skip freshness check and force sync
    #[arg(short, long)]
    pub force: bool,
}

impl SyncCommand {
    pub async fn execute(&self) -> Result<()> {
        // Check if we're in a workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            if !self.quiet {
                eprintln!("Not in a Metis workspace. Run 'metis init' to create one.");
            }
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }

        let metis_dir = metis_dir.unwrap();

        // Load config to check for upstream sync
        let config_path = metis_dir.join("config.toml");
        let config = if config_path.exists() {
            ConfigFile::load(&config_path).ok()
        } else {
            None
        };

        // If multi-workspace sync is configured, do git sync first
        if let Some(ref cfg) = config {
            if cfg.is_multi_workspace() {
                self.execute_git_sync(&metis_dir, cfg)?;
            }
        }

        // Always do local db sync (filesystem → database)
        self.execute_local_sync(&metis_dir).await?;

        Ok(())
    }

    /// Execute git-based multi-workspace sync (fetch → hydrate → dehydrate → push).
    fn execute_git_sync(
        &self,
        metis_dir: &std::path::Path,
        config: &ConfigFile,
    ) -> Result<()> {
        let upstream_url = config
            .upstream_url()
            .ok_or_else(|| anyhow::anyhow!("No upstream URL configured"))?;
        let workspace_prefix = config
            .workspace_prefix()
            .ok_or_else(|| anyhow::anyhow!("No workspace prefix configured"))?;

        let start = Instant::now();

        if !self.quiet {
            println!("Syncing with {}...", upstream_url);
        }

        // Flatten local workspace documents for pushing
        let flatten_result =
            metis_core::application::services::layout::flatten_workspace(metis_dir)
                .map_err(|e| anyhow::anyhow!("Failed to flatten workspace: {}", e))?;

        if !flatten_result.errors.is_empty() && !self.quiet {
            for (path, err) in &flatten_result.errors {
                eprintln!("  Warning: {} — {}", path.display(), err);
            }
        }

        // Convert FlatDocument → FlatDoc for metis-sync
        let local_documents: Vec<metis_sync::dehydration::FlatDoc> = flatten_result
            .documents
            .iter()
            .map(|d| metis_sync::dehydration::FlatDoc {
                short_code: d.short_code.clone(),
                filename: d.filename.clone(),
                content: d.content.clone(),
            })
            .collect();

        // Build sync config
        let sync_config = metis_sync::orchestration::SyncConfig {
            upstream_url: upstream_url.to_string(),
            workspace_prefix: workspace_prefix.to_string(),
            last_synced_commit: config.last_synced_commit().map(|s| s.to_string()),
        };

        let sync_options = metis_sync::orchestration::SyncOptions {
            force: self.force,
            ..metis_sync::orchestration::SyncOptions::new()
        };

        if self.dry_run {
            return self.execute_dry_run(metis_dir, &sync_config, &local_documents);
        }

        // Run the full sync cycle
        let result = metis_sync::orchestration::sync(
            &sync_config,
            metis_dir,
            &local_documents,
            &sync_options,
        )
        .map_err(|e| self.format_sync_error(e, upstream_url))?;

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

        // Output results
        if !self.quiet {
            self.print_git_sync_results(&result, elapsed);
        }

        // Print warnings
        if !self.quiet {
            for warning in &result.warnings {
                eprintln!("  Warning: {}", warning);
            }
        }

        Ok(())
    }

    /// Dry-run: fetch only (no push), show what would happen.
    fn execute_dry_run(
        &self,
        metis_dir: &std::path::Path,
        config: &metis_sync::orchestration::SyncConfig,
        local_documents: &[metis_sync::dehydration::FlatDoc],
    ) -> Result<()> {
        use metis_sync::SyncContext;

        if !self.quiet {
            println!("  [dry-run] Checking remote state...");
        }

        let mut ctx = SyncContext::new(&config.upstream_url, &config.workspace_prefix)
            .map_err(|e| self.format_sync_error(e, &config.upstream_url))?;

        let fetched_head = ctx
            .fetch()
            .map_err(|e| self.format_sync_error(e, &config.upstream_url))?;

        // Show what would be pulled
        if let Some(_head) = fetched_head {
            match metis_sync::hydration::hydrate(&ctx, metis_dir, &config.workspace_prefix) {
                Ok(hydration) => {
                    if !self.quiet {
                        if hydration.hydrated_workspaces.is_empty() {
                            println!("  [dry-run] Pull: no remote workspaces to hydrate");
                        } else {
                            println!("  [dry-run] Would pull:");
                            for ws in &hydration.hydrated_workspaces {
                                // Count files in that workspace dir
                                let ws_dir = metis_dir.join(ws);
                                let count = std::fs::read_dir(&ws_dir)
                                    .map(|entries| {
                                        entries
                                            .filter_map(|e| e.ok())
                                            .filter(|e| {
                                                e.path()
                                                    .extension()
                                                    .is_some_and(|ext| ext == "md")
                                            })
                                            .count()
                                    })
                                    .unwrap_or(0);
                                println!("    {}/ ({} docs)", ws, count);
                            }
                        }
                    }
                }
                Err(e) => {
                    if !self.quiet {
                        eprintln!("  [dry-run] Pull check failed: {}", e);
                    }
                }
            }
        } else if !self.quiet {
            println!("  [dry-run] Remote is empty, nothing to pull");
        }

        // Show what would be pushed
        if !self.quiet {
            if local_documents.is_empty() {
                println!("  [dry-run] Push: no local documents to push");
            } else {
                println!(
                    "  [dry-run] Would push: {}/ ({} docs)",
                    config.workspace_prefix,
                    local_documents.len()
                );
            }
        }

        Ok(())
    }

    /// Print git sync results in a human-readable format.
    fn print_git_sync_results(
        &self,
        result: &metis_sync::orchestration::SyncResult,
        elapsed: std::time::Duration,
    ) {
        if result.is_noop {
            println!("  Already up to date.");
        } else {
            // Pulled workspaces
            if let Some(ref hydration) = result.hydration {
                if !hydration.hydrated_workspaces.is_empty() {
                    let workspace_summary: Vec<String> = hydration
                        .hydrated_workspaces
                        .iter()
                        .map(|ws| format!("{}/", ws))
                        .collect();
                    println!(
                        "  Pulled: {} ({} files)",
                        workspace_summary.join(", "),
                        hydration.files_written
                    );
                }
                if !hydration.folders_removed.is_empty() {
                    println!(
                        "  Removed workspaces: {}",
                        hydration.folders_removed.join(", ")
                    );
                }
            }

            // Pushed
            if result.pushed() {
                println!("  Pushed: {} files", result.files_pushed());
            }

            // Retries
            if result.push_retries > 0 {
                println!(
                    "  Push retries: {} (resolved automatically)",
                    result.push_retries
                );
            }
        }

        println!("  Sync complete in {:.1}s", elapsed.as_secs_f64());
    }

    /// Convert SyncError to a user-friendly anyhow::Error.
    fn format_sync_error(
        &self,
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
                    "Could not reach {}: {}\nCheck your network connection and URL.",
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
            metis_sync::SyncError::RetriesExhausted { max_retries } => {
                anyhow::anyhow!(
                    "Push failed after {} retries — remote HEAD keeps moving.\nAnother workspace may be syncing. Try again in a moment.",
                    max_retries
                )
            }
            _ => anyhow::anyhow!("Sync failed: {}", error),
        }
    }

    /// Execute local filesystem → database sync.
    async fn execute_local_sync(&self, metis_dir: &std::path::Path) -> Result<()> {
        if !self.quiet {
            println!("Syncing local database...");
        }

        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;
        let app = Application::new(database);

        let sync_results = app.sync_directory(metis_dir).await?;

        if self.quiet {
            // In quiet mode, only report errors to stderr
            let errors: Vec<_> = sync_results
                .iter()
                .filter_map(|r| {
                    if let metis_core::application::services::synchronization::SyncResult::Error {
                        filepath,
                        error,
                    } = r
                    {
                        Some(format!("{}: {}", filepath, error))
                    } else {
                        None
                    }
                })
                .collect();

            if !errors.is_empty() {
                for err in &errors {
                    eprintln!("Error: {}", err);
                }
                anyhow::bail!("Local sync completed with {} errors", errors.len());
            }
            return Ok(());
        }

        // Verbose output for non-quiet mode
        let mut imported = 0;
        let mut updated = 0;
        let mut deleted = 0;
        let mut up_to_date = 0;
        let mut errors = 0;

        for result in &sync_results {
            match result {
                metis_core::application::services::synchronization::SyncResult::Imported {
                    filepath,
                } => {
                    println!("  [+] Imported: {}", filepath);
                    imported += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Updated {
                    filepath,
                } => {
                    println!("  [~] Updated: {}", filepath);
                    updated += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Deleted {
                    filepath,
                } => {
                    println!("  [-] Deleted: {}", filepath);
                    deleted += 1;
                }
                metis_core::application::services::synchronization::SyncResult::UpToDate {
                    ..
                } => {
                    up_to_date += 1;
                }
                metis_core::application::services::synchronization::SyncResult::NotFound {
                    filepath,
                } => {
                    println!("  [?] Not found: {}", filepath);
                }
                metis_core::application::services::synchronization::SyncResult::Error {
                    filepath,
                    error,
                } => {
                    eprintln!("  [!] Error syncing {}: {}", filepath, error);
                    errors += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Moved {
                    from,
                    to,
                } => {
                    println!("  [>] Moved: {} -> {}", from, to);
                    updated += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Renumbered {
                    filepath,
                    old_short_code,
                    new_short_code,
                } => {
                    println!(
                        "  [!] Renumbered: {} ({} -> {})",
                        filepath, old_short_code, new_short_code
                    );
                    updated += 1;
                }
            }
        }

        if imported > 0 || updated > 0 || deleted > 0 {
            println!(
                "  Database: {} imported, {} updated, {} deleted, {} up to date",
                imported, updated, deleted, up_to_date
            );
        } else {
            println!("  Database: {} documents up to date", up_to_date);
        }

        if errors > 0 {
            anyhow::bail!("Local sync completed with {} errors", errors);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_sync_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_command_with_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace first
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Create a test document file
        let test_strategy = temp_dir.path().join(".metis/strategies/test-strategy.md");
        fs::create_dir_all(test_strategy.parent().unwrap()).unwrap();
        fs::write(&test_strategy, "---\nid: test-strategy\nlevel: strategy\ntitle: \"Test Strategy\"\ncreated_at: 2025-01-01T00:00:00Z\nupdated_at: 2025-01-01T00:00:00Z\nparent: test-vision\nblocked_by: []\narchived: false\ntags:\n  - \"#strategy\"\n  - \"#phase/shaping\"\nexit_criteria_met: false\nsuccess_metrics: []\nrisk_level: medium\nstakeholders: []\n---\n\n# Test Strategy\n").unwrap();

        // Run sync command
        let cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        println!("Sync result: {:?}", result);

        // Check that the strategy file still exists
        assert!(test_strategy.exists());

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_command_no_upstream_configured() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create a minimal workspace (no upstream)
        let init_cmd = InitCommand {
            name: Some("Local Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Sync should succeed — just does local db sync
        let cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_command_quiet_mode() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("Quiet Test".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Quiet sync should succeed silently
        let cmd = SyncCommand {
            dry_run: false,
            quiet: true,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_command_dry_run_no_upstream() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        let init_cmd = InitCommand {
            name: Some("DryRun Test".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Dry run without upstream → just does local sync (dry_run only affects git sync)
        let cmd = SyncCommand {
            dry_run: true,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_with_upstream_auth_failure() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Auth Test".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: Some("AUTH".to_string()),
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Configure upstream with a bad URL
        let config_path = temp_dir.path().join(".metis/config.toml");
        let mut cfg = ConfigFile::load(&config_path).unwrap();
        cfg.set_workspace("auth".to_string(), None).unwrap();
        cfg.set_sync("git@nonexistent.invalid:org/repo.git".to_string())
            .unwrap();
        cfg.save(&config_path).unwrap();

        // Sync should fail with a clear error
        let cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_err());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    /// Helper: write upstream sync config to config.toml manually
    /// (bypasses URL validation which rejects file:// URLs used in tests)
    fn write_upstream_config(config_path: &std::path::Path, url: &str, ws_prefix: &str) {
        let existing = fs::read_to_string(config_path).unwrap();
        let with_sync = format!(
            "{}\n[workspace]\nprefix = \"{}\"\n\n[sync]\nupstream_url = \"{}\"\n",
            existing, ws_prefix, url
        );
        fs::write(config_path, with_sync).unwrap();
    }

    #[tokio::test]
    async fn test_sync_with_file_url_upstream() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Create a bare git repo as "central"
        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        // Create workspace in a subdirectory
        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let init_cmd = InitCommand {
            name: Some("E2E Sync Test".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: Some("SYNC".to_string()),
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        // Write upstream config manually (file:// URLs aren't accepted by set_sync)
        let config_path = project_dir.join(".metis/config.toml");
        write_upstream_config(&config_path, &central_url, "sync");

        // First sync — should push local documents to central
        let cmd = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok(), "First sync failed: {:?}", result);

        // Verify config was updated with last_synced_commit
        let cfg2 = ConfigFile::load(&config_path).unwrap();
        assert!(
            cfg2.last_synced_commit().is_some(),
            "last_synced_commit should be set after first sync"
        );

        // Second sync — should be a no-op
        let cmd2 = SyncCommand {
            dry_run: false,
            quiet: false,
            force: false,
        };
        let result2 = cmd2.execute().await;
        assert!(result2.is_ok(), "Second sync failed: {:?}", result2);

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_dry_run_with_upstream() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Create central repo
        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let init_cmd = InitCommand {
            name: Some("DryRun E2E".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: Some("DRY".to_string()),
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        let config_path = project_dir.join(".metis/config.toml");
        write_upstream_config(&config_path, &central_url, "dry");

        // Dry-run should not push anything
        let cmd = SyncCommand {
            dry_run: true,
            quiet: false,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok(), "Dry-run failed: {:?}", result);

        // Verify nothing was pushed (last_synced_commit should still be None)
        let cfg2 = ConfigFile::load(&config_path).unwrap();
        assert!(
            cfg2.last_synced_commit().is_none(),
            "Dry-run should not update last_synced_commit"
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_sync_quiet_with_upstream() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        let central_dir = temp_dir.path().join("central");
        git2::Repository::init_bare(&central_dir).unwrap();
        let central_url = format!("file://{}", central_dir.display());

        let project_dir = temp_dir.path().join("project");
        std::fs::create_dir_all(&project_dir).unwrap();
        std::env::set_current_dir(&project_dir).unwrap();

        let init_cmd = InitCommand {
            name: Some("Quiet E2E".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: Some("QT".to_string()),
            upstream: None,
            workspace_prefix: None,
            team: None,
        };
        init_cmd.execute().await.unwrap();

        let config_path = project_dir.join(".metis/config.toml");
        write_upstream_config(&config_path, &central_url, "qt");

        // Quiet sync with upstream should work silently
        let cmd = SyncCommand {
            dry_run: false,
            quiet: true,
            force: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_ok(), "Quiet sync failed: {:?}", result);

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
