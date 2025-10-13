use metis_core::{
    Application, Database,
    application::services::document::creation::{DocumentCreationService, DocumentCreationConfig},
    domain::documents::types::DocumentType,
};
use std::path::PathBuf;
use std::str::FromStr;
use tauri::State;
use serde::{Deserialize, Serialize};
use crate::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentInfo {
    pub id: String,
    pub title: String,
    pub document_type: String,
    pub short_code: String,
    pub filepath: String,
    pub phase: String,
    pub archived: bool,
    pub created_at: f64,
    pub updated_at: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocumentContent {
    pub id: String,
    pub title: String,
    pub content: String,
    pub frontmatter_json: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentRequest {
    pub document_type: String,
    pub title: String,
    pub parent_id: Option<String>,
    pub complexity: Option<String>,
    pub risk_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateDocumentResult {
    pub id: String,
    pub short_code: String,
    pub filepath: String,
}

fn find_strategy_short_code_for_initiative(
    metis_dir: &PathBuf,
    initiative_id: &str,
) -> Result<String, String> {
    let db_path = metis_dir.join("metis.db");
    let db = Database::new(db_path.to_str().unwrap()).map_err(|e| {
        format!("Database error: {}", e)
    })?;

    let mut repo = db.repository().map_err(|e| {
        format!("Repository error: {}", e)
    })?;

    // Find the initiative document in the database by short code
    let initiative = repo
        .find_by_short_code(initiative_id)
        .map_err(|e| {
            format!("Database lookup error: {}", e)
        })?
        .ok_or_else(|| {
            format!("Initiative '{}' not found in database", initiative_id)
        })?;

    // Get the strategy ID from the initiative, then find the strategy's short code
    let strategy_id = initiative.strategy_id.ok_or_else(|| {
        format!("Initiative '{}' has no parent strategy", initiative_id)
    })?;

    // Find the strategy by its short code (strategy_id now contains short codes)
    let strategy = repo
        .find_by_short_code(&strategy_id)
        .map_err(|e| {
            format!("Database lookup error: {}", e)
        })?
        .ok_or_else(|| {
            format!("Strategy '{}' not found in database", strategy_id)
        })?;

    Ok(strategy.short_code)
}

#[tauri::command]
pub async fn create_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    request: CreateDocumentRequest,
) -> Result<CreateDocumentResult, String> {
    let project_path = {
        let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state.current_project.as_ref()
            .ok_or("No project loaded")?
            .clone()
    };
    
    // Create the creation service for the metis directory
    let metis_dir = project_path.join(".metis");
    let creation_service = DocumentCreationService::new(&metis_dir);
    
    // Build the configuration
    let config = DocumentCreationConfig {
        title: request.title.clone(),
        description: None,
        parent_id: request.parent_id.as_ref().map(|id| id.clone().into()),
        tags: vec![],
        phase: None,
        complexity: request.complexity.as_ref().and_then(|c| c.parse().ok()),
        risk_level: request.risk_level.as_ref().and_then(|r| r.parse().ok()),
    };
    
    // Create document based on type
    let result = match request.document_type.as_str() {
        "vision" => creation_service.create_vision(config).await,
        "strategy" => creation_service.create_strategy(config).await,
        "adr" => creation_service.create_adr(config).await,
        "task" => {
            // Load flight configuration to determine proper task creation approach
            let db_path = metis_dir.join("metis.db");
            let database = Database::new(db_path.to_str().unwrap())
                .map_err(|e| format!("Failed to open database: {}", e))?;
            let mut config_repo = database.configuration_repository()
                .map_err(|e| format!("Failed to access configuration repository: {}", e))?;
            let flight_config = config_repo.get_flight_level_config()
                .map_err(|e| format!("Failed to load configuration: {}", e))?;
                
            if let Some(initiative_id) = request.parent_id.as_ref() {
                // Task with parent initiative
                let strategy_id = if flight_config.strategies_enabled {
                    // Full configuration: find actual strategy short code from initiative location
                    find_strategy_short_code_for_initiative(&metis_dir, initiative_id)?
                } else {
                    // Streamlined/Direct: use NULL as strategy placeholder
                    "NULL".to_string()
                };

                creation_service
                    .create_task_with_config(
                        config,
                        &strategy_id,
                        initiative_id,
                        &flight_config,
                    )
                    .await
            } else if flight_config.initiatives_enabled {
                // Initiatives enabled but no parent provided - this should be an error
                return Err(format!("Task requires a parent initiative ID in {} configuration. Provide an initiative_id or create as a backlog item.", flight_config.preset_name()));
            } else {
                // Direct configuration: create task without parents (use NULL for both)
                creation_service
                    .create_task_with_config(config, "NULL", "NULL", &flight_config)
                    .await
            }
        },
        "initiative" => {
            // Load flight configuration to determine proper initiative creation approach
            let db_path = metis_dir.join("metis.db");
            let database = Database::new(db_path.to_str().unwrap())
                .map_err(|e| format!("Failed to open database: {}", e))?;
            let mut config_repo = database.configuration_repository()
                .map_err(|e| format!("Failed to access configuration repository: {}", e))?;
            let flight_config = config_repo.get_flight_level_config()
                .map_err(|e| format!("Failed to load configuration: {}", e))?;

            // Determine parent strategy ID based on configuration
            let parent_strategy_id = if flight_config.strategies_enabled {
                // Full configuration: require explicit strategy parent
                request.parent_id.as_ref().ok_or_else(|| {
                    "Initiative requires a parent strategy short code in full configuration".to_string()
                })?.clone()
            } else {
                // Streamlined/Direct configuration: use NULL strategy placeholder
                "NULL".to_string()
            };

            creation_service
                .create_initiative_with_config(config, &parent_strategy_id, &flight_config)
                .await
        },
        _ => return Err(format!("Document type {} not supported yet", request.document_type)),
    }.map_err(|e| format!("Document creation error: {}", e))?;
    
    // Auto-sync after document creation to populate database
    let db_path = metis_dir.join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database for sync: {}", e))?;
    let app = Application::new(database);
    
    app.sync_directory(&metis_dir)
        .await
        .map_err(|e| format!("Failed to sync workspace after document creation: {}", e))?;
    
    Ok(CreateDocumentResult {
        id: result.document_id.to_string(),
        short_code: result.short_code,
        filepath: result.file_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
pub async fn list_documents(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<Vec<DocumentInfo>, String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    let documents = app.with_database(|service| -> Result<Vec<_>, metis_core::MetisError> {
        // Get all documents by type, same approach as TUI
        let mut all_docs = Vec::new();
        
        // Collect all document types using string literals like TUI does
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            if let Ok(mut docs) = service.find_by_type(DocumentType::from_str(doc_type).unwrap()) {
                all_docs.append(&mut docs);
            }
        }
        
        Ok(all_docs)
    }).map_err(|e| format!("Database error: {}", e))?;
    
    let doc_infos: Vec<DocumentInfo> = documents.into_iter()
        .filter(|doc| !doc.archived) // Filter out archived documents like TUI does
        .map(|doc| DocumentInfo {
            id: doc.id,
            title: doc.title,
            document_type: doc.document_type,
            short_code: doc.short_code,
            filepath: doc.filepath,
            phase: doc.phase,
            archived: doc.archived,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        })
        .collect();
    
    Ok(doc_infos)
}

#[tauri::command]
pub async fn read_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
) -> Result<DocumentContent, String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    let document = app.with_database(|service| {
        service.find_by_short_code(&short_code)
    }).map_err(|e| format!("Database error: {}", e))?
        .ok_or(format!("Document with short code {} not found", short_code))?;
    
    Ok(DocumentContent {
        id: document.id,
        title: document.title,
        content: document.content.unwrap_or_default(),
        frontmatter_json: document.frontmatter_json,
    })
}

#[tauri::command]
pub async fn search_documents(
    state: State<'_, std::sync::Mutex<AppState>>,
    query: String,
) -> Result<Vec<DocumentInfo>, String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    let documents = app.with_database(|service| {
        service.search_documents(&query)
    }).map_err(|e| format!("Search error: {}", e))?;
    
    let doc_infos: Vec<DocumentInfo> = documents.into_iter()
        .map(|doc| DocumentInfo {
            id: doc.id,
            title: doc.title,
            document_type: doc.document_type,
            short_code: doc.short_code,
            filepath: doc.filepath,
            phase: doc.phase,
            archived: doc.archived,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
        })
        .collect();
    
    Ok(doc_infos)
}

#[tauri::command]
pub async fn update_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
    content: String,
) -> Result<(), String> {
    let project_path = {
        let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state.current_project.as_ref()
            .ok_or("No project loaded")?
            .clone()
    };
    
    let metis_dir = project_path.join(".metis");
    let db_path = metis_dir.join("metis.db");
    
    // Resolve short code to file path
    let document_path = {
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| format!("Failed to open database: {}", e))?;
        
        let mut repo = database.repository()
            .map_err(|e| format!("Failed to get repository: {}", e))?;
        
        repo.resolve_short_code_to_filepath(&short_code)
            .map_err(|e| format!("Failed to resolve short code {}: {}", short_code, e))?
    };
    
    let full_document_path = metis_dir.join(&document_path);
    
    if !full_document_path.exists() {
        return Err(format!("Document not found for short code {}", short_code));
    }
    
    // Write the updated content to the file
    std::fs::write(&full_document_path, &content)
        .map_err(|e| format!("Failed to write document: {}", e))?;
    
    // Auto-sync after update to update database
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database for sync: {}", e))?;
    let app = Application::new(database);
    
    app.sync_directory(&metis_dir)
        .await
        .map_err(|e| format!("Failed to sync workspace: {}", e))?;
    
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParentOption {
    pub short_code: String,
    pub title: String,
    pub document_type: String,
    pub phase: String,
}

#[tauri::command]
pub async fn get_available_parents(
    state: State<'_, std::sync::Mutex<AppState>>,
    child_document_type: String,
) -> Result<Vec<ParentOption>, String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    // First check the flight level configuration using direct database access
    let mut config_repo = database.configuration_repository()
        .map_err(|e| format!("Failed to get config repository: {}", e))?;
    
    let config = config_repo.get_flight_level_config()
        .map_err(|e| format!("Failed to get config: {}", e))?;
    
    let mut app = Application::new(database);
    
    // Determine what parent type we need based on child type and configuration
    let parent_type = match child_document_type.as_str() {
        "task" => {
            // If initiatives are disabled (direct config), tasks don't need parents
            if !config.initiatives_enabled {
                return Ok(vec![]);
            }
            "initiative"
        },
        "initiative" => {
            // If strategies are disabled (streamlined/direct config), initiatives don't need parents
            if !config.strategies_enabled {
                return Ok(vec![]);
            }
            "strategy"
        }, 
        _ => return Ok(vec![]), // No parents needed for vision, strategy, adr
    };
    
    let documents = app.with_database(|service| -> Result<Vec<_>, metis_core::MetisError> {
        // Get documents of the parent type
        if let Ok(docs) = service.find_by_type(DocumentType::from_str(parent_type).unwrap()) {
            Ok(docs)
        } else {
            Ok(vec![])
        }
    }).map_err(|e| format!("Database error: {}", e))?;
    
    // Filter to only show non-archived parents
    let parent_options: Vec<ParentOption> = documents.into_iter()
        .filter(|doc| !doc.archived)
        .map(|doc| ParentOption {
            short_code: doc.short_code,
            title: doc.title,
            document_type: doc.document_type,
            phase: doc.phase,
        })
        .collect();
    
    Ok(parent_options)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::services::project::initialize_project;

    // Helper function to set up a project with state  
    async fn setup_test_project() -> (TempDir, AppState) {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();
        
        // Initialize project
        initialize_project(project_path.clone(), Some("TEST".to_string())).await.unwrap();
        
        // Create app state with loaded project
        let app_state = AppState {
            current_project: Some(temp_dir.path().to_path_buf()),
        };
        
        (temp_dir, app_state)
    }

    // Helper to test document creation directly without Tauri State
    async fn test_create_document_direct(
        metis_dir: &PathBuf,
        request: CreateDocumentRequest,
    ) -> Result<CreateDocumentResult, String> {
        let creation_service = DocumentCreationService::new(metis_dir);
        
        let config = DocumentCreationConfig {
            title: request.title.clone(),
            description: None,
            parent_id: request.parent_id.as_ref().map(|id| id.clone().into()),
            tags: vec![],
            phase: None,
            complexity: request.complexity.as_ref().and_then(|c| c.parse().ok()),
            risk_level: request.risk_level.as_ref().and_then(|r| r.parse().ok()),
        };
        
        let result = match request.document_type.as_str() {
            "vision" => creation_service.create_vision(config).await,
            "task" => {
                let db_path = metis_dir.join("metis.db");
                let database = Database::new(db_path.to_str().unwrap())
                    .map_err(|e| format!("Failed to open database: {}", e))?;
                let mut config_repo = database.configuration_repository()
                    .map_err(|e| format!("Failed to access configuration repository: {}", e))?;
                let flight_config = config_repo.get_flight_level_config()
                    .map_err(|e| format!("Failed to load configuration: {}", e))?;
                    
                creation_service
                    .create_task_with_config(config, "NULL", "NULL", &flight_config)
                    .await
            },
            _ => return Err(format!("Document type {} not supported in test", request.document_type)),
        }.map_err(|e| format!("Document creation error: {}", e))?;
        
        // Auto-sync after document creation
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| format!("Failed to open database for sync: {}", e))?;
        let app = Application::new(database);
        
        app.sync_directory(metis_dir)
            .await
            .map_err(|e| format!("Failed to sync workspace: {}", e))?;
        
        Ok(CreateDocumentResult {
            id: result.document_id.to_string(),
            short_code: result.short_code,
            filepath: result.file_path.to_string_lossy().to_string(),
        })
    }

    #[tokio::test]
    async fn test_create_adr_document() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();
        
        initialize_project(project_path.clone(), Some("TEST".to_string())).await.unwrap();
        let metis_dir = temp_dir.path().join(".metis");

        let creation_service = DocumentCreationService::new(&metis_dir);
        let config = DocumentCreationConfig {
            title: "Test ADR".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: None,
            risk_level: None,
        };
        
        let result = creation_service.create_adr(config).await;
        assert!(result.is_ok(), "ADR creation should succeed");
        
        let create_result = result.unwrap();
        assert!(create_result.short_code.starts_with("TEST-A-"));
    }

    #[tokio::test]
    async fn test_create_task_with_initiative_parent() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();
        
        initialize_project(project_path.clone(), Some("TEST".to_string())).await.unwrap();
        let metis_dir = temp_dir.path().join(".metis");

        // First create an initiative to be the parent
        let creation_service = DocumentCreationService::new(&metis_dir);
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap()).unwrap();
        let mut config_repo = database.configuration_repository().unwrap();
        let flight_config = config_repo.get_flight_level_config().unwrap();

        let initiative_config = DocumentCreationConfig {
            title: "Test Initiative".to_string(),
            description: None,
            parent_id: None,
            tags: vec![],
            phase: None,
            complexity: Some("m".parse().unwrap()),
            risk_level: None,
        };
        
        let initiative_result = creation_service
            .create_initiative_with_config(initiative_config, "NULL", &flight_config)
            .await
            .unwrap();

        // Sync the workspace
        let app = Application::new(Database::new(db_path.to_str().unwrap()).unwrap());
        app.sync_directory(&metis_dir).await.unwrap();

        // Now create a task with the initiative as parent
        let task_config = DocumentCreationConfig {
            title: "Test Task".to_string(),
            description: None,
            parent_id: Some(initiative_result.short_code.clone().into()),
            tags: vec![],
            phase: None,
            complexity: Some("s".parse().unwrap()),
            risk_level: None,
        };

        let task_result = creation_service
            .create_task_with_config(task_config, "NULL", &initiative_result.short_code, &flight_config)
            .await;

        assert!(task_result.is_ok(), "Task creation with initiative parent should succeed");
        
        let create_result = task_result.unwrap();
        assert!(create_result.short_code.starts_with("TEST-T-"));
    }
}