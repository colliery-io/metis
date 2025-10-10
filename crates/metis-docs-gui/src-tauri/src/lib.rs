use metis_core::{
    Application, Database, 
    application::services::{
        workspace::initialization::WorkspaceInitializationService,
        document::creation::{DocumentCreationService, DocumentCreationConfig},
    },
    domain::documents::types::{DocumentType},
};
use std::path::PathBuf;
use tauri::State;
use serde::{Deserialize, Serialize};

// Response types for Tauri commands
#[derive(Debug, Serialize, Deserialize)]
struct ProjectInfo {
    path: String,
    is_valid: bool,
    vision_exists: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentInfo {
    id: String,
    title: String,
    document_type: String,
    short_code: String,
    filepath: String,
    phase: String,
    archived: bool,
    created_at: f64,
    updated_at: f64,
}

#[derive(Debug, Serialize, Deserialize)]
struct DocumentContent {
    id: String,
    title: String,
    content: String,
    frontmatter_json: String,
}

// Application state
pub struct AppState {
    current_project: Option<PathBuf>,
}

// Tauri commands
#[derive(Debug, Serialize, Deserialize)]
struct InitializationResult {
    metis_dir: String,
    database_path: String,
    vision_path: String,
}

#[tauri::command]
async fn initialize_project(
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
    
    Ok(InitializationResult {
        metis_dir: result.metis_dir.to_string_lossy().to_string(),
        database_path: result.database_path.to_string_lossy().to_string(),
        vision_path: result.vision_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn load_project(
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
async fn list_documents(
    state: State<'_, std::sync::Mutex<AppState>>,
) -> Result<Vec<DocumentInfo>, String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    let documents = app.with_database(|service| {
        let mut all_docs = Vec::new();
        
        // Get all document types
        for doc_type in [
            DocumentType::Vision,
            DocumentType::Strategy, 
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ] {
            match service.find_by_type(doc_type) {
                Ok(docs) => all_docs.extend(docs),
                Err(e) => tracing::warn!("Failed to get {} documents: {}", doc_type, e),
            }
        }
        
        all_docs
    });
    
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
async fn read_document(
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
async fn search_documents(
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

#[derive(Debug, Serialize, Deserialize)]
struct CreateDocumentRequest {
    document_type: String,
    title: String,
    parent_id: Option<String>,
    complexity: Option<String>,
    risk_level: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CreateDocumentResult {
    id: String,
    short_code: String,
    filepath: String,
}

#[tauri::command]
async fn create_document(
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
        "adr" => creation_service.create_adr(config).await,
        "task" => {
            // For tasks, we need to check if it's a backlog item or has a parent initiative
            if request.parent_id.is_none() {
                creation_service.create_backlog_item(config).await
            } else {
                // For now, create as backlog item - we'll enhance this later
                creation_service.create_backlog_item(config).await
            }
        },
        _ => return Err(format!("Document type {} not supported yet", request.document_type)),
    }.map_err(|e| format!("Document creation error: {}", e))?;
    
    Ok(CreateDocumentResult {
        id: result.document_id.to_string(),
        short_code: result.short_code,
        filepath: result.file_path.to_string_lossy().to_string(),
    })
}

#[tauri::command]
async fn update_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
    content: String,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    // TODO: Implement proper document content update
    // For now, just return OK - we'll implement this properly later
    // app.with_database(|service| {
    //     service.update_document(&short_code, &content)
    // }).map_err(|e| format!("Document update error: {}", e))?;
    
    Ok(())
}

#[tauri::command]
async fn delete_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
) -> Result<(), String> {
    let app_state = state.lock().map_err(|e| format!("Failed to lock state: {}", e))?;
    
    let project_path = app_state.current_project.as_ref()
        .ok_or("No project loaded")?;
    
    let db_path = project_path.join(".metis").join("metis.db");
    let database = Database::new(db_path.to_str().unwrap())
        .map_err(|e| format!("Failed to open database: {}", e))?;
    
    let mut app = Application::new(database);
    
    // TODO: Implement proper document deletion
    // For now, just return OK - we'll implement this properly later
    // app.with_database(|service| {
    //     service.delete_document(&short_code)
    // }).map_err(|e| format!("Document deletion error: {}", e))?;
    
    Ok(())
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
            create_document,
            update_document,
            delete_document
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
