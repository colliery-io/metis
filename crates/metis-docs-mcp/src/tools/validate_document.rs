use metis_core::application::services::document::DocumentValidationService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[mcp_tool(
    name = "validate_document",
    description = "Validate a Metis document's structure and content",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ValidateDocumentTool {
    /// Path to the .metis folder containing the document
    pub project_path: String,
    /// Path to the document file (relative to project root)
    pub document_path: String,
}

impl ValidateDocumentTool {
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
        // The document_path is relative to the metis workspace
        let full_document_path = metis_dir.join(&self.document_path);

        if !full_document_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Document not found at {}", full_document_path.display()),
            )));
        }

        // Use the validation service
        let service = DocumentValidationService::new();
        let result = service.validate_document(&full_document_path).await;

        match result {
            Ok(validation_result) => {
                let response = serde_json::json!({
                    "is_valid": validation_result.is_valid,
                    "document_type": validation_result.document_type,
                    "document_path": self.document_path,
                    "full_path": full_document_path.to_string_lossy(),
                    "errors": validation_result.errors
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let response = serde_json::json!({
                    "is_valid": false,
                    "document_path": self.document_path,
                    "full_path": full_document_path.to_string_lossy(),
                    "error": format!("Validation failed: {}", e),
                    "errors": [format!("Validation service error: {}", e)]
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
