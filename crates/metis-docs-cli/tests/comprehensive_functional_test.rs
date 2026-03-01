//! Comprehensive functional tests for CLI covering complete workflows
//! These tests simulate real user command sequences through the CLI

use metis_core::{Application, Database};
use std::fs;
use std::path::PathBuf;

/// Helper to run CLI commands programmatically
mod cli_helpers {
    use super::*;
    use metis_docs_cli::commands::*;

    pub async fn init_workspace(
        path: &PathBuf,
        name: Option<&str>,
        prefix: Option<&str>,
        preset: Option<&str>,
    ) -> anyhow::Result<()> {
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(path)?;

        let cmd = init::InitCommand {
            name: name.map(|s| s.to_string()),
            prefix: prefix.map(|s| s.to_string()),
            preset: preset.map(|s| s.to_string()),
            strategies: None,
            initiatives: None,
            upstream: None,
            workspace_prefix: None,
            team: None,
        };

        let result = cmd.execute().await;

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        result
    }

    pub fn verify_workspace(path: &PathBuf) -> bool {
        let metis_dir = path.join(".metis");
        let db_path = metis_dir.join("metis.db");
        let config_path = metis_dir.join("config.toml");
        let vision_path = metis_dir.join("vision.md");

        metis_dir.exists() && db_path.exists() && config_path.exists() && vision_path.exists()
    }

    pub fn verify_config_toml(path: &PathBuf, expected_prefix: &str) -> bool {
        let config_path = path.join(".metis/config.toml");
        if let Ok(content) = fs::read_to_string(&config_path) {
            content.contains(&format!("prefix = \"{}\"", expected_prefix))
                && content.contains("[project]")
                && content.contains("[flight_levels]")
        } else {
            false
        }
    }
}

#[tokio::test]
async fn test_complete_streamlined_workflow() {
    println!("\n=== Testing Complete Streamlined Workflow ===");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Step 1: Initialize workspace with streamlined config (default)
    println!("Step 1: Initialize workspace");
    cli_helpers::init_workspace(&project_path, Some("Test Project"), Some("TEST"), None)
        .await
        .unwrap();

    assert!(
        cli_helpers::verify_workspace(&project_path),
        "Workspace should be initialized"
    );
    assert!(
        cli_helpers::verify_config_toml(&project_path, "TEST"),
        "config.toml should exist with correct prefix"
    );
    println!("✓ Workspace initialized");

    // Step 2: Verify database has correct configuration
    println!("\nStep 2: Verify database configuration");
    let db_path = project_path.join(".metis/metis.db");
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();

    let prefix = config_repo.get_project_prefix().unwrap().unwrap();
    assert_eq!(prefix, "TEST", "Prefix should be TEST");

    let flight_config = config_repo.get_flight_level_config().unwrap();
    assert_eq!(
        flight_config.preset_name(),
        "streamlined",
        "Should use streamlined preset"
    );
    assert!(
        !flight_config.strategies_enabled,
        "Strategies should be disabled"
    );
    assert!(
        flight_config.initiatives_enabled,
        "Initiatives should be enabled"
    );
    println!("✓ Database configuration verified");

    // Step 3: Verify vision document was created
    println!("\nStep 3: Verify vision document");
    let vision_path = project_path.join(".metis/vision.md");
    assert!(vision_path.exists(), "Vision document should exist");

    let vision_content = fs::read_to_string(&vision_path).unwrap();
    assert!(
        vision_content.contains("Test Project"),
        "Vision should contain project name"
    );
    assert!(
        vision_content.contains("short_code:"),
        "Vision should have short code"
    );
    assert!(
        vision_content.contains("TEST-V-0001"),
        "Vision should have correct short code format"
    );
    println!("✓ Vision document verified");

    // Step 4: Verify workspace can be synced
    println!("\nStep 4: Test sync operation");
    let app = Application::new(db);
    let metis_dir = project_path.join(".metis");
    let sync_results = app.sync_directory(&metis_dir).await;
    assert!(sync_results.is_ok(), "Sync should succeed");
    println!("✓ Workspace sync successful");

    println!("\n✅ Complete streamlined workflow test passed!");
}

#[tokio::test]
async fn test_complete_full_configuration_workflow() {
    println!("\n=== Testing Complete Full Configuration Workflow ===");

    let temp_dir = tempfile::tempdir().unwrap();

    // Create bare git repo as "central" for upstream
    let central_dir = temp_dir.path().join("central");
    git2::Repository::init_bare(&central_dir).unwrap();
    let central_url = format!("file://{}", central_dir.display());

    let project_path = temp_dir.path().join("project");
    std::fs::create_dir_all(&project_path).unwrap();

    // Step 1: Initialize workspace with full config + upstream (required for strategies)
    println!("Step 1: Initialize workspace with full configuration and upstream");
    {
        use metis_docs_cli::commands::init::InitCommand;
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(&project_path).unwrap();

        let cmd = InitCommand {
            name: Some("Full Config Test".to_string()),
            prefix: Some("FULL".to_string()),
            preset: Some("full".to_string()),
            strategies: None,
            initiatives: None,
            upstream: Some(central_url),
            workspace_prefix: Some("test-ws".to_string()),
            team: None,
        };
        cmd.execute().await.unwrap();

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    assert!(
        cli_helpers::verify_workspace(&project_path),
        "Workspace should be initialized"
    );
    println!("✓ Workspace initialized with full config");

    // Step 2: Verify full configuration is active
    println!("\nStep 2: Verify full flight level configuration");
    let db_path = project_path.join(".metis/metis.db");
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();

    let flight_config = config_repo.get_flight_level_config().unwrap();
    assert_eq!(
        flight_config.preset_name(),
        "full",
        "Should use full preset"
    );
    assert!(
        flight_config.strategies_enabled,
        "Strategies should be enabled"
    );
    assert!(
        flight_config.initiatives_enabled,
        "Initiatives should be enabled"
    );
    println!("✓ Full configuration verified");

    // Step 3: Verify workspace can be synced
    println!("\nStep 3: Test sync operation");
    let metis_dir = project_path.join(".metis");
    let app = Application::new(db);
    let sync_results = app.sync_directory(&metis_dir).await;
    assert!(sync_results.is_ok(), "Sync should succeed");
    println!("✓ Workspace sync successful");

    println!("\n✅ Complete full configuration workflow test passed!");
}

#[tokio::test]
async fn test_config_toml_persistence() {
    println!("\n=== Testing config.toml Persistence ===");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Step 1: Initialize workspace
    println!("Step 1: Initialize workspace");
    cli_helpers::init_workspace(&project_path, Some("Persistence Test"), Some("PERS"), None)
        .await
        .unwrap();

    // Step 2: Verify config.toml exists and has correct content
    println!("\nStep 2: Verify config.toml creation");
    let config_path = project_path.join(".metis/config.toml");
    assert!(config_path.exists(), "config.toml should exist");

    let config_content = fs::read_to_string(&config_path).unwrap();
    assert!(
        config_content.contains("prefix = \"PERS\""),
        "config.toml should have correct prefix"
    );
    assert!(
        config_content.contains("[project]"),
        "config.toml should have [project] section"
    );
    assert!(
        config_content.contains("[flight_levels]"),
        "config.toml should have [flight_levels] section"
    );
    println!("✓ config.toml created with correct content");

    // Step 3: Simulate database corruption by deleting it
    println!("\nStep 3: Simulate database corruption");
    let db_path = project_path.join(".metis/metis.db");
    fs::remove_file(&db_path).unwrap();
    assert!(!db_path.exists(), "Database should be deleted");
    println!("✓ Database deleted");

    // Step 4: Recreate database and sync - should recover from config.toml
    println!("\nStep 4: Recreate database and recover configuration");
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let app = Application::new(db);

    // Sync should trigger recovery
    let metis_dir = project_path.join(".metis");
    let _sync_results = app.sync_directory(&metis_dir).await.unwrap();
    println!("✓ Sync triggered recovery");

    // Step 5: Verify configuration was recovered from config.toml
    println!("\nStep 5: Verify configuration recovery");
    let db2 = Database::new(db_path.to_str().unwrap()).unwrap();
    let mut config_repo = db2.configuration_repository().unwrap();

    let recovered_prefix = config_repo.get_project_prefix().unwrap().unwrap();
    assert_eq!(
        recovered_prefix, "PERS",
        "Prefix should be recovered from config.toml"
    );

    let recovered_config = config_repo.get_flight_level_config().unwrap();
    assert_eq!(
        recovered_config.preset_name(),
        "streamlined",
        "Flight levels should be recovered"
    );
    println!("✓ Configuration recovered from config.toml");

    println!("\n✅ config.toml persistence test passed!");
}

#[tokio::test]
async fn test_custom_prefix_handling() {
    println!("\n=== Testing Custom Prefix Handling ===");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Test with custom prefix
    println!("Step 1: Initialize with custom prefix");
    cli_helpers::init_workspace(
        &project_path,
        Some("Custom Prefix Test"),
        Some("CUSTOM"),
        None,
    )
    .await
    .unwrap();

    // Verify prefix in database
    let db_path = project_path.join(".metis/metis.db");
    let db = Database::new(db_path.to_str().unwrap()).unwrap();
    let mut config_repo = db.configuration_repository().unwrap();

    let prefix = config_repo.get_project_prefix().unwrap().unwrap();
    assert_eq!(prefix, "CUSTOM", "Database should have custom prefix");

    // Verify prefix in config.toml
    assert!(
        cli_helpers::verify_config_toml(&project_path, "CUSTOM"),
        "config.toml should have custom prefix"
    );

    // Verify vision document has correct short code
    let vision_path = project_path.join(".metis/vision.md");
    let vision_content = fs::read_to_string(&vision_path).unwrap();
    assert!(
        vision_content.contains("CUSTOM-V-0001"),
        "Vision should use custom prefix in short code"
    );

    println!("✓ Custom prefix correctly applied");

    println!("\n✅ Custom prefix handling test passed!");
}

#[tokio::test]
async fn test_init_full_without_upstream_fails() {
    println!("\n=== Testing Init Full Without Upstream Fails ===");

    let temp_dir = tempfile::tempdir().unwrap();
    let project_path = temp_dir.path().to_path_buf();

    // Attempt to initialize with --preset full but no --upstream
    let result =
        cli_helpers::init_workspace(&project_path, Some("Gate Test"), Some("GATE"), Some("full"))
            .await;

    assert!(result.is_err(), "Init with --preset full and no --upstream should fail");
    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Strategies require multi-workspace sync"),
        "Error should mention sync requirement, got: {}",
        err_msg
    );

    println!("✓ Init with --preset full correctly rejected without --upstream");
    println!("\n✅ Init gate test passed!");
}
