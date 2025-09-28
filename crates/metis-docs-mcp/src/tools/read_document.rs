use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[mcp_tool(
    name = "read_document",
    description = "Read a document's content and structure. Always read documents before editing them.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ReadDocumentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Path to the document file (relative to project root)
    pub document_path: String,
}

impl ReadDocumentTool {
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
            "document_path": self.document_path,
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
                        let completed = checkbox_content.trim() == "x" || checkbox_content.trim() == "X";
                        
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