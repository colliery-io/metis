use crate::tools::{
    CheckPhaseTransitionTool, CreateDocumentTool, InitializeProjectTool, ListDocumentsTool,
    MetisTools, OpenVaultInObsidianTool, SearchDocumentsTool, TransitionPhaseTool,
    UpdateBlockedByTool, UpdateDocumentContentTool, UpdateExitCriterionTool, ValidateDocumentTool,
    ValidateExitCriteriaTool,
};
use crate::MetisServerConfig;
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult, TextContent},
    McpServer,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct MetisServerHandler {
    #[allow(dead_code)]
    config: Arc<MetisServerConfig>,
    // Track active projects for background sync
    active_projects: Arc<RwLock<HashMap<PathBuf, Arc<metis_core::SyncEngine>>>>,
}

impl MetisServerHandler {
    pub fn new(config: MetisServerConfig) -> Self {
        let handler = Self {
            config: Arc::new(config),
            active_projects: Arc::new(RwLock::new(HashMap::new())),
        };

        // Auto-discover existing projects on startup
        handler.discover_existing_projects();

        // Start background sync task
        handler.start_background_sync();

        handler
    }

    fn discover_existing_projects(&self) {
        let active_projects = Arc::clone(&self.active_projects);

        tokio::spawn(async move {
            info!("Discovering existing Metis projects...");

            // Start from current directory and search for .metis.db files
            if let Ok(current_dir) = std::env::current_dir() {
                match Self::find_metis_projects(&current_dir).await {
                    Ok(projects) => {
                        info!("Found {} existing Metis projects", projects.len());

                        let mut projects_map = active_projects.write().await;
                        for project_path in projects {
                            match Self::initialize_sync_engine(&project_path).await {
                                Ok(sync_engine) => {
                                    info!(
                                        "Registered project for sync: {}",
                                        project_path.display()
                                    );
                                    projects_map.insert(project_path, sync_engine);
                                }
                                Err(e) => {
                                    warn!(
                                        "Failed to initialize sync engine for {}: {}",
                                        project_path.display(),
                                        e
                                    );
                                }
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to discover existing projects: {}", e);
                    }
                }
            }
        });
    }

    async fn find_metis_projects(
        start_dir: &std::path::Path,
    ) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let mut projects = Vec::new();
        let mut current_dir = start_dir.to_path_buf();

        // Traverse upward from current directory looking for metis/ subdirectory
        loop {
            let metis_dir = current_dir.join("metis");
            let metis_db = metis_dir.join(".metis.db");

            if metis_dir.is_dir() && metis_db.is_file() {
                info!("Found Metis project at: {}", metis_dir.display());
                projects.push(metis_dir);
            }

            // Move to parent directory
            if let Some(parent) = current_dir.parent() {
                current_dir = parent.to_path_buf();
            } else {
                // Reached filesystem root
                break;
            }
        }

        Ok(projects)
    }

    async fn initialize_sync_engine(
        project_path: &Path,
    ) -> Result<Arc<metis_core::SyncEngine>, Box<dyn std::error::Error + Send + Sync>> {
        let db_path = project_path.join(".metis.db");

        // Create document store and sync engine
        let store = metis_core::DocumentStore::new(db_path.to_str().unwrap())
            .await
            .map_err(|e| format!("Failed to open document store: {}", e))?;
        let sync_engine = Arc::new(metis_core::SyncEngine::new(store));

        // Run initial sync
        info!(
            "Running initial sync for discovered project: {}",
            project_path.display()
        );
        match sync_engine.sync_from_filesystem(project_path).await {
            Ok(result) => {
                info!(
                    "Initial sync completed for {}: {} processed, {} updated, {} deleted",
                    project_path.display(),
                    result.files_processed,
                    result.files_updated,
                    result.files_deleted
                );

                if !result.errors.is_empty() {
                    warn!(
                        "Initial sync errors for {}: {} errors",
                        project_path.display(),
                        result.errors.len()
                    );
                    for error in &result.errors {
                        error!(
                            "Initial sync error in {}: {}",
                            error.file_path.display(),
                            error.error
                        );
                    }
                }
            }
            Err(e) => {
                warn!("Initial sync failed for {}: {}", project_path.display(), e);
            }
        }

        Ok(sync_engine)
    }

    fn start_background_sync(&self) {
        let active_projects = Arc::clone(&self.active_projects);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            info!("Background sync task started, checking every 5 seconds");

            loop {
                interval.tick().await;

                let projects = active_projects.read().await;
                debug!("Background sync tick: {} active projects", projects.len());
                for (project_path, sync_engine) in projects.iter() {
                    match sync_engine.sync_from_filesystem(project_path).await {
                        Ok(result) => {
                            if result.files_updated > 0 || result.files_deleted > 0 {
                                debug!(
                                    "Sync completed for {}: {} updated, {} deleted",
                                    project_path.display(),
                                    result.files_updated,
                                    result.files_deleted
                                );
                            }

                            if !result.errors.is_empty() {
                                warn!(
                                    "Sync errors for {}: {} errors",
                                    project_path.display(),
                                    result.errors.len()
                                );
                                for error in &result.errors {
                                    error!(
                                        "Sync error in {}: {}",
                                        error.file_path.display(),
                                        error.error
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("Sync failed for {}: {}", project_path.display(), e);
                        }
                    }
                }
            }
        });
    }

    async fn ensure_project_synced(
        &self,
        project_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "ensure_project_synced called for: {}",
            project_path.display()
        );
        let mut projects = self.active_projects.write().await;

        if !projects.contains_key(project_path) {
            info!(
                "Project not found in active projects, initializing: {}",
                project_path.display()
            );
            // Check if project exists
            let db_path = project_path.join(".metis.db");
            if !db_path.exists() {
                return Err(
                    format!("Metis project not found at: {}", project_path.display()).into(),
                );
            }

            // Create document store and sync engine
            let store = metis_core::DocumentStore::new(db_path.to_str().unwrap())
                .await
                .map_err(|e| format!("Failed to open document store: {}", e))?;
            let sync_engine = Arc::new(metis_core::SyncEngine::new(store));

            // Run initial sync
            info!(
                "Running initial sync for project: {}",
                project_path.display()
            );
            match sync_engine.sync_from_filesystem(project_path).await {
                Ok(result) => {
                    info!(
                        "Initial sync completed for {}: {} processed, {} updated, {} deleted",
                        project_path.display(),
                        result.files_processed,
                        result.files_updated,
                        result.files_deleted
                    );
                }
                Err(e) => {
                    warn!("Initial sync failed for {}: {}", project_path.display(), e);
                }
            }

            projects.insert(project_path.clone(), sync_engine);
        }

        Ok(())
    }
}

#[async_trait]
impl ServerHandler for MetisServerHandler {
    async fn handle_list_tools_request(
        &self,
        _request: ListToolsRequest,
        _server: &dyn McpServer,
    ) -> Result<ListToolsResult, rust_mcp_sdk::schema::RpcError> {
        Ok(ListToolsResult {
            tools: MetisTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        request: CallToolRequest,
        _server: &dyn McpServer,
    ) -> Result<CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError> {
        match request.params.name.as_str() {
            "initialize_project" => {
                let tool: InitializeProjectTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                // Initialize project at the specified path
                let base_path = PathBuf::from(&tool.project_path);

                let config = metis_core::ProjectConfig {
                    root_path: base_path.clone(),
                    name: tool.project_name.clone(),
                    description: tool.description.clone(),
                };

                match metis_core::initialize_project(config).await {
                    Ok(metadata) => {
                        // Register project for background sync using the metis directory path
                        let metis_dir = base_path.join("metis");
                        if let Err(e) = self.ensure_project_synced(&metis_dir).await {
                            warn!("Failed to register project for sync: {}", e);
                        } else {
                            info!(
                                "Successfully registered new project for sync: {}",
                                metis_dir.display()
                            );
                        }

                        let response = serde_json::json!({
                            "message": format!("Project '{}' initialized successfully", tool.project_name),
                            "project_path": metadata.project_path,
                            "database_path": metadata.database_path
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to initialize project '{}': {}", tool.project_name, e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "create_document" => {
                let tool: CreateDocumentTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                tool.call_tool().await
            }
            "validate_document" => {
                let tool: ValidateDocumentTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                tool.call_tool().await
            }
            "update_document_content" => {
                let tool: UpdateDocumentContentTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core update function
                match metis_core::update_document_content(
                    &document_path,
                    &tool.section_heading,
                    &tool.new_content,
                )
                .await
                {
                    Ok(()) => {
                        let response = serde_json::json!({
                            "message": format!("Successfully updated section '{}' in document", tool.section_heading),
                            "document_path": document_path,
                            "section_heading": tool.section_heading,
                            "updated": true
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to update document content: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "update_exit_criterion" => {
                let tool: UpdateExitCriterionTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core update function
                match metis_core::update_exit_criterion(
                    &document_path,
                    &tool.criterion_title,
                    tool.completed,
                )
                .await
                {
                    Ok(()) => {
                        let response = serde_json::json!({
                            "message": format!("Successfully updated exit criterion to {}", if tool.completed { "completed" } else { "incomplete" }),
                            "document_path": document_path,
                            "criterion_title": tool.criterion_title,
                            "completed": tool.completed,
                            "updated": true
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to update exit criterion: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "update_blocked_by" => {
                let tool: UpdateBlockedByTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core update function
                match metis_core::update_blocked_by(&document_path, tool.blocked_by.clone()).await {
                    Ok(()) => {
                        let response = serde_json::json!({
                            "message": "Successfully updated blocked_by relationships",
                            "document_path": document_path,
                            "blocked_by": tool.blocked_by,
                            "updated": true
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to update blocked_by: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "transition_phase" => {
                let tool: TransitionPhaseTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core phase transition function
                match metis_core::transition_phase(
                    &document_path,
                    &tool.new_phase,
                    tool.force.unwrap_or(false),
                )
                .await
                {
                    Ok(result_message) => {
                        let response = serde_json::json!({
                            "message": result_message,
                            "document_path": document_path,
                            "new_phase": tool.new_phase,
                            "force": tool.force.unwrap_or(false),
                            "updated": true
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to transition phase: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "check_phase_transition" => {
                let tool: CheckPhaseTransitionTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core phase checking function
                match metis_core::can_transition_to_phase(&document_path, &tool.target_phase).await
                {
                    Ok(can_transition) => {
                        let response = serde_json::json!({
                            "message": "Phase transition check completed",
                            "document_path": document_path,
                            "target_phase": tool.target_phase,
                            "can_transition": can_transition,
                            "valid": can_transition
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to check phase transition: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "validate_exit_criteria" => {
                let tool: ValidateExitCriteriaTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                let project_path = PathBuf::from(&tool.project_path);
                let document_path = project_path.join(&tool.document_path);

                // Use metis-core exit criteria validation function
                match metis_core::validate_exit_criteria(&document_path).await {
                    Ok(result) => {
                        let response = serde_json::json!({
                            "message": "Exit criteria validation completed",
                            "document_path": document_path,
                            "all_complete": result.met,
                            "completed_count": result.completed_criteria,
                            "total_count": result.total_criteria,
                            "missing_criteria": result.missing_criteria,
                            "valid": true
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Failed to validate exit criteria: {}", e)
                        });
                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?,
                        )]))
                    }
                }
            }
            "list_documents" => {
                let tool: ListDocumentsTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                tool.call_tool().await
            }
            "search_documents" => {
                let tool: SearchDocumentsTool = serde_json::from_value(serde_json::Value::Object(
                    request.params.arguments.unwrap_or_default(),
                ))
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                tool.call_tool().await
            }
            "open_vault_in_obsidian" => {
                let tool: OpenVaultInObsidianTool = serde_json::from_value(
                    serde_json::Value::Object(request.params.arguments.unwrap_or_default()),
                )
                .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;

                tool.call_tool().await
            }
            _ => Err(
                rust_mcp_sdk::schema::schema_utils::CallToolError::unknown_tool(
                    request.params.name,
                ),
            ),
        }
    }
}
