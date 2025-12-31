use metis_core::application::services::workspace::{
    BacklogCategory, ReassignmentService, WorkspaceInitializationService,
};
use metis_core::application::services::DatabaseService;
use metis_core::domain::documents::types::DocumentType;
use std::fs;
use tempfile::tempdir;

/// Helper to create a test workspace with vision, initiative, and task
async fn setup_test_workspace() -> (tempfile::TempDir, std::path::PathBuf) {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();

    // Initialize workspace - this creates vision with prefix TEST
    let result = WorkspaceInitializationService::initialize_workspace(base_path, "Test Project")
        .await
        .expect("Failed to initialize workspace");

    let metis_dir = result.metis_dir.clone();

    // The vision gets short code TEST-V-0001 from initialization
    let vision_short_code = "TEST-V-0001";

    // Create an initiative directory and file
    let init_dir = metis_dir.join("strategies/NULL/initiatives/test-initiative");
    fs::create_dir_all(&init_dir).expect("Failed to create initiative dir");
    fs::create_dir_all(init_dir.join("tasks")).expect("Failed to create tasks dir");

    let init_file = init_dir.join("initiative.md");
    let init_content = format!(
        "---\n\
id: test-initiative\n\
level: initiative\n\
title: \"Test Initiative\"\n\
short_code: \"TEST-I-0001\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: {}\n\
blocked_by: []\n\
archived: false\n\
\n\
tags:\n\
  - \"#initiative\"\n\
  - \"#phase/decompose\"\n\
\n\
exit_criteria_met: false\n\
estimated_complexity: M\n\
strategy_id: NULL\n\
initiative_id: test-initiative\n\
---\n\
\n\
# Test Initiative\n\
\n\
This is a test initiative.\n",
        vision_short_code
    );
    fs::write(&init_file, init_content).expect("Failed to write initiative");

    // Create a backlog task
    let backlog_dir = metis_dir.join("backlog/features");
    fs::create_dir_all(&backlog_dir).expect("Failed to create backlog dir");

    let task_file = backlog_dir.join("TEST-T-0001.md");
    let task_content = "---\n\
id: test-task\n\
level: task\n\
title: \"Test Task\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: \n\
blocked_by: []\n\
archived: false\n\
tags:\n\
  - \"#task\"\n\
  - \"#phase/backlog\"\n\
  - \"#feature\"\n\
exit_criteria_met: false\n\
---\n\
\n\
# Test Task\n\
\n\
This is a test task.\n";
    fs::write(&task_file, task_content).expect("Failed to write task");

    (temp_dir, metis_dir)
}

/// Test reassigning a backlog task to an initiative
#[tokio::test]
async fn test_reassign_backlog_to_initiative() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Sync the workspace to pick up the new documents
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Verify source exists at original location
    let source_path = metis_dir.join("backlog/features/TEST-T-0001.md");
    assert!(source_path.exists(), "Source task should exist in backlog");

    // Reassign to initiative
    let result = reassignment_service
        .reassign_to_initiative("TEST-T-0001", "TEST-I-0001", &mut db_service)
        .await
        .expect("Failed to reassign task");

    // Verify the move
    assert_eq!(result.short_code, "TEST-T-0001");
    assert!(!source_path.exists(), "Task should no longer be in backlog");

    let dest_path = metis_dir.join("strategies/NULL/initiatives/test-initiative/tasks/TEST-T-0001.md");
    assert!(dest_path.exists(), "Task should be in initiative tasks folder");
}

/// Test reassigning a task from initiative to backlog
#[tokio::test]
async fn test_reassign_task_to_backlog() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // First move the task into an initiative
    let source_path = metis_dir.join("backlog/features/TEST-T-0001.md");
    let init_task_path = metis_dir.join("strategies/NULL/initiatives/test-initiative/tasks/TEST-T-0001.md");

    // Manually move the file for this test
    fs::rename(&source_path, &init_task_path).expect("Failed to move task");

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Reassign back to backlog as tech-debt
    let result = reassignment_service
        .reassign_to_backlog("TEST-T-0001", BacklogCategory::TechDebt, &mut db_service)
        .await
        .expect("Failed to reassign task to backlog");

    // Verify the move
    assert_eq!(result.short_code, "TEST-T-0001");
    assert!(!init_task_path.exists(), "Task should no longer be in initiative");

    let backlog_path = metis_dir.join("backlog/tech-debt/TEST-T-0001.md");
    assert!(backlog_path.exists(), "Task should be in tech-debt backlog");
}

/// Test reassigning a task between initiatives
#[tokio::test]
async fn test_reassign_between_initiatives() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Create a second initiative
    let init2_dir = metis_dir.join("strategies/NULL/initiatives/second-initiative");
    fs::create_dir_all(&init2_dir).expect("Failed to create second initiative dir");
    fs::create_dir_all(init2_dir.join("tasks")).expect("Failed to create tasks dir");

    let init2_file = init2_dir.join("initiative.md");
    let init2_content = "---\n\
id: second-initiative\n\
level: initiative\n\
title: \"Second Initiative\"\n\
short_code: \"TEST-I-0002\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: TEST-V-0001\n\
blocked_by: []\n\
archived: false\n\
\n\
tags:\n\
  - \"#initiative\"\n\
  - \"#phase/active\"\n\
\n\
exit_criteria_met: false\n\
estimated_complexity: M\n\
strategy_id: NULL\n\
initiative_id: second-initiative\n\
---\n\
\n\
# Second Initiative\n";
    fs::write(&init2_file, init2_content).expect("Failed to write second initiative");

    // Move task from backlog to first initiative
    let source_path = metis_dir.join("backlog/features/TEST-T-0001.md");
    let init1_task_path = metis_dir.join("strategies/NULL/initiatives/test-initiative/tasks/TEST-T-0001.md");
    fs::rename(&source_path, &init1_task_path).expect("Failed to move task to init1");

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Reassign from first initiative to second initiative
    let result = reassignment_service
        .reassign_to_initiative("TEST-T-0001", "TEST-I-0002", &mut db_service)
        .await
        .expect("Failed to reassign task between initiatives");

    // Verify the move
    assert_eq!(result.short_code, "TEST-T-0001");
    assert!(!init1_task_path.exists(), "Task should no longer be in first initiative");

    let init2_task_path = metis_dir.join("strategies/NULL/initiatives/second-initiative/tasks/TEST-T-0001.md");
    assert!(init2_task_path.exists(), "Task should be in second initiative");
}

/// Test reassigning a task between initiatives under different strategies
#[tokio::test]
async fn test_reassign_across_strategies() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Create a strategy directory structure
    let strategy_dir = metis_dir.join("strategies/test-strategy");
    fs::create_dir_all(&strategy_dir).expect("Failed to create strategy dir");

    let strategy_file = strategy_dir.join("strategy.md");
    let strategy_content = "---\n\
id: test-strategy\n\
level: strategy\n\
title: \"Test Strategy\"\n\
short_code: \"TEST-S-0001\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: TEST-V-0001\n\
blocked_by: []\n\
archived: false\n\
\n\
tags:\n\
  - \"#strategy\"\n\
  - \"#phase/active\"\n\
\n\
exit_criteria_met: false\n\
success_metrics: []\n\
risk_level: medium\n\
stakeholders: []\n\
---\n\
\n\
# Test Strategy\n";
    fs::write(&strategy_file, strategy_content).expect("Failed to write strategy");

    // Create initiative under the strategy
    let init_under_strategy_dir = strategy_dir.join("initiatives/strategy-initiative");
    fs::create_dir_all(&init_under_strategy_dir).expect("Failed to create initiative dir");
    fs::create_dir_all(init_under_strategy_dir.join("tasks")).expect("Failed to create tasks dir");

    let init_file = init_under_strategy_dir.join("initiative.md");
    let init_content = "---\n\
id: strategy-initiative\n\
level: initiative\n\
title: \"Strategy Initiative\"\n\
short_code: \"TEST-I-0003\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: TEST-S-0001\n\
blocked_by: []\n\
archived: false\n\
\n\
tags:\n\
  - \"#initiative\"\n\
  - \"#phase/decompose\"\n\
\n\
exit_criteria_met: false\n\
estimated_complexity: M\n\
strategy_id: TEST-S-0001\n\
initiative_id: strategy-initiative\n\
---\n\
\n\
# Strategy Initiative\n";
    fs::write(&init_file, init_content).expect("Failed to write initiative");

    // Move task from backlog to first initiative (under NULL strategy)
    let source_path = metis_dir.join("backlog/features/TEST-T-0001.md");
    let init1_task_path = metis_dir.join("strategies/NULL/initiatives/test-initiative/tasks/TEST-T-0001.md");
    fs::rename(&source_path, &init1_task_path).expect("Failed to move task to init1");

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Reassign from NULL strategy initiative to test-strategy initiative
    let result = reassignment_service
        .reassign_to_initiative("TEST-T-0001", "TEST-I-0003", &mut db_service)
        .await
        .expect("Failed to reassign task across strategies");

    // Verify the move
    assert_eq!(result.short_code, "TEST-T-0001");
    assert!(!init1_task_path.exists(), "Task should no longer be in NULL strategy initiative");

    let strategy_task_path = metis_dir.join("strategies/test-strategy/initiatives/strategy-initiative/tasks/TEST-T-0001.md");
    assert!(strategy_task_path.exists(), "Task should be in strategy initiative");
}

/// Test that reassignment fails for non-task documents
#[tokio::test]
async fn test_reassign_non_task_fails() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Try to reassign the initiative (should fail)
    let result = reassignment_service
        .reassign_to_backlog("TEST-I-0001", BacklogCategory::Feature, &mut db_service)
        .await;

    assert!(result.is_err(), "Should not be able to reassign initiative");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Only tasks"),
        "Error should mention only tasks can be reassigned"
    );
}

/// Test that reassignment to non-initiative parent fails
#[tokio::test]
async fn test_reassign_to_non_initiative_fails() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Get the vision short code
    let visions = db_service.find_by_type(DocumentType::Vision).expect("Failed to get visions");
    let vision_code = &visions[0].short_code;

    // Try to reassign to vision (should fail)
    let result = reassignment_service
        .reassign_to_initiative("TEST-T-0001", vision_code, &mut db_service)
        .await;

    assert!(result.is_err(), "Should not be able to reassign to vision");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("must be an initiative"),
        "Error should mention parent must be initiative"
    );
}

/// Test that reassignment to initiative in wrong phase fails
#[tokio::test]
async fn test_reassign_to_wrong_phase_initiative_fails() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Modify initiative to be in discovery phase
    let init_file = metis_dir.join("strategies/NULL/initiatives/test-initiative/initiative.md");
    let content = fs::read_to_string(&init_file).expect("Failed to read initiative");
    let updated = content.replace("#phase/decompose", "#phase/discovery");
    fs::write(&init_file, updated).expect("Failed to update initiative");

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Try to reassign to initiative in discovery phase (should fail)
    let result = reassignment_service
        .reassign_to_initiative("TEST-T-0001", "TEST-I-0001", &mut db_service)
        .await;

    assert!(result.is_err(), "Should not be able to reassign to discovery phase initiative");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("phase"),
        "Error should mention phase requirement: {}",
        err
    );
}

/// Test reassignment with missing source document
#[tokio::test]
async fn test_reassign_missing_document_fails() {
    let (_temp_dir, metis_dir) = setup_test_workspace().await;

    // Sync workspace
    let detection_service =
        metis_core::application::services::workspace::WorkspaceDetectionService::new();
    let synced_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace");

    let mut db_service = DatabaseService::new(synced_db.into_repository());
    let reassignment_service = ReassignmentService::new(&metis_dir);

    // Try to reassign non-existent document
    let result = reassignment_service
        .reassign_to_initiative("NONEXISTENT-T-9999", "TEST-I-0001", &mut db_service)
        .await;

    assert!(result.is_err(), "Should fail for non-existent document");
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("not found"),
        "Error should mention document not found"
    );
}
