use crate::formatting::{error_result, ToolOutput};
use crate::viewer::ViewerDispatcher;
use metis_core::application::services::{workspace::WorkspaceDetectionService, FilesystemService};
use metis_core::domain::configuration::ViewerBackend;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[mcp_tool(
    name = "open_document",
    description = "Open a Metis document in an external viewer (VSCode, system editor, or Metis GUI). Use this to review documents in a proper editor. Supports opening an initiative with all its child tasks via include_children.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = true,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct OpenDocumentTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
    /// Open child tasks alongside the parent document (default: false)
    #[serde(default)]
    pub include_children: bool,
    /// Override the default viewer for this call ("sys_editor", "code", or "gui")
    pub viewer: Option<String>,
}

impl OpenDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        // This path is for when no dispatcher is available (shouldn't happen in practice)
        Ok(error_result(
            "Viewer not configured",
            "No viewer dispatcher available. The open_document tool requires viewer configuration.",
            Some("Configure [viewer] section in config.toml"),
        ))
    }

    pub async fn call_tool_with_dispatcher(
        &self,
        dispatcher: Arc<ViewerDispatcher>,
    ) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace
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

        // Collect paths to open
        let mut paths: Vec<PathBuf> = vec![full_document_path];

        // If include_children, gather child document paths
        if self.include_children {
            let doc_id = repo
                .resolve_short_code_to_document_id(&self.short_code)
                .map_err(|e| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("ID resolution error: {}", e),
                    ))
                })?;

            let children = repo.find_children(&doc_id).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Children lookup error: {}", e),
                ))
            })?;

            for child in children {
                let child_path = metis_dir.join(&child.filepath);
                if fs_service.file_exists(&child_path) {
                    paths.push(child_path);
                }
            }
        }

        // Parse viewer override
        let viewer_override = self.viewer.as_ref().and_then(|v| match v.as_str() {
            "sys_editor" => Some(ViewerBackend::SysEditor),
            "code" => Some(ViewerBackend::Code),
            "gui" => Some(ViewerBackend::Gui),
            _ => None,
        });

        // Dispatch to viewer
        match dispatcher.open(&paths, viewer_override.as_ref()) {
            Ok(result) => {
                let mut output = ToolOutput::new();

                if result.opened.is_empty() && !result.skipped.is_empty() {
                    output = output.text(&format!(
                        "All {} document(s) already open in {}",
                        result.skipped.len(),
                        result.viewer_used
                    ));
                } else {
                    let total = result.opened.len() + result.skipped.len();
                    output = output.text(&format!(
                        "Opened {} document(s) in {} ({} already open)",
                        result.opened.len(),
                        result.viewer_used,
                        result.skipped.len()
                    ));

                    if total > 1 {
                        output = output.text(&format!(
                            "\nDocuments: {} + {} child task(s)",
                            self.short_code,
                            total - 1
                        ));
                    }
                }

                Ok(output.build_result())
            }
            Err(e) => Ok(error_result(
                "Failed to open document",
                &e.to_string(),
                Some("Check viewer configuration in config.toml [viewer] section."),
            )),
        }
    }
}
