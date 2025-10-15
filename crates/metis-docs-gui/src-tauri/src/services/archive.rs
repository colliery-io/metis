use crate::AppState;
use metis_core::{
    application::services::{workspace::ArchiveService, DatabaseService},
    Application, Database,
};
use serde::{Deserialize, Serialize};
use tauri::State;

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchiveResult {
    pub total_archived: usize,
    pub archived_documents: Vec<ArchivedDocument>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchivedDocument {
    pub document_id: String,
    pub document_type: String,
    pub original_path: String,
    pub archived_path: String,
}

#[tauri::command]
pub async fn archive_document(
    state: State<'_, std::sync::Mutex<AppState>>,
    short_code: String,
) -> Result<ArchiveResult, String> {
    let project_path = {
        let app_state = state
            .lock()
            .map_err(|e| format!("Failed to lock state: {}", e))?;
        app_state
            .current_project
            .as_ref()
            .ok_or("No project loaded")?
            .clone()
    };

    let metis_dir = project_path.join(".metis");

    // Create the archive service with database optimization
    let db = Database::new(&metis_dir.join("metis.db").to_string_lossy())
        .map_err(|e| format!("Database initialization failed: {}", e))?;
    let mut db_service = DatabaseService::new(db.into_repository());
    let archive_service = ArchiveService::new(&metis_dir);

    // Check if document is already archived using short code
    match archive_service
        .is_document_archived_by_short_code(&short_code)
        .await
    {
        Ok(true) => {
            return Err(format!("Document '{}' is already archived", short_code));
        }
        Ok(false) => {
            // Continue with archiving
        }
        Err(e) => {
            return Err(format!("Failed to check archive status: {}", e));
        }
    }

    // Archive the document using short code
    let archive_result = archive_service
        .archive_document_by_short_code(&short_code, &mut db_service)
        .await
        .map_err(|e| format!("Failed to archive document: {}", e))?;

    // Auto-sync after archiving to update database
    let database = Database::new(metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| format!("Failed to open database for sync: {}", e))?;
    let app = Application::new(database);

    app.sync_directory(&metis_dir)
        .await
        .map_err(|e| format!("Failed to sync workspace: {}", e))?;

    let archived_docs: Vec<ArchivedDocument> = archive_result
        .archived_documents
        .iter()
        .map(|doc| ArchivedDocument {
            document_id: doc.document_id.clone(),
            document_type: format!("{:?}", doc.document_type),
            original_path: doc.original_path.to_string_lossy().to_string(),
            archived_path: doc.archived_path.to_string_lossy().to_string(),
        })
        .collect();

    Ok(ArchiveResult {
        total_archived: archive_result.total_archived,
        archived_documents: archived_docs,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::project::initialize_project;
    use tempfile::TempDir;


    #[tokio::test]
    async fn test_archive_service_creation() {
        let temp_dir = TempDir::new().unwrap();
        let project_path = temp_dir.path().to_string_lossy().to_string();

        initialize_project(project_path.clone(), Some("TEST".to_string()))
            .await
            .unwrap();
        let metis_dir = temp_dir.path().join(".metis");

        // Test that the archive service can be created
        let archive_service = ArchiveService::new(&metis_dir);

        // Test checking archive status of non-existent document
        let result = archive_service
            .is_document_archived_by_short_code("TEST-V-9999")
            .await;
        assert!(result.is_ok());
    }
}
