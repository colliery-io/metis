//! Comprehensive phase transition tests for all document types in TUI
//! Tests each document type through its complete phase lifecycle with file and DB validation

mod common;

use anyhow::Result;
use common::TestHelper;
use metis_core::application::services::FilesystemService;
use metis_docs_tui::models::{AppState, BoardType};
use tui_input::backend::crossterm::EventHandler;

/// Helper function to validate file and database state after phase transitions
async fn validate_phase_transition(
    helper: &TestHelper,
    file_path: &str,
    expected_phase: &str,
    document_type: &str,
    document_id: &str,
) -> Result<()> {
    // Validate file exists and contains expected phase tag
    let full_path = helper.metis_dir.join(file_path);
    assert!(
        full_path.exists(),
        "Document file should exist at: {}",
        full_path.display()
    );

    let file_content = std::fs::read_to_string(&full_path)?;
    let expected_tag = format!("#phase/{}", expected_phase);
    assert!(
        file_content.contains(&expected_tag),
        "File should contain phase tag '{}' but content was:\n{}",
        expected_tag,
        file_content
    );

    // Validate database state
    let db = metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    // Find document by ID
    let db_doc = repo
        .find_by_id(document_id)
        .map_err(|e| anyhow::anyhow!("Find by ID error: {}", e))?
        .ok_or_else(|| anyhow::anyhow!("Document '{}' not found in database", document_id))?;

    // Validate database fields
    assert_eq!(
        db_doc.phase, expected_phase,
        "Database phase should be '{}' but was '{}'",
        expected_phase, db_doc.phase
    );
    assert_eq!(
        db_doc.document_type, document_type,
        "Database document type should be '{}' but was '{}'",
        document_type, db_doc.document_type
    );
    assert_eq!(
        db_doc.filepath,
        full_path.to_string_lossy(),
        "Database filepath should be '{}' but was '{}'",
        full_path.display(),
        db_doc.filepath
    );

    // Validate file hash matches
    let current_file_hash = FilesystemService::compute_file_hash(&full_path)?;
    assert_eq!(
        db_doc.file_hash, current_file_hash,
        "Database file hash should match current file hash"
    );

    println!(
        "✅ Validated phase '{}' - File and DB are in sync",
        expected_phase
    );
    Ok(())
}

/// Test Strategy document phase transitions: shaping → design → ready → active → completed
#[tokio::test]
async fn test_strategy_phase_transitions() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test Strategy Phase Transitions ===");

    // Create a strategy
    app.load_documents().await?;
    app.jump_to_strategy_board();

    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    let strategy_title = "Digital Transformation Strategy";
    for ch in strategy_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }

    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify file was created in correct location
    let strategy_dir = helper
        .metis_dir
        .join("strategies")
        .join("digital-transformation-strategy");
    assert!(
        strategy_dir.exists(),
        "Strategy directory should exist at: {:?}",
        strategy_dir
    );
    assert!(strategy_dir.is_dir(), "Strategy path should be a directory");

    let strategy_file = strategy_dir.join("strategy.md");
    assert!(
        strategy_file.exists(),
        "Strategy file should exist at: {:?}",
        strategy_file
    );
    assert!(strategy_file.is_file(), "Strategy path should be a file");

    // Verify file content
    let file_content = std::fs::read_to_string(&strategy_file)?;
    assert!(
        file_content.contains("level: strategy"),
        "File should contain strategy level"
    );
    assert!(
        file_content.contains("title: \"Digital Transformation Strategy\""),
        "File should contain correct title"
    );
    // Parent should match project name from initialization
    assert!(
        file_content.contains("parent:"),
        "File should contain parent field"
    );
    assert!(
        file_content.contains("#phase/shaping"),
        "File should contain shaping phase tag"
    );

    println!("✅ Strategy file created at correct location with proper content");

    // Get strategy ID and validate initial phase
    let db = metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    // Verify database record
    assert_eq!(
        db_strategies.len(),
        1,
        "Should have exactly 1 strategy in database"
    );
    let strategy_id = db_strategies[0].id.clone();
    assert_eq!(
        db_strategies[0].title, "Digital Transformation Strategy",
        "Database should have correct title"
    );
    assert_eq!(
        db_strategies[0].document_type, "strategy",
        "Database should have correct type"
    );
    assert_eq!(
        db_strategies[0].phase, "shaping",
        "Database should have correct phase"
    );
    assert_eq!(
        db_strategies[0].archived, false,
        "Strategy should not be archived"
    );

    // Validate complete state including file hash
    validate_phase_transition(
        &helper,
        "strategies/digital-transformation-strategy/strategy.md",
        "shaping",
        "strategy",
        &strategy_id,
    )
    .await?;

    // Transition through all phases
    let phases = ["design", "ready", "active", "completed"];
    let expected_columns = [1, 2, 3, 4]; // Column indices for each phase

    for (i, (phase, expected_col)) in phases.iter().zip(expected_columns.iter()).enumerate() {
        // Load documents to refresh board
        app.load_documents().await?;

        // Select the strategy (it moves columns as it transitions)
        let current_col = if i == 0 { 0 } else { expected_columns[i - 1] };
        while app
            .selection_state
            .get_current_selection(BoardType::Strategy)
            .0
            != current_col
        {
            app.move_selection_right();
        }

        // Transition to next phase
        app.transition_selected_document().await?;

        // Validate the phase transition
        validate_phase_transition(
            &helper,
            "strategies/digital-transformation-strategy/strategy.md",
            phase,
            "strategy",
            &strategy_id,
        )
        .await?;
    }

    println!("   Complete lifecycle: shaping → design → ready → active → completed ✅");

    Ok(())
}

/// Test Initiative document phase transitions: discovery → design → ready → decompose → active → completed
#[tokio::test]
async fn test_initiative_phase_transitions() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test Initiative Phase Transitions ===");

    // Create a strategy first
    app.load_documents().await?;
    app.jump_to_strategy_board();

    app.start_document_creation();
    let strategy_title = "Growth Strategy";
    for ch in strategy_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_new_document().await?;

    // Create an initiative
    app.start_child_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingChildDocument);

    let initiative_title = "Launch New Product Line";
    for ch in initiative_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }

    app.create_child_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify initiative was created in correct hierarchical location
    let initiative_dir = helper
        .metis_dir
        .join("strategies")
        .join("growth-strategy")
        .join("initiatives")
        .join("launch-new-product-line");
    assert!(
        initiative_dir.exists(),
        "Initiative directory should exist at: {:?}",
        initiative_dir
    );
    assert!(
        initiative_dir.is_dir(),
        "Initiative path should be a directory"
    );

    let initiative_file = initiative_dir.join("initiative.md");
    assert!(
        initiative_file.exists(),
        "Initiative file should exist at: {:?}",
        initiative_file
    );
    assert!(
        initiative_file.is_file(),
        "Initiative path should be a file"
    );

    // Verify file content
    let file_content = std::fs::read_to_string(&initiative_file)?;
    assert!(
        file_content.contains("level: initiative"),
        "File should contain initiative level"
    );
    assert!(
        file_content.contains("title: \"Launch New Product Line\""),
        "File should contain correct title"
    );
    assert!(
        file_content.contains("parent: growth-strategy"),
        "File should contain parent reference"
    );
    assert!(
        file_content.contains("#phase/discovery"),
        "File should contain discovery phase tag"
    );

    println!("✅ Initiative file created at correct hierarchical location with proper content");

    // Get initiative ID and validate initial phase
    let db = metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_initiatives = repo
        .find_by_type("initiative")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    // Verify database record
    assert_eq!(
        db_initiatives.len(),
        1,
        "Should have exactly 1 initiative in database"
    );
    let initiative_id = db_initiatives[0].id.clone();
    assert_eq!(
        db_initiatives[0].title, "Launch New Product Line",
        "Database should have correct title"
    );
    assert_eq!(
        db_initiatives[0].document_type, "initiative",
        "Database should have correct type"
    );
    assert_eq!(
        db_initiatives[0].phase, "discovery",
        "Database should have correct phase"
    );
    assert_eq!(
        db_initiatives[0].archived, false,
        "Initiative should not be archived"
    );

    // Validate complete state including file hash
    validate_phase_transition(
        &helper,
        "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        "discovery",
        "initiative",
        &initiative_id,
    )
    .await?;

    // Switch to initiative board
    app.jump_to_initiative_board();
    assert_eq!(app.ui_state.current_board, BoardType::Initiative);

    // Transition through all phases
    let phases = ["design", "ready", "decompose", "active", "completed"];
    let expected_columns = [1, 2, 3, 4, 5]; // Column indices for each phase

    for (i, (phase, expected_col)) in phases.iter().zip(expected_columns.iter()).enumerate() {
        // Load documents to refresh board
        app.load_documents().await?;

        // Select the initiative (it moves columns as it transitions)
        let current_col = if i == 0 { 0 } else { expected_columns[i - 1] };
        while app
            .selection_state
            .get_current_selection(BoardType::Initiative)
            .0
            != current_col
        {
            app.move_selection_right();
        }

        // Transition to next phase
        app.transition_selected_document().await?;

        // Validate the phase transition
        validate_phase_transition(
            &helper,
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
            phase,
            "initiative",
            &initiative_id,
        )
        .await?;
    }

    println!(
        "   Complete lifecycle: discovery → design → ready → decompose → active → completed ✅"
    );

    Ok(())
}

/// Test Task document phase transitions: todo → active → completed
#[tokio::test]
async fn test_task_phase_transitions() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test Task Phase Transitions ===");

    // Create hierarchy: Strategy -> Initiative -> Task
    app.load_documents().await?;
    app.jump_to_strategy_board();

    // Create strategy
    app.start_document_creation();
    let strategy_title = "Technical Excellence";
    for ch in strategy_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_new_document().await?;

    // Create initiative
    app.start_child_document_creation();
    let initiative_title = "Upgrade Infrastructure";
    for ch in initiative_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_child_document().await?;

    // Switch to initiative board to create task
    app.jump_to_initiative_board();
    app.load_documents().await?;

    // Create task
    app.start_child_document_creation();
    let task_title = "Setup CI/CD Pipeline";
    for ch in task_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_child_document().await?;

    // Verify task was created in correct hierarchical location
    let task_dir = helper
        .metis_dir
        .join("strategies")
        .join("technical-excellence")
        .join("initiatives")
        .join("upgrade-infrastructure");
    assert!(
        task_dir.exists(),
        "Task parent directory should exist at: {:?}",
        task_dir
    );
    assert!(task_dir.is_dir(), "Task parent path should be a directory");

    let task_file = task_dir.join("setup-ci-cd-pipeline.md");
    assert!(
        task_file.exists(),
        "Task file should exist at: {:?}",
        task_file
    );
    assert!(task_file.is_file(), "Task path should be a file");

    // Verify file content
    let file_content = std::fs::read_to_string(&task_file)?;
    assert!(
        file_content.contains("level: task"),
        "File should contain task level"
    );
    assert!(
        file_content.contains("title: \"Setup CI/CD Pipeline\""),
        "File should contain correct title"
    );
    assert!(
        file_content.contains("parent: upgrade-infrastructure"),
        "File should contain parent reference"
    );
    assert!(
        file_content.contains("#phase/todo"),
        "File should contain todo phase tag"
    );

    println!("✅ Task file created at correct hierarchical location with proper content");

    // Get task ID and validate initial phase
    let db = metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_tasks = repo
        .find_by_type("task")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    // Verify database record
    assert_eq!(db_tasks.len(), 1, "Should have exactly 1 task in database");
    let task_id = db_tasks[0].id.clone();
    assert_eq!(
        db_tasks[0].title, "Setup CI/CD Pipeline",
        "Database should have correct title"
    );
    assert_eq!(
        db_tasks[0].document_type, "task",
        "Database should have correct type"
    );
    assert_eq!(
        db_tasks[0].phase, "todo",
        "Database should have correct phase"
    );
    assert_eq!(db_tasks[0].archived, false, "Task should not be archived");

    // Validate complete state including file hash
    let task_path = "strategies/technical-excellence/initiatives/upgrade-infrastructure/setup-ci-cd-pipeline.md";
    validate_phase_transition(&helper, task_path, "todo", "task", &task_id).await?;

    // Switch to task board
    app.jump_to_task_board();
    assert_eq!(app.ui_state.current_board, BoardType::Task);
    app.load_documents().await?;

    // Transition to active
    app.transition_selected_document().await?;
    validate_phase_transition(&helper, task_path, "active", "task", &task_id).await?;

    // Load documents and move to active column
    app.load_documents().await?;
    app.move_selection_right();

    // Transition to completed
    app.transition_selected_document().await?;
    validate_phase_transition(&helper, task_path, "completed", "task", &task_id).await?;

    println!("   Complete lifecycle: todo → active → completed ✅");

    Ok(())
}

/// Test ADR document phase transitions: draft → discussion → decided
#[tokio::test]
async fn test_adr_phase_transitions() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test ADR Phase Transitions ===");

    // Create an ADR
    app.load_documents().await?;
    app.jump_to_adr_board();

    app.start_document_creation();
    assert_eq!(*app.app_state(), AppState::CreatingDocument);

    let adr_title = "Use Rust for Backend Services";
    for ch in adr_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }

    app.create_new_document().await?;
    assert_eq!(*app.app_state(), AppState::Normal);

    // Verify ADR was created in correct location
    let adrs_dir = helper.metis_dir.join("adrs");
    assert!(
        adrs_dir.exists(),
        "ADRs directory should exist at: {:?}",
        adrs_dir
    );
    assert!(adrs_dir.is_dir(), "ADRs path should be a directory");

    // ADRs have numbered filenames
    let adr_file = adrs_dir.join("001-use-rust-for-backend-services.md");
    assert!(
        adr_file.exists(),
        "ADR file should exist at: {:?}",
        adr_file
    );
    assert!(adr_file.is_file(), "ADR path should be a file");

    // Verify file content
    let file_content = std::fs::read_to_string(&adr_file)?;
    assert!(
        file_content.contains("level: adr"),
        "File should contain adr level"
    );
    assert!(
        file_content.contains("title: \"Use Rust for Backend Services\""),
        "File should contain correct title"
    );
    assert!(
        file_content.contains("#phase/draft"),
        "File should contain draft phase tag"
    );

    println!("✅ ADR file created at correct location with proper content");

    // Get ADR ID and validate initial phase
    let db = metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
        .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

    let db_adrs = repo
        .find_by_type("adr")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    // Verify database record
    assert_eq!(db_adrs.len(), 1, "Should have exactly 1 ADR in database");
    let adr_id = db_adrs[0].id.clone();
    assert_eq!(
        db_adrs[0].title, "Use Rust for Backend Services",
        "Database should have correct title"
    );
    assert_eq!(
        db_adrs[0].document_type, "adr",
        "Database should have correct type"
    );
    assert_eq!(
        db_adrs[0].phase, "draft",
        "Database should have correct phase"
    );
    assert_eq!(db_adrs[0].archived, false, "ADR should not be archived");

    // Validate complete state including file hash
    let adr_path = "adrs/001-use-rust-for-backend-services.md";
    validate_phase_transition(&helper, adr_path, "draft", "adr", &adr_id).await?;

    // Transition to discussion
    app.load_documents().await?;
    app.transition_selected_document().await?;
    validate_phase_transition(&helper, adr_path, "discussion", "adr", &adr_id).await?;

    // Load documents and move to discussion column
    app.load_documents().await?;
    app.move_selection_right();

    // Transition to decided
    app.transition_selected_document().await?;
    validate_phase_transition(&helper, adr_path, "decided", "adr", &adr_id).await?;

    // ADRs cannot transition from decided to any other phase
    // This represents a final decision that cannot be changed
    println!("   Complete lifecycle: draft → discussion → decided ✅");
    println!("   Note: ADRs in 'decided' phase are final and cannot be superseded");

    Ok(())
}

/// Test invalid phase transitions
#[tokio::test]
async fn test_invalid_phase_transitions() -> Result<()> {
    let helper = TestHelper::new().await?;
    let mut app = helper.create_app();

    println!("=== Test Invalid Phase Transitions ===");

    // Create an ADR which has a defined final state (decided)
    app.load_documents().await?;
    app.jump_to_adr_board();

    app.start_document_creation();
    let adr_title = "Test Invalid Transition";
    for ch in adr_title.chars() {
        app.ui_state
            .input_title
            .handle_event(&crossterm::event::Event::Key(
                crossterm::event::KeyEvent::new(
                    crossterm::event::KeyCode::Char(ch),
                    crossterm::event::KeyModifiers::empty(),
                ),
            ));
    }
    app.create_new_document().await?;

    // Transition through valid phases: draft -> discussion -> decided
    app.load_documents().await?;
    app.transition_selected_document().await?; // draft -> discussion

    app.load_documents().await?;
    app.move_selection_right();
    app.transition_selected_document().await?; // discussion -> decided

    // Now try to transition from decided (which should fail)
    app.load_documents().await?;
    app.move_selection_right();

    // Attempt invalid transition from decided
    let result = app.transition_selected_document().await;

    // Check if we have an error
    if let Some(error) = app.error_message() {
        println!("✅ Invalid phase transition correctly rejected: {}", error);
        assert!(
            error.contains("Invalid phase transition") || error.contains("cannot transition"),
            "Error should indicate invalid transition"
        );
    } else if result.is_err() {
        println!("✅ Invalid phase transition correctly rejected with error");
    } else {
        // The transition might have silently failed, check that phase hasn't changed
        let db =
            metis_core::dal::Database::new(helper.metis_dir.join("metis.db").to_str().unwrap())
                .map_err(|e| anyhow::anyhow!("Database error: {}", e))?;

        let mut repo = db
            .repository()
            .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;

        let db_adrs = repo
            .find_by_type("adr")
            .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

        assert_eq!(
            db_adrs[0].phase, "decided",
            "ADR should still be in decided phase"
        );
        println!("✅ Invalid phase transition prevented - document remains in decided phase");
    }

    Ok(())
}
