//! Tests for MCP tools operating through the GitOverlay filesystem backend.
//!
//! These tests verify that all MCP tools (create, read, edit, list, search,
//! transition) work correctly when on a feature branch, where FilesystemService
//! uses the GitOverlay backend (reads from main's tree, writes to .pending/).
//!
//! This exercises the exact bug path where `create_document` wrote to the overlay
//! but `read_document`/`edit_document` used raw `tokio::fs` and couldn't find
//! the file at the canonical path.

use metis_mcp_server::tools::*;
use regex::Regex;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// Helper to extract text content from MCP response
fn extract_text(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    match result.content.first() {
        Some(rust_mcp_sdk::schema::ContentBlock::TextContent(tc)) => tc.text.clone(),
        Some(rust_mcp_sdk::schema::ContentBlock::EmbeddedResource(er)) => match &er.resource {
            rust_mcp_sdk::schema::EmbeddedResourceResource::TextResourceContents(tr) => {
                tr.text.clone()
            }
            _ => panic!("Unexpected resource type"),
        },
        _ => panic!("No text content in result"),
    }
}

/// Extract short code (e.g., PROJ-I-0001) from MCP result text
fn extract_short_code(result: &rust_mcp_sdk::schema::CallToolResult) -> String {
    let text = extract_text(result);
    let re = Regex::new(r"([A-Z]+-[VITSA]-\d{4})").unwrap();
    re.captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| panic!("No short code found in: {}", text))
}

/// Get the vision short code from list output
async fn get_vision_short_code(metis_path: &str) -> String {
    let list = ListDocumentsTool {
        project_path: metis_path.to_string(),
        include_archived: None,
    };
    let result = list.call_tool().await.unwrap();
    let text = extract_text(&result);
    let re = Regex::new(r"\|\s*vision\s*\|\s*([A-Z]+-V-\d{4})\s*\|").unwrap();
    re.captures(&text)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| panic!("No vision found in: {}", text))
}

/// Run a git command in the given directory, panicking on failure
fn git(dir: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("Failed to run git {:?}: {}", args, e));
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("git {:?} failed: {}", args, stderr);
    }
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// Initialize a metis project inside a git repo, commit to main, switch to feature branch.
/// Returns (temp_dir, metis_path_string, project_path).
async fn setup_git_overlay_project() -> (tempfile::TempDir, String, std::path::PathBuf) {
    let temp_dir = tempdir().unwrap();
    let project_path = temp_dir.path().to_path_buf();
    let metis_path = format!("{}/.metis", project_path.display());

    // Initialize git repo
    git(&project_path, &["init", "-b", "main"]);
    git(
        &project_path,
        &["config", "user.email", "test@test.com"],
    );
    git(&project_path, &["config", "user.name", "Test"]);

    // Initialize metis project
    let init_tool = InitializeProjectTool {
        project_path: project_path.to_string_lossy().to_string(),
        prefix: None,
    };
    init_tool.call_tool().await.unwrap();

    // Create a vision (auto-created by init) — commit everything to main
    git(&project_path, &["add", "-A"]);
    git(&project_path, &["commit", "-m", "initial metis setup"]);

    // Switch to feature branch
    git(&project_path, &["checkout", "-b", "feature/test-overlay"]);

    (temp_dir, metis_path, project_path)
}

/// Core test: create an initiative on a feature branch, then read it back.
/// This is the exact scenario that was broken: create wrote to .pending/
/// but read used raw fs and couldn't find the file.
#[tokio::test]
async fn test_create_then_read_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Create initiative on feature branch (goes to overlay)
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Overlay Test Initiative".to_string(),
        parent_id: Some(vision_sc),
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // Read it back — this was the failing path
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);

    assert!(
        text.contains("Overlay Test Initiative"),
        "Should read back the initiative title, got: {}",
        &text[..text.len().min(200)]
    );
    assert!(
        !text.contains("Document not found"),
        "Should NOT return 'not found' for overlay-created document"
    );
}

/// Create on feature branch, then edit — verifies both read and write go through overlay.
#[tokio::test]
async fn test_create_then_edit_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Create initiative
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Editable Initiative".to_string(),
        parent_id: Some(vision_sc),
        complexity: Some("s".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // Edit it (search/replace in the template content)
    let edit = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        search: "{Describe the context and background for this initiative}".to_string(),
        replace: "This initiative tests overlay editing.".to_string(),
        replace_all: None,
    };
    let result = edit.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("updated"),
        "Edit should report success, got: {}",
        text
    );

    // Read back and verify the edit persisted
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("This initiative tests overlay editing."),
        "Edited content should persist in overlay"
    );
}

/// Create on feature branch, then transition phase — verifies PhaseTransitionService
/// works with overlay documents.
#[tokio::test]
async fn test_create_then_transition_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Create initiative (starts in discovery)
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Transitionable Initiative".to_string(),
        parent_id: Some(vision_sc),
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // Transition discovery -> design
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        phase: Some("design".to_string()),
        force: None,
    };
    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "Phase transition should succeed on overlay document: {:?}",
        result.err()
    );
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("design"),
        "Should show transition to design phase"
    );

    // Verify phase persisted by reading back
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("#phase/design"),
        "Phase tag should be updated in overlay document"
    );
}

/// Create a task under an initiative that only exists in main's git tree
/// (not on disk on the feature branch). Verifies parent validation uses
/// overlay-aware file_exists.
#[tokio::test]
async fn test_create_task_under_main_only_initiative() {
    let (_temp, metis_path, project_path) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Switch back to main to create an initiative on disk
    git(&project_path, &["checkout", "main"]);

    let create_init = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Main-Only Initiative".to_string(),
        parent_id: Some(vision_sc),
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_init.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // Commit to main
    git(&project_path, &["add", "-A"]);
    git(
        &project_path,
        &["commit", "-m", "add initiative on main"],
    );

    // Switch to feature branch — initiative exists only in main's tree
    git(&project_path, &["checkout", "feature/test-overlay"]);

    // Create a task under that initiative
    let create_task = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Task under main-only initiative".to_string(),
        parent_id: Some(initiative_sc.clone()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_task.call_tool().await;
    assert!(
        result.is_ok(),
        "Creating task under main-only initiative should succeed: {:?}",
        result.err()
    );
    let task_sc = extract_short_code(&result.unwrap());

    // Read the task back
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: task_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("Task under main-only initiative"),
        "Should read back the task created under main-only initiative"
    );
}

/// Documents created on main should be visible from a feature branch
/// (read from git tree, not disk).
#[tokio::test]
async fn test_read_main_document_from_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    // Vision was created on main and committed — should be readable from feature branch
    let vision_sc = get_vision_short_code(&metis_path).await;

    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        !text.contains("Document not found"),
        "Documents committed on main should be readable from feature branch"
    );
}

/// Edit a document that exists on main (not on disk on feature branch).
/// The edit should go to the overlay.
#[tokio::test]
async fn test_edit_main_document_from_feature_branch() {
    let (_temp, metis_path, project_path) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Read the vision first to see its content
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);

    // Find a template placeholder to replace (safe substitution that doesn't break frontmatter)
    let search_text = "{Why this vision exists and what it aims to achieve}";
    assert!(
        text.contains(search_text),
        "Vision should contain template placeholder"
    );

    let edit = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_sc.clone(),
        search: search_text.to_string(),
        replace: "Edited from feature branch via overlay.".to_string(),
        replace_all: None,
    };
    let result = edit.call_tool().await;
    assert!(
        result.is_ok(),
        "Editing main document from feature branch should succeed: {:?}",
        result.err()
    );

    // Verify the edit is visible when reading back
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: vision_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("Edited from feature branch via overlay"),
        "Edit should persist in overlay and be readable"
    );

    // Verify the original file on main is untouched
    let vision_file = project_path.join(".metis/vision.md");
    if vision_file.exists() {
        let main_content = std::fs::read_to_string(&vision_file).unwrap();
        assert!(
            !main_content.contains("Edited from feature branch via overlay"),
            "Original file on disk should NOT be modified — edit should only be in overlay"
        );
    }
}

/// List documents should include both main tree documents and overlay documents.
#[tokio::test]
async fn test_list_shows_main_and_overlay_documents() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Create an initiative on the feature branch (overlay)
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Overlay Initiative For List".to_string(),
        parent_id: Some(vision_sc.clone()),
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // List should show both the vision (from main) and the initiative (from overlay)
    let list = ListDocumentsTool {
        project_path: metis_path.clone(),
        include_archived: None,
    };
    let result = list.call_tool().await.unwrap();
    let text = extract_text(&result);

    assert!(
        text.contains(&vision_sc),
        "Vision from main should appear in list"
    );
    assert!(
        text.contains(&initiative_sc),
        "Initiative from overlay should appear in list"
    );
}

/// Search should find documents in both main tree and overlay.
#[tokio::test]
async fn test_search_finds_overlay_documents() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // Create initiative with distinctive title
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Searchable Overlay Quokka".to_string(),
        parent_id: Some(vision_sc),
        complexity: Some("m".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    create.call_tool().await.unwrap();

    // Search for the distinctive term
    let search = SearchDocumentsTool {
        project_path: metis_path.clone(),
        query: "Quokka".to_string(),
        document_type: None,
        limit: None,
        include_archived: None,
    };
    let result = search.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("Quokka"),
        "Search should find overlay documents, got: {}",
        &text[..text.len().min(300)]
    );
}

/// Full lifecycle on feature branch: create → edit → transition → read.
/// Exercises the complete CRUD path through the overlay.
#[tokio::test]
async fn test_full_lifecycle_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    let vision_sc = get_vision_short_code(&metis_path).await;

    // 1. Create initiative
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "initiative".to_string(),
        title: "Full Lifecycle Initiative".to_string(),
        parent_id: Some(vision_sc.clone()),
        complexity: Some("l".to_string()),
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let initiative_sc = extract_short_code(&result);

    // 2. Edit initiative content
    let edit = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        search: "{Describe the context and background for this initiative}".to_string(),
        replace: "Full lifecycle test context.".to_string(),
        replace_all: None,
    };
    edit.call_tool().await.unwrap();

    // 3. Transition discovery -> design
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        phase: Some("design".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // 4. Transition design -> ready
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        phase: Some("ready".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // 5. Transition ready -> decompose
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
        phase: Some("decompose".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // 6. Create a task under the initiative
    let create_task = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Lifecycle Sub-Task".to_string(),
        parent_id: Some(initiative_sc.clone()),
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: None,
    };
    let result = create_task.call_tool().await;
    assert!(
        result.is_ok(),
        "Creating task under overlay initiative should succeed: {:?}",
        result.err()
    );
    let task_sc = extract_short_code(&result.unwrap());

    // 7. Transition task todo -> active
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: task_sc.clone(),
        phase: Some("active".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // 8. Edit the task
    let edit = EditDocumentTool {
        project_path: metis_path.clone(),
        short_code: task_sc.clone(),
        search: "{Clear statement of what this task accomplishes}".to_string(),
        replace: "Lifecycle sub-task objective.".to_string(),
        replace_all: None,
    };
    edit.call_tool().await.unwrap();

    // 9. Complete the task
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: task_sc.clone(),
        phase: Some("completed".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // 10. Read back and verify final state
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: task_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(text.contains("Lifecycle sub-task objective."));
    assert!(text.contains("#phase/completed"));

    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: initiative_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(text.contains("Full lifecycle test context."));
    assert!(text.contains("#phase/decompose"));
}

/// Create an ADR on a feature branch and verify full CRUD works.
#[tokio::test]
async fn test_adr_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    // Create ADR
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "adr".to_string(),
        title: "Use Overlay for Feature Branches".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: Some("architect".to_string()),
        backlog_category: None,
    };
    let result = create.call_tool().await.unwrap();
    let adr_sc = extract_short_code(&result);

    // Read it back
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: adr_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(text.contains("Use Overlay for Feature Branches"));

    // Transition draft -> discussion
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: adr_sc.clone(),
        phase: Some("discussion".to_string()),
        force: None,
    };
    let result = transition.call_tool().await;
    assert!(
        result.is_ok(),
        "ADR transition should work on feature branch: {:?}",
        result.err()
    );
}

/// Backlog items on a feature branch.
#[tokio::test]
async fn test_backlog_on_feature_branch() {
    let (_temp, metis_path, _project) = setup_git_overlay_project().await;

    // Create a bug backlog item
    let create = CreateDocumentTool {
        project_path: metis_path.clone(),
        document_type: "task".to_string(),
        title: "Overlay Bug Report".to_string(),
        parent_id: None,
        complexity: None,
        stakeholders: None,
        decision_maker: None,
        backlog_category: Some("bug".to_string()),
    };
    let result = create.call_tool().await.unwrap();
    let bug_sc = extract_short_code(&result);

    // Read it back
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: bug_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(
        text.contains("Overlay Bug Report"),
        "Backlog item should be readable from overlay"
    );

    // Transition backlog -> todo -> active
    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: bug_sc.clone(),
        phase: Some("todo".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    let transition = TransitionPhaseTool {
        project_path: metis_path.clone(),
        short_code: bug_sc.clone(),
        phase: Some("active".to_string()),
        force: None,
    };
    transition.call_tool().await.unwrap();

    // Verify phase
    let read = ReadDocumentTool {
        project_path: metis_path.clone(),
        short_code: bug_sc.clone(),
    };
    let result = read.call_tool().await.unwrap();
    let text = extract_text(&result);
    assert!(text.contains("#phase/active"));
}
