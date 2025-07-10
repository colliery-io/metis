use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    metis_docs_tui::run().await
}
