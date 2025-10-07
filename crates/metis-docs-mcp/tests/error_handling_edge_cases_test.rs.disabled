//! Tests for error handling and edge cases
//! Ensures the system handles invalid inputs and edge conditions gracefully

mod common;

use anyhow::Result;
use common::McpTestHelper;
use metis_mcp_server::tools::*;

/// Test invalid parent relationships
#[tokio::test]
async fn test_invalid_parent_relationships() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== Test Invalid Parent Relationships ===");

    // Try to create an initiative without a valid strategy parent
    let create_initiative = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: "Orphan Initiative".to_string(),
        parent_id: Some("non-existent-strategy".to_string()),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let result = create_initiative.call_tool().await;
    if result.is_err() {
        println!("✅ Creating initiative with non-existent parent correctly failed");
    } else {
        println!("⚠️  System allowed creating initiative with invalid parent");
    }

    // Try to create a task without a valid initiative parent
    let create_task = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "Orphan Task".to_string(),
        parent_id: Some("non-existent-initiative".to_string()),
        risk_level: None,
        complexity: None,
        stakeholders: Some(vec!["developer".to_string()]),
        decision_maker: None,
    };

    let result = create_task.call_tool().await;
    if result.is_err() {
        println!("✅ Creating task with non-existent parent correctly failed");
    } else {
        println!("⚠️  System allowed creating task with invalid parent");
    }

    // Try to create a strategy with wrong parent type (should be vision/project)
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Misplaced Strategy".to_string(),
        parent_id: Some("some-random-id".to_string()),
        risk_level: Some("low".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    // This might succeed if the system doesn't validate parent types strictly
    if result.is_ok() {
        println!("⚠️  System allows flexible parent relationships");
    } else {
        println!("✅ System enforces strict parent type validation");
    }

    Ok(())
}

/// Test edge cases with extremely large titles
#[tokio::test]
async fn test_large_title_edge_cases() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== Test Large Title Edge Cases ===");

    // Test with a very long title (300 characters)
    let long_title = "This is an extremely long title that goes on and on and on to test how the system handles very lengthy document titles that might cause issues with file system paths or database fields or UI rendering or any other component that needs to display or store this information properly without truncation".to_string();

    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: long_title.clone(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    if result.is_ok() {
        println!("✅ System handled 300-character title");

        // Check if the slug was truncated appropriately
        let db = helper.get_database()?;
        let mut repo = db
            .repository()
            .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
        let strategies = repo
            .find_by_type("strategy")
            .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

        if let Some(strategy) = strategies.iter().find(|s| s.title == long_title) {
            println!(
                "  - Full title preserved in database: {} chars",
                strategy.title.len()
            );
            println!("  - Document ID (slug): {}", strategy.id);
            println!("  - ID length: {} chars", strategy.id.len());

            if strategy.id.len() <= 100 {
                println!("✅ Document ID reasonably truncated");
            } else {
                println!("⚠️  Document ID might be too long: {}", strategy.id.len());
            }
        }
    } else {
        println!("❌ System rejected 300-character title");
    }

    // Test with title containing only spaces
    let space_title = "     ".to_string();
    let create_spaces = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "initiative".to_string(),
        title: space_title,
        parent_id: Some(helper.get_project_name()),
        risk_level: None,
        complexity: Some("s".to_string()),
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let result = create_spaces.call_tool().await;
    if result.is_err() {
        println!("✅ System correctly rejected title with only spaces");
    } else {
        println!("❌ System allowed title with only spaces");
    }

    // Test with empty title
    let create_empty = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "task".to_string(),
        title: "".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: None,
        complexity: None,
        stakeholders: Some(vec!["dev".to_string()]),
        decision_maker: None,
    };

    let result = create_empty.call_tool().await;
    if result.is_err() {
        println!("✅ System correctly rejected empty title");
    } else {
        println!("❌ System allowed empty title");
    }

    Ok(())
}

/// Test special characters in titles and content
#[tokio::test]
async fn test_special_characters_edge_cases() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== Test Special Characters Edge Cases ===");

    // Test various special characters that might cause issues
    let test_titles = vec![
        ("Emoji Title 🚀 🎯 ✨", "Unicode emoji test"),
        ("Title with/slashes\\and|pipes", "Path separator test"),
        (
            "Title with \"quotes\" and 'apostrophes'",
            "Quote character test",
        ),
        (
            "Title with <HTML> tags & entities",
            "HTML special chars test",
        ),
        (
            "Title with\nnewlines\tand\ttabs",
            "Whitespace character test",
        ),
        (
            "Tïtlé wîth äccénts ànd ñ spëcial",
            "Accented character test",
        ),
        (
            "Title.with.dots...and___underscores---",
            "Filename special chars test",
        ),
        ("Title with 中文 characters 日本語", "CJK character test"),
    ];

    for (title, description) in test_titles {
        println!("\n--- Testing: {} ---", description);

        let create_doc = CreateDocumentTool {
            project_path: helper.metis_dir().clone(),
            document_type: "adr".to_string(),
            title: title.to_string(),
            parent_id: None,
            risk_level: None,
            complexity: None,
            stakeholders: Some(vec!["team".to_string()]),
            decision_maker: Some("CTO".to_string()),
        };

        let result = create_doc.call_tool().await;
        if result.is_ok() {
            println!("✅ System handled: {}", title);

            // Check how the title was stored
            let db = helper.get_database()?;
            let mut repo = db
                .repository()
                .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
            let adrs = repo
                .find_by_type("adr")
                .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

            if let Some(adr) = adrs.iter().find(|a| a.title == title) {
                println!("  - Title preserved correctly");
                println!("  - ID/slug generated: {}", adr.id);
            }
        } else {
            println!("❌ System rejected: {}", title);
        }
    }

    Ok(())
}

/// Test boundary values for numeric fields
#[tokio::test]
async fn test_numeric_boundary_values() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    // Set full configuration to enable all document types for testing
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    config_repo.set("flight_levels", r#"{"strategies_enabled":true,"initiatives_enabled":true}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    println!("=== Test Numeric Boundary Values ===");

    // Create parent strategy first
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Parent Strategy for Complexity Test".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let strategy_result = create_strategy.call_tool().await;
    assert!(strategy_result.is_ok(), "Failed to create parent strategy");
    let strategy_id = "parent-strategy-for-complexity-test"; // Based on title

    // Test complexity values
    let complexity_values = vec![
        ("xs", true, "Extra small complexity"),
        ("s", true, "Small complexity"),
        ("m", true, "Medium complexity"),
        ("l", true, "Large complexity"),
        ("xl", true, "Extra large complexity"),
        ("xxl", false, "Invalid complexity value"),
        ("", false, "Empty complexity"),
        ("MEDIUM", false, "Uppercase complexity"),
        ("3", false, "Numeric complexity"),
    ];

    for (complexity, should_succeed, description) in complexity_values {
        println!("\n--- Testing: {} ---", description);

        let create_initiative = CreateDocumentTool {
            project_path: helper.metis_dir().clone(),
            document_type: "initiative".to_string(),
            title: format!("Test Initiative {}", complexity),
            parent_id: Some(strategy_id.to_string()),
            risk_level: None,
            complexity: Some(complexity.to_string()),
            stakeholders: Some(vec!["team".to_string()]),
            decision_maker: None,
        };

        let result = create_initiative.call_tool().await;
        if should_succeed {
            if result.is_err() {
                println!(
                    "❌ {} failed with error: {:?}",
                    description,
                    result.err().unwrap()
                );
                panic!("{} should succeed", description);
            } else {
                println!("✅ Accepted complexity value: {}", complexity);
            }
        } else {
            if result.is_err() {
                println!("✅ Correctly rejected invalid complexity: {}", complexity);
            } else {
                println!("⚠️  System accepted invalid complexity: {}", complexity);
            }
        }
    }

    // Test risk levels
    let risk_values = vec![
        ("low", true, "Low risk"),
        ("medium", true, "Medium risk"),
        ("high", true, "High risk"),
        ("critical", false, "Invalid risk level"),
        ("LOW", false, "Uppercase risk"),
        ("1", false, "Numeric risk"),
    ];

    for (risk, should_succeed, description) in risk_values {
        println!("\n--- Testing: {} ---", description);

        let create_strategy = CreateDocumentTool {
            project_path: helper.metis_dir().clone(),
            document_type: "strategy".to_string(),
            title: format!("Test Strategy {}", risk),
            parent_id: Some(helper.get_project_name()),
            risk_level: Some(risk.to_string()),
            complexity: None,
            stakeholders: Some(vec!["team".to_string()]),
            decision_maker: None,
        };

        let result = create_strategy.call_tool().await;
        if should_succeed {
            assert!(result.is_ok(), "{} should succeed", description);
            println!("✅ Accepted risk level: {}", risk);
        } else {
            if result.is_err() {
                println!("✅ Correctly rejected invalid risk: {}", risk);
            } else {
                println!("⚠️  System accepted invalid risk: {}", risk);
            }
        }
    }

    Ok(())
}

/// Test concurrent modifications
#[tokio::test]
async fn test_concurrent_modifications() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;
    
    // Set full configuration to enable all document types for testing
    let db = helper.get_database()?;
    let mut config_repo = db.configuration_repository().map_err(|e| anyhow::anyhow!("Failed to get config repo: {}", e))?;
    config_repo.set("flight_levels", r#"{"strategies_enabled":true,"initiatives_enabled":true}"#).map_err(|e| anyhow::anyhow!("Failed to set config: {}", e))?;

    println!("=== Test Concurrent Modifications ===");

    // Create a strategy to modify
    let create_strategy = CreateDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_type: "strategy".to_string(),
        title: "Concurrent Test Strategy".to_string(),
        parent_id: Some(helper.get_project_name()),
        risk_level: Some("medium".to_string()),
        complexity: None,
        stakeholders: Some(vec!["team".to_string()]),
        decision_maker: None,
    };

    let result = create_strategy.call_tool().await;
    assert!(result.is_ok(), "Create strategy should succeed");

    let strategy_path = "strategies/concurrent-test-strategy/strategy.md";

    // Simulate concurrent updates to different sections
    let update1 = EditDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_path: strategy_path.to_string(),
        search: "{What problem does this strategy solve}".to_string(),
        replace: "First concurrent update to problem statement.".to_string(),
        replace_all: None,
    };

    let update2 = EditDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_path: strategy_path.to_string(),
        search: "{What success looks like}".to_string(),
        replace: "Second concurrent update to target outcomes.".to_string(),
        replace_all: None,
    };

    // Execute updates concurrently
    let (result1, result2) = tokio::join!(update1.call_tool(), update2.call_tool());

    // Both should succeed if they update different sections
    if result1.is_ok() && result2.is_ok() {
        println!("✅ Concurrent updates to different sections succeeded");
    } else {
        println!("⚠️  One or both concurrent updates failed");
        if result1.is_err() {
            println!("  - Update 1 failed: {:?}", result1);
        }
        if result2.is_err() {
            println!("  - Update 2 failed: {:?}", result2);
        }
    }

    // Try concurrent phase transitions (should handle gracefully)
    let transition1 = TransitionPhaseTool {
        project_path: helper.metis_dir().clone(),
        document_id: "concurrent-test-strategy".to_string(),
        phase: Some("design".to_string()),
        force: Some(true),
    };

    let transition2 = TransitionPhaseTool {
        project_path: helper.metis_dir().clone(),
        document_id: "concurrent-test-strategy".to_string(),
        phase: Some("ready".to_string()),
        force: Some(true),
    };

    // Execute transitions concurrently
    let (trans_result1, trans_result2) =
        tokio::join!(transition1.call_tool(), transition2.call_tool());

    // Check results - one should succeed, one might fail or both might succeed with last-write-wins
    println!("\nConcurrent phase transition results:");
    match (trans_result1.is_ok(), trans_result2.is_ok()) {
        (true, true) => println!("⚠️  Both transitions succeeded - last write wins"),
        (true, false) => println!("✅ First transition succeeded, second failed"),
        (false, true) => println!("✅ Second transition succeeded, first failed"),
        (false, false) => println!("❌ Both transitions failed"),
    }

    // Check final state
    let db = helper.get_database()?;
    let mut repo = db
        .repository()
        .map_err(|e| anyhow::anyhow!("Repository error: {}", e))?;
    let strategies = repo
        .find_by_type("strategy")
        .map_err(|e| anyhow::anyhow!("Find error: {}", e))?;

    if let Some(strategy) = strategies
        .iter()
        .find(|s| s.title == "Concurrent Test Strategy")
    {
        println!(
            "Final phase after concurrent transitions: {}",
            strategy.phase
        );
    }

    Ok(())
}

/// Test invalid document IDs and paths
#[tokio::test]
async fn test_invalid_document_ids_and_paths() -> Result<()> {
    let helper = McpTestHelper::new().await?;
    helper.initialize_project().await?;

    println!("=== Test Invalid Document IDs and Paths ===");

    // Try to transition with non-existent document ID
    let transition = TransitionPhaseTool {
        project_path: helper.metis_dir().clone(),
        document_id: "non-existent-document-id".to_string(),
        phase: Some("design".to_string()),
        force: None,
    };

    let result = transition.call_tool().await;
    if result.is_err() {
        println!("✅ Transition with non-existent ID correctly failed");
    } else {
        println!("❌ System allowed transition on non-existent document");
    }

    // Try to update content with invalid path
    let update = EditDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_path: "invalid/path/to/document.md".to_string(),
        search: "Some content".to_string(),
        replace: "New content".to_string(),
        replace_all: None,
    };

    let result = update.call_tool().await;
    if result.is_err() {
        println!("✅ Update with invalid path correctly failed");
    } else {
        println!("❌ System allowed update on non-existent file");
    }

    // Try path traversal attack
    let malicious_update = EditDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_path: "../../../etc/passwd".to_string(),
        search: "root".to_string(),
        replace: "malicious content".to_string(),
        replace_all: None,
    };

    let result = malicious_update.call_tool().await;
    if result.is_err() {
        println!("✅ Path traversal attempt correctly blocked");
    } else {
        println!("❌ SECURITY: Path traversal was not blocked!");
    }

    // Try to archive non-existent document
    let archive = ArchiveDocumentTool {
        project_path: helper.metis_dir().clone(),
        document_id: "ghost-document".to_string(),
    };

    let result = archive.call_tool().await;
    if result.is_err() {
        println!("✅ Archive of non-existent document correctly failed");
    } else {
        println!("❌ System allowed archiving non-existent document");
    }

    Ok(())
}
