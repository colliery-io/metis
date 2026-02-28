//! Integration tests for configuration recovery from database corruption/loss
//!
//! Tests the complete recovery flow:
//! - Database deletion and recreation
//! - Counter recovery from filesystem
//! - Config.toml synchronization

use metis_core::application::services::workspace::{
    ConfigurationRecoveryService, WorkspaceInitializationService,
};
use metis_core::domain::configuration::{ConfigFile, FlightLevelConfig};
use metis_core::{Application, Database};
use tempfile::TempDir;

/// Helper to create a test workspace
async fn setup_test_workspace() -> (TempDir, String, String) {
    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_string_lossy().to_string();
    let metis_dir = format!("{}/.metis", project_path);

    // Initialize workspace
    WorkspaceInitializationService::initialize_workspace_with_prefix(
        &project_path,
        "Test Workspace",
        Some("TEST"),
    )
    .await
    .unwrap();

    (temp_dir, project_path, metis_dir)
}

#[tokio::test]
async fn test_recovery_from_complete_database_loss() {
    println!("\n=== Test: Recovery from Complete Database Loss ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);
    let config_file_path = format!("{}/config.toml", metis_dir);

    // Step 1: Verify initial state
    println!("Step 1: Verify initial setup");
    assert!(std::path::Path::new(&db_path).exists(), "DB should exist");
    assert!(
        std::path::Path::new(&config_file_path).exists(),
        "config.toml should exist"
    );

    // Verify config.toml has correct prefix
    let config = ConfigFile::load(&config_file_path).unwrap();
    assert_eq!(config.prefix(), "TEST");
    println!("✓ Initial workspace created with prefix: TEST");

    // Step 2: Create some documents to establish counters
    println!("\nStep 2: Create documents to establish counters");
    let db = Database::new(&db_path).unwrap();
    let app = Application::new(db);

    // Sync to create vision (already exists from init)
    let sync_results = app.sync_directory(&metis_dir).await.unwrap();
    println!("✓ Synced {} files", sync_results.len());

    // Step 3: Verify counters exist in database
    println!("\nStep 3: Verify initial counter state");
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let vision_counter = config_repo.get_counter("vision").unwrap();
    assert_eq!(vision_counter, 1, "Vision counter should be 1");
    println!("✓ Vision counter = {}", vision_counter);

    // Step 4: Delete the database (simulate corruption)
    println!("\nStep 4: Simulate database corruption by deleting it");
    std::fs::remove_file(&db_path).unwrap();
    assert!(
        !std::path::Path::new(&db_path).exists(),
        "DB should be deleted"
    );
    println!("✓ Database deleted");

    // Step 5: Verify recovery is needed
    println!("\nStep 5: Check if recovery is needed");
    let needs_recovery =
        ConfigurationRecoveryService::needs_recovery(std::path::Path::new(&metis_dir));
    assert!(needs_recovery, "Should detect missing database");
    println!("✓ Recovery correctly detected as needed");

    // Step 6: Perform recovery
    println!("\nStep 6: Perform full recovery");
    let _db = Database::new(&db_path).unwrap(); // Recreates empty DB
    let report = ConfigurationRecoveryService::recover_configuration(&metis_dir, &db_path).unwrap();

    assert!(
        report.had_recovery_actions(),
        "Recovery should have taken actions"
    );
    println!("✓ Recovery report:");
    println!("  - Config file created: {}", report.config_file_created);
    println!("  - Prefix synced: {}", report.prefix_synced);
    println!("  - Flight levels synced: {}", report.flight_levels_synced);
    println!("  - Counters recovered: {}", report.counters_recovered);

    // Step 7: Verify recovery results
    println!("\nStep 7: Verify recovery completeness");

    // Check prefix was restored
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let recovered_prefix = config_repo.get_project_prefix().unwrap().unwrap();
    assert_eq!(recovered_prefix, "TEST", "Prefix should be recovered");
    println!("✓ Prefix recovered: {}", recovered_prefix);

    // Check counter was recovered
    let recovered_vision_counter = config_repo.get_counter("vision").unwrap();
    assert_eq!(
        recovered_vision_counter, 1,
        "Vision counter should be recovered to 1"
    );
    println!("✓ Vision counter recovered: {}", recovered_vision_counter);

    println!("\n✅ Complete database recovery successful!");
}

#[tokio::test]
async fn test_counter_recovery_prevents_duplicates() {
    println!("\n=== Test: Counter Recovery Prevents Duplicate Short Codes ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);

    // Step 1: Create multiple documents via normal workflow
    println!("Step 1: Sync to establish baseline");
    let db = Database::new(&db_path).unwrap();
    let app = Application::new(db);
    app.sync_directory(&metis_dir).await.unwrap();

    // Step 2: Manually corrupt counter in database (set it to 0)
    println!("\nStep 2: Corrupt counter by setting it to 0");
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    config_repo.set_counter("vision", 0).unwrap();
    println!("✓ Vision counter corrupted to 0");

    // Step 3: Run recovery
    println!("\nStep 3: Run recovery to fix counter");
    let report = ConfigurationRecoveryService::recover_configuration(&metis_dir, &db_path).unwrap();
    println!("✓ Counters recovered: {}", report.counters_recovered);

    // Step 4: Verify counter was restored to prevent duplicates
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let recovered_counter = config_repo.get_counter("vision").unwrap();
    assert!(
        recovered_counter >= 1,
        "Counter should be at least 1 to prevent duplicate TEST-V-0001"
    );
    println!("✓ Counter recovered to {}", recovered_counter);

    // Step 5: Create a new document to verify no duplicates
    println!("\nStep 5: Generate new short code to verify uniqueness");
    let new_short_code = config_repo.generate_short_code("vision").unwrap();
    assert!(
        new_short_code != "TEST-V-0001",
        "Should not generate duplicate short code"
    );
    println!("✓ New short code generated: {}", new_short_code);

    println!("\n✅ Counter recovery prevents duplicates!");
}

#[tokio::test]
async fn test_config_sync_on_normal_operations() {
    println!("\n=== Test: Config.toml Syncs to DB on Normal Operations ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);
    let config_file_path = format!("{}/config.toml", metis_dir);

    // Step 1: Manually change config.toml
    println!("Step 1: Manually change config.toml flight levels");
    let mut new_config = ConfigFile::new("TEST".to_string(), FlightLevelConfig::full()).unwrap();
    // Strategies require sync config, so add workspace + sync sections
    new_config
        .set_workspace("test-ws".to_string(), None)
        .unwrap();
    new_config
        .set_sync("git@github.com:org/repo.git".to_string())
        .unwrap();
    new_config.save(&config_file_path).unwrap();
    println!("✓ Config.toml updated to full configuration");

    // Step 2: Verify DB still has old config
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let old_flight_levels = config_repo.get_flight_level_config().unwrap();
    assert_eq!(
        old_flight_levels,
        FlightLevelConfig::streamlined(),
        "DB should still have streamlined config"
    );
    println!("✓ DB still has streamlined configuration");

    // Step 3: Run sync (which should sync config)
    println!("\nStep 2: Run sync operation");
    let db = Database::new(&db_path).unwrap();
    let app = Application::new(db);
    app.sync_directory(&metis_dir).await.unwrap();
    println!("✓ Sync completed");

    // Step 4: Verify DB was updated from config.toml
    println!("\nStep 3: Verify DB was updated from config.toml");
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let updated_flight_levels = config_repo.get_flight_level_config().unwrap();
    assert_eq!(
        updated_flight_levels,
        FlightLevelConfig::full(),
        "DB should now have full config from config.toml"
    );
    println!("✓ DB updated to full configuration");

    println!("\n✅ Config.toml syncs to DB on normal sync operations!");
}

#[tokio::test]
async fn test_recovery_validates_short_code_format() {
    println!("\n=== Test: Recovery Validates Short Code Format ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);

    // Step 1: Create a document with invalid short code
    println!("Step 1: Create document with invalid short code");
    let invalid_doc_path = format!("{}/test-invalid.md", metis_dir);
    let invalid_doc_content = "---\n\
id: test-invalid\n\
level: vision\n\
title: \"Invalid Doc\"\n\
short_code: \"INVALID-FORMAT\"\n\
created_at: 2025-01-01T00:00:00Z\n\
updated_at: 2025-01-01T00:00:00Z\n\
archived: false\n\
tags:\n\
  - \"#vision\"\n\
exit_criteria_met: false\n\
---\n\
# Invalid Doc\n";
    std::fs::write(&invalid_doc_path, invalid_doc_content).unwrap();
    println!("✓ Created document with invalid short code: INVALID-FORMAT");

    // Step 2: Delete database
    println!("\nStep 2: Delete database");
    std::fs::remove_file(&db_path).unwrap();

    // Step 3: Run recovery
    println!("\nStep 3: Run recovery (should skip invalid short code)");
    let _db = Database::new(&db_path).unwrap();
    let report = ConfigurationRecoveryService::recover_configuration(&metis_dir, &db_path).unwrap();

    // Step 4: Verify recovery completed despite invalid document
    println!("✓ Recovery completed");
    println!("  - Counters recovered: {}", report.counters_recovered);

    // Step 5: Verify only valid short codes were used
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let vision_counter = config_repo.get_counter("vision").unwrap();
    // Should be 1 from the valid vision.md, not 2 (invalid one should be skipped)
    assert_eq!(
        vision_counter, 1,
        "Should only count valid short codes (TEST-V-0001)"
    );
    println!("✓ Only valid short codes counted");

    println!("\n✅ Recovery correctly validates and skips invalid short codes!");
}

#[tokio::test]
async fn test_migration_from_old_workspace_without_config_toml() {
    println!("\n=== Test: Migration from Old Workspace (No config.toml) ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);
    let config_file_path = format!("{}/config.toml", metis_dir);

    // Step 1: Delete config.toml to simulate old workspace
    println!("Step 1: Delete config.toml to simulate old workspace");
    std::fs::remove_file(&config_file_path).unwrap();
    assert!(
        !std::path::Path::new(&config_file_path).exists(),
        "config.toml should be deleted"
    );
    println!("✓ config.toml removed");

    // Step 2: Run recovery (should detect missing config.toml)
    println!("\nStep 2: Run recovery to migrate");
    let report = ConfigurationRecoveryService::recover_configuration(&metis_dir, &db_path).unwrap();

    assert!(
        report.config_file_created,
        "Should have created config.toml"
    );
    println!("✓ config.toml created during migration");

    // Step 3: Verify config.toml was created with DB values
    assert!(
        std::path::Path::new(&config_file_path).exists(),
        "config.toml should now exist"
    );

    let config = ConfigFile::load(&config_file_path).unwrap();
    assert_eq!(config.prefix(), "TEST", "Should preserve prefix from DB");
    println!(
        "✓ config.toml created with correct prefix: {}",
        config.prefix()
    );

    println!("\n✅ Migration from old workspace successful!");
}

#[tokio::test]
async fn test_recovery_from_corrupted_database_file() {
    println!("\n=== Test: Recovery from Corrupted Database File ===");

    let (_temp_dir, _project_path, metis_dir) = setup_test_workspace().await;
    let db_path = format!("{}/metis.db", metis_dir);
    let config_file_path = format!("{}/config.toml", metis_dir);

    // Step 1: Verify initial state
    println!("Step 1: Verify initial setup");
    assert!(std::path::Path::new(&db_path).exists(), "DB should exist");
    assert!(
        std::path::Path::new(&config_file_path).exists(),
        "config.toml should exist"
    );

    // Verify config.toml has correct prefix
    let config = ConfigFile::load(&config_file_path).unwrap();
    assert_eq!(config.prefix(), "TEST");
    println!("✓ Initial workspace created with prefix: TEST");

    // Step 2: Create some documents to establish counters
    println!("\nStep 2: Create documents to establish counters");
    let db = Database::new(&db_path).unwrap();
    let app = Application::new(db);
    let sync_results = app.sync_directory(&metis_dir).await.unwrap();
    println!("✓ Synced {} files", sync_results.len());

    // Step 3: Verify counters exist in database
    println!("\nStep 3: Verify initial counter state");
    let db = Database::new(&db_path).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();
    let vision_counter = config_repo.get_counter("vision").unwrap();
    assert_eq!(vision_counter, 1, "Vision counter should be 1");
    println!("✓ Vision counter = {}", vision_counter);

    // Step 4: Corrupt the database file by writing garbage to it
    println!("\nStep 4: Corrupt database file with garbage data");
    std::fs::write(
        &db_path,
        b"This is not a valid SQLite database file! Just garbage data to corrupt it.",
    )
    .unwrap();
    assert!(
        std::path::Path::new(&db_path).exists(),
        "DB file should still exist (but corrupted)"
    );
    println!("✓ Database file corrupted with garbage data");

    // Step 5: Verify database is truly corrupt
    println!("\nStep 5: Verify database is unreadable");
    let db_open_result = Database::new(&db_path);
    assert!(
        db_open_result.is_err(),
        "Opening corrupt database should fail"
    );
    println!("✓ Confirmed database is corrupt and unreadable");

    // Step 6: Verify needs_recovery detects the corruption
    println!("\nStep 6: Check if recovery detects corruption");
    let needs_recovery =
        ConfigurationRecoveryService::needs_recovery(std::path::Path::new(&metis_dir));
    assert!(needs_recovery, "Should detect corrupted database");
    println!("✓ Recovery correctly detected corrupted database");

    // Step 7: Delete corrupt DB and recreate it
    println!("\nStep 7: Remove corrupt database and recreate");
    std::fs::remove_file(&db_path).unwrap();
    let _db = Database::new(&db_path).unwrap(); // Recreates empty DB
    println!("✓ Corrupt database removed and recreated");

    // Step 8: Perform recovery
    println!("\nStep 8: Perform full recovery");
    let report = ConfigurationRecoveryService::recover_configuration(&metis_dir, &db_path).unwrap();

    assert!(
        report.had_recovery_actions(),
        "Recovery should have taken actions"
    );
    println!("✓ Recovery report:");
    println!("  - Config file created: {}", report.config_file_created);
    println!("  - Prefix synced: {}", report.prefix_synced);
    println!("  - Flight levels synced: {}", report.flight_levels_synced);
    println!("  - Counters recovered: {}", report.counters_recovered);

    // Step 9: Verify recovery results
    println!("\nStep 9: Verify recovery completeness");

    // Check database is now readable
    let db = Database::new(&db_path).unwrap();
    println!("✓ Database is now readable");

    // Check prefix was restored
    let mut config_repo = db.configuration_repository().unwrap();
    let recovered_prefix = config_repo.get_project_prefix().unwrap().unwrap();
    assert_eq!(recovered_prefix, "TEST", "Prefix should be recovered");
    println!("✓ Prefix recovered: {}", recovered_prefix);

    // Check counter was recovered
    let recovered_vision_counter = config_repo.get_counter("vision").unwrap();
    assert_eq!(
        recovered_vision_counter, 1,
        "Vision counter should be recovered to 1"
    );
    println!("✓ Vision counter recovered: {}", recovered_vision_counter);

    println!("\n✅ Recovery from corrupted database successful!");
}
