use metis_core::{Application, Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "list_documents",
    description = "List documents in a project with optional filtering. Returns document details including unique short codes (format: PREFIX-TYPE-NNNN).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsTool {
    /// Path to the .metis folder to list documents from
    pub project_path: String,
}

impl ListDocumentsTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Validate metis workspace exists
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Metis workspace not found at {}", metis_dir.display()),
            )));
        }

        // Connect to database
        let db_path = metis_dir.join("metis.db");
        if !db_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Database not found at {}. Run initialize_project first.",
                    db_path.display()
                ),
            )));
        }

        // Sync before reading to catch external edits
        self.sync_workspace(metis_dir).await?;

        let db = Database::new(db_path.to_str().unwrap()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database connection failed: {}", e),
            ))
        })?;
        let mut repo = db.into_repository();

        // List all documents
        let documents = self.list_all_documents(&mut repo)?;

        // Use all documents (no filtering)
        let limited_documents = documents;

        // Convert to response format
        let document_list: Vec<serde_json::Value> = limited_documents
            .iter()
            .map(|doc| {
                let updated = chrono::DateTime::from_timestamp(doc.updated_at as i64, 0)
                    .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                    .unwrap_or_else(|| "Unknown".to_string());

                serde_json::json!({
                    "id": doc.id,
                    "title": doc.title,
                    "document_type": doc.document_type,
                    "phase": doc.phase,
                    "filepath": doc.filepath,
                    "updated_at": updated,
                    "archived": doc.archived,
                    "short_code": doc.short_code
                })
            })
            .collect();

        let response = serde_json::json!({
            "documents": document_list,
            "total_count": limited_documents.len()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

    fn list_all_documents(
        &self,
        repo: &mut metis_core::dal::database::repository::DocumentRepository,
    ) -> Result<Vec<metis_core::dal::database::models::Document>, CallToolError> {
        let mut all_docs = Vec::new();

        // Collect all document types
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            let mut docs = repo.find_by_type(doc_type).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to query {} documents: {}", doc_type, e),
                ))
            })?;
            all_docs.append(&mut docs);
        }

        // Sort by updated_at descending
        all_docs.sort_by(|a, b| {
            b.updated_at
                .partial_cmp(&a.updated_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_docs)
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
