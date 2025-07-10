use metis_docs_core::{
    Application, Database,
    Document, Vision, Strategy, Initiative, Task, Adr,
    DocumentType, Phase, DocumentId,
    RiskLevel, Complexity,
    application::services::{
        document::{DocumentCreationService, DocumentCreationConfig, DocumentValidationService},
        SyncService, DatabaseService,
    },
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

#[mcp_tool(
    name = "create_document",
    description = "Create a new Metis document (vision, strategy, initiative, task, adr)",
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

    /// Parent document title (required for strategy, initiative, task)
    pub parent_title: Option<String>,

    /// Risk level for strategies (low, medium, high)
    pub risk_level: Option<String>,

    /// Complexity for initiatives (xs, s, m, l, xl)
    pub complexity: Option<String>,

    /// Decision maker for ADRs
    pub decision_maker: Option<String>,

    /// Stakeholders involved
    pub stakeholders: Option<Vec<String>>,

    /// Strategy ID for documents in the strategy hierarchy
    pub strategy_id: Option<String>,

    /// Initiative ID for tasks
    pub initiative_id: Option<String>,
}

impl CreateDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);

        // Parse document type
        let doc_type = DocumentType::from_str(&self.document_type).map_err(CallToolError::new)?;

        // Create the document creation service
        let creation_service = DocumentCreationService::new(&project_path);

        // Create configuration for document creation
        let config = DocumentCreationConfig {
            title: self.title.clone(),
            description: None, // Could be added as optional field to the tool
            parent_id: self.parent_title.as_ref().map(|t| DocumentId::from(t.clone())),
            tags: vec![],
            phase: None, // Will use default phase for document type
        };

        // Create the document based on type
        let result = match doc_type {
            DocumentType::Vision => {
                creation_service.create_vision(config).await
            }
            DocumentType::Strategy => {
                // Parse risk level if provided
                let risk_level = if let Some(risk) = &self.risk_level {
                    Some(match risk.to_lowercase().as_str() {
                        "low" => RiskLevel::Low,
                        "medium" => RiskLevel::Medium,
                        "high" => RiskLevel::High,
                        "critical" => RiskLevel::Critical,
                        _ => return Ok(CallToolResult::text_content(
                            vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                                "error": format!("Invalid risk level '{}'. Must be: low, medium, high, critical", risk)
                            })).map_err(CallToolError::new)?)]
                        ))
                    })
                } else {
                    None
                };

                creation_service.create_strategy(config).await
            }
            DocumentType::Initiative => {
                // Initiative requires a parent strategy
                if let Some(parent_id) = &self.parent_title {
                    // Parse complexity if provided
                    let complexity = if let Some(comp) = &self.complexity {
                        Some(match comp.to_lowercase().as_str() {
                            "s" => Complexity::S,
                            "m" => Complexity::M,
                            "l" => Complexity::L,
                            "xl" => Complexity::XL,
                            _ => return Ok(CallToolResult::text_content(
                                vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                                    "error": format!("Invalid complexity '{}'. Must be: s, m, l, xl", comp)
                                })).map_err(CallToolError::new)?)]
                            ))
                        })
                    } else {
                        None
                    };

                    creation_service.create_initiative(config, parent_id).await
                } else {
                    return Ok(CallToolResult::text_content(vec![TextContent::from(
                        serde_json::to_string_pretty(&serde_json::json!({
                            "error": "Initiative requires parent_title parameter (strategy ID)"
                        })).map_err(CallToolError::new)?
                    )]));
                }
            }
            DocumentType::Task => {
                // Task requires both strategy_id and initiative_id
                if let (Some(strategy_id), Some(initiative_id)) = (&self.strategy_id, &self.initiative_id) {
                    creation_service.create_task(config, strategy_id, initiative_id).await
                } else {
                    return Ok(CallToolResult::text_content(vec![TextContent::from(
                        serde_json::to_string_pretty(&serde_json::json!({
                            "error": "Task requires both strategy_id and initiative_id parameters"
                        })).map_err(CallToolError::new)?
                    )]));
                }
            }
            DocumentType::Adr => {
                let decision_maker = self.decision_maker.as_deref().unwrap_or("Team");
                let stakeholders = self.stakeholders.clone().unwrap_or_default();
                creation_service.create_adr(config, decision_maker, stakeholders).await
            }
        };

        match result {
            Ok(creation_result) => {
                // Sync the created document to database
                let db_path = project_path.join("metis.db");
                if let Ok(db) = Database::new(db_path.to_str().unwrap()) {
                    let mut app = Application::new(db);
                    let _ = app.sync_directory(&project_path).await;
                }

                let response = serde_json::json!({
                    "message": format!("{} '{}' created successfully", doc_type, self.title),
                    "document_id": creation_result.document_id.to_string(),
                    "file_path": creation_result.file_path.to_string_lossy(),
                    "document_type": self.document_type
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to create document '{}': {}", self.title, e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

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
        let project_path = PathBuf::from(&self.project_path);
        let document_path = project_path.join(&self.document_path);

        // Create validation service
        let validation_service = DocumentValidationService::new();

        // Validate the document
        match validation_service.validate_document(&document_path).await {
            Ok(validation_result) => {
                let response = serde_json::json!({
                    "message": format!("Document validation {}: {}",
                        if validation_result.is_valid { "passed" } else { "failed" },
                        if validation_result.is_valid {
                            "Document is valid".to_string()
                        } else {
                            format!("{} errors found", validation_result.errors.len())
                        }),
                    "is_valid": validation_result.is_valid,
                    "document_type": validation_result.document_type.to_string(),
                    "phase": validation_result.phase.map(|p| p.to_string()),
                    "errors": validation_result.errors,
                    "warnings": validation_result.warnings,
                    "project_path": self.project_path,
                    "document_path": self.document_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to validate document: {}", e),
                    "project_path": self.project_path,
                    "document_path": self.document_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response)
                        .map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
