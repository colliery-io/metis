use metis_core::{Application, Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "search_documents",
    description = "Search documents by content with optional filtering",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchDocumentsTool {
    /// Path to the .metis folder to search documents in
    pub project_path: String,
    /// Search query to match against document content
    pub query: String,
    /// Filter by document type (vision, strategy, initiative, task, adr)
    pub document_type: Option<String>,
    /// Maximum number of results to return
    pub limit: Option<u32>,
}

impl SearchDocumentsTool {
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

        let database = Database::new(db_path.to_str().unwrap()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open database: {}", e),
            ))
        })?;
        let mut app = Application::new(database);

        // Perform full-text search
        let results = app
            .with_database(|db_service| db_service.search_documents(&self.query))
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Search failed: {}", e),
                ))
            })?;

        // Apply document type filter if specified
        let filtered_results: Vec<_> = if let Some(doc_type) = &self.document_type {
            results
                .into_iter()
                .filter(|doc| doc.document_type == *doc_type)
                .collect()
        } else {
            results
        };

        // Apply limit if specified
        let limited_results: Vec<_> = if let Some(limit) = self.limit {
            filtered_results.into_iter().take(limit as usize).collect()
        } else {
            filtered_results
        };

        // Convert to response format
        let document_list: Vec<serde_json::Value> = limited_results
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
                    "archived": doc.archived
                })
            })
            .collect();

        let response = serde_json::json!({
            "documents": document_list,
            "total_count": limited_results.len(),
            "search_query": self.query,
            "filters": {
                "document_type": self.document_type,
                "limit": self.limit
            }
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }
}
