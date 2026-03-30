use anyhow::{Context, Result};
use clap::Args;
use git2::{Repository, Signature};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Args, Debug)]
pub struct FlushCommand;

impl FlushCommand {
    pub async fn execute(&self) -> Result<()> {
        // Find the .metis workspace
        let cwd = std::env::current_dir()?;
        let workspace = find_metis_workspace(&cwd)
            .context("Not inside a Metis workspace (no .metis directory found)")?;

        let pending_dir = workspace.join(".pending");

        // Check if there's anything to flush
        if !pending_dir.exists() || is_dir_empty(&pending_dir)? {
            // Nothing to flush — silent no-op
            return Ok(());
        }

        // Open the git repo
        let repo =
            Repository::discover(&workspace).context("Not inside a git repository")?;

        let repo_root = repo
            .workdir()
            .context("Cannot determine repo working directory")?
            .to_path_buf();

        // Resolve main branch
        let main_branch = resolve_main_branch(&repo)
            .context("Cannot find main or master branch")?;
        let main_ref = format!("refs/heads/{}", main_branch);

        // Collect overlay files and tombstones
        let (overlay_files, tombstones) = collect_overlay_contents(&pending_dir, &workspace)?;

        if overlay_files.is_empty() && tombstones.is_empty() {
            return Ok(());
        }

        // Get main's current tree
        let main_commit = repo
            .find_reference(&main_ref)?
            .peel_to_commit()?;
        let main_tree = main_commit.tree()?;

        // Build a new tree with overlay changes applied
        let new_tree_oid = build_merged_tree(
            &repo,
            &main_tree,
            &repo_root,
            &overlay_files,
            &tombstones,
        )?;

        // Don't create a commit if the tree hasn't changed
        if new_tree_oid == main_tree.id() {
            cleanup_pending(&pending_dir)?;
            return Ok(());
        }

        let new_tree = repo.find_tree(new_tree_oid)?;

        // Create a commit on main
        let sig = repo
            .signature()
            .unwrap_or_else(|_| Signature::now("Metis", "metis@localhost").unwrap());

        repo.commit(
            Some(&main_ref),
            &sig,
            &sig,
            "metis: sync document changes",
            &new_tree,
            &[&main_commit],
        )?;

        // Clean up the pending directory
        cleanup_pending(&pending_dir)?;

        eprintln!("metis: flushed pending changes to {}", main_branch);

        Ok(())
    }
}

fn find_metis_workspace(start: &Path) -> Option<PathBuf> {
    let mut current = start.to_path_buf();
    loop {
        let metis_dir = current.join(".metis");
        if metis_dir.is_dir() {
            return Some(metis_dir);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn resolve_main_branch(repo: &Repository) -> Option<String> {
    for name in &["main", "master"] {
        let refname = format!("refs/heads/{}", name);
        if repo.find_reference(&refname).is_ok() {
            return Some(name.to_string());
        }
    }
    None
}

fn is_dir_empty(dir: &Path) -> Result<bool> {
    Ok(std::fs::read_dir(dir)?.next().is_none())
}

/// Collect overlay files and tombstones.
/// Returns (HashMap<tree_relative_path, content>, Vec<tree_relative_path_to_delete>)
fn collect_overlay_contents(
    pending_dir: &Path,
    workspace_dir: &Path,
) -> Result<(HashMap<String, String>, Vec<String>)> {
    use walkdir::WalkDir;

    let mut files: HashMap<String, String> = HashMap::new();
    let mut tombstones: Vec<String> = Vec::new();

    // workspace_dir is e.g., /project/.metis
    // We need to produce tree-relative paths like .metis/initiatives/FOO/initiative.md
    let ws_name = workspace_dir
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();

    for entry in WalkDir::new(pending_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if !entry.file_type().is_file() {
            continue;
        }

        let rel_to_pending = entry.path().strip_prefix(pending_dir)?;
        let tree_path = format!("{}/{}", ws_name, rel_to_pending.to_string_lossy());

        let file_name = entry.file_name().to_string_lossy();
        if file_name.ends_with(".deleted") {
            // This is a tombstone — remove the .deleted suffix to get the real path
            let real_path = tree_path.trim_end_matches(".deleted").to_string();
            tombstones.push(real_path);
        } else {
            let content = std::fs::read_to_string(entry.path())?;
            files.insert(tree_path, content);
        }
    }

    Ok((files, tombstones))
}

/// Build a new tree from main's tree with overlay changes applied.
fn build_merged_tree(
    repo: &Repository,
    base_tree: &git2::Tree,
    _repo_root: &Path,
    overlay_files: &HashMap<String, String>,
    tombstones: &[String],
) -> Result<git2::Oid> {
    // Strategy: read the full tree, apply changes, write back.
    // For simplicity, use git2's TreeBuilder to reconstruct.
    // Since trees are nested, we need to handle directory structure.

    // Collect all entries from the base tree
    let mut entries: HashMap<String, Vec<u8>> = HashMap::new();
    base_tree.walk(git2::TreeWalkMode::PreOrder, |root, entry| {
        if let Some(name) = entry.name() {
            let full_path = if root.is_empty() {
                name.to_string()
            } else {
                format!("{}{}", root, name)
            };
            if entry.kind() == Some(git2::ObjectType::Blob) {
                if let Ok(blob) = repo.find_blob(entry.id()) {
                    entries.insert(full_path, blob.content().to_vec());
                }
            }
        }
        git2::TreeWalkResult::Ok
    })?;

    // Apply overlay files (add or update)
    for (path, content) in overlay_files {
        entries.insert(path.clone(), content.as_bytes().to_vec());
    }

    // Apply tombstones (remove)
    for path in tombstones {
        entries.remove(path);
    }

    // Rebuild the tree structure
    build_tree_from_entries(repo, &entries)
}

/// Recursively build a git tree from a flat map of path → content.
fn build_tree_from_entries(
    repo: &Repository,
    entries: &HashMap<String, Vec<u8>>,
) -> Result<git2::Oid> {
    // Group entries by top-level directory
    let mut dirs: HashMap<String, HashMap<String, Vec<u8>>> = HashMap::new();
    let mut blobs: Vec<(String, Vec<u8>)> = Vec::new();

    for (path, content) in entries {
        if let Some(slash_pos) = path.find('/') {
            let dir = &path[..slash_pos];
            let rest = &path[slash_pos + 1..];
            dirs.entry(dir.to_string())
                .or_default()
                .insert(rest.to_string(), content.clone());
        } else {
            blobs.push((path.clone(), content.clone()));
        }
    }

    let mut builder = repo.treebuilder(None)?;

    // Add blobs at this level
    for (name, content) in &blobs {
        let blob_oid = repo.blob(content)?;
        builder.insert(name, blob_oid, 0o100644)?;
    }

    // Recursively add subdirectories
    for (dir_name, sub_entries) in &dirs {
        let sub_tree_oid = build_tree_from_entries(repo, sub_entries)?;
        builder.insert(dir_name, sub_tree_oid, 0o040000)?;
    }

    let tree_oid = builder.write()?;
    Ok(tree_oid)
}

fn cleanup_pending(pending_dir: &Path) -> Result<()> {
    if pending_dir.exists() {
        std::fs::remove_dir_all(pending_dir)?;
    }
    Ok(())
}

const HOOK_MARKER: &str = "METIS_POST_COMMIT_HOOK";

const HOOK_CONTENT: &str = r#"#!/bin/sh
# Metis post-commit hook — flushes pending .metis/ overlay changes to main.
# Installed by `metis init` or auto-installed on first Metis operation.
# METIS_POST_COMMIT_HOOK — do not remove this marker comment.

# Only run if metis binary is available
if command -v metis >/dev/null 2>&1; then
    metis flush 2>/dev/null || true
fi
"#;

/// Ensure the git post-commit hook is installed. Idempotent — safe to call repeatedly.
/// Called from `metis init` and can be called from workspace detection for lazy install.
pub fn ensure_git_hook_installed(search_from: &Path) -> Result<()> {
    // Find the .git directory
    let repo = Repository::discover(search_from)
        .map_err(|_| anyhow::anyhow!("Not in a git repository"))?;
    let git_dir = repo.path(); // .git/ directory

    let hooks_dir = git_dir.join("hooks");
    std::fs::create_dir_all(&hooks_dir)?;

    let hook_path = hooks_dir.join("post-commit");

    if hook_path.exists() {
        // Check if our hook is already installed
        let existing = std::fs::read_to_string(&hook_path)?;
        if existing.contains(HOOK_MARKER) {
            return Ok(()); // Already installed
        }

        // Append our hook to the existing one
        let mut content = existing;
        if !content.ends_with('\n') {
            content.push('\n');
        }
        content.push('\n');
        // Skip the shebang line from our hook content when appending
        let hook_body = HOOK_CONTENT
            .lines()
            .skip(1) // skip #!/bin/sh
            .collect::<Vec<_>>()
            .join("\n");
        content.push_str(&hook_body);
        content.push('\n');
        std::fs::write(&hook_path, content)?;
    } else {
        std::fs::write(&hook_path, HOOK_CONTENT)?;
    }

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}
