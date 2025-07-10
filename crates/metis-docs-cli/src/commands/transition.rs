use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{
    application::services::workspace::PhaseTransitionService,
    domain::documents::types::Phase,
};

#[derive(Args)]
pub struct TransitionCommand {
    /// Document ID to transition
    pub document_id: String,
    
    /// Target phase to transition to (optional - if not provided, transitions to next phase)
    pub phase: Option<String>,
    
    /// Document type (vision, strategy, initiative, task, adr)
    #[arg(short = 't', long)]
    pub document_type: Option<String>,
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
            transition_service.transition_document(&self.document_id, target_phase).await?
        } else {
            transition_service.transition_to_next_phase(&self.document_id).await?
        };
        
        // 4. Report success
        println!("✓ Transitioned {} '{}' from {} to {}", 
            result.document_type, 
            result.document_id, 
            result.from_phase, 
            result.to_phase
        );
        
        // 5. TODO: Auto-sync workspace after transition
        
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
    use tempfile::tempdir;
    use crate::commands::InitCommand;
    use metis_core::{Vision, Strategy, Initiative, Document};
    
    #[test]
    fn test_parse_phase() {
        let cmd = TransitionCommand {
            document_id: "test".to_string(),
            phase: Some("active".to_string()),
            document_type: None,
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
            document_id: "test-doc".to_string(),
            phase: Some("active".to_string()),
            document_type: None,
        };
        
        let result = cmd.execute().await;
        
        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
        
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Not in a Metis workspace"));
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
        };
        init_cmd.execute().await.unwrap();
        
        let cmd = TransitionCommand {
            document_id: "non-existent-doc".to_string(),
            phase: Some("active".to_string()),
            document_type: None,
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
        };
        init_cmd.execute().await.unwrap();
        
        let vision_path = temp_dir.path().join(".metis").join("vision.md");
        let doc_id = "test-project";
        
        // Verify initial state (Draft)
        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Draft);
        
        // 1. Auto-transition: Draft → Review
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("vision".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Review);
        
        // 2. Auto-transition: Review → Published  
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("vision".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Published);
        
        // 3. Test auto-transition (should fail at Published - final phase)
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("vision".to_string()),
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
        
        let doc_id = "test-strategy";
        
        // Find strategy file path
        let strategies_dir = temp_dir.path().join(".metis").join("strategies");
        let strategy_dir = std::fs::read_dir(&strategies_dir).unwrap()
            .find(|entry| entry.as_ref().unwrap().path().is_dir())
            .unwrap().unwrap().path();
        let strategy_path = strategy_dir.join("strategy.md");
        
        // Verify initial state (Shaping)
        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Shaping);
        
        // 1. Auto-transition: Shaping → Design
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("strategy".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Design);
        
        // 2. Auto-transition: Design → Ready
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("strategy".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Ready);
        
        // 3. Auto-transition: Ready → Active
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("strategy".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let strategy = Strategy::from_file(&strategy_path).await.unwrap();
        assert_eq!(strategy.phase().unwrap(), Phase::Active);
        
        // 4. Auto-transition: Active → Completed
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("strategy".to_string()),
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
                strategy: "parent-strategy".to_string(),
            },
        };
        create_initiative_cmd.execute().await.unwrap();
        
        let doc_id = "test-initiative";
        
        // Check what phase the initiative actually starts with
        let initiative_path = temp_dir.path().join(".metis/strategies/parent-strategy/initiatives/test-initiative/initiative.md");
        let initiative = Initiative::from_file(&initiative_path).await.unwrap();
        println!("Initiative starts with phase: {:?}", initiative.phase());
        
        // 1. Auto-transition: Discovery → Shaping
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("initiative".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 2. Auto-transition: Shaping → Decompose
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("initiative".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 3. Auto-transition: Decompose → Active
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("initiative".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 4. Auto-transition: Active → Completed
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("initiative".to_string()),
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
                strategy: "parent-strategy".to_string(),
            },
        };
        create_initiative_cmd.execute().await.unwrap();
        
        // Create task
        let create_task_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Task {
                title: "Test Task".to_string(),
                initiative: "parent-initiative".to_string(),
            },
        };
        create_task_cmd.execute().await.unwrap();
        
        let doc_id = "test-task";
        
        // 1. Todo → Active
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: Some("active".to_string()),
            document_type: Some("task".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 2. Active → Completed
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: Some("completed".to_string()),
            document_type: Some("task".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 3. Test blocking workflow: Todo → Blocked → Active
        // Create another task
        let create_task_cmd2 = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Task {
                title: "Blocked Task".to_string(),
                initiative: "parent-initiative".to_string(),
            },
        };
        create_task_cmd2.execute().await.unwrap();
        
        let blocked_doc_id = "blocked-task";
        
        // Todo → Blocked
        let cmd = TransitionCommand {
            document_id: blocked_doc_id.to_string(),
            phase: Some("blocked".to_string()),
            document_type: Some("task".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // Blocked → Active (this tests the blocked → unblocked workflow)
        let cmd = TransitionCommand {
            document_id: blocked_doc_id.to_string(),
            phase: Some("active".to_string()),
            document_type: Some("task".to_string()),
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
        };
        init_cmd.execute().await.unwrap();
        
        // Create ADR
        let create_adr_cmd = crate::commands::CreateCommand {
            document_type: crate::commands::create::CreateCommands::Adr {
                title: "Test ADR".to_string(),
            },
        };
        create_adr_cmd.execute().await.unwrap();
        
        let doc_id = "001-test-adr";
        
        // 1. Auto-transition: Draft → Discussion
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("adr".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 2. Auto-transition: Discussion → Decided
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("adr".to_string()),
        };
        cmd.execute().await.unwrap();
        
        // 3. Test that decided ADRs cannot be transitioned further
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: Some("superseded".to_string()),
            document_type: Some("adr".to_string()),
        };
        let result = cmd.execute().await;
        assert!(result.is_err()); // Should fail as Decided has no valid transitions
        
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
        };
        init_cmd.execute().await.unwrap();
        
        let doc_id = "test-project";
        
        // Test invalid vision transition: Draft → Published (must go through Review)
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: Some("published".to_string()),
            document_type: Some("vision".to_string()),
        };
        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid phase transition"));
        
        // Test transition to invalid phase
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: Some("invalid-phase".to_string()),
            document_type: Some("vision".to_string()),
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
        };
        init_cmd.execute().await.unwrap();
        
        let doc_id = "test-project";
        let vision_path = temp_dir.path().join(".metis").join("vision.md");
        
        // Test auto-transition (no phase specified): Draft → Review
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("vision".to_string()),
        };
        cmd.execute().await.unwrap();
        
        let vision = Vision::from_file(&vision_path).await.unwrap();
        assert_eq!(vision.phase().unwrap(), Phase::Review);
        
        // Test auto-transition: Review → Published
        let cmd = TransitionCommand {
            document_id: doc_id.to_string(),
            phase: None, // Auto transition
            document_type: Some("vision".to_string()),
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