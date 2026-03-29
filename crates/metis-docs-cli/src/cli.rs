use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::filter::LevelFilter;

use crate::commands::{
    ArchiveCommand, ConfigCommand, CreateCommand, IndexCommand, InitCommand, ListCommand,
    McpCommand, SearchCommand, StatusCommand, SyncCommand, TransitionCommand, ValidateCommand,
};

#[derive(Parser)]
#[command(name = "metis")]
#[command(about = "A document management system for strategic planning")]
#[command(version)]
pub struct Cli {
    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new Metis workspace
    Init(InitCommand),
    /// Synchronize workspace with file system
    Sync(SyncCommand),
    /// Create new documents
    Create(CreateCommand),
    /// Search documents in the workspace
    Search(SearchCommand),
    /// Transition documents between phases
    Transition(TransitionCommand),
    /// List documents in the workspace
    List(ListCommand),
    /// Show workspace status and actionable items
    Status(StatusCommand),
    /// Archive completed documents and move them to archived folder
    Archive(ArchiveCommand),
    /// Validate a document file
    Validate(ValidateCommand),
    /// Launch the MCP server for external integrations
    Mcp(McpCommand),
    /// Manage flight level configuration
    Config(ConfigCommand),
    /// Generate code index for AI agent navigation
    Index(IndexCommand),
}

impl Cli {
    pub fn init_logging(&self) {
        let level = match self.verbose {
            0 => LevelFilter::WARN,
            1 => LevelFilter::INFO,
            2 => LevelFilter::DEBUG,
            _ => LevelFilter::TRACE,
        };

        tracing_subscriber::fmt()
            .with_max_level(level)
            .with_target(false)
            .init();
    }

    pub async fn execute(&self) -> Result<()> {
        match &self.command {
            Commands::Init(cmd) => cmd.execute().await,
            Commands::Sync(cmd) => cmd.execute().await,
            Commands::Create(cmd) => cmd.execute().await,
            Commands::Search(cmd) => cmd.execute().await,
            Commands::Transition(cmd) => cmd.execute().await,
            Commands::List(cmd) => cmd.execute().await,
            Commands::Status(cmd) => cmd.execute().await,
            Commands::Archive(cmd) => cmd.execute().await,
            Commands::Validate(cmd) => cmd.execute().await,
            Commands::Mcp(cmd) => cmd.execute().await,
            Commands::Config(cmd) => cmd.execute().await,
            Commands::Index(cmd) => cmd.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::create::CreateCommands;
    use crate::commands::{
        ArchiveCommand, CreateCommand, ListCommand, OutputFormat, SearchCommand, StatusCommand,
        SyncCommand, TransitionCommand, ValidateCommand,
    };
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_comprehensive_cli_workflow() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // 1. Initialize a new project
        let init_cmd = InitCommand {
            name: Some("Integration Test Project".to_string()),
            prefix: None,
            preset: None,
            initiatives: None,
        };
        init_cmd
            .execute()
            .await
            .expect("Failed to initialize project");

        let metis_dir = temp_dir.path().join(".metis");
        assert!(
            metis_dir.exists(),
            "Metis directory should exist after init"
        );
        assert!(
            metis_dir.join("vision.md").exists(),
            "Vision document should be created"
        );

        // 2. Sync the workspace to populate database
        let sync_cmd = SyncCommand {};
        sync_cmd.execute().await.expect("Failed to sync workspace");

        // 3. Create an initiative under the vision
        let create_initiative_cmd = CreateCommand {
            document_type: CreateCommands::Initiative {
                title: "Test Initiative".to_string(),
                vision: "TEST-V-0001".to_string(),
            },
        };
        create_initiative_cmd
            .execute()
            .await
            .expect("Failed to create initiative");

        // 4. Create an ADR
        let create_adr_cmd = CreateCommand {
            document_type: CreateCommands::Adr {
                title: "Test Architecture Decision".to_string(),
            },
        };
        create_adr_cmd
            .execute()
            .await
            .expect("Failed to create ADR");

        // Find the created ADR
        let adrs_dir = metis_dir.join("adrs");
        let adr_files: Vec<_> = fs::read_dir(&adrs_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
            .collect();
        assert!(!adr_files.is_empty(), "ADR file should be created");

        // 5. Sync after creating documents
        let sync_cmd2 = SyncCommand {};
        sync_cmd2
            .execute()
            .await
            .expect("Failed to sync after creating documents");

        // 6. Transition the vision to review phase
        let transition_vision_cmd = TransitionCommand {
            short_code: "TEST-V-0001".to_string(),
            phase: Some("review".to_string()),
        };
        transition_vision_cmd
            .execute()
            .await
            .expect("Failed to transition vision");

        // 7. Transition initiative through phases to decompose
        // Discovery → Design → Ready → Decompose
        for _i in 0..3 {
            let cmd = TransitionCommand {
                short_code: "TEST-I-0001".to_string(),
                phase: None,
            };
            cmd.execute()
                .await
                .expect("Failed to transition initiative");
        }

        // 8. Create a task under the initiative (now in decompose phase)
        let create_task_cmd = CreateCommand {
            document_type: CreateCommands::Task {
                title: "Test Task".to_string(),
                initiative: "TEST-I-0001".to_string(),
            },
        };
        create_task_cmd
            .execute()
            .await
            .expect("Failed to create task");

        // 9. Transition the task: Todo → Active → Completed
        let transition_task_to_active_cmd = TransitionCommand {
            short_code: "TEST-T-0001".to_string(),
            phase: Some("active".to_string()),
        };
        transition_task_to_active_cmd
            .execute()
            .await
            .expect("Failed to transition task to active");

        let transition_task_to_completed_cmd = TransitionCommand {
            short_code: "TEST-T-0001".to_string(),
            phase: Some("completed".to_string()),
        };
        transition_task_to_completed_cmd
            .execute()
            .await
            .expect("Failed to transition task to completed");

        // 10. Archive the completed task
        let archive_task_cmd = ArchiveCommand {
            short_code: "TEST-T-0001".to_string(),
            document_type: Some("task".to_string()),
        };
        archive_task_cmd
            .execute()
            .await
            .expect("Failed to archive task");

        // 11. List all documents to verify they exist
        let list_cmd = ListCommand {
            document_type: None,
            phase: None,
            all: true,
            include_archived: true,
            format: OutputFormat::Table,
        };
        list_cmd.execute().await.expect("Failed to list documents");

        // 12. Test status command
        let status_cmd = StatusCommand {
            include_archived: false,
            format: OutputFormat::Table,
        };
        status_cmd.execute().await.expect("Failed to get status");

        // 13. Search for content
        let search_cmd = SearchCommand {
            query: "test".to_string(),
            limit: 10,
            format: OutputFormat::Table,
        };
        search_cmd
            .execute()
            .await
            .expect("Failed to search documents");

        // 14. Validate a document file
        let validate_cmd = ValidateCommand {
            file_path: metis_dir.join("vision.md"),
        };
        validate_cmd
            .execute()
            .await
            .expect("Failed to validate document");

        // Restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        println!("✓ Comprehensive CLI workflow test completed successfully");
    }
}
