mod adr;
mod initiative;
mod task;

use crate::commands::SyncCommand;
use anyhow::Result;
use clap::{Args, Subcommand};

#[derive(Args)]
pub struct CreateCommand {
    #[command(subcommand)]
    pub document_type: CreateCommands,
}

#[derive(Subcommand)]
pub enum CreateCommands {
    /// Create a new initiative document
    Initiative {
        /// Initiative title
        title: String,
        /// Parent vision short code (e.g., PROJ-V-0001)
        #[arg(short, long)]
        vision: String,
    },
    /// Create a new task document
    Task {
        /// Task title
        title: String,
        /// Parent initiative ID
        #[arg(short, long)]
        initiative: String,
    },
    /// Create a new ADR document
    Adr {
        /// ADR title
        title: String,
    },
}

impl CreateCommand {
    pub async fn execute(&self) -> Result<()> {
        match &self.document_type {
            CreateCommands::Initiative { title, vision } => {
                initiative::create_new_initiative(title, vision).await?;
            }
            CreateCommands::Task { title, initiative } => {
                task::create_new_task(title, initiative).await?;
            }
            CreateCommands::Adr { title } => {
                adr::create_new_adr(title).await?;
            }
        }

        // Auto-sync after creating documents to update the database index
        println!("\nSyncing workspace...");
        let sync_cmd = SyncCommand {};
        sync_cmd.execute().await?;

        Ok(())
    }
}
