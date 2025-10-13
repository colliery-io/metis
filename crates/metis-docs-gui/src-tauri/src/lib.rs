use std::path::PathBuf;

mod services;

use services::{
    initialize_project, load_project, get_project_config,
    create_document, update_document, list_documents, read_document, search_documents, get_available_parents,
    archive_document,
    transition_phase,
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
            get_project_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}