use metis_core::{
    application::services::document::{creation::DocumentCreationConfig, DocumentCreationService},
    domain::documents::types::DocumentType,
    Application, Database,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[mcp_tool(
    name = "create_document",
    description = "Create a new Metis document (vision, strategy, initiative, task, adr). Parent documents should be referenced by their ID (kebab-case string)",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentTool {
    /// Path to the .metis folder where the document will be created
    pub project_path: String,
    /// Document type: vision, strategy, initiative, task, adr
    pub document_type: String,
    /// Title of the document
    pub title: String,
    /// Parent document ID (required for strategy, initiative, task)
    pub parent_id: Option<String>,
    /// Risk level for strategies (low, medium, high)
    pub risk_level: Option<String>,
    /// Complexity for initiatives (xs, s, m, l, xl)
    pub complexity: Option<String>,
    /// Stakeholders involved
    pub stakeholders: Option<Vec<String>>,
    /// Decision maker for ADRs
    pub decision_maker: Option<String>,
}

impl CreateDocumentTool {
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

        // Parse document type
        let doc_type = DocumentType::from_str(&self.document_type).map_err(|_| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid document type: {}", self.document_type),
            ))
        })?;

        // Create the document creation service
        let creation_service = DocumentCreationService::new(metis_dir);

        // Build configuration
        let config = DocumentCreationConfig {
            title: self.title.clone(),
            description: None,
            parent_id: self
                .parent_id
                .as_ref()
                .map(|id| metis_core::domain::documents::types::DocumentId::from(id.clone())),
            tags: vec![],
            phase: None, // Will use defaults
        };

        // Create the document based on type
        let result = match doc_type {
            DocumentType::Vision => {
                if self.parent_id.is_some() {
                    return Err(CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Vision documents cannot have a parent",
                    )));
                }
                creation_service
                    .create_vision(config)
                    .await
                    .map_err(|e| CallToolError::new(e))?
            }
            DocumentType::Strategy => creation_service
                .create_strategy(config)
                .await
                .map_err(|e| CallToolError::new(e))?,
            DocumentType::Initiative => {
                let parent_id = self.parent_id.as_ref().ok_or_else(|| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Initiative requires a parent strategy ID",
                    ))
                })?;
                creation_service
                    .create_initiative(config, parent_id)
                    .await
                    .map_err(|e| CallToolError::new(e))?
            }
            DocumentType::Task => {
                let parent_id = self.parent_id.as_ref().ok_or_else(|| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Task requires a parent initiative ID",
                    ))
                })?;
                // For tasks, we need to resolve the strategy from the initiative
                creation_service
                    .create_task(config, parent_id, parent_id)
                    .await
                    .map_err(|e| CallToolError::new(e))?
            }
            DocumentType::Adr => creation_service
                .create_adr(config)
                .await
                .map_err(|e| CallToolError::new(e))?,
        };

        // Auto-sync after creation to update database
        self.sync_workspace(metis_dir).await?;

        let response = serde_json::json!({
            "success": true,
            "document_id": result.document_id.to_string(),
            "document_type": self.document_type,
            "title": self.title,
            "file_path": result.file_path.to_string_lossy(),
            "parent_id": self.parent_id,
            "auto_synced": true
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
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
