use crate::tools::{
    ArchiveDocumentTool, CreateDocumentTool, EditDocumentTool, InitializeProjectTool,
    ListDocumentsTool, MetisTools, ReadDocumentTool, SearchDocumentsTool, TransitionPhaseTool,
};
use crate::MetisServerConfig;
use async_trait::async_trait;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError},
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
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: MetisTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, rust_mcp_sdk::schema::schema_utils::CallToolError> {
        let args = serde_json::Value::Object(params.arguments.unwrap_or_default());

        match params.name.as_str() {
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
            "read_document" => {
                let tool: ReadDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "create_document" => {
                let tool: CreateDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "transition_phase" => {
                let tool: TransitionPhaseTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "edit_document" => {
                let tool: EditDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "archive_document" => {
                let tool: ArchiveDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            _ => Err(
                rust_mcp_sdk::schema::schema_utils::CallToolError::unknown_tool(
                    params.name,
                ),
            ),
        }
    }
}
