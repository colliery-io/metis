//! Dehydration: local → central document push.
//!
//! Serializes the owned workspace's documents into a git commit for push
//! to the central repo. Only the owned workspace folder is written —
//! other workspaces' folders are preserved untouched.

use crate::{FileEntry, SyncContext, SyncError};
use std::collections::BTreeSet;
use tracing::debug;

/// Result of a dehydration operation.
#[derive(Debug)]
pub struct DehydrationResult {
    /// Commit OID created (None if no changes to push)
    pub commit_oid: Option<git2::Oid>,
    /// Files pushed to central
    pub files_pushed: usize,
    /// Files removed from central
    pub files_removed: usize,
    /// Whether a push was performed
    pub pushed: bool,
}

/// A flattened document ready for dehydration. Mirrors the layout module's
/// FlatDocument but avoids a dependency on metis-docs-core.
#[derive(Debug, Clone)]
pub struct FlatDoc {
    /// The short code (e.g., "API-T-0001")
    pub short_code: String,
    /// The filename (e.g., "API-T-0001.md")
    pub filename: String,
    /// Full file content including frontmatter
    pub content: String,
}

/// Dehydrate: push local workspace documents to the central repo.
///
/// Takes a list of flattened documents (from the layout module's `flatten_workspace()`)
/// and pushes them to the central repo under `<prefix>/`.
///
/// # Arguments
///
/// * `ctx` - SyncContext (must have been fetched already)
/// * `documents` - Flattened documents from the local workspace
/// * `prefix` - The owned workspace prefix
///
/// # Returns
///
/// `DehydrationResult` with commit OID, file counts, and whether a push occurred.
pub fn dehydrate(
    ctx: &mut SyncContext,
    documents: &[FlatDoc],
    prefix: &str,
) -> Result<DehydrationResult, SyncError> {
    // Build file entries for the commit
    let files: Vec<FileEntry> = documents
        .iter()
        .map(|doc| FileEntry {
            path: format!("{}/{}", prefix, doc.filename),
            content: doc.content.as_bytes().to_vec(),
        })
        .collect();

    // Determine which files currently exist in central for this workspace
    // so we can compute removals
    let central_files: BTreeSet<String> = if ctx.fetched_head().is_some() {
        ctx.list_workspace_files(prefix)?
            .into_iter()
            .map(|(name, _)| format!("{}/{}", prefix, name))
            .collect()
    } else {
        BTreeSet::new()
    };

    let local_files: BTreeSet<String> = files.iter().map(|f| f.path.clone()).collect();

    // Files in central but not in local → removals
    let removals: Vec<String> = central_files
        .difference(&local_files)
        .cloned()
        .collect();

    // Check if there are any actual changes
    if files.is_empty() && removals.is_empty() {
        debug!("no changes to dehydrate");
        return Ok(DehydrationResult {
            commit_oid: None,
            files_pushed: 0,
            files_removed: 0,
            pushed: false,
        });
    }

    // Check if content actually changed (avoid empty commits)
    if removals.is_empty() && ctx.fetched_head().is_some() {
        let all_same = files.iter().all(|file| {
            if let Ok(existing) = ctx.read_blob(ctx.fetched_head().unwrap(), &file.path) {
                existing == file.content
            } else {
                false // File doesn't exist in central, so it's a new addition
            }
        });
        if all_same && files.len() == central_files.len() {
            debug!("no content changes, skipping push");
            return Ok(DehydrationResult {
                commit_oid: None,
                files_pushed: 0,
                files_removed: 0,
                pushed: false,
            });
        }
    }

    let files_pushed = files.len();
    let files_removed = removals.len();

    // Build commit message with timestamp
    let timestamp = chrono_lite_now();
    let message = format!("sync: {} @ {}", prefix, timestamp);

    debug!(
        prefix,
        files_pushed,
        files_removed,
        %message,
        "dehydrating workspace"
    );

    let commit_oid = ctx.commit_update(&files, &removals, &message)?;
    ctx.push()?;

    Ok(DehydrationResult {
        commit_oid: Some(commit_oid),
        files_pushed,
        files_removed,
        pushed: true,
    })
}

/// Simple UTC timestamp without pulling in the full chrono crate.
fn chrono_lite_now() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();

    // Convert epoch seconds to ISO 8601-ish UTC timestamp
    let days = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, day) = days_to_date(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

/// Convert days since Unix epoch to (year, month, day).
fn days_to_date(days: u64) -> (u64, u64, u64) {
    // Civil days algorithm
    let z = days + 719468;
    let era = z / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{build::TreeUpdateBuilder, Repository, Signature};
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper: create a bare remote
    fn create_bare_remote() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        Repository::init_bare(dir.path()).unwrap();
        let url = format!("file://{}", dir.path().display());
        (dir, url)
    }

    /// Helper: create a bare remote with an initial commit
    fn create_remote_with_commit(files: &[(&str, &str)]) -> (TempDir, String, git2::Oid) {
        let remote_dir = TempDir::new().unwrap();
        let bare = Repository::init_bare(remote_dir.path()).unwrap();

        let sig = Signature::now("test", "test@test.com").unwrap();
        let empty_tree_oid = bare.treebuilder(None).unwrap().write().unwrap();
        let empty_tree = bare.find_tree(empty_tree_oid).unwrap();

        let mut builder = TreeUpdateBuilder::new();
        for (path, content) in files {
            let blob = bare.blob(content.as_bytes()).unwrap();
            builder.upsert(path, blob, git2::FileMode::Blob);
        }
        let tree_oid = builder.create_updated(&bare, &empty_tree).unwrap();
        let tree = bare.find_tree(tree_oid).unwrap();

        let commit_oid = bare
            .commit(Some("refs/heads/main"), &sig, &sig, "initial", &tree, &[])
            .unwrap();

        let url = format!("file://{}", remote_dir.path().display());
        (remote_dir, url, commit_oid)
    }

    fn make_docs(items: &[(&str, &str, &str)]) -> Vec<FlatDoc> {
        items
            .iter()
            .map(|(sc, fn_, content)| FlatDoc {
                short_code: sc.to_string(),
                filename: fn_.to_string(),
                content: content.to_string(),
            })
            .collect()
    }

    // ─── Tree Surgery ─────────────────────────────────────────────────────

    #[test]
    fn test_first_push_to_empty_central() {
        let (remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "# Vision"),
            ("API-T-0001", "API-T-0001.md", "# Task 1"),
        ]);

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        assert!(result.commit_oid.is_some());
        assert_eq!(result.files_pushed, 2);
        assert_eq!(result.files_removed, 0);
        assert!(result.pushed);

        // Verify on remote
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();
        let api_entry = tree.get_name("api").unwrap();
        let api_tree = bare.find_tree(api_entry.id()).unwrap();
        assert_eq!(api_tree.len(), 2);
    }

    #[test]
    fn test_update_existing_workspace() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "v1 vision"),
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "v2 updated vision"),
            ("API-T-0001", "API-T-0001.md", "new task"),
        ]);

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_pushed, 2);
    }

    #[test]
    fn test_other_workspaces_preserved() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
            ("strat/STRAT-V-0001.md", "strat vision"),
            ("alpha/ALPHA-V-0001.md", "alpha vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "api vision updated"),
        ]);

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);

        // Verify other workspaces are untouched
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();

        // Read strat content
        let strat_entry = tree.get_path(Path::new("strat/STRAT-V-0001.md")).unwrap();
        let strat_blob = bare.find_blob(strat_entry.id()).unwrap();
        assert_eq!(
            std::str::from_utf8(strat_blob.content()).unwrap(),
            "strat vision"
        );

        // Read alpha content
        let alpha_entry = tree.get_path(Path::new("alpha/ALPHA-V-0001.md")).unwrap();
        let alpha_blob = bare.find_blob(alpha_entry.id()).unwrap();
        assert_eq!(
            std::str::from_utf8(alpha_blob.content()).unwrap(),
            "alpha vision"
        );
    }

    #[test]
    fn test_deleted_document_removed_from_central() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
            ("api/API-T-0001.md", "task to delete"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Only push the vision, task is deleted
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_removed, 1);

        // Verify task is gone from remote
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();
        assert!(tree.get_path(Path::new("api/API-T-0001.md")).is_err());
        assert!(tree.get_path(Path::new("api/API-V-0001.md")).is_ok());
    }

    #[test]
    fn test_mixed_operations() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "old vision"),
            ("api/API-T-0001.md", "task to keep but update"),
            ("api/API-T-0002.md", "task to delete"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "new vision"),        // modified
            ("API-T-0001", "API-T-0001.md", "updated task"),      // modified
            ("API-I-0001", "API-I-0001.md", "new initiative"),    // added
        ]);
        // API-T-0002 is implicitly deleted (not in docs)

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_pushed, 3);
        assert_eq!(result.files_removed, 1);

        // Verify on remote
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();

        // New initiative exists
        let init_entry = tree.get_path(Path::new("api/API-I-0001.md")).unwrap();
        let init_blob = bare.find_blob(init_entry.id()).unwrap();
        assert_eq!(
            std::str::from_utf8(init_blob.content()).unwrap(),
            "new initiative"
        );

        // Deleted task is gone
        assert!(tree.get_path(Path::new("api/API-T-0002.md")).is_err());
    }

    #[test]
    fn test_empty_workspace_push() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Push empty workspace — removes all api files
        let docs: Vec<FlatDoc> = vec![];
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_removed, 1);

        // Verify strat is still there
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();
        assert!(tree.get_path(Path::new("strat/STRAT-V-0001.md")).is_ok());
    }

    // ─── Write Scope ──────────────────────────────────────────────────────

    #[test]
    fn test_only_owned_prefix_written() {
        let (remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "api vision"),
        ]);

        dehydrate(&mut ctx, &docs, "api").unwrap();

        // Only api/ should exist in the remote
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        let tree = commit.tree().unwrap();
        assert_eq!(tree.len(), 1); // only "api"
        assert!(tree.get_name("api").is_some());
    }

    // ─── Commit Quality ───────────────────────────────────────────────────

    #[test]
    fn test_commit_message_format() {
        let (_remote, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        let commit = ctx
            .repo()
            .find_commit(result.commit_oid.unwrap())
            .unwrap();
        let msg = commit.message().unwrap();

        assert!(msg.starts_with("sync: api @ "), "got: {}", msg);
        assert!(msg.contains("T"), "should have time component: {}", msg);
        assert!(msg.ends_with("Z"), "should be UTC: {}", msg);
    }

    #[test]
    fn test_commit_parent_is_fetched_head() {
        let (_remote, url, initial_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "v2")]);
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        let commit = ctx
            .repo()
            .find_commit(result.commit_oid.unwrap())
            .unwrap();
        assert_eq!(commit.parent_count(), 1);
        assert_eq!(commit.parent_id(0).unwrap(), initial_oid);
    }

    #[test]
    fn test_single_commit_per_sync() {
        let (_remote, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "vision"),
            ("API-T-0001", "API-T-0001.md", "task 1"),
            ("API-T-0002", "API-T-0002.md", "task 2"),
        ]);

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert_eq!(result.files_pushed, 3);

        // Should be exactly one commit
        let commit = ctx
            .repo()
            .find_commit(result.commit_oid.unwrap())
            .unwrap();
        assert_eq!(commit.parent_count(), 0); // initial commit, no parent
    }

    // ─── Integration ──────────────────────────────────────────────────────

    #[test]
    fn test_sequential_pushes() {
        let (remote_dir, url) = create_bare_remote();

        // First push
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();
        let docs1 = make_docs(&[("API-V-0001", "API-V-0001.md", "v1")]);
        dehydrate(&mut ctx, &docs1, "api").unwrap();

        // Second push with updates
        let mut ctx2 = SyncContext::new(&url, "api").unwrap();
        ctx2.fetch().unwrap();
        let docs2 = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "v2 updated"),
            ("API-T-0001", "API-T-0001.md", "new task"),
        ]);
        dehydrate(&mut ctx2, &docs2, "api").unwrap();

        // Verify latest state
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        assert_eq!(commit.parent_count(), 1); // second commit parents on first

        let tree = commit.tree().unwrap();
        let vision_entry = tree.get_path(Path::new("api/API-V-0001.md")).unwrap();
        let vision_blob = bare.find_blob(vision_entry.id()).unwrap();
        assert_eq!(
            std::str::from_utf8(vision_blob.content()).unwrap(),
            "v2 updated"
        );
    }

    #[test]
    fn test_push_after_hydration_doesnt_leak() {
        let (_remote_dir, url) = create_bare_remote();

        // Strat workspace pushes
        let mut ctx_strat = SyncContext::new(&url, "strat").unwrap();
        ctx_strat.fetch().unwrap();
        let strat_docs =
            make_docs(&[("STRAT-V-0001", "STRAT-V-0001.md", "strat vision")]);
        dehydrate(&mut ctx_strat, &strat_docs, "strat").unwrap();

        // API workspace fetches (hydrates strat) then pushes own content
        let mut ctx_api = SyncContext::new(&url, "api").unwrap();
        ctx_api.fetch().unwrap();

        // API pushes only its own docs, not strat's
        let api_docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let result = dehydrate(&mut ctx_api, &api_docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_pushed, 1);

        // Verify strat is still in central (not removed by api's push)
        let mut verify_ctx = SyncContext::new(&url, "verify").unwrap();
        let head = verify_ctx.fetch().unwrap().unwrap();
        let strat_content = verify_ctx
            .read_blob(head, "strat/STRAT-V-0001.md")
            .unwrap();
        assert_eq!(
            String::from_utf8(strat_content).unwrap(),
            "strat vision"
        );
    }

    // ─── No-change detection ──────────────────────────────────────────────

    #[test]
    fn test_no_changes_skips_push() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Push same content as already in central
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        assert!(!result.pushed);
        assert!(result.commit_oid.is_none());
    }

    #[test]
    fn test_no_docs_no_central_skips() {
        let (_remote, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Empty docs, empty central → nothing to do
        let docs: Vec<FlatDoc> = vec![];
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        assert!(!result.pushed);
        assert!(result.commit_oid.is_none());
    }

    // ─── Edge Cases ───────────────────────────────────────────────────────

    #[test]
    fn test_large_workspace_push() {
        let (_remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs: Vec<FlatDoc> = (1..=100)
            .map(|i| FlatDoc {
                short_code: format!("API-T-{:04}", i),
                filename: format!("API-T-{:04}.md", i),
                content: format!("# Task {}\n\nContent.", i),
            })
            .collect();

        let result = dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_pushed, 100);
    }

    #[test]
    fn test_unicode_preserved() {
        let (_remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let content = "---\nshort_code: API-V-0001\n---\n\n# 愿景\n\n中文 🎉 ñ àéîõü";
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", content)]);
        let result = dehydrate(&mut ctx, &docs, "api").unwrap();

        // Read back from repo
        let oid = result.commit_oid.unwrap();
        let read_back = ctx.read_blob(oid, "api/API-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(read_back).unwrap(), content);
    }

    #[test]
    fn test_timestamp_format() {
        let ts = chrono_lite_now();
        // Should be ISO 8601-ish: YYYY-MM-DDTHH:MM:SSZ
        assert!(ts.len() == 20, "unexpected timestamp length: {}", ts);
        assert!(ts.ends_with('Z'));
        assert!(ts.contains('T'));
    }
}
