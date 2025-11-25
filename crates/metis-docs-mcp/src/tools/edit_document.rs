use crate::formatting::{format_error, format_not_found, ToolOutput};
use metis_core::application::services::workspace::WorkspaceDetectionService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

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
    /// Path to the .metis folder containing the document
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

        if !full_document_path.exists() {
            let output = format_not_found(
                "Document",
                &self.short_code,
                Some("Use `list_documents` to see available documents."),
            );
            return Ok(CallToolResult::text_content(vec![TextContent::from(output)]));
        }

        // Read the current document content
        let content = fs::read_to_string(&full_document_path)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Perform the edit operation
        let (updated_content, replacements_made) = self.perform_edit(&content)?;

        if replacements_made == 0 {
            let output = format_error(
                &format!("No match found in {}", self.short_code),
                &format!("Search text not found:\n```\n{}\n```", self.search),
                Some("Use `read_document` to view current content."),
            );
            return Ok(CallToolResult::text_content(vec![TextContent::from(output)]));
        }

        // Write the updated content back to the file
        fs::write(&full_document_path, &updated_content)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Build formatted output with diff
        let replace_all = self.replace_all.unwrap_or(false);
        let mut output = ToolOutput::new()
            .header("Document Updated")
            .text(&format!("{} modified", self.short_code));

        if replace_all && replacements_made > 1 {
            output = output.subheader(&format!("Changes ({} replacements)", replacements_made));
        } else {
            output = output.subheader("Change");
        }

        output = output.diff(&self.search, &self.replace);

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output.build(),
        )]))
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
