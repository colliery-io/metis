use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct TuiCommand {
    // No additional arguments needed for now
}

impl TuiCommand {
    pub async fn execute(&self) -> Result<()> {
        // Call the TUI main function directly
        metis_docs_tui::run().await
    }
}
