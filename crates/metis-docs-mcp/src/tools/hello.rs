use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};

#[mcp_tool(
    name = "hello_world",
    description = "A simple hello world tool",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct HelloWorldTool {
    /// Name to greet
    pub name: Option<String>,
}

impl HelloWorldTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let name = self.name.as_deref().unwrap_or("World");
        let message = format!("Hello, {}!", name);

        let response = serde_json::json!({
            "message": message,
            "timestamp": chrono::Utc::now().to_rfc3339()
        });

        Ok(CallToolResult::text_content(vec![TextContent::from(
            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
        )]))
    }
}
