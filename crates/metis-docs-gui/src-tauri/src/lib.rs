use std::path::PathBuf;

mod services;

use services::{
    archive_document, create_document, get_available_parents, get_project_config,
    initialize_project, list_documents, load_project, read_document, search_documents,
    sync_project, transition_phase, update_document,
};

// Application state
pub struct AppState {
    current_project: Option<PathBuf>,
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
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .manage(std::sync::Mutex::new(AppState {
            current_project: None,
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
            sync_project
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
