use crate::formatting::ToolOutput;
use metis_core::{Application, application::services::workspace::WorkspaceDetectionService};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "search_documents",
    description = "Search documents by content with optional filtering. Returns matching documents with their unique short codes (format: PREFIX-TYPE-NNNN).",
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
    /// Sanitize search query to prevent FTS syntax errors
    fn sanitize_search_query(&self, query: &str) -> String {
        // If query is very short or contains problematic FTS characters, quote it
        let problematic_chars = ['#', '*', ':', '(', ')', '[', ']', '{', '}', '^', '~', '?'];

        if query.len() <= 2 || query.chars().any(|c| problematic_chars.contains(&c)) {
            // Wrap in double quotes and escape any internal quotes
            format!("\"{}\"", query.replace('"', "\"\""))
        } else {
            // For longer, safe queries, use as-is
            query.to_string()
        }
    }

    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let mut app = Application::new(db);

        // Sanitize query for FTS search - escape special characters and handle edge cases
        let sanitized_query = self.sanitize_search_query(&self.query);

        // Perform full-text search
        let results = app
            .with_database(|db_service| db_service.search_documents(&sanitized_query))
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Search failed: {}. Try using simpler search terms without special characters.", e),
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

        // Build formatted output
        let result_count = limited_results.len();

        let mut output = ToolOutput::new()
            .header(&format!("Search Results for \"{}\"", self.query))
            .text(&format!("Found {} match{}", result_count, if result_count == 1 { "" } else { "es" }));

        if result_count > 0 {
            let rows: Vec<Vec<String>> = limited_results
                .iter()
                .map(|doc| {
                    vec![
                        doc.short_code.clone(),
                        doc.title.clone(),
                        doc.document_type.clone(),
                    ]
                })
                .collect();

            output = output.table(&["Code", "Title", "Type"], rows);
        }

        Ok(output.build_result())
    }
}
