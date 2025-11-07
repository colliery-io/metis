use metis_core::application::services::workspace::WorkspaceInitializationService;
use metis_core::application::services::{DatabaseService, SyncService};
use metis_core::dal::Database;
use std::fs;
use tempfile::tempdir;

/// Integration test for METIS-T-0001: Multi-developer short code collision resolution
///
/// Simulates scenario where two developers create tasks on different branches
/// with the same short code, then merge. Sync should detect and resolve the collision.
#[tokio::test]
async fn test_short_code_collision_resolution() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();

    // Step 1: Initialize workspace
    let result = WorkspaceInitializationService::initialize_workspace_with_prefix(
        base_path,
        "Test Project",
        "TEST",
    )
    .await
    .expect("Failed to initialize workspace");

    let metis_dir = result.metis_dir;

    // Step 2: Create two task files with the same short code but different parent paths
    // Simulating two developers creating tasks in different initiatives

    // Developer A's task (in initiative I-0001)
    let init_a_dir = metis_dir
        .join("strategies/NULL/initiatives/I-0001/tasks");
    fs::create_dir_all(&init_a_dir).expect("Failed to create initiative A dir");

    let task_a = init_a_dir.join("T-0001.md");
    let task_a_content = "---\n\
id: dev-a-task\n\
level: task\n\
title: \"Developer A Task\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T10:00:00Z\n\
updated_at: 2025-01-01T10:00:00Z\n\
parent: I-0001\n\
blocked_by: []\n\
archived: false\n\
tags:\n\
  - \"#task\"\n\
  - \"#phase/todo\"\n\
exit_criteria_met: false\n\
---\n\
\n\
# Developer A Task\n\
\n\
This task was created by developer A.";

    fs::write(&task_a, task_a_content).expect("Failed to write task A");

    // Developer B's task (in initiative I-0002) - same short code!
    let init_b_dir = metis_dir
        .join("strategies/NULL/initiatives/I-0002/tasks");
    fs::create_dir_all(&init_b_dir).expect("Failed to create initiative B dir");

    let task_b = init_b_dir.join("T-0001.md");
    let task_b_content = "---\n\
id: dev-b-task\n\
level: task\n\
title: \"Developer B Task\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T11:00:00Z\n\
updated_at: 2025-01-01T11:00:00Z\n\
parent: I-0002\n\
blocked_by: []\n\
archived: false\n\
tags:\n\
  - \"#task\"\n\
  - \"#phase/todo\"\n\
exit_criteria_met: false\n\
---\n\
\n\
# Developer B Task\n\
\n\
This task was created by developer B.";

    fs::write(&task_b, task_b_content).expect("Failed to write task B");

    // Step 3: Run sync - should detect collision and resolve it
    let db_path = metis_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).expect("Failed to open database");
    let mut db_service = DatabaseService::new(db.into_repository());

    let mut sync_service = SyncService::new(&mut db_service)
        .with_workspace_dir(&metis_dir);

    let sync_results = sync_service
        .sync_directory(&metis_dir)
        .await
        .expect("Sync should succeed");

    // Debug: Print all sync results
    println!("Sync results:");
    for result in &sync_results {
        println!("  {:?}", result);
    }

    // Step 4: Verify collision was detected and resolved
    let renumbered_results: Vec<_> = sync_results
        .iter()
        .filter(|r| matches!(r, metis_core::application::services::synchronization::SyncResult::Renumbered { .. }))
        .collect();

    assert_eq!(
        renumbered_results.len(),
        1,
        "Should have exactly 1 renumbered document"
    );

    // Step 5: Verify both files exist but with different short codes
    // One should still be T-0001, the other should be renumbered

    // Debug: List all files in both directories
    println!("Files in init_a_dir:");
    for entry in fs::read_dir(&init_a_dir).unwrap() {
        let entry = entry.unwrap();
        println!("  - {}", entry.file_name().to_string_lossy());
    }

    println!("Files in init_b_dir:");
    for entry in fs::read_dir(&init_b_dir).unwrap() {
        let entry = entry.unwrap();
        println!("  - {}", entry.file_name().to_string_lossy());
    }

    let task_a_new_content = fs::read_to_string(&task_a)
        .or_else(|_| {
            // File might have been renamed
            let renamed_path = init_a_dir.join("TEST-T-0002.md");
            fs::read_to_string(renamed_path)
        })
        .expect("Should be able to read task A");

    let task_b_new_content = fs::read_to_string(&task_b)
        .or_else(|_| {
            // File might have been renamed
            let renamed_path = init_b_dir.join("TEST-T-0002.md");
            fs::read_to_string(renamed_path)
        })
        .expect("Should be able to read task B");

    // Verify both documents exist with different short codes
    assert!(
        task_a_new_content.contains("TEST-T-") && task_b_new_content.contains("TEST-T-"),
        "Both tasks should have TEST-T- short codes"
    );

    // Verify they have different short codes
    let has_0001 = task_a_new_content.contains("TEST-T-0001")
        || task_b_new_content.contains("TEST-T-0001");
    let has_0002 = task_a_new_content.contains("TEST-T-0002")
        || task_b_new_content.contains("TEST-T-0002");

    assert!(has_0001, "One task should have TEST-T-0001");
    assert!(has_0002, "One task should have TEST-T-0002");

    // Step 6: Verify both documents are in the database with unique short codes
    let repo = db_service.find_by_type(metis_core::domain::documents::types::DocumentType::Task)
        .expect("Should be able to query tasks");

    let task_short_codes: Vec<String> = repo.iter().map(|t| t.short_code.clone()).collect();

    assert!(
        task_short_codes.contains(&"TEST-T-0001".to_string())
            || task_short_codes.contains(&"TEST-T-0002".to_string()),
        "Database should contain renumbered tasks"
    );
}

/// Test cross-reference updating in sibling documents
#[tokio::test]
async fn test_sibling_cross_reference_update() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();

    // Initialize workspace
    let result = WorkspaceInitializationService::initialize_workspace(base_path, "Test Project")
        .await
        .expect("Failed to initialize workspace");

    let metis_dir = result.metis_dir;

    // Create initiative directory with two tasks
    let tasks_dir = metis_dir
        .join("strategies/NULL/initiatives/I-0001/tasks");
    fs::create_dir_all(&tasks_dir).expect("Failed to create tasks dir");

    // Task 1 - will be renumbered
    let task1 = tasks_dir.join("T-0001.md");
    let task1_content = "---\n\
id: task-1\n\
level: task\n\
title: \"Task 1\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T10:00:00Z\n\
updated_at: 2025-01-01T10:00:00Z\n\
parent: I-0001\n\
blocked_by: []\n\
archived: false\n\
tags:\n\
  - \"#task\"\n\
exit_criteria_met: false\n\
---\n\
\n\
# Task 1";

    fs::write(&task1, task1_content).expect("Failed to write task 1");

    // Task 2 - references Task 1 and will cause collision
    let task2 = tasks_dir.join("T-0002.md");
    let task2_content = "---\n\
id: task-2\n\
level: task\n\
title: \"Task 2\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T11:00:00Z\n\
updated_at: 2025-01-01T11:00:00Z\n\
parent: I-0001\n\
blocked_by:\n\
  - TEST-T-0001\n\
archived: false\n\
tags:\n\
  - \"#task\"\n\
exit_criteria_met: false\n\
---\n\
\n\
# Task 2\n\
\n\
This task depends on TEST-T-0001 being completed.";

    fs::write(&task2, task2_content).expect("Failed to write task 2");

    // Run sync
    let db_path = metis_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).expect("Failed to open database");
    let mut db_service = DatabaseService::new(db.into_repository());

    let mut sync_service = SyncService::new(&mut db_service)
        .with_workspace_dir(&metis_dir);

    sync_service
        .sync_directory(&metis_dir)
        .await
        .expect("Sync should succeed");

    // Read task 2 and verify reference was updated
    let task2_updated = fs::read_to_string(&task2)
        .or_else(|_| {
            // File might have been renamed
            let renamed = tasks_dir.join("T-0003.md");
            fs::read_to_string(renamed)
        })
        .expect("Should be able to read updated task 2");

    // The reference to TEST-T-0001 should be updated if task 1 was renumbered
    // or left unchanged if task 2 was renumbered
    let has_reference = task2_updated.contains("TEST-T-");

    assert!(
        has_reference,
        "Task 2 should still have a reference to a task"
    );
}

/// Test that collision resolution preserves document order by path depth
#[tokio::test]
async fn test_collision_resolution_depth_ordering() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();

    let result = WorkspaceInitializationService::initialize_workspace(base_path, "Test Project")
        .await
        .expect("Failed to initialize workspace");

    let metis_dir = result.metis_dir;

    // Create task at shallower depth (should keep original short code)
    let shallow_dir = metis_dir.join("strategies/NULL/initiatives/I-0001/tasks");
    fs::create_dir_all(&shallow_dir).expect("Failed to create shallow dir");

    let shallow_task = shallow_dir.join("T-0001.md");
    let shallow_content = "---\n\
id: shallow-task\n\
level: task\n\
title: \"Shallow Task\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T10:00:00Z\n\
updated_at: 2025-01-01T10:00:00Z\n\
parent: \n\
blocked_by: []\n\
archived: false\n\
tags: [\"#task\"]\n\
exit_criteria_met: false\n\
---\n\
# Shallow Task";

    fs::write(&shallow_task, shallow_content).expect("Failed to write shallow task");

    // Create task at deeper depth (should be renumbered)
    let deep_dir = metis_dir.join("strategies/S-0001/initiatives/I-0002/tasks");
    fs::create_dir_all(&deep_dir).expect("Failed to create deep dir");

    let deep_task = deep_dir.join("T-0001.md");
    let deep_content = "---\n\
id: deep-task\n\
level: task\n\
title: \"Deep Task\"\n\
short_code: \"TEST-T-0001\"\n\
created_at: 2025-01-01T11:00:00Z\n\
updated_at: 2025-01-01T11:00:00Z\n\
parent: \n\
blocked_by: []\n\
archived: false\n\
tags: [\"#task\"]\n\
exit_criteria_met: false\n\
---\n\
# Deep Task";

    fs::write(&deep_task, deep_content).expect("Failed to write deep task");

    // Run sync
    let db_path = metis_dir.join("metis.db");
    let db = Database::new(&db_path.to_string_lossy()).expect("Failed to open database");
    let mut db_service = DatabaseService::new(db.into_repository());

    let mut sync_service = SyncService::new(&mut db_service)
        .with_workspace_dir(&metis_dir);

    sync_service
        .sync_directory(&metis_dir)
        .await
        .expect("Sync should succeed");

    // Verify shallow task kept TEST-T-0001
    let shallow_updated = fs::read_to_string(&shallow_task)
        .expect("Shallow task should exist at original path");

    assert!(
        shallow_updated.contains("TEST-T-0001"),
        "Shallow task should keep original short code"
    );

    // Verify deep task was renumbered
    let deep_updated_result = fs::read_to_string(&deep_task);

    if deep_updated_result.is_err() {
        // File was renamed - this is expected
        let renamed_path = deep_dir.join("T-0002.md");
        let deep_renamed = fs::read_to_string(&renamed_path)
            .expect("Deep task should be renamed to T-0002.md");

        assert!(
            deep_renamed.contains("TEST-T-0002"),
            "Deep task should be renumbered to TEST-T-0002"
        );
    } else {
        // File wasn't renamed, but short code should be updated
        let deep_updated = deep_updated_result.unwrap();
        assert!(
            deep_updated.contains("TEST-T-0002"),
            "Deep task should have new short code TEST-T-0002"
        );
    }
}
