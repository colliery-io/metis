use metis_core::{update_blocked_by, update_document_content, update_exit_criterion};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        match update_document_content(&document_path, &self.section_heading, &self.new_content)
            .await
        {
            Ok(_) => {
                let response = serde_json::json!({
                    "message": format!("Successfully updated section '{}' in document", self.section_heading),
                    "project_path": self.project_path,
                    "document_path": self.document_path,
                    "section_heading": self.section_heading
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to update document content: {}", e),
                    "project_path": self.project_path,
                    "document_path": self.document_path,
                    "section_heading": self.section_heading
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

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
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        match update_exit_criterion(&document_path, &self.criterion_title, self.completed).await {
            Ok(_) => {
                let response = serde_json::json!({
                    "message": format!("Successfully updated exit criterion '{}' to {}",
                        self.criterion_title,
                        if self.completed { "completed" } else { "pending" }
                    ),
                    "project_path": self.project_path,
                    "document_path": self.document_path,
                    "criterion_title": self.criterion_title,
                    "completed": self.completed
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to update exit criterion: {}", e),
                    "project_path": self.project_path,
                    "document_path": self.document_path,
                    "criterion_title": self.criterion_title
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

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
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        match update_blocked_by(&document_path, self.blocked_by.clone()).await {
            Ok(_) => {
                let response = serde_json::json!({
                    "message": format!("Successfully updated blocked_by for document (now blocked by {} items)", self.blocked_by.len()),
                    "project_path": self.project_path,
                    "document_path": self.document_path,
                    "blocked_by": self.blocked_by
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to update blocked_by: {}", e),
                    "project_path": self.project_path,
                    "document_path": self.document_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
