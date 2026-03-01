use metis_core::application::services::workspace::{
    WorkspaceDetectionService, WorkspaceInitializationService,
};
use std::fs;
use tempfile::tempdir;

/// Integration test for METIS-T-0012: Database as cache only
///
/// Tests the core behavior:
/// 1. Initialize workspace
/// 2. Add documents
/// 3. Delete database
/// 4. Call workspace detection/preparation
/// 5. Verify database reformed with all data intact
#[tokio::test]
async fn test_database_auto_reconstruction() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();

    // Step 1: Initialize workspace
    let result = WorkspaceInitializationService::initialize_workspace(base_path, "Test Project")
        .await
        .expect("Failed to initialize workspace");

    let metis_dir = result.metis_dir;
    let db_path = result.database_path.clone();

    // Verify initial state
    assert!(metis_dir.exists());
    assert!(db_path.exists());

    // Step 2: Add some documents by creating markdown files
    let strategies_dir = metis_dir.join("strategies");
    fs::create_dir_all(&strategies_dir).expect("Failed to create strategies dir");

    let test_doc = strategies_dir.join("test-strategy.md");
    let test_content = "---\n\
id: test-strategy\n\
level: strategy\n\
title: \"Test Strategy\"\n\
short_code: \"TEST-S-0001\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
parent: TEST-V-0001\n\
blocked_by: []\n\
archived: false\n\
tags:\n\
  - \"#strategy\"\n\
  - \"#phase/draft\"\n\
exit_criteria_met: false\n\
success_metrics: []\n\
risk_level: medium\n\
stakeholders: []\n\
---\n\
\n\
# Test Strategy\n\
\n\
This is a test strategy document.";

    fs::write(&test_doc, test_content).expect("Failed to write test document");

    // Step 3: Delete the database
    fs::remove_file(&db_path).expect("Failed to delete database");
    assert!(!db_path.exists(), "Database should be deleted");

    // Step 4: Call workspace preparation (should reconstruct database)
    let detection_service = WorkspaceDetectionService::new();
    let reconstructed_db = detection_service
        .prepare_workspace(&metis_dir)
        .await
        .expect("Failed to prepare workspace - database should be auto-reconstructed");

    // Step 5: Verify database was reformed
    assert!(db_path.exists(), "Database should be reconstructed");

    // Step 6: Validate data integrity - check that documents are in the database
    let mut repo = reconstructed_db.into_repository();

    // Check for vision document (created during init)
    let visions = repo
        .find_by_type("vision")
        .expect("Failed to query visions");
    assert_eq!(visions.len(), 1, "Should have 1 vision document");
    assert!(
        visions[0].title.contains("Test Project"),
        "Vision should have correct title"
    );

    // Check for strategy document (created manually)
    let strategies = repo
        .find_by_type("strategy")
        .expect("Failed to query strategies");
    assert_eq!(strategies.len(), 1, "Should have 1 strategy document");
    assert_eq!(
        strategies[0].title, "Test Strategy",
        "Strategy should have correct title"
    );
    assert_eq!(
        strategies[0].short_code, "TEST-S-0001",
        "Strategy should have correct short code"
    );
}

/// Test that workspace detection works without database present
#[tokio::test]
async fn test_workspace_detection_without_database() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();
    let metis_dir = base_path.join(".metis");

    // Create just the .metis directory, no database
    fs::create_dir_all(&metis_dir).expect("Failed to create .metis dir");

    // Workspace detection should succeed (only checks for .metis directory)
    let detection_service = WorkspaceDetectionService::new();
    let result = detection_service.find_workspace_from(base_path);

    assert!(result.is_ok(), "Workspace detection should succeed");
    assert!(
        result.unwrap().is_some(),
        "Should find workspace with just .metis directory"
    );
}

/// Test that is_workspace only checks for .metis directory
#[test]
fn test_is_workspace_without_database() {
    let temp_dir = tempdir().unwrap();
    let base_path = temp_dir.path();
    let metis_dir = base_path.join(".metis");

    // Initially not a workspace
    assert!(!WorkspaceInitializationService::is_workspace(base_path));

    // Create .metis directory - should now be a workspace
    fs::create_dir_all(&metis_dir).unwrap();
    assert!(WorkspaceInitializationService::is_workspace(base_path));

    // No database file needed
    let db_path = metis_dir.join("metis.db");
    assert!(
        !db_path.exists(),
        "Test should work without database file"
    );
}
