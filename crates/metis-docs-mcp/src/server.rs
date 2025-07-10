use crate::tools::{HelloWorldTool, MetisTools};
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
        match request.params.name.as_str() {
            "hello_world" => {
                let tool: HelloWorldTool = serde_json::from_value(
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