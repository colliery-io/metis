use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing_subscriber::filter::LevelFilter;

use crate::commands::{
    ArchiveCommand, ConfigCommand, CreateCommand, InitCommand, ListCommand, McpCommand,
    SearchCommand, StatusCommand, SyncCommand, TransitionCommand, TuiCommand, ValidateCommand,
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
    /// Launch the interactive TUI interface
    Tui(TuiCommand),
    /// Launch the MCP server for external integrations
    Mcp(McpCommand),
    /// Manage flight level configuration
    Config(ConfigCommand),
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
            Commands::Tui(cmd) => cmd.execute().await,
            Commands::Mcp(cmd) => cmd.execute().await,
            Commands::Config(cmd) => cmd.execute().await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::create::CreateCommands;
    use crate::commands::{
        ArchiveCommand, CreateCommand, ListCommand, SearchCommand, StatusCommand, SyncCommand,
        TransitionCommand, ValidateCommand,
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
            preset: None,
            strategies: None,
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

        // 3. Create a strategy
        let create_strategy_cmd = CreateCommand {
            document_type: CreateCommands::Strategy {
                title: "Test Strategy for Integration".to_string(),
                vision: Some("integration-test-project".to_string()),
            },
        };
        create_strategy_cmd
            .execute()
            .await
            .expect("Failed to create strategy");

        // 4. Create an initiative under the strategy
        let create_initiative_cmd = CreateCommand {
            document_type: CreateCommands::Initiative {
                title: "Test Initiative".to_string(),
                strategy: "TEST-S-0001".to_string(),
            },
        };
        create_initiative_cmd
            .execute()
            .await
            .expect("Failed to create initiative");

        // 5. Create a task under the initiative
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

        // 6. Create an ADR
        let create_adr_cmd = CreateCommand {
            document_type: CreateCommands::Adr {
                title: "Test Architecture Decision".to_string(),
            },
        };
        create_adr_cmd
            .execute()
            .await
            .expect("Failed to create ADR");

        // Find the created ADR (it will have a number prefix)
        let adrs_dir = metis_dir.join("adrs");
        let adr_files: Vec<_> = fs::read_dir(&adrs_dir)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
            .collect();
        assert!(!adr_files.is_empty(), "ADR file should be created");

        // 7. Sync after creating documents
        let sync_cmd2 = SyncCommand {};
        sync_cmd2
            .execute()
            .await
            .expect("Failed to sync after creating documents");

        // 8. Transition the vision to review phase
        let transition_vision_cmd = TransitionCommand {
            document_id: "integration-test-project".to_string(),
            document_type: Some("vision".to_string()),
            phase: Some("review".to_string()),
        };
        transition_vision_cmd
            .execute()
            .await
            .expect("Failed to transition vision");

        // 9. Transition the strategy through its phases: Shaping → Design → Ready → Active
        let transition_strategy_to_design_cmd = TransitionCommand {
            document_id: "test-strategy-for-integration".to_string(),
            document_type: Some("strategy".to_string()),
            phase: Some("design".to_string()),
        };
        transition_strategy_to_design_cmd
            .execute()
            .await
            .expect("Failed to transition strategy to design");

        let transition_strategy_to_ready_cmd = TransitionCommand {
            document_id: "test-strategy-for-integration".to_string(),
            document_type: Some("strategy".to_string()),
            phase: Some("ready".to_string()),
        };
        transition_strategy_to_ready_cmd
            .execute()
            .await
            .expect("Failed to transition strategy to ready");

        let transition_strategy_to_active_cmd = TransitionCommand {
            document_id: "test-strategy-for-integration".to_string(),
            document_type: Some("strategy".to_string()),
            phase: Some("active".to_string()),
        };
        transition_strategy_to_active_cmd
            .execute()
            .await
            .expect("Failed to transition strategy to active");

        // 10. Transition the task through its phases: Todo → Active → Completed
        let transition_task_to_active_cmd = TransitionCommand {
            document_id: "test-task".to_string(),
            document_type: Some("task".to_string()),
            phase: Some("active".to_string()),
        };
        transition_task_to_active_cmd
            .execute()
            .await
            .expect("Failed to transition task to active");

        let transition_task_to_completed_cmd = TransitionCommand {
            document_id: "test-task".to_string(),
            document_type: Some("task".to_string()),
            phase: Some("completed".to_string()),
        };
        transition_task_to_completed_cmd
            .execute()
            .await
            .expect("Failed to transition task to completed");

        // 11. Archive the completed task
        let archive_task_cmd = ArchiveCommand {
            document_id: "test-task".to_string(),
            document_type: Some("task".to_string()),
        };
        archive_task_cmd
            .execute()
            .await
            .expect("Failed to archive task");

        // 12. List all documents to verify they exist
        let list_cmd = ListCommand {
            document_type: None,
            phase: None,
            all: true,
            include_archived: true,
        };
        list_cmd.execute().await.expect("Failed to list documents");

        // 13. Test status command
        let status_cmd = StatusCommand {
            include_archived: false,
        };
        status_cmd.execute().await.expect("Failed to get status");

        // 14. Search for content
        let search_cmd = SearchCommand {
            query: "test".to_string(),
            limit: 10,
        };
        search_cmd
            .execute()
            .await
            .expect("Failed to search documents");

        // 15. Validate a document file
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
