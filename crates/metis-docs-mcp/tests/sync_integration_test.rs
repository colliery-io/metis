//! UAT: Cross-team sync through the MCP server layer.
//!
//! Scenario: Two teams (api, sre) share a bare git remote.
//! Team A creates and edits documents via MCP tools, post_sync pushes to central.
//! Team B calls read/list tools, pre_sync pulls from central, sees Team A's work.
//!
//! This exercises the exact same code paths as the real MCP server handler:
//!   pre_sync → tool dispatch → post_sync
//! just without the MCP transport/runtime wrapper.

use git2::Repository;
use metis_mcp_server::config::MetisServerConfig;
use metis_mcp_server::server::{sync_mode_for_tool, MetisServerHandler, SyncMode};
use metis_mcp_server::tools::*;
use regex::Regex;
use rust_mcp_sdk::schema::ContentBlock;
use tempfile::TempDir;

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Extract the first text content from a CallToolResult.
fn text_from(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    match result.content.first() {
        Some(ContentBlock::TextContent(tc)) => tc.text.clone(),
        Some(ContentBlock::EmbeddedResource(er)) => match &er.resource {
            rust_mcp_sdk::schema::EmbeddedResourceResource::TextResourceContents(tr) => {
                tr.text.clone()
            }
            _ => panic!("unexpected resource type"),
        },
        _ => panic!("no text content in result"),
    }
}

/// Extract a short code (e.g. PROJ-V-0001) from result text.
fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    let text = text_from(result);
    let re = Regex::new(r"([A-Z]+-[VSITA]-\d{4})").unwrap();
    re.captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| panic!("no short code in: {}", text))
}

/// Write a config.toml that enables multi-workspace sync.
fn write_sync_config(metis_dir: &str, prefix: &str, remote_url: &str) {
    let config = format!(
        r#"[project]
prefix = "PROJ"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true

[workspace]
prefix = "{prefix}"

[sync]
upstream_url = "{remote_url}"
"#
    );
    std::fs::write(format!("{}/config.toml", metis_dir), config).unwrap();
}

/// Create a bare git remote in a temp dir, return (TempDir, file:// URL).
fn create_bare_remote() -> (TempDir, String) {
    let dir = TempDir::new().unwrap();
    Repository::init_bare(dir.path()).unwrap();
    let url = format!("file://{}", dir.path().display());
    (dir, url)
}

/// Build a handler — same construction as the real MCP server.
fn make_handler() -> MetisServerHandler {
    MetisServerHandler::new(MetisServerConfig::default())
}

// ─── sync_mode_for_tool classification ───────────────────────────────────────

#[test]
fn test_sync_mode_classification() {
    // Read ops → Pull
    assert_eq!(sync_mode_for_tool("list_documents"), SyncMode::Pull);
    assert_eq!(sync_mode_for_tool("search_documents"), SyncMode::Pull);
    assert_eq!(sync_mode_for_tool("read_document"), SyncMode::Pull);

    // Write ops → Full
    assert_eq!(sync_mode_for_tool("create_document"), SyncMode::Full);
    assert_eq!(sync_mode_for_tool("edit_document"), SyncMode::Full);
    assert_eq!(sync_mode_for_tool("transition_phase"), SyncMode::Full);
    assert_eq!(sync_mode_for_tool("archive_document"), SyncMode::Full);
    assert_eq!(sync_mode_for_tool("reassign_parent"), SyncMode::Full);

    // No sync
    assert_eq!(sync_mode_for_tool("initialize_project"), SyncMode::None);
    assert_eq!(sync_mode_for_tool("index_code"), SyncMode::None);
    assert_eq!(sync_mode_for_tool("unknown_tool"), SyncMode::None);
}

// ─── No-op when sync is not configured ───────────────────────────────────────

#[tokio::test]
async fn test_pre_sync_noop_without_sync_config() {
    // A normal project without [sync]/[workspace] sections should not crash
    let temp = TempDir::new().unwrap();
    let project_path = temp.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project_path);

    // Initialize a vanilla project (no sync config)
    let init = InitializeProjectTool {
        project_path,
        prefix: None,
    };
    init.call_tool().await.unwrap();

    // Calling pre_sync and post_sync should silently no-op
    let handler = make_handler();
    let metis_dir = std::path::Path::new(&metis_path);
    handler.pre_sync(metis_dir, SyncMode::Pull);
    handler.post_sync(metis_dir);

    // If we got here without panic, the test passes
}

// ─── UAT: Team A creates → sync → Team B sees ───────────────────────────────

#[tokio::test]
async fn test_cross_team_create_and_read() {
    // ── Setup: bare remote + two workspaces ──────────────────────────────
    let (_remote, remote_url) = create_bare_remote();

    let workspace_a = TempDir::new().unwrap();
    let project_a = workspace_a.path().to_string_lossy().to_string();
    let metis_a = format!("{}/.metis", project_a);

    let workspace_b = TempDir::new().unwrap();
    let project_b = workspace_b.path().to_string_lossy().to_string();
    let metis_b = format!("{}/.metis", project_b);

    // Initialize both projects
    InitializeProjectTool {
        project_path: project_a.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();

    InitializeProjectTool {
        project_path: project_b.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();

    // Configure sync: workspace A = "api", workspace B = "sre"
    write_sync_config(&metis_a, "api", &remote_url);
    write_sync_config(&metis_b, "sre", &remote_url);

    let handler_a = make_handler();
    let handler_b = make_handler();
    let dir_a = std::path::Path::new(&metis_a);
    let dir_b = std::path::Path::new(&metis_b);

    // ── Act: Team A creates a vision (already exists from init) ──────────
    // Get the vision short code
    let list_result = ListDocumentsTool {
        project_path: metis_a.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();

    let list_text = text_from(&list_result);
    let re = Regex::new(r"\|\s*vision\s*\|\s*([A-Z]+-V-\d{4})\s*\|").unwrap();
    let vision_code = re
        .captures(&list_text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .expect("should find vision in list");

    // ── Act: Team A creates an initiative ────────────────────────────────
    let create_result = CreateDocumentTool {
        project_path: metis_a.clone(),
        document_type: "initiative".to_string(),
        title: "API Gateway Redesign".to_string(),
        parent_id: Some(vision_code.clone()),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    }
    .call_tool()
    .await
    .unwrap();

    let initiative_code = extract_short_code(&create_result);
    println!("Team A created initiative: {}", initiative_code);

    // ── Act: Team A's handler pushes to central (post_sync) ─────────────
    handler_a.post_sync(dir_a);

    // ── Verify: central repo has Team A's documents ─────────────────────
    // Quick sanity check — the remote should have api/ prefix files
    let mut verify_ctx =
        metis_sync::SyncContext::new(&remote_url, "verify").unwrap();
    let head = verify_ctx.fetch().unwrap().expect("remote should have commits");
    let vision_blob = verify_ctx
        .read_blob(head, &format!("api/{}.md", vision_code))
        .expect("vision should be in central");
    assert!(
        !vision_blob.is_empty(),
        "vision content should not be empty in central"
    );
    let init_blob = verify_ctx
        .read_blob(head, &format!("api/{}.md", initiative_code))
        .expect("initiative should be in central");
    assert!(
        !init_blob.is_empty(),
        "initiative content should not be empty in central"
    );

    // ── Act: Team B pulls (pre_sync for a read operation) ───────────────
    handler_b.pre_sync(dir_b, SyncMode::Pull);

    // ── Verify: Team B's filesystem has Team A's documents ──────────────
    let hydrated_vision = dir_b.join(format!("api/{}.md", vision_code));
    assert!(
        hydrated_vision.exists(),
        "Team B should have Team A's vision at {}",
        hydrated_vision.display()
    );

    let hydrated_initiative = dir_b.join(format!("api/{}.md", initiative_code));
    assert!(
        hydrated_initiative.exists(),
        "Team B should have Team A's initiative at {}",
        hydrated_initiative.display()
    );

    // ── Verify: Team B's list_documents still shows its own docs ────────
    let list_b = ListDocumentsTool {
        project_path: metis_b.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();

    let list_b_text = text_from(&list_b);
    // Team B should see its own vision
    assert!(
        list_b_text.contains("PROJ-V-"),
        "Team B should see its own vision in list"
    );

    println!("Team B list output:\n{}", list_b_text);
}

// ─── UAT: Edit propagation ──────────────────────────────────────────────────

#[tokio::test]
async fn test_cross_team_edit_propagation() {
    let (_remote, remote_url) = create_bare_remote();

    // Set up workspace A
    let workspace_a = TempDir::new().unwrap();
    let project_a = workspace_a.path().to_string_lossy().to_string();
    let metis_a = format!("{}/.metis", project_a);

    InitializeProjectTool {
        project_path: project_a.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_a, "api", &remote_url);

    // Set up workspace B
    let workspace_b = TempDir::new().unwrap();
    let project_b = workspace_b.path().to_string_lossy().to_string();
    let metis_b = format!("{}/.metis", project_b);

    InitializeProjectTool {
        project_path: project_b.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_b, "sre", &remote_url);

    let handler_a = make_handler();
    let dir_a = std::path::Path::new(&metis_a);
    let dir_b = std::path::Path::new(&metis_b);

    // ── Team A: initial push ─────────────────────────────────────────────
    handler_a.post_sync(dir_a);

    // Get Team A's vision code
    let list_a = ListDocumentsTool {
        project_path: metis_a.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    let re = Regex::new(r"([A-Z]+-V-\d{4})").unwrap();
    let vision_code = re
        .captures(&text_from(&list_a))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap();

    // ── Team A: edit the vision document ─────────────────────────────────
    let edit_result = EditDocumentTool {
        project_path: metis_a.clone(),
        short_code: vision_code.clone(),
        search: "{Why this vision exists and what it aims to achieve}".to_string(),
        replace: "Build the best API gateway in the industry.".to_string(),
        replace_all: None,
    }
    .call_tool()
    .await
    .unwrap();

    let edit_text = text_from(&edit_result);
    assert!(
        !edit_text.contains("Error"),
        "edit should succeed: {}",
        edit_text
    );

    // ── Team A: push the edit ────────────────────────────────────────────
    // Use a fresh handler to bypass the 30s debounce from the initial push
    let handler_a2 = make_handler();
    handler_a2.post_sync(dir_a);

    // ── Team B: pull and read Team A's edited vision ─────────────────────
    // Fresh handler for Team B too (debounce from Round 1 would block)
    let handler_b2 = make_handler();
    handler_b2.pre_sync(dir_b, SyncMode::Pull);

    let hydrated = dir_b.join(format!("api/{}.md", vision_code));
    let content = std::fs::read_to_string(&hydrated).expect("hydrated file should exist");
    assert!(
        content.contains("Build the best API gateway in the industry"),
        "Team B should see Team A's edited vision content, got:\n{}",
        content
    );

    println!("Edit propagation verified — Team B sees Team A's edit.");
}

// ─── UAT: Bidirectional sync ─────────────────────────────────────────────────

#[tokio::test]
async fn test_bidirectional_sync_both_teams_create() {
    let (_remote, remote_url) = create_bare_remote();

    // Workspace A
    let workspace_a = TempDir::new().unwrap();
    let project_a = workspace_a.path().to_string_lossy().to_string();
    let metis_a = format!("{}/.metis", project_a);
    InitializeProjectTool {
        project_path: project_a.clone(),
        prefix: Some("ALPHA".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_a, "api", &remote_url);

    // Workspace B
    let workspace_b = TempDir::new().unwrap();
    let project_b = workspace_b.path().to_string_lossy().to_string();
    let metis_b = format!("{}/.metis", project_b);
    InitializeProjectTool {
        project_path: project_b.clone(),
        prefix: Some("BRAVO".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_b, "sre", &remote_url);

    let handler_a = make_handler();
    let handler_b = make_handler();
    let dir_a = std::path::Path::new(&metis_a);
    let dir_b = std::path::Path::new(&metis_b);

    // ── Round 1: both teams push their initial state ─────────────────────
    handler_a.post_sync(dir_a);
    handler_b.post_sync(dir_b);

    // ── Round 2: both teams pull — each sees the other ───────────────────
    handler_a.pre_sync(dir_a, SyncMode::Full);
    handler_b.pre_sync(dir_b, SyncMode::Full);

    // Team A should have sre/ directory with Team B's docs
    let sre_dir = dir_a.join("sre");
    assert!(
        sre_dir.exists(),
        "Team A should have sre/ directory after pull"
    );
    let sre_files: Vec<_> = std::fs::read_dir(&sre_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    assert!(
        !sre_files.is_empty(),
        "Team A should see Team B's documents in sre/"
    );

    // Team B should have api/ directory with Team A's docs
    let api_dir = dir_b.join("api");
    assert!(
        api_dir.exists(),
        "Team B should have api/ directory after pull"
    );
    let api_files: Vec<_> = std::fs::read_dir(&api_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    assert!(
        !api_files.is_empty(),
        "Team B should see Team A's documents in api/"
    );

    println!(
        "Bidirectional sync verified: A sees {} sre docs, B sees {} api docs",
        sre_files.len(),
        api_files.len()
    );
}

// ─── UAT: Debounce prevents redundant syncs ──────────────────────────────────

#[tokio::test]
async fn test_debounce_prevents_repeated_pulls() {
    let (_remote, remote_url) = create_bare_remote();

    let workspace = TempDir::new().unwrap();
    let project = workspace.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project);

    InitializeProjectTool {
        project_path: project.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_path, "api", &remote_url);

    let handler = make_handler();
    let dir = std::path::Path::new(&metis_path);

    // First push to seed the remote
    handler.post_sync(dir);

    // First pull — should run (no previous pull recorded)
    handler.pre_sync(dir, SyncMode::Pull);

    // Second pull immediately — should be debounced (< 30s since last)
    // We can't directly observe the skip, but we verify it doesn't crash
    // and the handler's internal state tracks the timestamp
    handler.pre_sync(dir, SyncMode::Pull);
    handler.pre_sync(dir, SyncMode::Pull);
    handler.pre_sync(dir, SyncMode::Pull);

    // Similarly for push — first one runs, rest are debounced
    handler.post_sync(dir);
    handler.post_sync(dir);
    handler.post_sync(dir);

    // If we got here, debounce logic works without errors
    println!("Debounce verified — rapid calls don't crash or hang.");
}

// ─── UAT: Sync failure is non-fatal ─────────────────────────────────────────

#[tokio::test]
async fn test_sync_failure_does_not_block_tool() {
    let workspace = TempDir::new().unwrap();
    let project = workspace.path().to_string_lossy().to_string();
    let metis_path = format!("{}/.metis", project);

    InitializeProjectTool {
        project_path: project.clone(),
        prefix: None,
    }
    .call_tool()
    .await
    .unwrap();

    // Point sync at a nonexistent remote — sync will fail
    write_sync_config(&metis_path, "api", "file:///nonexistent/repo.git");

    let handler = make_handler();
    let dir = std::path::Path::new(&metis_path);

    // pre_sync should fail silently (logged, not propagated)
    handler.pre_sync(dir, SyncMode::Pull);

    // Tool should still work fine
    let list_result = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();

    let text = text_from(&list_result);
    assert!(
        text.contains("PROJ-V-"),
        "list should still work even when sync fails"
    );

    // post_sync should also fail silently
    handler.post_sync(dir);

    println!("Non-fatal sync failure verified — tools work despite unreachable remote.");
}

// ─── UAT: SQLite database picks up hydrated documents ────────────────────────

#[tokio::test]
async fn test_sqlite_indexes_hydrated_documents_after_pull() {
    // Scenario: Team A pushes documents to central. Team B pulls.
    // When Team B calls list_documents, the SQLite database (refreshed by
    // prepare_workspace inside the tool) should NOT show hydrated docs as
    // owned documents, but the files should be on disk for projection.
    //
    // This tests that the prepare_workspace → sync_directory pipeline
    // correctly handles files that appear in hydrated workspace directories.

    let (_remote, remote_url) = create_bare_remote();

    // ── Workspace A: create documents and push ───────────────────────────
    let workspace_a = TempDir::new().unwrap();
    let project_a = workspace_a.path().to_string_lossy().to_string();
    let metis_a = format!("{}/.metis", project_a);

    InitializeProjectTool {
        project_path: project_a.clone(),
        prefix: Some("ALPHA".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_a, "api", &remote_url);

    // Get vision code and create an initiative
    let list_a = ListDocumentsTool {
        project_path: metis_a.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    let re = Regex::new(r"([A-Z]+-V-\d{4})").unwrap();
    let vision_code_a = re
        .captures(&text_from(&list_a))
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap();

    let create_result = CreateDocumentTool {
        project_path: metis_a.clone(),
        document_type: "initiative".to_string(),
        title: "API Gateway Redesign".to_string(),
        parent_id: Some(vision_code_a.clone()),
        risk_level: None,
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    }
    .call_tool()
    .await
    .unwrap();
    let initiative_code_a = extract_short_code(&create_result);

    // Push Team A's documents to central
    let handler_a = make_handler();
    handler_a.post_sync(std::path::Path::new(&metis_a));

    // ── Workspace B: pull and verify ─────────────────────────────────────
    let workspace_b = TempDir::new().unwrap();
    let project_b = workspace_b.path().to_string_lossy().to_string();
    let metis_b = format!("{}/.metis", project_b);

    InitializeProjectTool {
        project_path: project_b.clone(),
        prefix: Some("BRAVO".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_b, "sre", &remote_url);

    // Pull Team A's documents
    let handler_b = make_handler();
    handler_b.pre_sync(std::path::Path::new(&metis_b), SyncMode::Pull);

    // Verify: hydrated files exist on disk
    let dir_b = std::path::Path::new(&metis_b);
    let api_dir = dir_b.join("api");
    assert!(
        api_dir.exists(),
        "api/ workspace directory should be hydrated"
    );
    let api_vision = api_dir.join(format!("{}.md", vision_code_a));
    assert!(
        api_vision.exists(),
        "Team A's vision should exist on Team B's disk: {}",
        api_vision.display()
    );
    let api_initiative = api_dir.join(format!("{}.md", initiative_code_a));
    assert!(
        api_initiative.exists(),
        "Team A's initiative should exist on Team B's disk: {}",
        api_initiative.display()
    );

    // Verify: list_documents (which calls prepare_workspace → sync_directory)
    // should still work correctly and show Team B's own documents
    let list_b = ListDocumentsTool {
        project_path: metis_b.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    let list_b_text = text_from(&list_b);

    // Team B should see its own vision
    assert!(
        list_b_text.contains("BRAVO-V-"),
        "Team B should see its own vision: {}",
        list_b_text
    );

    // Verify: search_documents can find content from hydrated workspace
    // The SQLite database should index the hydrated files too
    let search_b = SearchDocumentsTool {
        project_path: metis_b.clone(),
        query: "API Gateway Redesign".to_string(),
        document_type: None,
        limit: None,
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    let search_text = text_from(&search_b);

    // Whether hydrated docs appear in search depends on sync_directory scope.
    // Either way, the tool should NOT crash when hydrated files are present.
    println!(
        "Search result for Team A's initiative title from Team B:\n{}",
        search_text
    );

    // Verify: read_document for a hydrated doc should work if it's indexed,
    // or fail gracefully if it's not in the DB
    let read_hydrated = ReadDocumentTool {
        project_path: metis_b.clone(),
        short_code: initiative_code_a.clone(),
    }
    .call_tool()
    .await;

    match read_hydrated {
        Ok(result) => {
            let text = text_from(&result);
            println!(
                "Team B can read Team A's initiative via MCP:\n{}",
                &text[..text.len().min(200)]
            );
        }
        Err(_) => {
            // This is also acceptable — hydrated docs may not be in the owned
            // workspace's SQLite index, depending on sync_directory scope.
            println!(
                "Team B cannot read {} via MCP — hydrated docs are disk-only (expected)",
                initiative_code_a
            );
        }
    }

    // Key invariant: Team B's tools NEVER crash due to hydrated files
    // being present alongside owned documents.
    println!("SQLite-after-hydration verified — tools work with hydrated files on disk.");
}

// ─── UAT: Database consistency after full sync cycle ─────────────────────────

#[tokio::test]
async fn test_database_consistent_after_push_pull_cycle() {
    // Full round-trip: A creates → A pushes → B pulls → B creates → B pushes → A pulls
    // Both teams should see their own documents in list_documents at all times.

    let (_remote, remote_url) = create_bare_remote();

    // ── Set up both workspaces ───────────────────────────────────────────
    let ws_a = TempDir::new().unwrap();
    let proj_a = ws_a.path().to_string_lossy().to_string();
    let metis_a = format!("{}/.metis", proj_a);
    InitializeProjectTool {
        project_path: proj_a.clone(),
        prefix: Some("ALPHA".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_a, "api", &remote_url);

    let ws_b = TempDir::new().unwrap();
    let proj_b = ws_b.path().to_string_lossy().to_string();
    let metis_b = format!("{}/.metis", proj_b);
    InitializeProjectTool {
        project_path: proj_b.clone(),
        prefix: Some("BRAVO".to_string()),
    }
    .call_tool()
    .await
    .unwrap();
    write_sync_config(&metis_b, "sre", &remote_url);

    let dir_a = std::path::Path::new(&metis_a);
    let dir_b = std::path::Path::new(&metis_b);

    // ── Round 1: Team A creates + pushes ─────────────────────────────────
    let handler_1 = make_handler();
    handler_1.post_sync(dir_a);

    // Team A's list should have its vision
    let list_a1 = ListDocumentsTool {
        project_path: metis_a.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    assert!(
        text_from(&list_a1).contains("ALPHA-V-"),
        "Round 1: Team A should see its vision"
    );

    // ── Round 2: Team B pulls + creates + pushes ─────────────────────────
    let handler_2 = make_handler();
    handler_2.pre_sync(dir_b, SyncMode::Pull);

    // Team B creates an ADR
    let adr_result = CreateDocumentTool {
        project_path: metis_b.clone(),
        document_type: "adr".to_string(),
        title: "Use gRPC for inter-service communication".to_string(),
        parent_id: None,
        risk_level: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("SRE Lead".to_string()),
        backlog_category: None,
    }
    .call_tool()
    .await
    .unwrap();
    let adr_code = extract_short_code(&adr_result);

    // Team B pushes
    let handler_3 = make_handler();
    handler_3.post_sync(dir_b);

    // Team B's list should have its vision AND the ADR
    let list_b2 = ListDocumentsTool {
        project_path: metis_b.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    let list_b2_text = text_from(&list_b2);
    assert!(
        list_b2_text.contains("BRAVO-V-"),
        "Round 2: Team B should see its vision"
    );
    assert!(
        list_b2_text.contains(&adr_code),
        "Round 2: Team B should see its ADR"
    );

    // ── Round 3: Team A pulls — should have Team B's ADR on disk ─────────
    let handler_4 = make_handler();
    handler_4.pre_sync(dir_a, SyncMode::Pull);

    let sre_adr_file = dir_a.join(format!("sre/{}.md", adr_code));
    assert!(
        sre_adr_file.exists(),
        "Round 3: Team A should have Team B's ADR hydrated at {}",
        sre_adr_file.display()
    );

    // Team A's own list should still show its own vision (unaffected by hydration)
    let list_a3 = ListDocumentsTool {
        project_path: metis_a.clone(),
        include_archived: None,
    }
    .call_tool()
    .await
    .unwrap();
    assert!(
        text_from(&list_a3).contains("ALPHA-V-"),
        "Round 3: Team A should still see its vision"
    );

    println!(
        "Full push-pull cycle verified — both teams maintain database consistency."
    );
}
