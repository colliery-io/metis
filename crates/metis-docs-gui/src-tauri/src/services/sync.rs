use crate::{AppState, SyncStatus};
use metis_core::domain::configuration::ConfigFile;
use metis_core::{Application, Database};
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::{Emitter, State};

// ─── Local DB Sync (existing behavior, renamed for clarity) ──────────────────

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub imported: u32,
    pub updated: u32,
    pub deleted: u32,
    pub up_to_date: u32,
    pub errors: u32,
    pub messages: Vec<String>,
}

/// Sync local filesystem → database (existing behavior).
#[tauri::command]
pub async fn sync_project(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<SyncResult, String> {
    let current_project = {
        let app_state = state
            .lock()
            .map_err(|e| format!("Failed to get app state: {}", e))?;
        app_state.current_project.clone()
    };

    let project_path = current_project.ok_or("No project currently loaded")?;
    let metis_dir = project_path.join(".metis");
    if !metis_dir.exists() {
        return Err("Not a valid Metis project directory".to_string());
    }

    execute_local_sync(&metis_dir).await
}

// ─── Git-based Multi-Workspace Sync ──────────────────────────────────────────

/// Full sync result returned to the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkspaceSyncResult {
    /// Whether git sync was performed (false if no upstream configured)
    pub git_sync_performed: bool,
    /// Workspaces pulled from remote
    pub pulled_workspaces: Vec<String>,
    /// Number of files pushed to remote
    pub files_pushed: u32,
    /// Number of push retries (0 = first attempt succeeded)
    pub push_retries: u32,
    /// Whether the sync was a no-op (everything up to date)
    pub is_noop: bool,
    /// Human-readable summary message
    pub summary: String,
    /// Elapsed time in seconds
    pub elapsed_secs: f64,
    /// Local db sync results
    pub local_sync: SyncResult,
}

/// Trigger a full workspace sync: git sync (if upstream configured) + local db sync.
/// Debounced — returns error if sync is already in progress.
#[tauri::command]
pub async fn sync_workspace(
    state: State<'_, std::sync::Mutex<AppState>>,
    app_handle: tauri::AppHandle,
) -> Result<WorkspaceSyncResult, String> {
    let (project_path, is_already_syncing) = {
        let mut app_state = state
            .lock()
            .map_err(|e| format!("Failed to get app state: {}", e))?;
        let path = app_state
            .current_project
            .as_ref()
            .ok_or("No project loaded")?
            .clone();
        let already = app_state.sync_status.in_progress;
        if !already {
            app_state.sync_status.in_progress = true;
            app_state.sync_status.last_error = None;
        }
        (path, already)
    };

    // Debounce: only one sync at a time
    if is_already_syncing {
        return Err("Sync already in progress".to_string());
    }

    let metis_dir = project_path.join(".metis");
    if !metis_dir.exists() {
        set_sync_error(&state, "Not a valid Metis project directory");
        return Err("Not a valid Metis project directory".to_string());
    }

    // Run the sync and handle errors
    let result = execute_full_sync(&metis_dir).await;

    // Update sync status based on result
    match &result {
        Ok(ref sync_result) => {
            let mut app_state = state
                .lock()
                .map_err(|e| format!("Failed to update sync status: {}", e))?;
            app_state.sync_status.in_progress = false;
            app_state.sync_status.last_synced =
                Some(chrono::Utc::now().to_rfc3339());
            app_state.sync_status.last_error = None;
            app_state.sync_status.last_result_summary =
                Some(sync_result.summary.clone());
        }
        Err(ref err) => {
            set_sync_error(&state, err);
        }
    }

    // Emit event so frontend can refresh views
    let _ = app_handle.emit("sync-completed", ());

    result
}

/// Get the current sync status (for UI status indicator).
#[tauri::command]
pub async fn get_sync_status(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<SyncStatus, String> {
    let app_state = state
        .lock()
        .map_err(|e| format!("Failed to get app state: {}", e))?;
    Ok(app_state.sync_status.clone())
}

/// Check if upstream is configured (to show/hide sync UI elements).
#[tauri::command]
pub async fn is_upstream_configured(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<bool, String> {
    let project_path = {
        let app_state = state
            .lock()
            .map_err(|e| format!("Failed to get app state: {}", e))?;
        match app_state.current_project.as_ref() {
            Some(p) => p.clone(),
            None => return Ok(false),
        }
    };

    let config_path = project_path.join(".metis/config.toml");
    if !config_path.exists() {
        return Ok(false);
    }

    match ConfigFile::load(&config_path) {
        Ok(config) => Ok(config.is_multi_workspace()),
        Err(_) => Ok(false),
    }
}

// ─── Internal helpers ────────────────────────────────────────────────────────

/// Execute the full sync cycle: git sync (if configured) + local db sync.
async fn execute_full_sync(
    metis_dir: &std::path::Path,
) -> Result<WorkspaceSyncResult, String> {
    let start = Instant::now();

    // Check for upstream configuration
    let config_path = metis_dir.join("config.toml");
    let config = if config_path.exists() {
        ConfigFile::load(&config_path).ok()
    } else {
        None
    };

    let mut git_sync_performed = false;
    let mut pulled_workspaces = Vec::new();
    let mut files_pushed = 0u32;
    let mut push_retries = 0u32;
    let mut is_noop = true;

    // Git sync (if multi-workspace configured)
    if let Some(ref cfg) = config {
        if cfg.is_multi_workspace() {
            let git_result = execute_git_sync(metis_dir, cfg)?;
            git_sync_performed = true;
            is_noop = git_result.is_noop;

            if let Some(ref hydration) = git_result.hydration {
                pulled_workspaces = hydration.hydrated_workspaces.clone();
            }

            if git_result.pushed() {
                files_pushed = git_result.files_pushed() as u32;
            }

            push_retries = git_result.push_retries;

            // Update last_synced_commit in config
            if let Some(ref new_sha) = git_result.new_synced_commit {
                if let Ok(mut cfg) = ConfigFile::load(&config_path) {
                    if cfg.update_last_synced_commit(new_sha).is_ok() {
                        let _ = cfg.save(&config_path);
                    }
                }
            }
        }
    }

    // Always do local db sync
    let local_sync = execute_local_sync(metis_dir).await?;

    let elapsed = start.elapsed();

    // Build summary message
    let summary = build_summary(
        git_sync_performed,
        &pulled_workspaces,
        files_pushed,
        is_noop,
        &local_sync,
    );

    Ok(WorkspaceSyncResult {
        git_sync_performed,
        pulled_workspaces,
        files_pushed,
        push_retries,
        is_noop,
        summary,
        elapsed_secs: elapsed.as_secs_f64(),
        local_sync,
    })
}

/// Execute git-based multi-workspace sync.
fn execute_git_sync(
    metis_dir: &std::path::Path,
    config: &ConfigFile,
) -> Result<metis_sync::orchestration::SyncResult, String> {
    let upstream_url = config
        .upstream_url()
        .ok_or("No upstream URL configured")?;
    let workspace_prefix = config
        .workspace_prefix()
        .ok_or("No workspace prefix configured")?;

    // Flatten local workspace documents
    let flatten_result =
        metis_core::application::services::layout::flatten_workspace(metis_dir)
            .map_err(|e| format!("Failed to flatten workspace: {}", e))?;

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

    let sync_config = metis_sync::orchestration::SyncConfig {
        upstream_url: upstream_url.to_string(),
        workspace_prefix: workspace_prefix.to_string(),
        last_synced_commit: config.last_synced_commit().map(|s| s.to_string()),
    };

    let sync_options = metis_sync::orchestration::SyncOptions::new();

    metis_sync::orchestration::sync(&sync_config, metis_dir, &local_documents, &sync_options)
        .map_err(|e| format_sync_error(e, upstream_url))
}

/// Execute local filesystem → database sync.
async fn execute_local_sync(
    metis_dir: &std::path::Path,
) -> Result<SyncResult, String> {
    let db_path = metis_dir.join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    let app = Application::new(database);

    let sync_results = app
        .sync_directory(metis_dir)
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    let mut imported = 0;
    let mut updated = 0;
    let mut deleted = 0;
    let mut up_to_date = 0;
    let mut errors = 0;
    let mut messages = Vec::new();

    for result in &sync_results {
        match result {
            metis_core::application::services::synchronization::SyncResult::Imported {
                filepath,
            } => {
                messages.push(format!("Imported: {}", filepath));
                imported += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Updated {
                filepath,
            } => {
                messages.push(format!("Updated: {}", filepath));
                updated += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Deleted {
                filepath,
            } => {
                messages.push(format!("Deleted: {}", filepath));
                deleted += 1;
            }
            metis_core::application::services::synchronization::SyncResult::UpToDate { .. } => {
                up_to_date += 1;
            }
            metis_core::application::services::synchronization::SyncResult::NotFound {
                filepath,
            } => {
                messages.push(format!("Not found: {}", filepath));
            }
            metis_core::application::services::synchronization::SyncResult::Error {
                filepath,
                error,
            } => {
                messages.push(format!("Error syncing {}: {}", filepath, error));
                errors += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Moved { from, to } => {
                messages.push(format!("Moved: {} -> {}", from, to));
                updated += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Renumbered {
                filepath,
                old_short_code,
                new_short_code,
            } => {
                messages.push(format!(
                    "Renumbered: {} ({} -> {})",
                    filepath, old_short_code, new_short_code
                ));
                updated += 1;
            }
        }
    }

    Ok(SyncResult {
        imported,
        updated,
        deleted,
        up_to_date,
        errors,
        messages,
    })
}

/// Set sync error state and clear in_progress flag.
fn set_sync_error(state: &State<'_, std::sync::Mutex<AppState>>, error: &str) {
    if let Ok(mut app_state) = state.lock() {
        app_state.sync_status.in_progress = false;
        app_state.sync_status.last_error = Some(error.to_string());
    }
}

/// Build a human-readable summary of the sync result.
fn build_summary(
    git_sync_performed: bool,
    pulled_workspaces: &[String],
    files_pushed: u32,
    is_noop: bool,
    local_sync: &SyncResult,
) -> String {
    let mut parts = Vec::new();

    if git_sync_performed {
        if is_noop {
            parts.push("Up to date with remote".to_string());
        } else {
            if !pulled_workspaces.is_empty() {
                parts.push(format!(
                    "Pulled from {} workspace{}",
                    pulled_workspaces.len(),
                    if pulled_workspaces.len() == 1 { "" } else { "s" }
                ));
            }
            if files_pushed > 0 {
                parts.push(format!(
                    "Pushed {} file{}",
                    files_pushed,
                    if files_pushed == 1 { "" } else { "s" }
                ));
            }
        }
    }

    let local_changes = local_sync.imported + local_sync.updated + local_sync.deleted;
    if local_changes > 0 {
        parts.push(format!("Database: {} changes", local_changes));
    }

    if parts.is_empty() {
        "Everything up to date".to_string()
    } else {
        parts.join(". ")
    }
}

/// Convert SyncError to a user-friendly error message.
fn format_sync_error(error: metis_sync::SyncError, upstream_url: &str) -> String {
    match &error {
        metis_sync::SyncError::Auth { message } => {
            format!(
                "Authentication failed for {}. Check your SSH keys or credentials. ({})",
                upstream_url, message
            )
        }
        metis_sync::SyncError::FetchFailed { url, reason } => {
            format!(
                "Cannot reach {}. Check your network connection. ({})",
                url, reason
            )
        }
        metis_sync::SyncError::InvalidUrl { url } => {
            format!("Invalid upstream URL: {}", url)
        }
        metis_sync::SyncError::RetriesExhausted { max_retries } => {
            format!(
                "Another team synced first. Push failed after {} retries. Try again in a moment.",
                max_retries
            )
        }
        _ => format!("Sync failed: {}", error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_default() {
        let status = SyncStatus::default();
        assert!(!status.in_progress);
        assert!(status.last_synced.is_none());
        assert!(status.last_error.is_none());
        assert!(status.last_result_summary.is_none());
    }

    #[test]
    fn test_build_summary_noop() {
        let local = SyncResult {
            imported: 0,
            updated: 0,
            deleted: 0,
            up_to_date: 5,
            errors: 0,
            messages: vec![],
        };
        let summary = build_summary(true, &[], 0, true, &local);
        assert_eq!(summary, "Up to date with remote");
    }

    #[test]
    fn test_build_summary_pulled_and_pushed() {
        let local = SyncResult {
            imported: 2,
            updated: 0,
            deleted: 0,
            up_to_date: 3,
            errors: 0,
            messages: vec![],
        };
        let summary = build_summary(
            true,
            &["alpha".to_string(), "beta".to_string()],
            3,
            false,
            &local,
        );
        assert!(summary.contains("Pulled from 2 workspaces"));
        assert!(summary.contains("Pushed 3 files"));
        assert!(summary.contains("Database: 2 changes"));
    }

    #[test]
    fn test_build_summary_no_git_sync() {
        let local = SyncResult {
            imported: 1,
            updated: 2,
            deleted: 0,
            up_to_date: 5,
            errors: 0,
            messages: vec![],
        };
        let summary = build_summary(false, &[], 0, true, &local);
        assert_eq!(summary, "Database: 3 changes");
    }

    #[test]
    fn test_build_summary_everything_up_to_date() {
        let local = SyncResult {
            imported: 0,
            updated: 0,
            deleted: 0,
            up_to_date: 5,
            errors: 0,
            messages: vec![],
        };
        let summary = build_summary(false, &[], 0, true, &local);
        assert_eq!(summary, "Everything up to date");
    }

    #[test]
    fn test_build_summary_single_workspace() {
        let local = SyncResult {
            imported: 0,
            updated: 0,
            deleted: 0,
            up_to_date: 0,
            errors: 0,
            messages: vec![],
        };
        let summary = build_summary(true, &["api".to_string()], 1, false, &local);
        assert!(summary.contains("Pulled from 1 workspace"));
        assert!(summary.contains("Pushed 1 file"));
    }

    #[test]
    fn test_format_sync_error_auth() {
        let err = metis_sync::SyncError::Auth {
            message: "bad key".to_string(),
        };
        let msg = format_sync_error(err, "git@example.com:org/repo.git");
        assert!(msg.contains("Authentication failed"));
        assert!(msg.contains("SSH keys"));
    }

    #[test]
    fn test_format_sync_error_network() {
        let err = metis_sync::SyncError::FetchFailed {
            url: "git@example.com:org/repo.git".to_string(),
            reason: "timeout".to_string(),
        };
        let msg = format_sync_error(err, "git@example.com:org/repo.git");
        assert!(msg.contains("Cannot reach"));
        assert!(msg.contains("network connection"));
    }

    #[test]
    fn test_format_sync_error_retries() {
        let err = metis_sync::SyncError::RetriesExhausted { max_retries: 5 };
        let msg = format_sync_error(err, "git@example.com:org/repo.git");
        assert!(msg.contains("Another team synced first"));
        assert!(msg.contains("5 retries"));
    }
}
