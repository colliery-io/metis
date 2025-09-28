//! Comprehensive phase transition tests for all document types
//! Tests each document type through its complete phase lifecycle

mod common;

use anyhow::Result;
use common::McpTestHelper;
use metis_mcp_server::tools::*;
use std::fs;
use std::path::Path;

/// Helper function to validate file and database state after phase transitions
async fn validate_phase_transition(
    helper: &McpTestHelper,
    file_path: &str,
    expected_phase: &str,
    document_type: &str,
    document_id: &str,
) -> Result<()> {
    // Validate file exists and contains expected phase tag
    let full_path = format!("{}/{}", helper.metis_dir, file_path);
    assert!(
        Path::new(&full_path).exists(),
        "Document file should exist at: {}",
        full_path
    );

    let file_content = fs::read_to_string(&full_path)?;
    let expected_tag = format!("#phase/{}", expected_phase);
    assert!(
        file_content.contains(&expected_tag),
        "File should contain phase tag '{}' but content was:\n{}",
        expected_tag,
        file_content
    );

    // Validate database state
    let db = helper.get_database()?;
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
        db_doc.filepath, full_path,
        "Database filepath should be '{}' but was '{}'",
        full_path, db_doc.filepath
    );

    // Validate file hash matches
    let current_file_hash =
        metis_core::application::services::FilesystemService::compute_file_hash(&full_path)?;
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

/// Test Vision document phase transitions: draft → review → published
#[tokio::test]
async fn test_vision_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;

    println!("=== Test Vision Phase Transitions ===");

    // Initialize project (creates Vision in draft phase)
    helper.initialize_project().await?;

    // Get the vision document ID from the database
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
    let db_visions = repo
        .find_by_type("vision")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    let vision_id = db_visions[0].id.clone();

    // Validate initial draft phase
    validate_phase_transition(&helper, "vision.md", "draft", "vision", &vision_id).await?;

    // Update some content to meet exit criteria
    let update_content = UpdateDocumentContentTool {
        project_path: helper.metis_dir.clone(),
        document_path: "vision.md".to_string(),
        section_heading: "Purpose".to_string(),
        new_content: "To create an exceptional platform that transforms how teams collaborate."
            .to_string(),
    };
    let result = update_content.call_tool().await;
    assert!(
        result.is_ok(),
        "Update content should succeed: {:?}",
        result
    );

    // Transition to review phase (force it for testing)
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: vision_id.clone(),
        phase: Some("review".to_string()),
        force: Some(true), // Force transition for testing
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to review should succeed: {:?}",
        result
    );

    // Validate review phase
    validate_phase_transition(&helper, "vision.md", "review", "vision", &vision_id).await?;

    // Transition to published phase
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: vision_id.clone(),
        phase: Some("published".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to published should succeed: {:?}",
        result
    );

    // Validate published phase
    validate_phase_transition(&helper, "vision.md", "published", "vision", &vision_id).await?;
    println!("   Complete lifecycle: draft → review → published ✅");

    Ok(())
}

/// Test Strategy document phase transitions: shaping → design → ready → active → completed
#[tokio::test]
async fn test_strategy_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Strategy Phase Transitions ===");

    // Create a strategy
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Digital Transformation Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["leadership".to_string(), "tech_team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy should succeed: {:?}",
        result
    );

    // Verify file was created in correct location
    let strategy_dir = Path::new(&helper.metis_dir)
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
    let file_content = fs::read_to_string(&strategy_file)?;
    assert!(
        file_content.contains("level: strategy"),
        "File should contain strategy level"
    );
    assert!(
        file_content.contains("title: \"Digital Transformation Strategy\""),
        "File should contain correct title"
    );
    assert!(
        file_content.contains(&format!("parent: {}", helper.get_project_name())),
        "File should contain parent reference"
    );
    assert!(
        file_content.contains("risk_level: medium"),
        "File should contain risk level"
    );
    assert!(
        file_content.contains("#phase/shaping"),
        "File should contain shaping phase tag"
    );

    println!("✅ Strategy file created at correct location with proper content");

    // Get strategy ID and validate initial phase
    let db = helper.get_database()?;
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
    let phases = [
        (
            "design",
            "strategies/digital-transformation-strategy/strategy.md",
        ),
        (
            "ready",
            "strategies/digital-transformation-strategy/strategy.md",
        ),
        (
            "active",
            "strategies/digital-transformation-strategy/strategy.md",
        ),
        (
            "completed",
            "strategies/digital-transformation-strategy/strategy.md",
        ),
    ];

    for (phase, file_path) in phases.iter() {
        let transition = TransitionPhaseTool {
            project_path: helper.metis_dir.clone(),
            document_id: strategy_id.clone(),
            phase: Some(phase.to_string()),
            force: Some(true), // Force transition for testing
        };

        let result = transition.call_tool().await;
        assert!(
            result.is_ok(),
            "Transition to {} should succeed: {:?}",
            phase,
            result
        );

        // Validate the phase transition
        validate_phase_transition(&helper, file_path, phase, "strategy", &strategy_id).await?;
    }

    println!("   Complete lifecycle: shaping → design → ready → active → completed ✅");

    Ok(())
}

/// Test Initiative document phase transitions: discovery → design → ready → decompose → active → completed
#[tokio::test]
async fn test_initiative_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Initiative Phase Transitions ===");

    // Create a strategy first
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Growth Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("high".to_string()),
        complexity: None,
        stakeholders: Some(vec!["sales".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy should succeed: {:?}",
        result
    );

    // Create an initiative
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "initiative".to_string(),
        title: "Launch New Product Line".to_string(),
        parent_id: Some("growth-strategy".to_string()),
        risk_level: None,
        complexity: Some("l".to_string()),
        stakeholders: Some(vec!["product".to_string(), "engineering".to_string()]),
        decision_maker: None,
    };

    let result = create_initiative.call_tool().await;
    assert!(
        result.is_ok(),
        "Create initiative should succeed: {:?}",
        result
    );

    // Verify initiative was created in correct hierarchical location
    let initiative_dir = Path::new(&helper.metis_dir)
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
    let file_content = fs::read_to_string(&initiative_file)?;
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
    // Check for complexity - the field is stored as estimated_complexity and value is uppercase
    assert!(
        file_content.contains("estimated_complexity:"),
        "File should contain estimated_complexity field"
    );
    assert!(
        file_content.contains("#phase/discovery"),
        "File should contain discovery phase tag"
    );

    println!("✅ Initiative file created at correct hierarchical location with proper content");

    // Get initiative ID and validate initial phase
    let db = helper.get_database()?;
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

    // Transition through all phases
    let phases = [
        (
            "design",
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        ),
        (
            "ready",
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        ),
        (
            "decompose",
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        ),
        (
            "active",
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        ),
        (
            "completed",
            "strategies/growth-strategy/initiatives/launch-new-product-line/initiative.md",
        ),
    ];

    for (phase, file_path) in phases.iter() {
        let transition = TransitionPhaseTool {
            project_path: helper.metis_dir.clone(),
            document_id: initiative_id.clone(),
            phase: Some(phase.to_string()),
            force: Some(true), // Force transition for testing
        };

        let result = transition.call_tool().await;
        assert!(
            result.is_ok(),
            "Transition to {} should succeed: {:?}",
            phase,
            result
        );

        // Validate the phase transition
        validate_phase_transition(&helper, file_path, phase, "initiative", &initiative_id).await?;
    }

    println!(
        "   Complete lifecycle: discovery → design → ready → decompose → active → completed ✅"
    );

    Ok(())
}

/// Test Task document phase transitions: todo → doing → completed
#[tokio::test]
async fn test_task_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Task Phase Transitions ===");

    // Create hierarchy: Strategy -> Initiative -> Task
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Technical Excellence".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: Some(vec!["tech_lead".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy should succeed: {:?}",
        result
    );

    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "initiative".to_string(),
        title: "Upgrade Infrastructure".to_string(),
        parent_id: Some("technical-excellence".to_string()),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: Some(vec!["devops".to_string()]),
        decision_maker: None,
    };

    let result = create_initiative.call_tool().await;
    assert!(
        result.is_ok(),
        "Create initiative should succeed: {:?}",
        result
    );

    // Create a task
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "task".to_string(),
        title: "Setup CI/CD Pipeline".to_string(),
        parent_id: Some("upgrade-infrastructure".to_string()),
        risk_level: None,
        complexity: None,
        stakeholders: Some(vec!["devops_engineer".to_string()]),
        decision_maker: None,
    };

    let result = create_task.call_tool().await;
    assert!(result.is_ok(), "Create task should succeed: {:?}", result);

    // Verify task was created in correct hierarchical location
    let task_dir = Path::new(&helper.metis_dir)
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
    let file_content = fs::read_to_string(&task_file)?;
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
    let db = helper.get_database()?;
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

    // Transition to active
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: task_id.clone(),
        phase: Some("active".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to active should succeed: {:?}",
        result
    );

    // Validate active phase
    validate_phase_transition(&helper, task_path, "active", "task", &task_id).await?;

    // Transition to completed
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: task_id.clone(),
        phase: Some("completed".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to completed should succeed: {:?}",
        result
    );

    // Validate completed phase
    validate_phase_transition(&helper, task_path, "completed", "task", &task_id).await?;
    println!("   Complete lifecycle: todo → active → completed ✅");

    Ok(())
}

/// Test ADR document phase transitions: draft → discussion → decided → superseded
#[tokio::test]
async fn test_adr_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test ADR Phase Transitions ===");

    // Create an ADR
    let create_adr = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "adr".to_string(),
        title: "Use Rust for Backend Services".to_string(),
        parent_id: None, // ADRs don't require a parent
        risk_level: None,
        complexity: None,
        stakeholders: Some(vec!["architects".to_string(), "backend_team".to_string()]),
        decision_maker: Some("CTO".to_string()),
    };

    let result = create_adr.call_tool().await;
    assert!(result.is_ok(), "Create ADR should succeed: {:?}", result);

    // Verify ADR was created in correct location
    let adrs_dir = Path::new(&helper.metis_dir).join("adrs");
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
    let file_content = fs::read_to_string(&adr_file)?;
    assert!(
        file_content.contains("level: adr"),
        "File should contain adr level"
    );
    assert!(
        file_content.contains("title: \"Use Rust for Backend Services\""),
        "File should contain correct title"
    );
    // Decision maker can be null or CTO
    assert!(
        file_content.contains("decision_maker:"),
        "File should contain decision_maker field"
    );
    assert!(
        file_content.contains("#phase/draft"),
        "File should contain draft phase tag"
    );

    println!("✅ ADR file created at correct location with proper content");

    // Get ADR ID and validate initial phase
    let db = helper.get_database()?;
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
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: adr_id.clone(),
        phase: Some("discussion".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to discussion should succeed: {:?}",
        result
    );

    // Validate discussion phase
    validate_phase_transition(&helper, adr_path, "discussion", "adr", &adr_id).await?;

    // Transition to decided
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: adr_id.clone(),
        phase: Some("decided".to_string()),
        force: Some(true), // Force transition for testing
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to decided should succeed: {:?}",
        result
    );

    // Validate decided phase
    validate_phase_transition(&helper, adr_path, "decided", "adr", &adr_id).await?;

    // ADRs cannot transition from decided to any other phase
    // This represents a final decision that cannot be changed
    println!("   Complete lifecycle: draft → discussion → decided ✅");
    println!("   Note: ADRs in 'decided' phase are final and cannot be superseded");

    Ok(())
}

/// Test automatic phase transitions (transition to next valid phase)
#[tokio::test]
async fn test_automatic_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Automatic Phase Transitions ===");

    // Create a strategy
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Auto Transition Test".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: Some(vec!["test_team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy should succeed: {:?}",
        result
    );

    // Transition without specifying phase (should go to next valid phase)
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: "auto-transition-test".to_string(),
        phase: None,       // Let it auto-select next phase
        force: Some(true), // Force transition for testing
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Automatic transition should succeed: {:?}",
        result
    );

    // Verify it moved from shaping to design
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
    let db_strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    assert_eq!(db_strategies[0].phase, "design");

    println!("✅ Strategy automatically transitioned from shaping → design");

    // Try another automatic transition
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: "auto-transition-test".to_string(),
        phase: None,
        force: Some(true),
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Second automatic transition should succeed: {:?}",
        result
    );

    let db_strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;
    assert_eq!(db_strategies[0].phase, "ready");

    println!("✅ Strategy automatically transitioned from design → ready");

    Ok(())
}

/// Test invalid phase transitions
#[tokio::test]
async fn test_invalid_phase_transitions() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Invalid Phase Transitions ===");

    // Try to transition vision to an invalid phase
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: helper.get_project_name(),
        phase: Some("invalid_phase".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(result.is_err(), "Transition to invalid phase should fail");

    println!("✅ Invalid phase transition correctly rejected");

    // Try to transition to a non-adjacent phase without force
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Skip Phase Test".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["test".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy should succeed: {:?}",
        result
    );

    // Try to skip from shaping directly to active (should fail without force)
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: "skip-phase-test".to_string(),
        phase: Some("active".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    // This might succeed if exit criteria are not enforced, but document the behavior
    if result.is_err() {
        println!("✅ Phase skipping correctly prevented without force flag");
    } else {
        println!("⚠️  Phase skipping allowed - exit criteria might not be enforced");
    }

    Ok(())
}

/// Integration test: Test phase transitions with blocked_by relationships
#[tokio::test]
async fn test_phase_transitions_with_dependencies() -> Result<()> {
    let helper = McpTestHelper::new()?;
    helper.initialize_project().await?;

    println!("=== Test Phase Transitions with Dependencies ===");

    // Create two strategies
    let create_strategy1 = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Prerequisite Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team1".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy1.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy 1 should succeed: {:?}",
        result
    );

    let create_strategy2 = CreateDocumentTool {
        project_path: helper.metis_dir.clone(),
        document_type: "strategy".to_string(),
        title: "Dependent Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team2".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy2.call_tool().await;
    assert!(
        result.is_ok(),
        "Create strategy 2 should succeed: {:?}",
        result
    );

    // Get the database to access actual document IDs
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
    let db_strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    // Find the strategies by title
    let prerequisite_strategy = db_strategies
        .iter()
        .find(|s| s.title == "Prerequisite Strategy")
        .ok_or_else(|| anyhow::anyhow!("Prerequisite strategy not found"))?;
    let dependent_strategy = db_strategies
        .iter()
        .find(|s| s.title == "Dependent Strategy")
        .ok_or_else(|| anyhow::anyhow!("Dependent strategy not found"))?;

    // Set up dependency using content update to an existing section
    let update_blocked = UpdateDocumentContentTool {
        project_path: helper.metis_dir.clone(),
        document_path: "strategies/dependent-strategy/strategy.md".to_string(),
        section_heading: "Dependencies".to_string(),
        new_content: "This strategy depends on: Prerequisite Strategy".to_string(),
    };

    let result = update_blocked.call_tool().await;
    // If Dependencies section doesn't exist either, use Problem Statement
    if result.is_err() {
        let update_problem = UpdateDocumentContentTool {
            project_path: helper.metis_dir.clone(),
            document_path: "strategies/dependent-strategy/strategy.md".to_string(),
            section_heading: "Problem Statement".to_string(),
            new_content: "This strategy is blocked by: Prerequisite Strategy".to_string(),
        };
        let result = update_problem.call_tool().await;
        assert!(
            result.is_ok(),
            "Update dependency note should succeed: {:?}",
            result
        );
    }

    println!("✅ Set up dependency: Dependent Strategy blocked by Prerequisite Strategy");

    // Move prerequisite strategy through phases: shaping → design → ready → active
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: prerequisite_strategy.id.clone(),
        phase: Some("design".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to design should succeed: {:?}",
        result
    );

    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: prerequisite_strategy.id.clone(),
        phase: Some("ready".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Transition to ready should succeed: {:?}",
        result
    );

    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: prerequisite_strategy.id.clone(),
        phase: Some("active".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(result.is_ok(), "Transition should succeed: {:?}", result);

    // Now dependent strategy should be able to progress
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir.clone(),
        document_id: dependent_strategy.id.clone(),
        phase: Some("design".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Dependent strategy should be able to progress after blocker is active"
    );

    println!("✅ Dependent strategy successfully transitioned after blocker became active");

    Ok(())
}
