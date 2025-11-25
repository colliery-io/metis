use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{
    application::services::workspace::PhaseTransitionService, domain::documents::types::Phase,
    Application, Database,
};

#[derive(Args)]
pub struct TransitionCommand {
    /// Document short code to transition (e.g., PROJ-V-0001)
    pub short_code: String,

    /// Target phase to transition to (optional - if not provided, transitions to next phase)
    pub phase: Option<String>,
}

impl TransitionCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Create the phase transition service
        let transition_service = PhaseTransitionService::new(&metis_dir);

        // 3. Perform the transition
        let result = if let Some(phase_str) = &self.phase {
            let target_phase = self.parse_phase(phase_str)?;
            transition_service
                .transition_document(&self.short_code, target_phase)
                .await?
        } else {
            transition_service
                .transition_to_next_phase(&self.short_code)
                .await?
        };

        // 4. Report success
        println!(
            "[+] Transitioned {} '{}' from {} to {}",
            result.document_type, result.document_id, result.from_phase, result.to_phase
        );

        // 5. Auto-sync workspace after transition
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database for sync: {}", e))?;
        let app = Application::new(database);
        app.sync_directory(&metis_dir)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync workspace: {}", e))?;

        Ok(())
    }

    fn parse_phase(&self, phase_str: &str) -> Result<Phase> {
        match phase_str.to_lowercase().as_str() {
            "draft" => Ok(Phase::Draft),
            "review" => Ok(Phase::Review),
            "published" => Ok(Phase::Published),
            "discussion" => Ok(Phase::Discussion),
            "decided" => Ok(Phase::Decided),
            "superseded" => Ok(Phase::Superseded),
            "backlog" => Ok(Phase::Backlog),
            "todo" => Ok(Phase::Todo),
            "active" => Ok(Phase::Active),
            "blocked" => Ok(Phase::Blocked),
            "completed" => Ok(Phase::Completed),
            "shaping" => Ok(Phase::Shaping),
            "design" => Ok(Phase::Design),
            "ready" => Ok(Phase::Ready),
            "decompose" => Ok(Phase::Decompose),
            "discovery" => Ok(Phase::Discovery),
            _ => anyhow::bail!("Unknown phase: {}", phase_str),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use metis_core::{Adr, Document, Initiative, Strategy, Task, Vision};
    use tempfile::tempdir;

    #[test]
    fn test_parse_phase() {
        let cmd = TransitionCommand {
            short_code: "test".to_string(),
            phase: Some("active".to_string()),
        };

        assert_eq!(cmd.parse_phase("draft").unwrap(), Phase::Draft);
        assert_eq!(cmd.parse_phase("ACTIVE").unwrap(), Phase::Active);
        assert_eq!(cmd.parse_phase("completed").unwrap(), Phase::Completed);
        assert!(cmd.parse_phase("invalid").is_err());
    }

    #[tokio::test]
    async fn test_transition_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return; // Skip test if we can't change directory
        }

        let cmd = TransitionCommand {
            short_code: "test-doc".to_string(),
            phase: Some("active".to_string()),
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));
    }

    #[tokio::test]
    async fn test_find_document_not_found() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let cmd = TransitionCommand {
            short_code: "TEST-T-9999".to_string(),
            phase: Some("active".to_string()),
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[tokio::test]
    async fn test_vision_full_transition_sequence() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let vision_path = temp_dir.path().join(".metis").join("vision.md");

        // Verify initial state (Draft)
        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Draft);
        let short_code = vision.metadata().short_code.clone();

        // 1. Auto-transition: Draft → Review
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Review);

        // 2. Auto-transition: Review → Published
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Published);

        // 3. Test auto-transition (should fail at Published - final phase)
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        let result = cmd.execute().await;
        assert!(result.is_err()); // Should fail as Published is final

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_strategy_full_transition_sequence() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create strategy using create command
        let create_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Strategy {
                title: "Test Strategy".to_string(),
                vision: None,
            },
        };
        create_cmd.execute().await.unwrap();

        // Find strategy file path
        let strategies_dir = temp_dir.path().join(".metis").join("strategies");
        let strategy_dir = std::fs::read_dir(&strategies_dir)
            .unwrap()
            .find(|entry| entry.as_ref().unwrap().path().is_dir())
            .unwrap()
            .unwrap()
            .path();
        let strategy_path = strategy_dir.join("strategy.md");

        // Verify initial state (Shaping)
        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Shaping);
        let short_code = strategy.metadata().short_code.clone();

        // 1. Auto-transition: Shaping → Design
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Design);

        // 2. Auto-transition: Design → Ready
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Ready);

        // 3. Auto-transition: Ready → Active
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Active);

        // 4. Auto-transition: Active → Completed
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Completed);

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_initiative_full_transition_sequence() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create strategy first
        let create_strategy_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Strategy {
                title: "Parent Strategy".to_string(),
                vision: None,
            },
        };
        create_strategy_cmd.execute().await.unwrap();

        // Create initiative
        let create_initiative_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Initiative {
                title: "Test Initiative".to_string(),
                strategy: "TEST-S-0001".to_string(),
            },
        };
        create_initiative_cmd.execute().await.unwrap();

        // Check what phase the initiative actually starts with
        let initiative_path = temp_dir
            .path()
            .join(".metis/strategies/TEST-S-0001/initiatives/TEST-I-0001/initiative.md");
        let initiative = Initiative::from_file(&initiative_path).await.unwrap();
        println!("Initiative starts with phase: {:?}", initiative.phase());
        let short_code = initiative.metadata().short_code.clone();

        // 1. Auto-transition: Discovery → Shaping
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // 2. Auto-transition: Shaping → Decompose
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // 3. Auto-transition: Decompose → Active
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // 4. Auto-transition: Active → Completed
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_task_full_transition_sequence() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create strategy
        let create_strategy_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Strategy {
                title: "Parent Strategy".to_string(),
                vision: None,
            },
        };
        create_strategy_cmd.execute().await.unwrap();

        // Create initiative
        let create_initiative_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Initiative {
                title: "Parent Initiative".to_string(),
                strategy: "TEST-S-0001".to_string(),
            },
        };
        create_initiative_cmd.execute().await.unwrap();

        // Create task
        let create_task_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Task {
                title: "Test Task".to_string(),
                initiative: "TEST-I-0001".to_string(),
            },
        };
        create_task_cmd.execute().await.unwrap();

        // Load the task to get its short code
        let task_path = temp_dir
            .path()
            .join(".metis/strategies/TEST-S-0001/initiatives/TEST-I-0001/tasks/TEST-T-0001.md");
        let task = Task::from_file(&task_path).await.unwrap();
        let short_code = task.metadata().short_code.clone();

        // 1. Todo → Active
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: Some("active".to_string()),
        };
        cmd.execute().await.unwrap();

        // 2. Active → Completed
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: Some("completed".to_string()),
        };
        cmd.execute().await.unwrap();

        // 3. Test blocking workflow: Todo → Blocked → Active
        // Create another task
        let create_task_cmd2 = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Task {
                title: "Blocked Task".to_string(),
                initiative: "TEST-I-0001".to_string(),
            },
        };
        create_task_cmd2.execute().await.unwrap();

        let blocked_doc_id = "TEST-T-0002";

        // Todo → Blocked
        let cmd = TransitionCommand {
            short_code: blocked_doc_id.to_string(),
            phase: Some("blocked".to_string()),
        };
        cmd.execute().await.unwrap();

        // Blocked → Active (this tests the blocked → unblocked workflow)
        let cmd = TransitionCommand {
            short_code: blocked_doc_id.to_string(),
            phase: Some("active".to_string()),
        };
        cmd.execute().await.unwrap();

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_adr_full_transition_sequence() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Create ADR
        let create_adr_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Adr {
                title: "Test ADR".to_string(),
            },
        };
        create_adr_cmd.execute().await.unwrap();

        // Load the ADR to get its short code
        let adr_path = temp_dir.path().join(".metis/adrs/TEST-A-0001.md");
        let adr = Adr::from_file(&adr_path).await.unwrap();
        let short_code = adr.metadata().short_code.clone();

        // 1. Auto-transition: Draft → Discussion
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // 2. Auto-transition: Discussion → Decided
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        // 3. Test transition from decided to superseded (should work)
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: Some("superseded".to_string()),
        };
        cmd.execute().await.unwrap(); // Should succeed as Decided → Superseded is valid

        // 4. Test that superseded ADRs cannot be transitioned further
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        let result = cmd.execute().await;
        assert!(result.is_err()); // Should fail as Superseded has no valid transitions

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_invalid_transitions() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        // Load the vision to get its short code
        let vision_path = temp_dir.path().join(".metis").join("vision.md");
        let vision = Vision::from_file(&vision_path).await.unwrap();
        let short_code = vision.metadata().short_code.clone();

        // Test invalid vision transition: Draft → Published (must go through Review)
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: Some("published".to_string()),
        };
        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid phase transition"));

        // Test transition to invalid phase
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: Some("invalid-phase".to_string()),
        };
        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown phase"));

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_auto_transitions() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let vision_path = temp_dir.path().join(".metis").join("vision.md");

        // Load the vision to get its short code
        let vision = Vision::from_file(&vision_path).await.unwrap();
        let short_code = vision.metadata().short_code.clone();

        // Test auto-transition (no phase specified): Draft → Review
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Review);

        // Test auto-transition: Review → Published
        let cmd = TransitionCommand {
            short_code: short_code.clone(),
            phase: None, // Auto transition
        };
        cmd.execute().await.unwrap();

        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Published);

        // Always restore original directory
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
