use crate::formatting::ToolOutput;
use metis_core::application::services::workspace::{ArchiveService, WorkspaceDetectionService};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "archive_document",
    description = "Archive a document and all its children using its short code (e.g., PROJ-V-0001). The document and its children will be moved to the archived folder and marked as archived.",
    idempotent_hint = true,
    destructive_hint = true,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ArchiveDocumentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
}

impl ArchiveDocumentTool {
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

        let mut db_service =
            metis_core::application::services::DatabaseService::new(db.into_repository());
        let archive_service = ArchiveService::new(metis_dir);

        // Check if document is already archived using short code
        match archive_service
            .is_document_archived_by_short_code(&self.short_code)
            .await
        {
            Ok(true) => {
                return Err(CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!("Document '{}' is already archived", self.short_code),
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

        // Archive the document using short code
        let archive_result = archive_service
            .archive_document_by_short_code(&self.short_code, &mut db_service)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to archive document: {}", e),
                ))
            })?;

        // Build formatted output
        let total = archive_result.total_archived;
        let children = if total > 1 {
            format!(" and {} children", total - 1)
        } else {
            String::new()
        };

        let mut output = ToolOutput::new()
            .header("Document Archived")
            .text(&format!("{}{} archived", self.short_code, children))
            .blank()
            .text("Archived:");

        // Build indented list
        let items: Vec<(bool, String)> = archive_result
            .archived_documents
            .iter()
            .map(|doc| {
                (true, format!("{} ({})", doc.document_id, format!("{:?}", doc.document_type).to_lowercase()))
            })
            .collect();

        let items_ref: Vec<(bool, &str)> = items.iter().map(|(b, s)| (*b, s.as_str())).collect();
        output = output.indented(items_ref);

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output.build(),
        )]))
    }
}
