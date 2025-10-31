use metis_core::{dal::Database, Application};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
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

        // Sync before reading to catch external edits
        self.sync_workspace(metis_dir).await?;

        // Resolve short code to document path
        let document_path = self.resolve_short_code_to_path(metis_dir)?;
        let full_document_path = metis_dir.join(&document_path);

        if !full_document_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Document not found for short code {}", self.short_code),
            )));
        }

        // Read the document content
        let content = fs::read_to_string(&full_document_path)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Extract sections for convenience
        let sections = self.extract_sections(&content);

        // Extract exit criteria completion info
        let exit_criteria = self.extract_exit_criteria(&content);
        let completed_criteria = exit_criteria.iter().filter(|c| c.completed).count();
        let total_criteria = exit_criteria.len();

        let response = serde_json::json!({
            "short_code": self.short_code,
            "document_path": document_path,
            "content": content,
            "sections": sections,
            "exit_criteria_summary": {
                "total": total_criteria,
                "completed": completed_criteria,
                "completion_percentage": if total_criteria > 0 {
                    (completed_criteria as f64 / total_criteria as f64 * 100.0).round()
                } else {
                    0.0
                }
            },
            "document_stats": {
                "lines": content.lines().count(),
                "characters": content.len(),
                "words": content.split_whitespace().count()
            }
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

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

#[derive(Debug, Serialize, Deserialize)]
struct ExitCriterion {
    text: String,
    completed: bool,
}
