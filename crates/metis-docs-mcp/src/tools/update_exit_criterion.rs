use metis_core::{application::Application, dal::Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[mcp_tool(
    name = "update_exit_criterion",
    description = "Update the completion status of an exit criterion",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateExitCriterionTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Path to the document file (relative to project root)
    pub document_path: String,
    /// Title of the exit criterion to update
    pub criterion_title: String,
    /// New completion status (true/false)
    pub completed: bool,
    /// Optional notes or evidence for the completion
    pub notes: Option<String>,
}

impl UpdateExitCriterionTool {
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

        // Construct the full document path
        let full_document_path = metis_dir.join(&self.document_path);

        if !full_document_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Document not found at {}", full_document_path.display()),
            )));
        }

        // Read the current document content
        let content = fs::read_to_string(&full_document_path)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Update the exit criterion
        let updated_content = self.update_exit_criterion(&content)?;

        // Write the updated content back to the file
        fs::write(&full_document_path, updated_content)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Auto-sync after update to update database
        self.sync_workspace(metis_dir).await?;

        let response = serde_json::json!({
            "success": true,
            "document_path": self.document_path,
            "criterion_title": self.criterion_title,
            "completed": self.completed,
            "notes": self.notes,
            "message": format!(
                "Successfully {} exit criterion '{}'",
                if self.completed { "completed" } else { "unchecked" },
                self.criterion_title
            ),
            "auto_synced": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

    fn update_exit_criterion(&self, content: &str) -> Result<String, CallToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines = Vec::new();
        let mut found_criterion = false;

        for line in lines {
            let trimmed = line.trim();

            // Look for markdown checkbox patterns
            if trimmed.starts_with("- [") {
                if let Some(checkbox_end) = trimmed.find(']') {
                    if checkbox_end >= 3 {
                        // Extract the criterion text after the checkbox
                        let criterion_text = if trimmed.len() > checkbox_end + 1 {
                            trimmed[checkbox_end + 1..].trim()
                        } else {
                            ""
                        };

                        // Check if this is the criterion we want to update
                        if criterion_text.contains(&self.criterion_title)
                            || criterion_text.trim() == self.criterion_title.trim()
                        {
                            found_criterion = true;

                            // Preserve the original indentation
                            let indentation = &line[..line.len() - line.trim_start().len()];

                            // Create the updated checkbox
                            let checkbox_mark = if self.completed { "x" } else { " " };
                            let mut updated_line =
                                format!("{}− [{}] {}", indentation, checkbox_mark, criterion_text);

                            // Add notes if provided and completing the criterion
                            if self.completed && self.notes.is_some() {
                                let notes = self.notes.as_ref().unwrap();
                                if !notes.is_empty() {
                                    updated_line = format!("{} ({})", updated_line, notes);
                                }
                            }

                            result_lines.push(updated_line);
                        } else {
                            result_lines.push(line.to_string());
                        }
                    } else {
                        result_lines.push(line.to_string());
                    }
                } else {
                    result_lines.push(line.to_string());
                }
            } else {
                result_lines.push(line.to_string());
            }
        }

        if !found_criterion {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Exit criterion '{}' not found in document",
                    self.criterion_title
                ),
            )));
        }

        Ok(result_lines.join("\n"))
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
