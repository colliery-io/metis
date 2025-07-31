use metis_core::{application::Application, dal::Database};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[mcp_tool(
    name = "update_blocked_by",
    description = "Update the blocked_by relationship for a document",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateBlockedByTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Path to the document file (relative to project root)
    pub document_path: String,
    /// List of document titles that this document is blocked by
    pub blocked_by: Vec<String>,
}

impl UpdateBlockedByTool {
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

        // Update the blocked_by field in the frontmatter
        let updated_content = self.update_blocked_by_frontmatter(&content)?;

        // Write the updated content back to the file
        fs::write(&full_document_path, updated_content)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Auto-sync after update to update database
        self.sync_workspace(metis_dir).await?;

        let response = serde_json::json!({
            "success": true,
            "document_path": self.document_path,
            "blocked_by": self.blocked_by,
            "message": format!(
                "Successfully updated blocked_by relationships. Document is {} blocked by {} documents",
                if self.blocked_by.is_empty() { "not" } else { "now" },
                self.blocked_by.len()
            ),
            "auto_synced": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }

    fn update_blocked_by_frontmatter(&self, content: &str) -> Result<String, CallToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines = Vec::new();
        let mut in_frontmatter = false;
        let mut found_blocked_by = false;
        let mut frontmatter_ended = false;

        for (i, line) in lines.iter().enumerate() {
            if i == 0 && line.trim() == "---" {
                in_frontmatter = true;
                result_lines.push(line.to_string());
                continue;
            }

            if in_frontmatter && line.trim() == "---" {
                // End of frontmatter
                if !found_blocked_by {
                    // Add blocked_by field before closing the frontmatter
                    self.add_blocked_by_field(&mut result_lines);
                }
                result_lines.push(line.to_string());
                in_frontmatter = false;
                frontmatter_ended = true;
                continue;
            }

            if in_frontmatter {
                let trimmed = line.trim();
                if trimmed.starts_with("blocked_by:") {
                    found_blocked_by = true;
                    // Replace the blocked_by field
                    self.add_blocked_by_field(&mut result_lines);
                    continue;
                } else {
                    result_lines.push(line.to_string());
                }
            } else {
                result_lines.push(line.to_string());
            }
        }

        if !frontmatter_ended {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Document does not have valid YAML frontmatter",
            )));
        }

        Ok(result_lines.join("\n"))
    }

    fn add_blocked_by_field(&self, result_lines: &mut Vec<String>) {
        if self.blocked_by.is_empty() {
            result_lines.push("blocked_by: []".to_string());
        } else {
            result_lines.push("blocked_by:".to_string());
            for blocked_document in &self.blocked_by {
                result_lines.push(format!("  - \"{}\"", blocked_document));
            }
        }
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
