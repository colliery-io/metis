use crate::tools::{
    CreateDocumentTool, InitializeProjectTool, ListDocumentsTool, MetisTools, SearchDocumentsTool,
    TransitionPhaseTool, UpdateBlockedByTool, UpdateDocumentContentTool, UpdateExitCriterionTool,
    ValidateDocumentTool, ValidateExitCriteriaTool,
};
use crate::MetisServerConfig;
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{CallToolRequest, CallToolResult, ListToolsRequest, ListToolsResult},
    McpServer,
};
use std::sync::Arc;
use tracing::info;

pub struct MetisServerHandler {
    #[allow(dead_code)]
    config: Arc<MetisServerConfig>,
}

impl MetisServerHandler {
    pub fn new(config: MetisServerConfig) -> Self {
        info!("Initializing Metis MCP Server");
        Self {
            config: Arc::new(config),
        }
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
        let args = serde_json::Value::Object(request.params.arguments.unwrap_or_default());

        match request.params.name.as_str() {
            "initialize_project" => {
                let tool: InitializeProjectTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "list_documents" => {
                let tool: ListDocumentsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "search_documents" => {
                let tool: SearchDocumentsTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "create_document" => {
                let tool: CreateDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "validate_document" => {
                let tool: ValidateDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "transition_phase" => {
                let tool: TransitionPhaseTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "validate_exit_criteria" => {
                let tool: ValidateExitCriteriaTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "update_document_content" => {
                let tool: UpdateDocumentContentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "update_exit_criterion" => {
                let tool: UpdateExitCriterionTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "update_blocked_by" => {
                let tool: UpdateBlockedByTool = serde_json::from_value(args)
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
