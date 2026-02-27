use crate::formatting::ToolOutput;
use metis_core::{
    application::services::{
        document::{creation::DocumentCreationConfig, DocumentCreationService},
        workspace::WorkspaceDetectionService,
    },
    domain::documents::types::DocumentType,
    Database,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::str::FromStr;

#[mcp_tool(
    name = "create_document",
    description = "Create a new Metis document (vision, strategy, initiative, task, adr). Each document gets a unique short code in format PREFIX-TYPE-NNNN (e.g., PROJ-V-0001). Parent documents should be referenced by their short code (e.g., PROJ-V-0001). Document type availability depends on current flight level configuration. For standalone work items not tied to initiatives, use document_type='task' with backlog_category to create a backlog item.",
    idempotent_hint = false,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct CreateDocumentTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Document type: vision, strategy, initiative, task, adr
    pub document_type: String,
    /// Title of the document
    pub title: String,
    /// Parent document short code (required for strategy, initiative, task). Omit for backlog items.
    pub parent_id: Option<String>,
    /// Risk level for strategies (low, medium, high)
    pub risk_level: Option<String>,
    /// Complexity for initiatives (xs, s, m, l, xl)
    pub complexity: Option<String>,
    /// Stakeholders involved
    pub stakeholders: Option<Vec<String>>,
    /// Decision maker for ADRs
    pub decision_maker: Option<String>,
    /// Backlog category for standalone tasks not tied to initiatives (bug, feature, tech-debt). When specified, creates a backlog item instead of a regular task.
    pub backlog_category: Option<String>,
}

impl CreateDocumentTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let database = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        // Parse document type
        let doc_type = DocumentType::from_str(&self.document_type).map_err(|_| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid document type: {}", self.document_type),
            ))
        })?;

        let mut config_repo = database.configuration_repository().map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to access configuration repository: {}", e),
            ))
        })?;

        let flight_config = config_repo.get_flight_level_config().map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load configuration: {}", e),
            ))
        })?;

        // Validate document type is enabled in current configuration
        let enabled_types = flight_config.enabled_document_types();
        if !enabled_types.contains(&doc_type) {
            let available_types: Vec<String> =
                enabled_types.iter().map(|t| t.to_string()).collect();
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!(
                    "{} creation is disabled in current configuration ({} mode). Available document types: {}. To enable {}, use 'metis config set --preset full' or configure individually with 'metis config set --strategies true --initiatives true'",
                    doc_type,
                    flight_config.preset_name(),
                    available_types.join(", "),
                    doc_type
                ),
            )));
        }

        // Create the document creation service
        let creation_service = DocumentCreationService::new(metis_dir);

        // Parse complexity if provided
        let complexity = self
            .complexity
            .as_ref()
            .map(|c| c.parse())
            .transpose()
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid complexity: {}", e),
                ))
            })?;

        // Parse risk level if provided
        let risk_level = self
            .risk_level
            .as_ref()
            .map(|r| r.parse())
            .transpose()
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("Invalid risk level: {}", e),
                ))
            })?;

        // Pass parent_id as-is (short codes are now handled directly by core services)
        let resolved_parent_id = self.parent_id.clone();

        let config = DocumentCreationConfig {
            title: self.title.clone(),
            description: None,
            parent_id: resolved_parent_id
                .as_ref()
                .map(|id| metis_core::domain::documents::types::DocumentId::from(id.clone())),
            tags: vec![],
            phase: None, // Will use defaults
            complexity,
            risk_level,
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
                // Determine parent strategy ID based on configuration
                let parent_strategy_id = if flight_config.strategies_enabled {
                    // Full configuration: require explicit strategy parent
                    resolved_parent_id.as_ref().ok_or_else(|| {
                        CallToolError::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Initiative requires a parent strategy short code in full configuration",
                        ))
                    })?.clone()
                } else {
                    // Streamlined/Direct configuration: use NULL strategy placeholder
                    "NULL".to_string()
                };

                creation_service
                    .create_initiative_with_config(config, &parent_strategy_id, &flight_config)
                    .await
                    .map_err(|e| CallToolError::new(e))?
            }
            DocumentType::Task => {
                // Check if this should be a backlog item
                if let Some(category) = &self.backlog_category {
                    // Creating a backlog item - add category tag
                    let category_tag = match category.to_lowercase().as_str() {
                        "bug" => {
                            metis_core::domain::documents::types::Tag::Label("bug".to_string())
                        }
                        "feature" => {
                            metis_core::domain::documents::types::Tag::Label("feature".to_string())
                        }
                        "tech-debt" | "techdebt" | "tech_debt" => {
                            metis_core::domain::documents::types::Tag::Label(
                                "tech-debt".to_string(),
                            )
                        }
                        _ => {
                            return Err(CallToolError::new(std::io::Error::new(
                                std::io::ErrorKind::InvalidInput,
                                format!("Invalid backlog category '{}'. Valid options: bug, feature, tech-debt", category),
                            )));
                        }
                    };

                    let backlog_config = DocumentCreationConfig {
                        title: self.title.clone(),
                        description: None,
                        parent_id: None,
                        tags: vec![category_tag],
                        phase: None,
                        complexity: None,
                        risk_level: None,
                    };

                    creation_service
                        .create_backlog_item(backlog_config)
                        .await
                        .map_err(|e| CallToolError::new(e))?
                } else if let Some(initiative_id) = resolved_parent_id.as_ref() {
                    // Task with parent initiative
                    let strategy_id = if flight_config.strategies_enabled {
                        // Full configuration: use actual strategy short code from initiative location
                        self.find_strategy_short_code_for_initiative(&database, initiative_id)?
                    } else {
                        // Streamlined/Direct: use NULL as strategy placeholder
                        "NULL".to_string()
                    };

                    creation_service
                        .create_task_with_config(
                            config,
                            &strategy_id,
                            initiative_id,
                            &flight_config,
                        )
                        .await
                        .map_err(|e| CallToolError::new(e))?
                } else if flight_config.initiatives_enabled {
                    // Initiatives enabled but no parent provided - suggest backlog
                    return Err(CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Task requires a parent initiative ID in {} configuration. Either provide parent_id with an initiative short code, or use backlog_category (bug, feature, tech-debt) to create a standalone backlog item.", flight_config.preset_name()),
                    )));
                } else {
                    // Direct configuration: create task without parents (use NULL for both)
                    creation_service
                        .create_task_with_config(config, "NULL", "NULL", &flight_config)
                        .await
                        .map_err(|e| CallToolError::new(e))?
                }
            }
            DocumentType::Adr => creation_service
                .create_adr(config)
                .await
                .map_err(|e| CallToolError::new(e))?,
        };

        let parent_display = self.parent_id.as_ref().map(|s| s.as_str()).unwrap_or("-");

        let result_output = ToolOutput::new()
            .header("Document Created")
            .text(&format!("{} created successfully", result.short_code))
            .table(
                &["Field", "Value"],
                vec![
                    vec!["Title".to_string(), self.title.clone()],
                    vec!["Type".to_string(), self.document_type.clone()],
                    vec!["Short Code".to_string(), result.short_code.clone()],
                    vec!["Parent".to_string(), parent_display.to_string()],
                ],
            )
            .text(&format!("Path: `{}`", result.file_path.to_string_lossy()))
            .build_result();

        Ok(result_output)
    }

    fn find_strategy_short_code_for_initiative(
        &self,
        database: &Database,
        initiative_id: &str,
    ) -> Result<String, CallToolError> {
        let mut repo = database.repository().map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Repository error: {}", e),
            ))
        })?;

        // Find the initiative document in the database by short code
        let initiative = repo
            .find_by_short_code(initiative_id)
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database lookup error: {}", e),
                ))
            })?
            .ok_or_else(|| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Initiative '{}' not found in database", initiative_id),
                ))
            })?;

        // Get the strategy ID from the initiative, then find the strategy's short code
        let strategy_id = initiative.strategy_id.ok_or_else(|| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Initiative '{}' has no parent strategy", initiative_id),
            ))
        })?;

        // Find the strategy by its short code (strategy_id now contains short codes)
        let strategy = repo
            .find_by_short_code(&strategy_id)
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Database lookup error: {}", e),
                ))
            })?
            .ok_or_else(|| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Strategy '{}' not found in database", strategy_id),
                ))
            })?;

        Ok(strategy.short_code)
    }
}
