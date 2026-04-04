use crate::formatting::{error_result, ToolOutput};
use crate::read_tracker::DocumentReadTracker;
use crate::viewer::ViewerDispatcher;
use metis_core::application::services::{workspace::WorkspaceDetectionService, FilesystemService};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tracing::warn;

#[mcp_tool(
    name = "edit_document",
    description = "Edit document content using search-and-replace. Use short codes (e.g., PROJ-V-0001) to identify documents.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct EditDocumentTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Text to search for (will be replaced)
    pub search: String,
    /// Text to replace the search text with
    pub replace: String,
    /// Whether to replace all occurrences (default: false, only first match)
    pub replace_all: Option<bool>,
}

impl EditDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        self.call_tool_inner(None, None).await
    }

    #[allow(dead_code)]
    pub async fn call_tool_with_tracker(
        &self,
        tracker: Arc<DocumentReadTracker>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        self.call_tool_inner(Some(tracker), None).await
    }

    pub async fn call_tool_with_tracker_and_dispatcher(
        &self,
        tracker: Arc<DocumentReadTracker>,
        dispatcher: Arc<ViewerDispatcher>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        self.call_tool_inner(Some(tracker), Some(dispatcher)).await
    }

    async fn call_tool_inner(
        &self,
        tracker: Option<Arc<DocumentReadTracker>>,
        dispatcher: Option<Arc<ViewerDispatcher>>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
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

        // Resolve short code to file path
        let mut repo = db.repository().map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Repository error: {}", e),
            ))
        })?;

        let document_path = repo
            .resolve_short_code_to_filepath(&self.short_code)
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Resolution error: {}", e),
                ))
            })?;

        let full_document_path = metis_dir.join(&document_path);

        // Use FilesystemService for overlay-aware file access
        let fs_service = FilesystemService::new(metis_dir);

        if !fs_service.file_exists(&full_document_path) {
            return Ok(error_result(
                &format!("Document not found: {}", self.short_code),
                &format!(
                    "No document with identifier \"{}\" exists in this project.",
                    self.short_code
                ),
                Some("Use `list_documents` to see available documents."),
            ));
        }

        // Check read-before-edit guard (if tracker is available)
        if let Some(ref tracker) = tracker {
            if let Err(guard_err) = tracker.check_edit_allowed(&full_document_path) {
                return Ok(error_result(
                    &format!("Edit rejected for {}", self.short_code),
                    &guard_err.to_string(),
                    None,
                ));
            }
        }

        // Read the current document content through FilesystemService (overlay-aware)
        let content = fs_service.read_file(&full_document_path).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        // Perform the edit operation
        let (updated_content, replacements_made) = self.perform_edit(&content)?;

        if replacements_made == 0 {
            return Ok(error_result(
                &format!("No match found in {}", self.short_code),
                &format!("Search text not found:\n```\n{}\n```", self.search),
                Some("Use `read_document` to view current content."),
            ));
        }

        // Write the updated content back through FilesystemService (overlay-aware)
        fs_service
            .write_file(&full_document_path, &updated_content)
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Update the tracker after our own successful write
        if let Some(ref tracker) = tracker {
            tracker.record_edit(&full_document_path);
        }

        // Build concise output with diff
        let replace_all = self.replace_all.unwrap_or(false);
        let count_msg = if replace_all && replacements_made > 1 {
            format!(" ({} replacements)", replacements_made)
        } else {
            String::new()
        };

        let output = ToolOutput::new()
            .text(&format!("✓ {} updated{}", self.short_code, count_msg))
            .diff(&self.search, &self.replace)
            .build_result();

        // Proactive open: open the document in the viewer if not already open
        if let Some(dispatcher) = dispatcher {
            if !dispatcher.is_proactive_opening_suppressed() {
                if let Err(e) = dispatcher.open(&[full_document_path.to_path_buf()], None) {
                    warn!("Proactive open after edit failed: {}", e);
                }
            }
        }

        Ok(output)
    }

    fn perform_edit(&self, content: &str) -> Result<(String, usize), CallToolError> {
        let replace_all = self.replace_all.unwrap_or(false);

        if replace_all {
            // Replace all occurrences
            let replacements = content.matches(&self.search).count();
            let updated_content = content.replace(&self.search, &self.replace);
            Ok((updated_content, replacements))
        } else {
            // Replace only the first occurrence
            if let Some(pos) = content.find(&self.search) {
                let mut updated_content = String::new();
                updated_content.push_str(&content[..pos]);
                updated_content.push_str(&self.replace);
                updated_content.push_str(&content[pos + self.search.len()..]);
                Ok((updated_content, 1))
            } else {
                Ok((content.to_string(), 0))
            }
        }
    }
}
