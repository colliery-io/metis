use std::path::PathBuf;

mod services;

use services::{
    archive_document, auto_install_cli, create_document, get_app_version, get_available_parents,
    get_cli_install_status, get_project_config, get_sync_status, initialize_project, install_cli,
    install_cli_elevated, is_upstream_configured, list_documents, load_project, read_document,
    search_documents, sync_project, sync_workspace, transition_phase, uninstall_cli,
    update_document,
};

/// Sync status tracked in application state.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SyncStatus {
    /// Whether a sync is currently in progress
    pub in_progress: bool,
    /// ISO 8601 timestamp of the last successful sync (None if never synced)
    pub last_synced: Option<String>,
    /// Last error message (None if last sync succeeded or no sync yet)
    pub last_error: Option<String>,
    /// Summary of last sync result
    pub last_result_summary: Option<String>,
}

impl Default for SyncStatus {
    fn default() -> Self {
        Self {
            in_progress: false,
            last_synced: None,
            last_error: None,
            last_result_summary: None,
        }
    }
}

// Application state
pub struct AppState {
    pub current_project: Option<PathBuf>,
    pub sync_status: SyncStatus,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Auto-install CLI on first launch or when update needed
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                auto_install_cli(app_handle).await;
            });

            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .manage(std::sync::Mutex::new(AppState {
            current_project: None,
            sync_status: SyncStatus::default(),
        }))
        .invoke_handler(tauri::generate_handler![
            initialize_project,
            load_project,
            list_documents,
            read_document,
            search_documents,
            get_available_parents,
            create_document,
            update_document,
            archive_document,
            transition_phase,
            get_project_config,
            sync_project,
            sync_workspace,
            get_sync_status,
            is_upstream_configured,
            get_app_version,
            // CLI installer commands
            get_cli_install_status,
            install_cli,
            install_cli_elevated,
            uninstall_cli
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
