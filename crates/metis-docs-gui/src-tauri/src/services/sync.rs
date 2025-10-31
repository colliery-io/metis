use crate::AppState;
use metis_core::{Application, Database};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub imported: u32,
    pub updated: u32,
    pub deleted: u32,
    pub up_to_date: u32,
    pub errors: u32,
    pub messages: Vec<String>,
}

#[tauri::command]
pub async fn sync_project(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<SyncResult, String> {
    let current_project = {
        let app_state = state.lock().map_err(|e| format!("Failed to get app state: {}", e))?;
        app_state.current_project.clone()
    };

    let project_path = current_project.ok_or("No project currently loaded")?;
    
    // Find the .metis directory
    let metis_dir = project_path.join(".metis");
    if !metis_dir.exists() {
        return Err("Not a valid Metis project directory".to_string());
    }

    // Initialize application with database
    let db_path = metis_dir.join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to initialize database: {}", e))?;
    let app = Application::new(database);

    // Sync the workspace directory
    let sync_results = app
        .sync_directory(&metis_dir)
        .await
        .map_err(|e| format!("Sync failed: {}", e))?;

    // Process results
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
                messages.push(format!("✓ Imported: {}", filepath));
                imported += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Updated {
                filepath,
            } => {
                messages.push(format!("✓ Updated: {}", filepath));
                updated += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Deleted {
                filepath,
            } => {
                messages.push(format!("✓ Deleted: {}", filepath));
                deleted += 1;
            }
            metis_core::application::services::synchronization::SyncResult::UpToDate {
                filepath,
            } => {
                messages.push(format!("• Up to date: {}", filepath));
                up_to_date += 1;
            }
            metis_core::application::services::synchronization::SyncResult::NotFound {
                filepath,
            } => {
                messages.push(format!("? Not found: {}", filepath));
            }
            metis_core::application::services::synchronization::SyncResult::Error {
                filepath,
                error,
            } => {
                messages.push(format!("✗ Error syncing {}: {}", filepath, error));
                errors += 1;
            }
            metis_core::application::services::synchronization::SyncResult::Moved {
                from,
                to,
            } => {
                messages.push(format!("↻ Moved: {} → {}", from, to));
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