//! Hydration: central → local document sync.
//!
//! Fetches all workspace folders from the central repo and writes remote
//! documents to local `.metis/<prefix>/` subfolders. The owned workspace
//! is skipped (don't overwrite local state with central's copy).

use crate::{SyncContext, SyncError};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use tracing::debug;

/// Result of a hydration operation.
#[derive(Debug)]
pub struct HydrationResult {
    /// Workspace prefixes that were hydrated
    pub hydrated_workspaces: Vec<String>,
    /// Files written to disk
    pub files_written: usize,
    /// Stale files removed from disk
    pub files_removed: usize,
    /// Stale workspace folders removed from disk
    pub folders_removed: Vec<String>,
    /// Errors encountered (non-fatal — other workspaces still processed)
    pub errors: Vec<(String, String)>,
}

/// Hydrate remote workspaces from the central repo to the local `.metis/` directory.
///
/// For each workspace folder in central that is NOT the owned workspace:
/// 1. List all `.md` files in the remote workspace folder
/// 2. Write each file to `.metis/<prefix>/<filename>`
/// 3. Remove any local `.metis/<prefix>/*.md` files not in central
/// 4. Remove workspace folders that no longer exist in central
///
/// # Arguments
///
/// * `ctx` - The SyncContext (must have been fetched already)
/// * `metis_dir` - Path to the local `.metis/` directory
/// * `owned_prefix` - The owned workspace prefix to skip
pub fn hydrate(
    ctx: &SyncContext,
    metis_dir: &Path,
    owned_prefix: &str,
) -> Result<HydrationResult, SyncError> {
    let mut result = HydrationResult {
        hydrated_workspaces: Vec::new(),
        files_written: 0,
        files_removed: 0,
        folders_removed: Vec::new(),
        errors: Vec::new(),
    };

    // If remote is empty, nothing to hydrate
    if ctx.fetched_head().is_none() {
        debug!("remote is empty, nothing to hydrate");
        return Ok(result);
    }

    // Get all workspace folders from central
    let central_folders = ctx.list_workspace_folders()?;
    let remote_prefixes: BTreeSet<&str> = central_folders
        .iter()
        .filter(|p| p.as_str() != owned_prefix)
        .map(|s| s.as_str())
        .collect();

    debug!(
        remote_count = remote_prefixes.len(),
        owned = owned_prefix,
        "hydrating remote workspaces"
    );

    // Hydrate each remote workspace
    for prefix in &remote_prefixes {
        match hydrate_workspace(ctx, metis_dir, prefix) {
            Ok((written, removed)) => {
                result.hydrated_workspaces.push(prefix.to_string());
                result.files_written += written;
                result.files_removed += removed;
            }
            Err(e) => {
                result
                    .errors
                    .push((prefix.to_string(), format!("{}", e)));
            }
        }
    }

    // Remove local workspace folders that no longer exist in central
    let removed = remove_stale_workspace_folders(metis_dir, owned_prefix, &central_folders)?;
    result.folders_removed = removed;

    // Update .gitignore with hydrated workspace entries
    if let Err(e) = update_gitignore(metis_dir, &result.hydrated_workspaces) {
        result
            .errors
            .push((".gitignore".to_string(), format!("{}", e)));
    }

    Ok(result)
}

/// Hydrate a single workspace: write files from central, remove stale local files.
/// Returns (files_written, files_removed).
fn hydrate_workspace(
    ctx: &SyncContext,
    metis_dir: &Path,
    prefix: &str,
) -> Result<(usize, usize), SyncError> {
    let workspace_dir = metis_dir.join(prefix);

    // Create the workspace directory if it doesn't exist
    fs::create_dir_all(&workspace_dir)?;

    // Get all .md files from central for this workspace
    let central_files = ctx.list_workspace_files(prefix)?;
    let central_filenames: BTreeSet<String> = central_files
        .iter()
        .map(|(name, _)| name.clone())
        .collect();

    // Write files from central to local
    let mut written = 0;
    for (filename, content) in &central_files {
        let file_path = workspace_dir.join(filename);
        fs::write(&file_path, content)?;
        written += 1;
    }

    // Remove local .md files not present in central
    let mut removed = 0;
    if let Ok(entries) = fs::read_dir(&workspace_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };
            if !filename.ends_with(".md") {
                continue;
            }
            if !central_filenames.contains(&filename) {
                fs::remove_file(&path)?;
                removed += 1;
            }
        }
    }

    debug!(prefix, written, removed, "workspace hydrated");
    Ok((written, removed))
}

/// Remove local workspace folders that no longer exist in central.
/// Returns the list of removed folder names.
fn remove_stale_workspace_folders(
    metis_dir: &Path,
    owned_prefix: &str,
    central_folders: &[String],
) -> Result<Vec<String>, SyncError> {
    let central_set: BTreeSet<&str> = central_folders.iter().map(|s| s.as_str()).collect();
    let mut removed = Vec::new();

    // Known non-workspace entries in .metis/ that should never be touched
    let reserved_names: BTreeSet<&str> = [
        "archived",
        "strategies",
        "adrs",
        "backlog",
        "templates",
        "code-index.md",
        "code-index-hashes.json",
        "code-index-symbols.json",
        "config.toml",
        "metis.db",
        "metis.db-journal",
        "metis.db-wal",
        "metis.db-shm",
        ".gitignore",
        ".index-dirty",
    ]
    .into_iter()
    .collect();

    if let Ok(entries) = fs::read_dir(metis_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let dirname = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            // Skip the owned workspace
            if dirname == owned_prefix {
                continue;
            }

            // Skip reserved/known non-workspace directories
            if reserved_names.contains(dirname.as_str()) {
                continue;
            }

            // Skip hidden directories
            if dirname.starts_with('.') {
                continue;
            }

            // If this folder was previously a hydrated workspace but no longer
            // exists in central, remove it
            if !central_set.contains(dirname.as_str()) {
                // Only remove if it looks like a hydrated workspace
                // (contains only .md files or is empty)
                if is_hydrated_workspace(&path) {
                    fs::remove_dir_all(&path)?;
                    removed.push(dirname);
                }
            }
        }
    }

    Ok(removed)
}

/// Check if a directory looks like a hydrated workspace (only contains .md files).
fn is_hydrated_workspace(dir: &Path) -> bool {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return false,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            return false; // Hydrated workspaces are flat
        }
        if let Some(ext) = path.extension() {
            if ext != "md" {
                return false; // Non-md files suggest this isn't a hydrated workspace
            }
        } else {
            return false; // Files without extension are suspicious
        }
    }

    true // Empty dirs or dirs with only .md files
}

/// Update `.metis/.gitignore` to include hydrated workspace folders.
fn update_gitignore(metis_dir: &Path, hydrated_prefixes: &[String]) -> Result<(), SyncError> {
    let gitignore_path = metis_dir.join(".gitignore");

    // Read existing .gitignore content
    let existing = if gitignore_path.exists() {
        fs::read_to_string(&gitignore_path)?
    } else {
        String::new()
    };

    let existing_lines: BTreeSet<&str> = existing.lines().collect();

    // Add entries for each hydrated workspace folder
    let mut needs_update = false;
    let mut new_entries = Vec::new();
    for prefix in hydrated_prefixes {
        let entry = format!("{}/", prefix);
        if !existing_lines.contains(entry.as_str()) {
            new_entries.push(entry);
            needs_update = true;
        }
    }

    if !needs_update {
        return Ok(());
    }

    // Append new entries to gitignore
    let mut content = existing;
    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    // Add a section header if we're adding the first hydrated workspace entries
    let has_hydration_header = content.contains("# Hydrated remote workspaces");
    if !has_hydration_header && !new_entries.is_empty() {
        content.push_str("# Hydrated remote workspaces\n");
    }

    for entry in &new_entries {
        content.push_str(entry);
        content.push('\n');
    }

    fs::write(&gitignore_path, content)?;
    debug!(
        entries = new_entries.len(),
        "updated .gitignore with hydrated workspace entries"
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FileEntry;
    use git2::{build::TreeUpdateBuilder, Repository, Signature};
    use tempfile::TempDir;

    /// Helper: create a bare remote with an initial commit on `main`
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

    /// Helper: add a commit on top of an existing bare repo
    fn add_commit_to_remote(
        remote_path: &Path,
        files: &[(&str, &str)],
        removals: &[&str],
        message: &str,
    ) -> git2::Oid {
        let bare = Repository::open_bare(remote_path).unwrap();
        let parent_ref = bare.find_reference("refs/heads/main").unwrap();
        let parent_oid = parent_ref.target().unwrap();
        let parent = bare.find_commit(parent_oid).unwrap();
        let parent_tree = parent.tree().unwrap();

        let mut builder = TreeUpdateBuilder::new();
        for (path, content) in files {
            let blob = bare.blob(content.as_bytes()).unwrap();
            builder.upsert(path, blob, git2::FileMode::Blob);
        }
        for removal in removals {
            builder.remove(removal);
        }
        let new_tree_oid = builder.create_updated(&bare, &parent_tree).unwrap();
        let new_tree = bare.find_tree(new_tree_oid).unwrap();

        let sig = Signature::now("test", "test@test.com").unwrap();
        bare.commit(
            Some("refs/heads/main"),
            &sig,
            &sig,
            message,
            &new_tree,
            &[&parent],
        )
        .unwrap()
    }

    /// Helper: create a temp .metis/ directory
    fn create_metis_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    // ─── Hydration Core ───────────────────────────────────────────────────

    #[test]
    fn test_single_remote_workspace() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "# Vision"),
            ("strat/STRAT-I-0001.md", "# Initiative"),
            ("strat/STRAT-T-0001.md", "# Task"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.hydrated_workspaces, vec!["strat"]);
        assert_eq!(result.files_written, 3);
        assert_eq!(result.files_removed, 0);

        // Verify files on disk
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
        assert!(metis_dir.path().join("strat/STRAT-I-0001.md").exists());
        assert!(metis_dir.path().join("strat/STRAT-T-0001.md").exists());
    }

    #[test]
    fn test_multiple_remote_workspaces() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
            ("alpha/ALPHA-V-0001.md", "alpha vision"),
            ("sre/SRE-V-0001.md", "sre vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.hydrated_workspaces.len(), 3);
        assert_eq!(result.files_written, 3);

        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
        assert!(metis_dir.path().join("alpha/ALPHA-V-0001.md").exists());
        assert!(metis_dir.path().join("sre/SRE-V-0001.md").exists());
    }

    #[test]
    fn test_owned_workspace_skipped() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision from central"),
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        // Only strat should be hydrated, not api
        assert_eq!(result.hydrated_workspaces, vec!["strat"]);
        assert!(!metis_dir.path().join("api").exists());
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
    }

    #[test]
    fn test_new_file_in_remote() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap(),
            "vision"
        );

        // Add a new file to remote
        add_commit_to_remote(
            remote_dir.path(),
            &[("strat/STRAT-T-0001.md", "new task")],
            &[],
            "add task",
        );

        // Re-fetch and re-hydrate
        ctx.fetch().unwrap();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.files_written, 2); // both files written
        assert!(metis_dir.path().join("strat/STRAT-T-0001.md").exists());
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("strat/STRAT-T-0001.md")).unwrap(),
            "new task"
        );
    }

    #[test]
    fn test_updated_file_in_remote() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "v1 content"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap(),
            "v1 content"
        );

        // Update the file in remote
        add_commit_to_remote(
            remote_dir.path(),
            &[("strat/STRAT-V-0001.md", "v2 updated content")],
            &[],
            "update",
        );

        ctx.fetch().unwrap();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap(),
            "v2 updated content"
        );
    }

    #[test]
    fn test_deleted_file_in_remote() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
            ("strat/STRAT-T-0001.md", "task to delete"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert!(metis_dir.path().join("strat/STRAT-T-0001.md").exists());

        // Delete file from remote
        add_commit_to_remote(
            remote_dir.path(),
            &[],
            &["strat/STRAT-T-0001.md"],
            "delete task",
        );

        ctx.fetch().unwrap();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.files_removed, 1);
        assert!(!metis_dir.path().join("strat/STRAT-T-0001.md").exists());
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
    }

    #[test]
    fn test_deleted_workspace_in_remote() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
            ("alpha/ALPHA-V-0001.md", "alpha vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert!(metis_dir.path().join("alpha").exists());

        // Remove all alpha files from remote (effectively removing the workspace)
        add_commit_to_remote(
            remote_dir.path(),
            &[],
            &["alpha/ALPHA-V-0001.md"],
            "remove alpha",
        );

        ctx.fetch().unwrap();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        // alpha folder should be removed since it's now empty in central
        // (central may keep the tree entry or not — either way, local files removed)
        assert!(result.files_removed >= 1 || result.folders_removed.contains(&"alpha".to_string()));
    }

    #[test]
    fn test_new_workspace_appears() {
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert!(!metis_dir.path().join("sre").exists());

        // Add a new workspace to remote
        add_commit_to_remote(
            remote_dir.path(),
            &[("sre/SRE-V-0001.md", "sre vision")],
            &[],
            "add sre workspace",
        );

        ctx.fetch().unwrap();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert!(result.hydrated_workspaces.contains(&"sre".to_string()));
        assert!(metis_dir.path().join("sre/SRE-V-0001.md").exists());
    }

    #[test]
    fn test_first_sync_no_prior_folders() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat"),
            ("alpha/ALPHA-V-0001.md", "alpha"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        // No prior folders exist
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.hydrated_workspaces.len(), 2);
        assert_eq!(result.files_written, 2);
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
        assert!(metis_dir.path().join("alpha/ALPHA-V-0001.md").exists());
    }

    #[test]
    fn test_empty_remote() {
        let (_remote_dir, url) = {
            let dir = TempDir::new().unwrap();
            Repository::init_bare(dir.path()).unwrap();
            let url = format!("file://{}", dir.path().display());
            (dir, url)
        };

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert!(result.hydrated_workspaces.is_empty());
        assert_eq!(result.files_written, 0);
    }

    // ─── Idempotency ──────────────────────────────────────────────────────

    #[test]
    fn test_double_hydration_idempotent() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision content"),
            ("strat/STRAT-T-0001.md", "task content"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();

        let result1 = hydrate(&ctx, metis_dir.path(), "api").unwrap();
        let content1 =
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap();

        let result2 = hydrate(&ctx, metis_dir.path(), "api").unwrap();
        let content2 =
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap();

        assert_eq!(result1.files_written, result2.files_written);
        assert_eq!(result2.files_removed, 0);
        assert_eq!(content1, content2);
    }

    #[test]
    fn test_content_preserved_exactly() {
        let content = "---\nshort_code: STRAT-V-0001\nlevel: vision\n---\n\n# Vision\n\nUnicode: 中文 🎉 ñ";
        let (_remote, url, _) =
            create_remote_with_commit(&[("strat/STRAT-V-0001.md", content)]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        let local_content =
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap();
        assert_eq!(local_content, content);
    }

    // ─── File Filtering ───────────────────────────────────────────────────

    #[test]
    fn test_non_md_files_ignored() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
            ("strat/config.toml", "should not be copied"),
            ("strat/notes.txt", "should not be copied"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.files_written, 1); // Only the .md file
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());
        assert!(!metis_dir.path().join("strat/config.toml").exists());
        assert!(!metis_dir.path().join("strat/notes.txt").exists());
    }

    // ─── Integration: Push then Hydrate ───────────────────────────────────

    #[test]
    fn test_push_then_hydrate() {
        // Create an empty remote
        let (_remote_dir, url) = {
            let dir = TempDir::new().unwrap();
            Repository::init_bare(dir.path()).unwrap();
            let url = format!("file://{}", dir.path().display());
            (dir, url)
        };

        // Workspace A pushes
        let mut ctx_a = SyncContext::new(&url, "api").unwrap();
        ctx_a.fetch().unwrap();
        ctx_a
            .commit_update(
                &[
                    FileEntry {
                        path: "api/API-V-0001.md".to_string(),
                        content: b"api vision".to_vec(),
                    },
                    FileEntry {
                        path: "api/API-T-0001.md".to_string(),
                        content: b"api task".to_vec(),
                    },
                ],
                &[],
                "api initial push",
            )
            .unwrap();
        ctx_a.push().unwrap();

        // Workspace B fetches and hydrates
        let mut ctx_b = SyncContext::new(&url, "sre").unwrap();
        ctx_b.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx_b, metis_dir.path(), "sre").unwrap();

        assert_eq!(result.hydrated_workspaces, vec!["api"]);
        assert_eq!(result.files_written, 2);

        let vision = fs::read_to_string(metis_dir.path().join("api/API-V-0001.md")).unwrap();
        assert_eq!(vision, "api vision");

        let task = fs::read_to_string(metis_dir.path().join("api/API-T-0001.md")).unwrap();
        assert_eq!(task, "api task");
    }

    #[test]
    fn test_sequential_syncs() {
        let (_remote_dir, url) = {
            let dir = TempDir::new().unwrap();
            Repository::init_bare(dir.path()).unwrap();
            let url = format!("file://{}", dir.path().display());
            (dir, url)
        };

        // Workspace A pushes initial content
        let mut ctx_a = SyncContext::new(&url, "api").unwrap();
        ctx_a.fetch().unwrap();
        ctx_a
            .commit_update(
                &[FileEntry {
                    path: "api/API-V-0001.md".to_string(),
                    content: b"api v1".to_vec(),
                }],
                &[],
                "api v1",
            )
            .unwrap();
        ctx_a.push().unwrap();

        // Workspace B hydrates
        let mut ctx_b = SyncContext::new(&url, "sre").unwrap();
        ctx_b.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx_b, metis_dir.path(), "sre").unwrap();
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("api/API-V-0001.md")).unwrap(),
            "api v1"
        );

        // Workspace A pushes update
        let mut ctx_a2 = SyncContext::new(&url, "api").unwrap();
        ctx_a2.fetch().unwrap();
        ctx_a2
            .commit_update(
                &[FileEntry {
                    path: "api/API-V-0001.md".to_string(),
                    content: b"api v2 updated".to_vec(),
                }],
                &[],
                "api v2",
            )
            .unwrap();
        ctx_a2.push().unwrap();

        // Workspace B re-hydrates and sees the update
        ctx_b.fetch().unwrap();
        hydrate(&ctx_b, metis_dir.path(), "sre").unwrap();
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("api/API-V-0001.md")).unwrap(),
            "api v2 updated"
        );
    }

    // ─── Edge Cases ───────────────────────────────────────────────────────

    #[test]
    fn test_large_workspace_hydration() {
        // Create remote with 100 files
        let files: Vec<(String, String)> = (1..=100)
            .map(|i| {
                (
                    format!("strat/STRAT-T-{:04}.md", i),
                    format!("# Task {}", i),
                )
            })
            .collect();

        let file_refs: Vec<(&str, &str)> = files
            .iter()
            .map(|(p, c)| (p.as_str(), c.as_str()))
            .collect();

        let (_remote, url, _) = create_remote_with_commit(&file_refs);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        let result = hydrate(&ctx, metis_dir.path(), "api").unwrap();

        assert_eq!(result.files_written, 100);
    }

    // ─── Gitignore ────────────────────────────────────────────────────────

    #[test]
    fn test_gitignore_created_for_hydrated_workspaces() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
            ("alpha/ALPHA-V-0001.md", "vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        let gitignore = fs::read_to_string(metis_dir.path().join(".gitignore")).unwrap();
        assert!(gitignore.contains("strat/"));
        assert!(gitignore.contains("alpha/"));
        assert!(gitignore.contains("# Hydrated remote workspaces"));
    }

    #[test]
    fn test_gitignore_preserves_existing_entries() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
        ]);

        let metis_dir = create_metis_dir();

        // Write existing gitignore
        fs::write(
            metis_dir.path().join(".gitignore"),
            "metis.db\ncode-index-hashes.json\n",
        )
        .unwrap();

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        let gitignore = fs::read_to_string(metis_dir.path().join(".gitignore")).unwrap();
        assert!(gitignore.contains("metis.db"));
        assert!(gitignore.contains("code-index-hashes.json"));
        assert!(gitignore.contains("strat/"));
    }

    #[test]
    fn test_gitignore_no_duplicate_entries() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();

        // Hydrate twice
        hydrate(&ctx, metis_dir.path(), "api").unwrap();
        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        let gitignore = fs::read_to_string(metis_dir.path().join(".gitignore")).unwrap();
        let strat_count = gitignore.matches("strat/").count();
        assert_eq!(strat_count, 1, "strat/ should appear exactly once");
    }

    // ─── Reserved directory safety ────────────────────────────────────────

    #[test]
    fn test_reserved_directories_not_removed() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "vision"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let metis_dir = create_metis_dir();

        // Create some reserved directories that should never be removed
        fs::create_dir_all(metis_dir.path().join("archived")).unwrap();
        fs::create_dir_all(metis_dir.path().join("strategies")).unwrap();
        fs::create_dir_all(metis_dir.path().join("adrs")).unwrap();

        hydrate(&ctx, metis_dir.path(), "api").unwrap();

        // These should still exist
        assert!(metis_dir.path().join("archived").exists());
        assert!(metis_dir.path().join("strategies").exists());
        assert!(metis_dir.path().join("adrs").exists());
    }
}
