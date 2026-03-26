use crate::read_tracker::DocumentReadTracker;
use crate::tools::{
    ArchiveDocumentTool, CreateDocumentTool, EditDocumentTool, IndexCodeTool,
    InitializeProjectTool, ListDocumentsTool, MetisTools, OpenDocumentTool, ReadDocumentTool,
    SearchDocumentsTool, TransitionPhaseTool,
};
use crate::viewer::ViewerDispatcher;
use crate::MetisServerConfig;
use async_trait::async_trait;
use metis_core::domain::configuration::ViewerConfig;
use rust_mcp_sdk::{
    mcp_server::ServerHandler,
    schema::{
        CallToolRequestParams, CallToolResult, ListToolsResult, PaginatedRequestParams, RpcError,
    },
    McpServer,
};
use std::sync::Arc;
use tracing::info;

pub struct MetisServerHandler {
    #[allow(dead_code)]
    config: Arc<MetisServerConfig>,
    read_tracker: Arc<DocumentReadTracker>,
    viewer_dispatcher: Arc<ViewerDispatcher>,
}

impl MetisServerHandler {
    pub fn new(config: MetisServerConfig) -> Self {
        Self::with_viewer_config(config, ViewerConfig::default())
    }

    pub fn with_viewer_config(config: MetisServerConfig, viewer_config: ViewerConfig) -> Self {
        info!("Initializing Metis MCP Server");

        // Build viewer backends
        let backends: Vec<Box<dyn crate::viewer::DocumentViewer>> = vec![
            Box::new(crate::viewer::VscodeViewer::new()),
            Box::new(crate::viewer::SysEditorViewer::new()),
        ];

        Self {
            config: Arc::new(config),
            read_tracker: Arc::new(DocumentReadTracker::new()),
            viewer_dispatcher: Arc::new(ViewerDispatcher::new(viewer_config, backends)),
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
                tool.call_tool_with_tracker(self.read_tracker.clone()).await
            }
            "create_document" => {
                let tool: CreateDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool_with_dispatcher(self.viewer_dispatcher.clone())
                    .await
            }
            "transition_phase" => {
                let tool: TransitionPhaseTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "edit_document" => {
                let tool: EditDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool_with_tracker_and_dispatcher(
                    self.read_tracker.clone(),
                    self.viewer_dispatcher.clone(),
                )
                .await
            }
            "archive_document" => {
                let tool: ArchiveDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "index_code" => {
                let tool: IndexCodeTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool().await
            }
            "open_document" => {
                let tool: OpenDocumentTool = serde_json::from_value(args)
                    .map_err(rust_mcp_sdk::schema::schema_utils::CallToolError::new)?;
                tool.call_tool_with_dispatcher(self.viewer_dispatcher.clone())
                    .await
            }
            _ => Err(rust_mcp_sdk::schema::schema_utils::CallToolError::unknown_tool(params.name)),
        }
    }
}
