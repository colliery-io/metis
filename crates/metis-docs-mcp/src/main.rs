use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    metis_mcp_server::run().await
}
