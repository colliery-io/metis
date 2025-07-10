use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct McpCommand {
    /// Log level for the MCP server (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    pub log_level: String,
}

impl McpCommand {
    pub async fn execute(&self) -> Result<()> {
        // Set the log level environment variable
        std::env::set_var("METIS_LOG_LEVEL", &self.log_level);
        
        // Call the MCP server main function directly
        metis_mcp_server::run().await
    }
}