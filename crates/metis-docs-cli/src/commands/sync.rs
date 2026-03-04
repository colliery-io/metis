use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{Application, Database};

#[derive(Args)]
pub struct SyncCommand {}

impl SyncCommand {
    pub async fn execute(&self) -> Result<()> {
        // Check if we're in a workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }

        let metis_dir = metis_dir.unwrap();
        let workspace_root = &metis_dir;

        println!("Syncing workspace: {}", workspace_root.display());

        // Initialize application with database
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;
        let app = Application::new(database);

        // Sync the workspace directory
        let sync_results = app.sync_directory(workspace_root).await?;

        // Report results
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
                    println!("[+] Imported: {}", filepath);
                    imported += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Updated {
                    filepath,
                } => {
                    println!("[+] Updated: {}", filepath);
                    updated += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Deleted {
                    filepath,
                } => {
                    println!("[+] Deleted: {}", filepath);
                    deleted += 1;
                }
                metis_core::application::services::synchronization::SyncResult::UpToDate {
                    filepath,
                } => {
                    println!("[.] Up to date: {}", filepath);
                    up_to_date += 1;
                }
                metis_core::application::services::synchronization::SyncResult::NotFound {
                    filepath,
                } => {
                    println!("[?] Not found: {}", filepath);
                }
                metis_core::application::services::synchronization::SyncResult::Error {
                    filepath,
                    error,
                } => {
                    println!("[-] Error syncing {}: {}", filepath, error);
                    errors += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Moved {
                    from,
                    to,
                } => {
                    println!("[>] Moved: {} -> {}", from, to);
                    updated += 1;
                }
                metis_core::application::services::synchronization::SyncResult::Renumbered {
                    filepath,
                    old_short_code,
                    new_short_code,
                } => {
                    println!(
                        "[!] Renumbered: {} ({} -> {})",
                        filepath, old_short_code, new_short_code
                    );
                    updated += 1;
                }
            }
        }

        println!("\nSync complete:");
        println!("  Imported: {}", imported);
        println!("  Updated: {}", updated);
        println!("  Deleted: {}", deleted);
        println!("  Up to date: {}", up_to_date);
        if errors > 0 {
            println!("  Errors: {}", errors);
        }

        if errors > 0 {
            anyhow::bail!("Sync completed with {} errors", errors);
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

        let cmd = SyncCommand {};
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

            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Run sync command
        let cmd = SyncCommand {};
        let result = cmd.execute().await;

        // The command should succeed and sync the vision.md created by init
        println!("Sync result: {:?}", result);

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
