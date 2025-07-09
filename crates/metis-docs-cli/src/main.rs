mod cli;
mod commands;
mod utils;
mod workspace;

use anyhow::Result;
use clap::Parser;
use cli::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    // Parse CLI arguments first to get verbosity level
    let cli = Cli::parse();
    
    // Initialize tracing based on verbosity
    cli.init_logging();
    
    // Execute the command
    cli.execute().await
}
