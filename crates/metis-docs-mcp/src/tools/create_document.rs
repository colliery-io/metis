use metis_core::{
    application::services::document::{creation::DocumentCreationConfig, DocumentCreationService},
    domain::{documents::types::DocumentType, configuration::FlightLevelConfig},
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
    description = "Create a new Metis document (vision, strategy, initiative, task, adr). Parent documents should be referenced by their ID (kebab-case string). Document type availability depends on current flight level configuration.",
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

        // Load current flight level configuration
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_string_lossy().as_ref()).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open database: {}", e),
            ))
        })?;
        
        let mut config_repo = database
            .configuration_repository()
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to access configuration repository: {}", e),
                ))
            })?;
            
        let flight_config = config_repo
            .get_flight_level_config()
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to load configuration: {}", e),
                ))
            })?;

        // Validate document type is enabled in current configuration
        let enabled_types = flight_config.enabled_document_types();
        if !enabled_types.contains(&doc_type) {
            let available_types: Vec<String> = enabled_types.iter().map(|t| t.to_string()).collect();
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
                    self.parent_id.as_ref().ok_or_else(|| {
                        CallToolError::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "Initiative requires a parent strategy ID in full configuration",
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
                // Determine task creation approach based on configuration and provided parent
                if let Some(initiative_id) = self.parent_id.as_ref() {
                    // Task with parent initiative
                    let strategy_id = if flight_config.strategies_enabled {
                        // Full configuration: resolve actual strategy from initiative location
                        self.find_strategy_id_for_initiative(metis_dir, initiative_id)?
                    } else {
                        // Streamlined/Direct: use NULL as strategy placeholder
                        "NULL".to_string()
                    };

                    creation_service
                        .create_task_with_config(config, &strategy_id, initiative_id, &flight_config)
                        .await
                        .map_err(|e| CallToolError::new(e))?
                } else if flight_config.initiatives_enabled {
                    // Initiatives enabled but no parent provided - this should be an error
                    return Err(CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("Task requires a parent initiative ID in {} configuration. Provide an initiative_id or create as a backlog item.", flight_config.preset_name()),
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

    fn find_strategy_id_for_initiative(
        &self,
        metis_dir: &Path,
        initiative_id: &str,
    ) -> Result<String, CallToolError> {
        let strategies_dir = metis_dir.join("strategies");

        if !strategies_dir.exists() {
            return Err(CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "No strategies directory found",
            )));
        }

        // Search through all strategy directories to find the one containing this initiative
        for entry in std::fs::read_dir(&strategies_dir).map_err(CallToolError::new)? {
            let entry = entry.map_err(CallToolError::new)?;
            let strategy_path = entry.path();

            if strategy_path.is_dir() {
                let initiatives_dir = strategy_path.join("initiatives").join(initiative_id);
                if initiatives_dir.exists() && initiatives_dir.is_dir() {
                    // Found the strategy containing this initiative
                    if let Some(strategy_name) = strategy_path.file_name() {
                        if let Some(strategy_id) = strategy_name.to_str() {
                            return Ok(strategy_id.to_string());
                        }
                    }
                }
            }
        }

        Err(CallToolError::new(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Parent initiative '{}' not found", initiative_id),
        )))
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
