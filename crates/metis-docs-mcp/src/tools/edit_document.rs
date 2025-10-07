use metis_core::{application::Application, dal::Database};
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

        // Resolve short code to document path
        let document_path = self.resolve_short_code_to_path(metis_dir)?;
        let full_document_path = metis_dir.join(&document_path);

        if !full_document_path.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Document not found for short code {}", self.short_code),
            )));
        }

        // Read the current document content
        let content = fs::read_to_string(&full_document_path)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Perform the edit operation
        let (updated_content, replacements_made) = self.perform_edit(&content)?;

        if replacements_made == 0 {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "Search text '{}' not found in document. Use read_document to see the current content.",
                    self.search
                ),
            )));
        }

        // Write the updated content back to the file
        fs::write(&full_document_path, &updated_content)
            .await
            .map_err(|e| CallToolError::new(e))?;

        // Auto-sync after update to update database
        self.sync_workspace(metis_dir).await?;

        let response = serde_json::json!({
            "success": true,
            "short_code": self.short_code,
            "document_path": document_path,
            "replacements_made": replacements_made,
            "search_text": self.search,
            "replace_text": self.replace,
            "replace_all": self.replace_all.unwrap_or(false),
            "message": format!(
                "Successfully made {} replacement(s) in document {}",
                replacements_made, self.short_code
            ),
            "auto_synced": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
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
