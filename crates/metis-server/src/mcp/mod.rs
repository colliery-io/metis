//! MCP server over the service layer (METIS-T-0132).
//!
//! Exposes the work-item verbs to agents via `rust-mcp-sdk` over **stdio**. The
//! handler holds an [`McpState`] (pool + object store + the actor this session
//! acts as) and each tool calls `metis-core::service::*`. Hosted
//! streamable-HTTP transport with per-connection auth is a follow-up; the tool
//! layer here is transport-agnostic so it can be reused.

pub mod tools;

use std::sync::Arc;

use async_trait::async_trait;
use metis_core::actor::Actor;
use metis_core::service::user;
use rust_mcp_sdk::mcp_server::{server_runtime, McpServerOptions, ServerHandler};
use rust_mcp_sdk::schema::{
    schema_utils::CallToolError, CallToolRequestParams, CallToolResult, Implementation,
    InitializeResult, ListToolsResult, PaginatedRequestParams, RpcError, ServerCapabilities,
    ServerCapabilitiesTools, LATEST_PROTOCOL_VERSION,
};
use rust_mcp_sdk::{McpServer, StdioTransport, ToMcpServerHandler, TransportOptions};

use crate::state::AppState;
use tools::{
    ArchiveItemTool, CreateItemTool, CreateProjectTool, EditItemTool, LinkItemTool, ListItemsTool,
    McpState, MetisItemTools, ReadItemTool, ReadProjectTool, TransitionPhaseTool,
};

/// Resolve the session actor and build the MCP state from an [`AppState`].
///
/// In local mode the implicit local user acts. In team mode a bearer token is
/// required and resolved to its [`Actor`] (so events are attributed correctly).
pub fn mcp_state_from(app: &AppState, token: Option<String>) -> anyhow::Result<McpState> {
    let actor = if app.config.local {
        Actor::human(app.local_user.expect("local mode sets local_user"))
    } else {
        let token =
            token.ok_or_else(|| anyhow::anyhow!("team mode requires --token (or use --local)"))?;
        let mut conn = app.pool.get()?;
        user::resolve_actor(&mut conn, &token)?
            .ok_or_else(|| anyhow::anyhow!("invalid or revoked token"))?
    };
    Ok(McpState {
        pool: app.pool.clone(),
        store: app.store.clone(),
        actor,
    })
}

/// Run the MCP server on stdio until the client disconnects.
pub async fn run_stdio(state: McpState) -> anyhow::Result<()> {
    let server_details = InitializeResult {
        server_info: Implementation {
            name: "Metis".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            title: Some("Metis MCP Server".to_string()),
            description: Some("Work-item management over the Metis 3.0 service layer".to_string()),
            icons: vec![],
            website_url: None,
        },
        capabilities: ServerCapabilities {
            tools: Some(ServerCapabilitiesTools { list_changed: None }),
            ..Default::default()
        },
        meta: None,
        instructions: Some(
            "Metis work-item tools. Items are addressed by short code (e.g. METIS-T-0001). \
             Create items under a project slug with a type from the project config; transitions \
             enforce the type's workflow unless forced."
                .to_string(),
        ),
        protocol_version: LATEST_PROTOCOL_VERSION.to_string(),
    };

    let transport = StdioTransport::new(TransportOptions::default())
        .map_err(|e| anyhow::anyhow!("failed to create stdio transport: {e}"))?;
    let handler = MetisMcpHandler {
        state: Arc::new(state),
    }
    .to_mcp_server_handler();

    let server = server_runtime::create_server(McpServerOptions {
        server_details,
        transport,
        handler,
        task_store: None,
        client_task_store: None,
    });
    server
        .start()
        .await
        .map_err(|e| anyhow::anyhow!("MCP server failed: {e}"))?;
    Ok(())
}

/// The MCP request handler: lists tools and dispatches calls by name.
pub struct MetisMcpHandler {
    state: Arc<McpState>,
}

#[async_trait]
impl ServerHandler for MetisMcpHandler {
    async fn handle_list_tools_request(
        &self,
        _params: Option<PaginatedRequestParams>,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<ListToolsResult, RpcError> {
        Ok(ListToolsResult {
            tools: MetisItemTools::tools(),
            meta: None,
            next_cursor: None,
        })
    }

    async fn handle_call_tool_request(
        &self,
        params: CallToolRequestParams,
        _runtime: Arc<dyn McpServer>,
    ) -> Result<CallToolResult, CallToolError> {
        let args = serde_json::Value::Object(params.arguments.clone().unwrap_or_default());
        let state = &self.state;

        macro_rules! dispatch {
            ($($name:literal => $ty:ty),* $(,)?) => {
                match params.name.as_str() {
                    $(
                        $name => {
                            let tool: $ty = serde_json::from_value(args).map_err(CallToolError::new)?;
                            tool.run(state).await
                        }
                    )*
                    _ => Err(CallToolError::unknown_tool(params.name.clone())),
                }
            };
        }

        dispatch! {
            "create_project" => CreateProjectTool,
            "read_project" => ReadProjectTool,
            "list_items" => ListItemsTool,
            "read_item" => ReadItemTool,
            "create_item" => CreateItemTool,
            "edit_item" => EditItemTool,
            "transition_phase" => TransitionPhaseTool,
            "link_item" => LinkItemTool,
            "archive_item" => ArchiveItemTool,
        }
    }
}
