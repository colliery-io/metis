use metis_core::{
    Application, Database,
    application::services::workspace::initialization::WorkspaceInitializationService,
};
use std::path::PathBuf;
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInfo {
    pub path: String,
    pub is_valid: bool,
    pub vision_exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InitializationResult {
    pub metis_dir: String,
    pub database_path: String,
    pub vision_path: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub strategies_enabled: bool,
    pub initiatives_enabled: bool,
    pub preset_name: String,
}

#[tauri::command]
pub async fn initialize_project(
    path: String,
    prefix: Option<String>,
) -> Result<InitializationResult, String> {
    let project_path = PathBuf::from(&path);
    
    let result = WorkspaceInitializationService::initialize_workspace_with_prefix(
        &project_path,
        "New Project",
        prefix.as_deref()
    )
    .await
    .map_err(|e| format!("Failed to initialize project: {}", e))?;
    
    // Auto-sync after project initialization to populate database
    let database = Database::new(result.database_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database for sync: {}", e))?;
    let app = Application::new(database);
    
    app.sync_directory(&result.metis_dir)
        .await
        .map_err(|e| format!("Failed to sync workspace after initialization: {}", e))?;
    
    Ok(InitializationResult {
        metis_dir: result.metis_dir.to_string_lossy().to_string(),
        database_path: result.database_path.to_string_lossy().to_string(),
        vision_path: result.vision_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn load_project(
    state: State<'_, std::sync::Mutex<AppState>>,
    path: String,
) -> Result<ProjectInfo, String> {
    let project_path = PathBuf::from(&path);
    let metis_dir = project_path.join(".metis");
    
    let is_valid = WorkspaceInitializationService::is_workspace(&project_path);
    let vision_exists = metis_dir.join("vision.md").exists();
    
    if is_valid {
        let mut app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state.current_project = Some(project_path);
    }
    
    Ok(ProjectInfo {
        path,
        is_valid,
        vision_exists,
    })
}

#[tauri::command]
pub async fn get_project_config(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<ProjectConfig, String> {
    let project_path = {
        let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state.current_project.as_ref()
            .ok_or("No project loaded")?
            .clone()
    };
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut config_repo = database.configuration_repository()
        .map_err(|e| format!("Failed to get config repository: {}", e))?;
    
    let config = config_repo.get_flight_level_config()
        .map_err(|e| format!("Failed to get config: {}", e))?;
    
    Ok(ProjectConfig {
        strategies_enabled: config.strategies_enabled,
        initiatives_enabled: config.initiatives_enabled,
        preset_name: config.preset_name().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_app_state() -> std::sync::Mutex<AppState> {
        std::sync::Mutex::new(AppState {
            current_project: None,
        })
    }

    #[tokio::test]
    async fn test_initialize_project_success() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();

        let result = initialize_project(project_path.clone(), Some("TEST".to_string())).await;

        assert!(result.is_ok(), "Project initialization should succeed");
        let init_result = result.unwrap();
        
        // Verify the metis directory was created
        assert!(temp_dir.path().join(".metis").exists());
        assert!(temp_dir.path().join(".metis").join("metis.db").exists());
        assert!(temp_dir.path().join(".metis").join("vision.md").exists());
        
        // Verify return values
        assert!(init_result.metis_dir.contains(".metis"));
        assert!(init_result.database_path.contains("metis.db"));
        assert!(init_result.vision_path.contains("vision.md"));
    }

    #[tokio::test]
    async fn test_initialize_project_with_default_prefix() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();

        let result = initialize_project(project_path, None).await;

        assert!(result.is_ok(), "Project initialization with default prefix should succeed");
    }
}