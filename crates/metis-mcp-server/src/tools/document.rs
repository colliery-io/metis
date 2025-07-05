use metis_core::{
    render, validate, Complexity, DocumentContext, DocumentStore, DocumentType, RiskLevel,
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
}

impl CreateDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);

        // Parse document type
        let doc_type = DocumentType::from_str(&self.document_type).map_err(CallToolError::new)?;

        // Create document context
        let mut context = DocumentContext::new(self.title.clone());

        // Add optional fields based on document type
        if let Some(parent) = &self.parent_title {
            context = context.with_parent(parent.clone());
        }

        if let Some(risk) = &self.risk_level {
            let risk_level = match risk.to_lowercase().as_str() {
                "low" => RiskLevel::Low,
                "medium" => RiskLevel::Medium,
                "high" => RiskLevel::High,
                "critical" => RiskLevel::Critical,
                _ => return Ok(CallToolResult::text_content(
                    vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                        "error": format!("Invalid risk level '{}'. Must be: low, medium, high, critical", risk)
                    })).map_err(CallToolError::new)?)]
                ))
            };
            context = context.with_risk_level(risk_level);
        }

        if let Some(complexity) = &self.complexity {
            let complexity_level = match complexity.to_lowercase().as_str() {
                "s" => Complexity::S,
                "m" => Complexity::M,
                "l" => Complexity::L,
                "xl" => Complexity::XL,
                _ => return Ok(CallToolResult::text_content(
                    vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                        "error": format!("Invalid complexity '{}'. Must be: s, m, l, xl", complexity)
                    })).map_err(CallToolError::new)?)]
                ))
            };
            context = context.with_complexity(complexity_level);
        }

        if let Some(decision_maker) = &self.decision_maker {
            context = context.with_decision_maker(decision_maker.clone());
        }

        if let Some(stakeholders) = &self.stakeholders {
            context = context.with_stakeholders(stakeholders.clone());
        }

        // Validate context for document type
        if let Err(validation_error) = context.validate_for_type(&doc_type) {
            return Ok(CallToolResult::text_content(vec![TextContent::from(
                serde_json::to_string_pretty(&serde_json::json!({
                    "error": "Document validation failed",
                    "validation_error": format!("{}", validation_error)
                }))
                .map_err(CallToolError::new)?,
            )]));
        }

        // Render document using metis-core render function
        match render(doc_type.clone(), context, &project_path).await {
            Ok(file_path) => {
                // Store document in database
                let db_path = project_path.join(".metis.db");
                match DocumentStore::new(db_path.to_str().unwrap()).await {
                    Ok(store) => {
                        // Sync the created document to database
                        let sync_engine = metis_core::SyncEngine::new(store);
                        if let Err(e) = sync_engine.sync_from_filesystem(&project_path).await {
                            eprintln!("Warning: Failed to sync document to database: {}", e);
                        }

                        let response = serde_json::json!({
                            "message": format!("{} '{}' created successfully", doc_type, self.title),
                            "file_path": file_path.to_string_lossy(),
                            "document_type": self.document_type
                        });

                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to connect to database: {}", e)
                        });

                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(CallToolError::new)?,
                        )]))
                    }
                }
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

        // Read the document file
        match tokio::fs::read_to_string(&document_path).await {
            Ok(_content) => {
                // Validate the document using metis-core
                match validate(&document_path).await {
                    Ok(validation_result) => {
                        let response = serde_json::json!({
                            "message": format!("Document validation {}: {}",
                                if validation_result.is_valid { "passed" } else { "failed" },
                                if validation_result.is_valid {
                                    "Document is valid".to_string()
                                } else {
                                    format!("{} errors found", validation_result.frontmatter_errors.len())
                                }),
                            "is_valid": validation_result.is_valid,
                            "document_type": validation_result.document_type,
                            "frontmatter_errors": validation_result.frontmatter_errors,
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
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to read document file: {}", e),
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
