use metis_core::{application::services::workspace::ArchiveService, Application, Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "archive_document",
    description = "Archive a document and all its children. The document and its children will be moved to the archived folder and marked as archived.",
    idempotent_hint = true,
    destructive_hint = true,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveDocumentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Document ID to archive
    pub document_id: String,
}

impl ArchiveDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Validate metis workspace exists
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Metis workspace not found at {}. Run initialize_project first.",
                    metis_dir.display()
                ),
            )));
        }

        // Create the archive service
        let archive_service = ArchiveService::new(metis_dir);

        // Check if document is already archived
        match archive_service
            .is_document_archived(&self.document_id)
            .await
        {
            Ok(true) => {
                return Err(CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Document '{}' is already archived", self.document_id),
                )));
            }
            Ok(false) => {
                // Continue with archiving
            }
            Err(e) => {
                return Err(CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to check archive status: {}", e),
                )));
            }
        }

        // Archive the document
        let archive_result = archive_service
            .archive_document(&self.document_id)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to archive document: {}", e),
                ))
            })?;

        // Auto-sync after archiving to update database
        self.sync_workspace(metis_dir).await?;

        let archived_docs: Vec<serde_json::Value> = archive_result
            .archived_documents
            .iter()
            .map(|doc| {
                serde_json::json!({
                    "document_id": doc.document_id,
                    "document_type": format!("{:?}", doc.document_type),
                    "original_path": doc.original_path.to_string_lossy(),
                    "archived_path": doc.archived_path.to_string_lossy()
                })
            })
            .collect();

        let response = serde_json::json!({
            "success": true,
            "document_id": self.document_id,
            "total_archived": archive_result.total_archived,
            "archived_documents": archived_docs,
            "auto_synced": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

    async fn sync_workspace(&self, metis_dir: &Path) -> Result<(), CallToolError> {
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open database for sync: {}", e),
            ))
        })?;
        let app = Application::new(database);

        app.sync_directory(metis_dir)
            .await
            .map_err(|e| CallToolError::new(e))?;

        Ok(())
    }
}
