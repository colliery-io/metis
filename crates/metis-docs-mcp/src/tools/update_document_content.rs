use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[mcp_tool(
    name = "update_document_content",
    description = "Update content of a specific H2 section in a document",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct UpdateDocumentContentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Path to the document file (relative to project root)
    pub document_path: String,
    /// Section heading to update - must be an H2 level heading (e.g., "Problem Statement" targets "## Problem Statement")
    pub section_heading: String,
    /// New content for the section
    pub new_content: String,
}

impl UpdateDocumentContentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);
        
        // Validate metis workspace exists
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Metis workspace not found at {}. Run initialize_project first.", metis_dir.display())
            )));
        }
        
        // Construct the full document path
        let full_document_path = metis_dir.join(&self.document_path);
        
        if !full_document_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Document not found at {}", full_document_path.display())
            )));
        }
        
        // Read the current document content
        let content = fs::read_to_string(&full_document_path).await
            .map_err(|e| CallToolError::new(e))?;
        
        // Update the section content
        let updated_content = self.update_section_content(&content)?;
        
        // Write the updated content back to the file
        fs::write(&full_document_path, updated_content).await
            .map_err(|e| CallToolError::new(e))?;
        
        let response = serde_json::json!({
            "success": true,
            "document_path": self.document_path,
            "section_heading": self.section_heading,
            "updated": true,
            "message": format!("Successfully updated section '{}' in document", self.section_heading)
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }
    
    fn update_section_content(&self, content: &str) -> Result<String, CallToolError> {
        let lines: Vec<&str> = content.lines().collect();
        let mut result_lines = Vec::new();
        let mut in_target_section = false;
        let mut found_section = false;
        
        let target_heading = format!("## {}", self.section_heading);
        
        for line in lines {
            if line.trim() == target_heading.trim() {
                // Found the target section
                found_section = true;
                in_target_section = true;
                result_lines.push(line);
                
                // Add the new content after the heading
                result_lines.push("");
                result_lines.push(&self.new_content);
                result_lines.push("");
                continue;
            }
            
            if in_target_section {
                // Check if we've reached the next section (another ## heading)
                if line.trim().starts_with("## ") {
                    in_target_section = false;
                    result_lines.push(line);
                } else {
                    // Skip lines in the target section (they're being replaced)
                    continue;
                }
            } else {
                // Not in target section, keep the line
                result_lines.push(line);
            }
        }
        
        if !found_section {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Section heading '{}' not found in document", self.section_heading)
            )));
        }
        
        Ok(result_lines.join("\n"))
    }
}