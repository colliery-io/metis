//! Archive cascading behavior tests for TUI to verify initiative archiving

mod common;

use anyhow::Result;
use common::TestHelper;
use metis_docs_tui::app::App;
use metis_docs_tui::models::BoardType;
use tui_input::backend::crossterm::EventHandler;

/// Helper function to type text into TUI input
fn type_text(app: &mut App, text: &str) {
    for ch in text.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
}

/// Test TUI archive cascading behavior for initiatives with tasks
#[tokio::test]
async fn test_tui_archive_initiative_cascade() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== TUI Archive Initiative Cascade Test ===");

    // Step 1: Create hierarchy - Strategy -> Initiative -> 2 Tasks
    println!("\n=== Step 1: Create Test Hierarchy ===");

    // Create strategy first
    app.ui_state.current_board = BoardType::Strategy;
    app.start_document_creation();
    type_text(&mut app, "Test Strategy for Archive");

    let result = app.create_new_document().await;
    assert!(
        result.is_ok(),
        "Strategy creation should succeed: {:?}",
        result
    );

    // Load documents to get the strategy in the board
    app.load_documents().await?;

    // Verify strategy exists
    let board = &app.ui_state.strategy_board;
    assert_eq!(board.columns[0].items.len(), 1, "Should have 1 strategy");

    // Create initiative under the strategy (strategy should be selected by default at 0,0)
    app.start_child_document_creation();
    type_text(&mut app, "Test Initiative for Archive");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "Initiative creation should succeed: {:?}",
        result
    );

    // Load documents and switch to initiative board
    app.load_documents().await?;
    app.ui_state.current_board = BoardType::Initiative;

    // Verify initiative exists
    let board = &app.ui_state.initiative_board;
    assert_eq!(board.columns[0].items.len(), 1, "Should have 1 initiative");

    // Create first task under the initiative (initiative should be selected by default)
    app.start_child_document_creation();
    type_text(&mut app, "Task One for Archive");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "First task creation should succeed: {:?}",
        result
    );

    // Create second task under the initiative
    app.start_child_document_creation();
    type_text(&mut app, "Task Two for Archive");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "Second task creation should succeed: {:?}",
        result
    );

    // Load documents to get everything in the boards
    app.load_documents().await?;

    println!("✅ Created complete hierarchy: Strategy -> Initiative -> 2 Tasks");

    // Step 2: Verify initial state in database
    println!("\n=== Step 2: Verify Initial Database State ===");

    let db_path = helper.metis_dir().join("metis.db");
    let db = metis_core::dal::Database::new(&db_path.to_string_lossy())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    let mut repo = db.into_repository();

    let strategies = repo.find_by_type("strategy")?;
    let initiatives = repo.find_by_type("initiative")?;
    let tasks = repo.find_by_type("task")?;

    assert_eq!(strategies.len(), 1, "Should have 1 strategy");
    assert_eq!(initiatives.len(), 1, "Should have 1 initiative");
    assert_eq!(tasks.len(), 2, "Should have 2 tasks");

    // All should be active (not archived)
    assert!(!strategies[0].archived, "Strategy should be active");
    assert!(!initiatives[0].archived, "Initiative should be active");
    assert_eq!(
        tasks.iter().filter(|t| t.archived).count(),
        0,
        "No tasks should be archived"
    );

    println!("✅ Initial state verified: 1 strategy, 1 initiative, 2 tasks (all active)");

    // Step 3: Archive the initiative (should cascade to its tasks)
    println!("\n=== Step 3: Archive Initiative (Cascade Test) ===");

    // Switch to initiative board (initiative should be selected by default at 0,0)
    app.ui_state.current_board = BoardType::Initiative;

    // Verify we have the initiative selected
    let selected_item = app.get_selected_item();
    assert!(selected_item.is_some(), "Should have initiative selected");
    assert_eq!(selected_item.unwrap().id(), "test-initiative-for-archive");

    // Before archiving, let's see what's in the initiative directory
    let strategy_dir = helper
        .metis_dir()
        .join("strategies")
        .join("test-strategy-for-archive");
    let initiative_dir = strategy_dir
        .join("initiatives")
        .join("test-initiative-for-archive");

    println!("Before archive - Initiative directory contents:");
    if let Ok(entries) = std::fs::read_dir(&initiative_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = std::fs::metadata(&path);
            println!("  {:?} - {:?}", path, metadata);
        }
    }

    // Attempt to archive the initiative
    let result = app.archive_selected_document().await;

    // After archive attempt, let's see what's left in the directory
    println!("After archive attempt - Initiative directory contents:");
    if let Ok(entries) = std::fs::read_dir(&initiative_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let metadata = std::fs::metadata(&path);
            println!("  {:?} - {:?}", path, metadata);
        }
    } else {
        println!("  Directory no longer exists (good!)");
    }

    if result.is_ok() {
        println!("✅ Initiative archive succeeded");

        // Reload documents and check database state
        app.load_documents().await?;

        let strategies_after = repo.find_by_type("strategy")?;
        let initiatives_after = repo.find_by_type("initiative")?;
        let tasks_after = repo.find_by_type("task")?;

        // Strategy should still be active
        assert!(
            !strategies_after[0].archived,
            "Strategy should still be active"
        );

        // Initiative should be archived
        assert!(
            initiatives_after[0].archived,
            "Initiative should be archived"
        );

        // Both tasks should be archived due to cascade
        let archived_tasks = tasks_after.iter().filter(|t| t.archived).count();
        assert_eq!(
            archived_tasks, 2,
            "Both tasks should be archived due to cascade"
        );

        println!("✅ Cascade archiving successful:");
        println!("   - Strategy: active (not cascaded)");
        println!("   - Initiative: archived");
        println!("   - Tasks: {} archived (cascaded)", archived_tasks);

        // Step 4: Verify file system state
        println!("\n=== Step 4: Verify File System State ===");

        let strategies_dir = helper.metis_dir().join("strategies");
        let archived_dir = helper.metis_dir().join("archived");

        // Strategy directory should still exist (strategy not archived)
        assert!(
            strategies_dir.exists(),
            "Strategies directory should still exist"
        );

        // Archived directory should contain our initiative
        assert!(archived_dir.exists(), "Archived directory should exist");

        // Check for archived initiative structure
        let archived_strategy_dir = archived_dir
            .join("strategies")
            .join("test-strategy-for-archive");
        let archived_initiative_dir = archived_strategy_dir
            .join("initiatives")
            .join("test-initiative-for-archive");

        if archived_initiative_dir.exists() {
            println!("✅ Archived initiative directory structure exists");
        } else {
            println!("⚠️  Archived initiative directory structure not found at expected location");
            // List what's actually in the archived directory for debugging
            if let Ok(entries) = std::fs::read_dir(&archived_dir) {
                println!("Contents of archived directory:");
                for entry in entries.flatten() {
                    println!("  {:?}", entry.path());
                }
            }
        }
    } else {
        let err = result.unwrap_err();
        println!("❌ Initiative archive failed: {:?}", err);

        // Check if this is the "Directory not empty" error we're debugging
        let err_msg = format!("{:?}", err);
        if err_msg.contains("Directory not empty") || err_msg.contains("os error 66") {
            println!("🔍 Detected 'Directory not empty' error - this is the bug we're fixing");

            // Let's examine what's actually in the initiative directory
            let strategy_dir = helper
                .metis_dir()
                .join("strategies")
                .join("test-strategy-for-archive");
            let initiative_dir = strategy_dir
                .join("initiatives")
                .join("test-initiative-for-archive");

            if initiative_dir.exists() {
                println!("Initiative directory contents:");
                if let Ok(entries) = std::fs::read_dir(&initiative_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        let metadata = std::fs::metadata(&path);
                        println!("  {:?} - {:?}", path, metadata);
                    }
                }
            } else {
                println!("Initiative directory doesn't exist at expected location");
            }

            return Err(anyhow::anyhow!(
                "Archive failed with directory not empty error - this confirms the bug"
            ));
        }

        return Err(anyhow::anyhow!("Archive failed unexpectedly: {:?}", err));
    }

    println!("\n=== TUI Archive Initiative Test Summary ===");
    println!("1. Created hierarchy (Strategy -> Initiative -> 2 Tasks) ✅");
    println!("2. Verified initial database state ✅");
    println!("3. Archived initiative (cascade to tasks) ✅");
    println!("4. Verified cascade behavior ✅");
    println!("5. Verified file system state ✅");

    Ok(())
}

/// Test archiving an initiative with no tasks (should work without issues)
#[tokio::test]
async fn test_tui_archive_empty_initiative() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== TUI Archive Empty Initiative Test ===");

    // Create strategy -> initiative (no tasks)
    app.ui_state.current_board = BoardType::Strategy;
    app.start_document_creation();
    type_text(&mut app, "Empty Test Strategy");

    let result = app.create_new_document().await;
    assert!(result.is_ok(), "Strategy creation should succeed");

    app.load_documents().await?;

    // Create initiative (no tasks)
    app.start_child_document_creation();
    type_text(&mut app, "Empty Test Initiative");

    let result = app.create_child_document().await;
    assert!(result.is_ok(), "Initiative creation should succeed");

    app.load_documents().await?;
    app.ui_state.current_board = BoardType::Initiative;

    // Archive the empty initiative (should succeed)
    let result = app.archive_selected_document().await;
    assert!(
        result.is_ok(),
        "Empty initiative archive should succeed: {:?}",
        result
    );

    println!("✅ Empty initiative archived successfully");

    Ok(())
}

/// Test TUI archive cascading with partial archive state (like MCP test)
/// This tests archiving individual task first, then archiving strategy
#[tokio::test]
async fn test_tui_archive_strategy_with_partial_archive() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== TUI Archive Strategy with Partial Archive Test ===");

    // Step 1: Create hierarchy - Strategy -> Initiative -> 2 Tasks
    println!("\n=== Step 1: Create Test Hierarchy ===");

    // Create strategy first
    app.ui_state.current_board = BoardType::Strategy;
    app.start_document_creation();
    type_text(&mut app, "Digital Transformation Strategy");

    let result = app.create_new_document().await;
    assert!(
        result.is_ok(),
        "Strategy creation should succeed: {:?}",
        result
    );

    // Load documents to get the strategy in the board
    app.load_documents().await?;

    // Create initiative under the strategy
    app.start_child_document_creation();
    type_text(&mut app, "Modernize Legacy Systems");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "Initiative creation should succeed: {:?}",
        result
    );

    // Load documents and switch to initiative board
    app.load_documents().await?;
    app.ui_state.current_board = BoardType::Initiative;

    // Create first task under the initiative
    app.start_child_document_creation();
    type_text(&mut app, "Audit Current Database Schema");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "First task creation should succeed: {:?}",
        result
    );

    // Create second task under the initiative
    app.start_child_document_creation();
    type_text(&mut app, "Plan Migration Roadmap");

    let result = app.create_child_document().await;
    assert!(
        result.is_ok(),
        "Second task creation should succeed: {:?}",
        result
    );

    // Load documents to get everything in the boards
    app.load_documents().await?;
    app.ui_state.current_board = BoardType::Task;

    println!("✅ Created complete hierarchy: Strategy -> Initiative -> 2 Tasks");

    // Step 2: Verify initial state
    println!("\n=== Step 2: Verify Initial Database State ===");

    let db_path = helper.metis_dir().join("metis.db");
    let db = metis_core::dal::Database::new(&db_path.to_string_lossy())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;
    let mut repo = db.into_repository();

    let strategies = repo.find_by_type("strategy")?;
    let initiatives = repo.find_by_type("initiative")?;
    let tasks = repo.find_by_type("task")?;

    assert_eq!(strategies.len(), 1, "Should have 1 strategy");
    assert_eq!(initiatives.len(), 1, "Should have 1 initiative");
    assert_eq!(tasks.len(), 2, "Should have 2 tasks");

    // All should be active (not archived)
    assert!(!strategies[0].archived, "Strategy should be active");
    assert!(!initiatives[0].archived, "Initiative should be active");
    assert_eq!(
        tasks.iter().filter(|t| t.archived).count(),
        0,
        "No tasks should be archived"
    );

    println!("✅ Initial state verified: 1 strategy, 1 initiative, 2 tasks (all active)");

    // Step 3: Archive individual task (should not cascade)
    println!("\n=== Step 3: Archive Individual Task ===");

    // Verify we have a task selected (first item selected by default)
    let selected_item = app.get_selected_item();
    assert!(selected_item.is_some(), "Should have task selected");
    println!("About to archive task: {}", selected_item.unwrap().id());

    // Archive the individual task
    let result = app.archive_selected_document().await;
    assert!(
        result.is_ok(),
        "Individual task archive should succeed: {:?}",
        result
    );

    // Reload and verify only one task is archived
    app.load_documents().await?;

    let db_tasks_after_single = repo.find_by_type("task")?;
    let db_strategies_after_single = repo.find_by_type("strategy")?;
    let db_initiatives_after_single = repo.find_by_type("initiative")?;

    let archived_tasks = db_tasks_after_single.iter().filter(|t| t.archived).count();
    let active_tasks = db_tasks_after_single.iter().filter(|t| !t.archived).count();

    assert_eq!(archived_tasks, 1, "Should have 1 archived task");
    assert_eq!(active_tasks, 1, "Should have 1 active task");
    assert!(
        !db_strategies_after_single[0].archived,
        "Strategy should still be active"
    );
    assert!(
        !db_initiatives_after_single[0].archived,
        "Initiative should still be active"
    );

    println!("✅ Individual task archived, no cascade effect");

    // Step 4: Archive strategy (should cascade to remaining children)
    println!("\n=== Step 4: Archive Strategy (Cascade Test) ===");

    // Switch to strategy board (first strategy selected by default)
    app.ui_state.current_board = BoardType::Strategy;

    // Verify we have the strategy selected
    let selected_item = app.get_selected_item();
    assert!(selected_item.is_some(), "Should have strategy selected");
    println!("About to archive strategy: {}", selected_item.unwrap().id());

    // Archive the strategy (should cascade to initiative and remaining task)
    let result = app.archive_selected_document().await;

    if result.is_ok() {
        println!("✅ Strategy archive succeeded - cascade working!");

        // Reload and verify full cascade happened
        app.load_documents().await?;

        let db_strategies_final = repo.find_by_type("strategy")?;
        let db_initiatives_final = repo.find_by_type("initiative")?;
        let db_tasks_final = repo.find_by_type("task")?;

        assert!(
            db_strategies_final[0].archived,
            "Strategy should be archived"
        );
        assert!(
            db_initiatives_final[0].archived,
            "Initiative should be archived due to cascade"
        );

        let final_archived_tasks = db_tasks_final.iter().filter(|t| t.archived).count();
        assert_eq!(
            final_archived_tasks, 2,
            "Both tasks should be archived (1 individual + 1 cascaded)"
        );

        println!("✅ Full cascade archiving successful");
        println!("   - Strategy: archived");
        println!("   - Initiative: archived (cascaded)");
        println!(
            "   - Tasks: {} archived (1 individual + 1 cascaded)",
            final_archived_tasks
        );
    } else {
        println!("❌ Strategy archive failed: {:?}", result);
        return Err(anyhow::anyhow!(
            "Strategy archive should have succeeded: {:?}",
            result
        ));
    }

    println!("\n=== TUI Archive Strategy with Partial Archive Test Summary ===");
    println!("1. Created hierarchy (Strategy -> Initiative -> 2 Tasks) ✅");
    println!("2. Verified initial database state ✅");
    println!("3. Archived individual task (no cascade) ✅");
    println!("4. Archived strategy (cascade to remaining) ✅");
    println!("5. Verified cascade behavior with partial archive ✅");

    Ok(())
}
