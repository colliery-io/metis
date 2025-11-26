use crate::formatting::{error_result, ToolOutput};
use metis_core::{application::services::workspace::WorkspaceDetectionService, dal::Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[mcp_tool(
    name = "read_document",
    description = "Read a document's content and structure using its short code (e.g., PROJ-V-0001).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadDocumentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Document short code (e.g., PROJ-V-0001) to identify the document
    pub short_code: String,
}

impl ReadDocumentTool {
    /// Resolve short code to file path
    fn resolve_short_code_to_path(&self, metis_dir: &Path) -> Result<String, CallToolError> {
        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Database error: {}", e),
            ))
        })?;

        let mut repo = db.repository().map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Repository error: {}", e),
            ))
        })?;

        // Use the core DAL method
        repo.resolve_short_code_to_filepath(&self.short_code)
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Resolution error: {}", e),
                ))
            })
    }

    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let _db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Resolve short code to document path
        let document_path = self.resolve_short_code_to_path(metis_dir)?;
        let full_document_path = metis_dir.join(&document_path);

        if !full_document_path.exists() {
            return Ok(error_result(
                &format!("Document not found: {}", self.short_code),
                &format!(
                    "No document with identifier \"{}\" exists in this project.",
                    self.short_code
                ),
                Some("Use `list_documents` to see available documents."),
            ));
        }

        // Read the document content
        let content = fs::read_to_string(&full_document_path)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Extract metadata from frontmatter
        let (doc_type, phase, _created, _archived, title) = self.extract_metadata(&content);

        // Build simplified output with inline metadata
        let output = ToolOutput::new()
            .header(&format!("{}: {} ({}, {})", self.short_code, title, doc_type, phase))
            .text(&content);

        Ok(output.build_result())
    }

    fn extract_metadata(&self, content: &str) -> (String, String, String, String, String) {
        let mut doc_type = "unknown".to_string();
        let mut phase = "unknown".to_string();
        let mut created = "unknown".to_string();
        let mut archived = "No".to_string();
        let mut title = "Untitled".to_string();

        let mut in_frontmatter = false;

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed == "---" {
                if in_frontmatter {
                    break; // End of frontmatter
                }
                in_frontmatter = true;
                continue;
            }

            if in_frontmatter {
                if let Some((key, value)) = trimmed.split_once(':') {
                    let key = key.trim();
                    let value = value.trim().trim_matches('"');
                    match key {
                        "level" => doc_type = value.to_string(),
                        "title" => title = value.to_string(),
                        "created_at" => {
                            // Parse and format date
                            if let Some(date_part) = value.split('T').next() {
                                created = date_part.to_string();
                            } else {
                                created = value.to_string();
                            }
                        }
                        "archived" => {
                            archived = if value == "true" {
                                "Yes".to_string()
                            } else {
                                "No".to_string()
                            };
                        }
                        _ => {}
                    }
                }
                // Extract phase from tags
                if trimmed.contains("#phase/") {
                    if let Some(start) = trimmed.find("#phase/") {
                        let phase_start = start + 7;
                        let phase_end = trimmed[phase_start..]
                            .find(|c: char| !c.is_alphanumeric() && c != '_')
                            .map(|i| phase_start + i)
                            .unwrap_or(trimmed.len());
                        phase = trimmed[phase_start..phase_end].trim_matches('"').to_string();
                    }
                }
            }
        }

        (doc_type, phase, created, archived, title)
    }

    #[allow(dead_code)]
    fn extract_sections(&self, content: &str) -> Vec<String> {
        let mut sections = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("## ") && !trimmed.starts_with("### ") {
                let section_name = trimmed[3..].trim().to_string();
                sections.push(section_name);
            }
        }

        sections
    }

    fn extract_exit_criteria(&self, content: &str) -> Vec<ExitCriterion> {
        let mut criteria = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();

            // Look for markdown checkbox patterns
            if trimmed.starts_with("- [") {
                if let Some(checkbox_end) = trimmed.find(']') {
                    if checkbox_end >= 3 {
                        let checkbox_content = &trimmed[3..checkbox_end];
                        let completed =
                            checkbox_content.trim() == "x" || checkbox_content.trim() == "X";

                        // Extract the criterion text after the checkbox
                        let criterion_text = if trimmed.len() > checkbox_end + 1 {
                            trimmed[checkbox_end + 1..].trim().to_string()
                        } else {
                            "".to_string()
                        };

                        if !criterion_text.is_empty() {
                            criteria.push(ExitCriterion {
                                text: criterion_text,
                                completed,
                            });
                        }
                    }
                }
            }
        }

        criteria
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct ExitCriterion {
    text: String,
    completed: bool,
}
