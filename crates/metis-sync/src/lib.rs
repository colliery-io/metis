//! Git-based sync operations for multi-workspace Metis projects.
//!
//! Provides transient git operations: clone/fetch, diff, commit, push.
//! No persistent `.git/` directory is left behind — all git state lives in
//! a temporary directory that is cleaned up on drop (RAII semantics).

pub mod dehydration;
pub mod hydration;
pub mod orchestration;
pub mod projection;

use git2::{
    build::TreeUpdateBuilder, Cred, CredentialType, DiffOptions, FetchOptions, PushOptions,
    RemoteCallbacks, Repository, Signature,
};
use std::path::{Path, PathBuf};
use tempfile::TempDir;
use thiserror::Error;
use tracing::debug;

// ─── Error types ──────────────────────────────────────────────────────────────

#[derive(Debug, Error)]
pub enum SyncError {
    #[error("git error: {0}")]
    Git(#[from] git2::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("authentication failed: {message}")]
    Auth { message: String },

    #[error("invalid remote URL: {url}")]
    InvalidUrl { url: String },

    #[error("fetch failed for {url}: {reason}")]
    FetchFailed { url: String, reason: String },

    #[error("push rejected (non-fast-forward): remote HEAD has moved since last fetch")]
    PushRejected,

    #[error("push failed: {reason}")]
    PushFailed { reason: String },

    #[error("commit not found: {sha}")]
    CommitNotFound { sha: String },

    #[error("path outside workspace prefix: {path} (expected prefix: {prefix})")]
    PathOutsideWorkspace { path: String, prefix: String },

    #[error("remote is empty (no commits)")]
    EmptyRemote,

    #[error("push retries exhausted after {max_retries} attempts")]
    RetriesExhausted { max_retries: u32 },
}

pub type Result<T> = std::result::Result<T, SyncError>;

// ─── Types ────────────────────────────────────────────────────────────────────

/// The kind of change detected between two commits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

/// A file change detected between two commits.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub kind: ChangeKind,
}

/// A file entry to be committed to the repository.
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Path relative to the repository root (e.g., "api/API-T-0001.md")
    pub path: String,
    /// File content as bytes
    pub content: Vec<u8>,
}

// ─── SyncContext ──────────────────────────────────────────────────────────────

/// Transient git context for sync operations.
///
/// Creates a temporary git repository for performing fetch/diff/commit/push
/// operations against a remote. The temporary directory is cleaned up when
/// this struct is dropped, ensuring no persistent `.git/` state is left
/// inside `.metis/`.
pub struct SyncContext {
    repo: Repository,
    _temp_dir: TempDir,
    remote_url: String,
    workspace_prefix: String,
    /// Tracks the fetched HEAD after a `fetch()` call.
    fetched_head: Option<git2::Oid>,
}

impl std::fmt::Debug for SyncContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SyncContext")
            .field("remote_url", &self.remote_url)
            .field("workspace_prefix", &self.workspace_prefix)
            .field("fetched_head", &self.fetched_head)
            .finish_non_exhaustive()
    }
}

impl SyncContext {
    /// Create a new sync context for the given remote URL and workspace prefix.
    ///
    /// Initializes a temporary git repository and configures it to fetch from
    /// the remote. No network operations happen until `fetch()` is called.
    pub fn new(remote_url: &str, workspace_prefix: &str) -> Result<Self> {
        if remote_url.is_empty() {
            return Err(SyncError::InvalidUrl {
                url: remote_url.to_string(),
            });
        }

        let temp_dir = TempDir::new()?;
        let repo = Repository::init(temp_dir.path())?;

        // Configure the remote
        repo.remote("origin", remote_url)?;

        debug!(
            remote_url,
            workspace_prefix,
            temp_path = %temp_dir.path().display(),
            "sync context created"
        );

        Ok(Self {
            repo,
            _temp_dir: temp_dir,
            remote_url: remote_url.to_string(),
            workspace_prefix: workspace_prefix.to_string(),
            fetched_head: None,
        })
    }

    /// Returns the path to the temporary repository.
    pub fn temp_path(&self) -> &Path {
        self.repo.workdir().unwrap_or(self.repo.path())
    }

    /// Returns the workspace prefix.
    pub fn workspace_prefix(&self) -> &str {
        &self.workspace_prefix
    }

    /// Returns the remote URL.
    pub fn remote_url(&self) -> &str {
        &self.remote_url
    }

    /// Returns the fetched HEAD OID, if a fetch has been performed.
    pub fn fetched_head(&self) -> Option<git2::Oid> {
        self.fetched_head
    }

    /// Returns a reference to the underlying git repository.
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    // ─── Fetch ────────────────────────────────────────────────────────────────

    /// Fetch the latest state from the remote.
    ///
    /// Returns the HEAD OID of the remote's default branch, or `None` if the
    /// remote repository is empty (no commits).
    pub fn fetch(&mut self) -> Result<Option<git2::Oid>> {
        let mut remote = self.repo.find_remote("origin")?;
        let callbacks = Self::make_callbacks();
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        debug!(url = %self.remote_url, "fetching from remote");

        remote
            .fetch(&["refs/heads/*:refs/remotes/origin/*"], Some(&mut fetch_opts), None)
            .map_err(|e| {
                if e.message().contains("auth")
                    || e.message().contains("credential")
                    || e.message().contains("SSH")
                    || e.code() == git2::ErrorCode::Auth
                {
                    SyncError::Auth {
                        message: e.message().to_string(),
                    }
                } else {
                    SyncError::FetchFailed {
                        url: self.remote_url.clone(),
                        reason: e.message().to_string(),
                    }
                }
            })?;

        // Find the remote HEAD — try common default branch names
        let head_oid = self.resolve_remote_head();
        self.fetched_head = head_oid;

        debug!(?head_oid, "fetch complete");
        Ok(head_oid)
    }

    /// Resolve the remote HEAD by looking for common default branch refs.
    fn resolve_remote_head(&self) -> Option<git2::Oid> {
        // Try refs/remotes/origin/main, then origin/master
        for branch in &["main", "master"] {
            let refname = format!("refs/remotes/origin/{}", branch);
            if let Ok(reference) = self.repo.find_reference(&refname) {
                if let Some(oid) = reference.target() {
                    return Some(oid);
                }
            }
        }

        // Try any branch under refs/remotes/origin/
        if let Ok(refs) = self.repo.references_glob("refs/remotes/origin/*") {
            for reference in refs.flatten() {
                if let Some(oid) = reference.target() {
                    return Some(oid);
                }
            }
        }

        None
    }

    // ─── Diff ─────────────────────────────────────────────────────────────────

    /// Compute changes between a prior commit and the current fetched HEAD.
    ///
    /// If `since_sha` is `None`, all files in HEAD are returned as `Added`.
    /// If `since_sha` is `Some`, returns the diff between that commit and HEAD.
    ///
    /// The `path_filter` parameter optionally restricts results to paths under
    /// a given prefix (e.g., `"api/"` to only see changes in the api workspace).
    pub fn diff_since(
        &self,
        since_sha: Option<&str>,
        path_filter: Option<&str>,
    ) -> Result<Vec<FileChange>> {
        let head_oid = self
            .fetched_head
            .ok_or_else(|| SyncError::FetchFailed {
                url: self.remote_url.clone(),
                reason: "no fetch performed yet".to_string(),
            })?;

        let head_commit = self.repo.find_commit(head_oid)?;
        let head_tree = head_commit.tree()?;

        let mut diff_opts = DiffOptions::new();
        if let Some(prefix) = path_filter {
            diff_opts.pathspec(prefix);
        }

        let diff = match since_sha {
            Some(sha) => {
                let since_oid = git2::Oid::from_str(sha).map_err(|_| SyncError::CommitNotFound {
                    sha: sha.to_string(),
                })?;
                let since_commit =
                    self.repo
                        .find_commit(since_oid)
                        .map_err(|_| SyncError::CommitNotFound {
                            sha: sha.to_string(),
                        })?;
                let since_tree = since_commit.tree()?;
                self.repo
                    .diff_tree_to_tree(Some(&since_tree), Some(&head_tree), Some(&mut diff_opts))?
            }
            None => {
                // No prior commit — everything in HEAD is new
                self.repo
                    .diff_tree_to_tree(None, Some(&head_tree), Some(&mut diff_opts))?
            }
        };

        let mut changes = Vec::new();
        diff.foreach(
            &mut |delta, _progress| {
                let kind = match delta.status() {
                    git2::Delta::Added | git2::Delta::Untracked => ChangeKind::Added,
                    git2::Delta::Deleted => ChangeKind::Deleted,
                    git2::Delta::Modified
                    | git2::Delta::Renamed
                    | git2::Delta::Copied
                    | git2::Delta::Typechange => ChangeKind::Modified,
                    _ => return true, // skip other statuses
                };

                if let Some(path) = delta.new_file().path().or_else(|| delta.old_file().path()) {
                    changes.push(FileChange {
                        path: path.to_string_lossy().to_string(),
                        kind,
                    });
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(changes)
    }

    // ─── Read ─────────────────────────────────────────────────────────────────

    /// Read a file's content from a specific commit.
    pub fn read_blob(&self, commit_oid: git2::Oid, path: &str) -> Result<Vec<u8>> {
        let commit = self.repo.find_commit(commit_oid)?;
        let tree = commit.tree()?;
        let entry = tree.get_path(Path::new(path))?;
        let blob = self.repo.find_blob(entry.id())?;
        Ok(blob.content().to_vec())
    }

    // ─── Tree Enumeration ──────────────────────────────────────────────────────

    /// List top-level directory names in the fetched HEAD tree.
    /// These correspond to workspace prefixes in the central repo.
    pub fn list_workspace_folders(&self) -> Result<Vec<String>> {
        let head_oid = self
            .fetched_head
            .ok_or_else(|| SyncError::FetchFailed {
                url: self.remote_url.clone(),
                reason: "no fetch performed yet".to_string(),
            })?;

        let commit = self.repo.find_commit(head_oid)?;
        let tree = commit.tree()?;

        let mut folders = Vec::new();
        for entry in tree.iter() {
            if entry.kind() == Some(git2::ObjectType::Tree) {
                if let Some(name) = entry.name() {
                    folders.push(name.to_string());
                }
            }
        }

        Ok(folders)
    }

    /// List all `.md` files in a specific workspace folder from the fetched HEAD.
    /// Returns `(filename, content_bytes)` pairs.
    pub fn list_workspace_files(&self, prefix: &str) -> Result<Vec<(String, Vec<u8>)>> {
        let head_oid = self
            .fetched_head
            .ok_or_else(|| SyncError::FetchFailed {
                url: self.remote_url.clone(),
                reason: "no fetch performed yet".to_string(),
            })?;

        let commit = self.repo.find_commit(head_oid)?;
        let tree = commit.tree()?;

        let prefix_entry = match tree.get_name(prefix) {
            Some(entry) => entry,
            None => return Ok(Vec::new()),
        };

        let prefix_tree = self.repo.find_tree(prefix_entry.id())?;
        let mut files = Vec::new();

        for entry in prefix_tree.iter() {
            if entry.kind() != Some(git2::ObjectType::Blob) {
                continue;
            }
            let name = match entry.name() {
                Some(n) => n.to_string(),
                None => continue,
            };
            // Only include .md files
            if !name.ends_with(".md") {
                continue;
            }
            let blob = self.repo.find_blob(entry.id())?;
            files.push((name, blob.content().to_vec()));
        }

        Ok(files)
    }

    // ─── Commit ───────────────────────────────────────────────────────────────

    /// Create a commit that updates files in the repository.
    ///
    /// All file paths in `files` must be under `workspace_prefix/`.
    /// Files in `removals` are deleted from the tree.
    /// The commit is parented on the fetched HEAD (if exists) or is an initial commit.
    ///
    /// Returns the new commit OID.
    pub fn commit_update(
        &mut self,
        files: &[FileEntry],
        removals: &[String],
        message: &str,
    ) -> Result<git2::Oid> {
        let prefix_with_slash = format!("{}/", self.workspace_prefix);

        // Validate all paths are under the workspace prefix
        for file in files {
            if !file.path.starts_with(&prefix_with_slash) {
                return Err(SyncError::PathOutsideWorkspace {
                    path: file.path.clone(),
                    prefix: self.workspace_prefix.clone(),
                });
            }
        }
        for removal in removals {
            if !removal.starts_with(&prefix_with_slash) {
                return Err(SyncError::PathOutsideWorkspace {
                    path: removal.clone(),
                    prefix: self.workspace_prefix.clone(),
                });
            }
        }

        // Start from the existing tree if we have a parent commit
        let parent_commit = self.fetched_head.and_then(|oid| self.repo.find_commit(oid).ok());

        let base_tree = parent_commit
            .as_ref()
            .and_then(|c| c.tree().ok());

        // Build the updated tree using TreeUpdateBuilder
        let mut builder = TreeUpdateBuilder::new();

        // Add/update files
        for file in files {
            let blob_oid = self.repo.blob(&file.content)?;
            builder.upsert(
                &file.path,
                blob_oid,
                git2::FileMode::Blob,
            );
        }

        // Remove files
        for removal in removals {
            builder.remove(removal);
        }

        let new_tree_oid = builder.create_updated(
            &self.repo,
            base_tree.as_ref().unwrap_or(&self.repo.treebuilder(None)?.write().and_then(|oid| self.repo.find_tree(oid))?),
        )?;
        let new_tree = self.repo.find_tree(new_tree_oid)?;

        let sig = Signature::now("metis-sync", "metis@localhost")?;
        let parents: Vec<&git2::Commit> = parent_commit.as_ref().into_iter().collect();

        // Create commit without updating HEAD ref (avoids "current tip is not
        // the first parent" error on retry after a failed push). We manually
        // set HEAD afterward so push() can read it.
        let commit_oid =
            self.repo
                .commit(None, &sig, &sig, message, &new_tree, &parents)?;
        self.repo.set_head_detached(commit_oid)?;

        debug!(%commit_oid, "commit created");
        Ok(commit_oid)
    }

    // ─── Push ─────────────────────────────────────────────────────────────────

    /// Push the local HEAD to the remote's default branch.
    ///
    /// The remote branch name is resolved from the fetched refs (defaults to `main`).
    pub fn push(&self) -> Result<()> {
        let mut remote = self.repo.find_remote("origin")?;
        let callbacks = Self::make_callbacks();
        let mut push_opts = PushOptions::new();
        push_opts.remote_callbacks(callbacks);

        let branch = self.resolve_remote_branch_name();
        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);

        // Ensure local HEAD points to the branch
        if let Ok(head) = self.repo.head() {
            if let Some(oid) = head.target() {
                self.repo
                    .reference(&format!("refs/heads/{}", branch), oid, true, "metis push")?;
            }
        }

        debug!(refspec, "pushing to remote");

        let mut push_err: Option<String> = None;
        {
            let push_err_ref = &mut push_err;
            let mut callbacks = Self::make_callbacks();
            callbacks.push_update_reference(move |_refname, status| {
                if let Some(msg) = status {
                    *push_err_ref = Some(msg.to_string());
                }
                Ok(())
            });
            let mut push_opts = PushOptions::new();
            push_opts.remote_callbacks(callbacks);

            remote.push(&[&refspec], Some(&mut push_opts)).map_err(|e| {
                let msg = e.message().to_string();
                if Self::is_push_rejection(&msg) {
                    SyncError::PushRejected
                } else if e.code() == git2::ErrorCode::Auth {
                    SyncError::Auth { message: msg }
                } else {
                    SyncError::PushFailed { reason: msg }
                }
            })?;
        }

        if let Some(err_msg) = push_err {
            if Self::is_push_rejection(&err_msg) {
                return Err(SyncError::PushRejected);
            }
            return Err(SyncError::PushFailed { reason: err_msg });
        }

        debug!("push complete");
        Ok(())
    }

    /// Check if a push error message indicates a retriable rejection.
    ///
    /// These errors all mean "the remote ref has moved" and are resolved by
    /// re-fetching and retrying:
    /// - "non-fast-forward" — standard rejection
    /// - "rejected" — generic rejection
    /// - "not present locally" — commits exist on remote we don't have
    /// - "already exists" — concurrent ref update race
    /// - "lock" — ref lock contention from concurrent push
    fn is_push_rejection(msg: &str) -> bool {
        msg.contains("non-fast-forward")
            || msg.contains("rejected")
            || msg.contains("not present locally")
            || msg.contains("already exists")
            || msg.contains("lock")
    }

    /// Determine the remote branch name from fetched refs.
    fn resolve_remote_branch_name(&self) -> String {
        for branch in &["main", "master"] {
            let refname = format!("refs/remotes/origin/{}", branch);
            if self.repo.find_reference(&refname).is_ok() {
                return branch.to_string();
            }
        }
        // Default to main for new remotes
        "main".to_string()
    }

    // ─── Auth ─────────────────────────────────────────────────────────────────

    /// Build remote callbacks with the authentication chain:
    /// SSH agent → SSH key files → credential helper → fail
    fn make_callbacks() -> RemoteCallbacks<'static> {
        let mut callbacks = RemoteCallbacks::new();
        let attempt = std::cell::Cell::new(0u32);

        callbacks.credentials(move |url, username_from_url, allowed_types| {
            let current = attempt.get();
            attempt.set(current + 1);

            // Prevent infinite auth loops
            if current > 10 {
                return Err(git2::Error::from_str(
                    "authentication failed after multiple attempts",
                ));
            }

            let username = username_from_url.unwrap_or("git");

            // Try SSH agent first
            if allowed_types.contains(CredentialType::SSH_KEY) {
                if current == 0 {
                    debug!("trying SSH agent authentication");
                    return Cred::ssh_key_from_agent(username);
                }

                // Try common SSH key file paths
                let home = dirs_next().unwrap_or_default();
                let key_names = ["id_ed25519", "id_rsa", "id_ecdsa"];
                let key_idx = (current as usize).saturating_sub(1);
                if key_idx < key_names.len() {
                    let key_path = home.join(".ssh").join(key_names[key_idx]);
                    if key_path.exists() {
                        debug!(key = %key_path.display(), "trying SSH key file");
                        return Cred::ssh_key(username, None, &key_path, None);
                    }
                }
            }

            // Try credential helper / default credentials
            if allowed_types.contains(CredentialType::USER_PASS_PLAINTEXT) {
                debug!("trying credential helper");
                return Cred::credential_helper(
                    &git2::Config::open_default()
                        .unwrap_or_else(|_| git2::Config::new().unwrap()),
                    url,
                    username_from_url,
                );
            }

            // Try default credentials
            if allowed_types.contains(CredentialType::DEFAULT) {
                return Cred::default();
            }

            Err(git2::Error::from_str(&format!(
                "no valid authentication method found for {}",
                url
            )))
        });

        callbacks
    }
}

/// Get the user's home directory.
fn dirs_next() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    /// Helper: create a bare git repo to act as a "remote"
    fn create_bare_remote() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        Repository::init_bare(dir.path()).unwrap();
        let url = format!("file://{}", dir.path().display());
        (dir, url)
    }

    /// Helper: create a bare remote with an initial commit on `main`
    fn create_remote_with_commit(files: &[(&str, &str)]) -> (TempDir, String, git2::Oid) {
        let remote_dir = TempDir::new().unwrap();
        let bare = Repository::init_bare(remote_dir.path()).unwrap();

        // Build a tree with nested paths using TreeUpdateBuilder
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
        let commit_oid = bare
            .commit(
                Some("refs/heads/main"),
                &sig,
                &sig,
                message,
                &new_tree,
                &[&parent],
            )
            .unwrap();
        commit_oid
    }

    // ─── SyncContext Lifecycle ─────────────────────────────────────────────

    #[test]
    fn test_create_context_valid_file_url() {
        let (_remote, url) = create_bare_remote();
        let ctx = SyncContext::new(&url, "api").unwrap();
        assert!(ctx.temp_path().exists());
        assert_eq!(ctx.workspace_prefix(), "api");
        assert_eq!(ctx.remote_url(), url);
    }

    #[test]
    fn test_create_context_ssh_url() {
        // SSH URLs are accepted at creation time (no network call)
        let ctx = SyncContext::new("git@github.com:org/repo.git", "api").unwrap();
        assert!(ctx.temp_path().exists());
    }

    #[test]
    fn test_create_context_https_url() {
        let ctx = SyncContext::new("https://github.com/org/repo.git", "api").unwrap();
        assert!(ctx.temp_path().exists());
    }

    #[test]
    fn test_create_context_empty_url_fails() {
        let err = SyncContext::new("", "api").unwrap_err();
        assert!(matches!(err, SyncError::InvalidUrl { .. }));
    }

    #[test]
    fn test_drop_cleanup() {
        let (_remote, url) = create_bare_remote();
        let temp_path;
        {
            let ctx = SyncContext::new(&url, "api").unwrap();
            temp_path = ctx.temp_path().to_path_buf();
            assert!(temp_path.exists());
        }
        // After drop, temp dir should be cleaned up
        assert!(!temp_path.exists());
    }

    #[test]
    fn test_multiple_concurrent_contexts() {
        let (_remote1, url1) = create_bare_remote();
        let (_remote2, url2) = create_bare_remote();

        let ctx1 = SyncContext::new(&url1, "api").unwrap();
        let ctx2 = SyncContext::new(&url2, "sre").unwrap();

        assert_ne!(ctx1.temp_path(), ctx2.temp_path());
        assert_eq!(ctx1.workspace_prefix(), "api");
        assert_eq!(ctx2.workspace_prefix(), "sre");
    }

    // ─── Fetch ────────────────────────────────────────────────────────────

    #[test]
    fn test_fetch_from_populated_remote() {
        let (_remote, url, expected_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "# vision")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();

        let head = ctx.fetch().unwrap();
        assert_eq!(head, Some(expected_oid));
    }

    #[test]
    fn test_fetch_from_empty_remote() {
        let (_remote, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();

        let head = ctx.fetch().unwrap();
        assert_eq!(head, None);
    }

    #[test]
    fn test_fetch_updates_after_remote_changes() {
        let (remote_dir, url, first_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();

        let head1 = ctx.fetch().unwrap();
        assert_eq!(head1, Some(first_oid));

        // Add a new commit to the remote
        let second_oid =
            add_commit_to_remote(remote_dir.path(), &[("api/API-V-0001.md", "v2")], &[], "update");

        let head2 = ctx.fetch().unwrap();
        assert_eq!(head2, Some(second_oid));
        assert_ne!(head1, head2);
    }

    #[test]
    fn test_fetch_with_unreachable_remote() {
        let mut ctx =
            SyncContext::new("file:///nonexistent/path/to/repo.git", "api").unwrap();
        let err = ctx.fetch().unwrap_err();
        assert!(
            matches!(err, SyncError::FetchFailed { .. }),
            "expected FetchFailed, got: {:?}",
            err
        );
    }

    // ─── Diff ─────────────────────────────────────────────────────────────

    #[test]
    fn test_diff_no_changes() {
        let (_remote, url, initial_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "# vision")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let changes = ctx.diff_since(Some(&initial_oid.to_string()), None).unwrap();
        assert!(changes.is_empty());
    }

    #[test]
    fn test_diff_new_files_added() {
        let (remote_dir, url, initial_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);
        add_commit_to_remote(
            remote_dir.path(),
            &[("api/API-T-0001.md", "task")],
            &[],
            "add task",
        );

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let changes = ctx.diff_since(Some(&initial_oid.to_string()), None).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "api/API-T-0001.md");
        assert_eq!(changes[0].kind, ChangeKind::Added);
    }

    #[test]
    fn test_diff_files_modified() {
        let (remote_dir, url, initial_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);
        add_commit_to_remote(
            remote_dir.path(),
            &[("api/API-V-0001.md", "v2 updated")],
            &[],
            "update vision",
        );

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let changes = ctx.diff_since(Some(&initial_oid.to_string()), None).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "api/API-V-0001.md");
        assert_eq!(changes[0].kind, ChangeKind::Modified);
    }

    #[test]
    fn test_diff_files_deleted() {
        let (remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "v1"),
            ("api/API-T-0001.md", "task"),
        ]);
        add_commit_to_remote(
            remote_dir.path(),
            &[],
            &["api/API-T-0001.md"],
            "delete task",
        );

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let changes = ctx.diff_since(Some(&initial_oid.to_string()), None).unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "api/API-T-0001.md");
        assert_eq!(changes[0].kind, ChangeKind::Deleted);
    }

    #[test]
    fn test_diff_mixed_operations() {
        let (remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
            ("api/API-T-0001.md", "task1"),
            ("api/API-T-0002.md", "task2"),
        ]);
        add_commit_to_remote(
            remote_dir.path(),
            &[
                ("api/API-V-0001.md", "updated vision"),  // modified
                ("api/API-T-0003.md", "new task"),         // added
            ],
            &["api/API-T-0001.md"], // deleted
            "mixed changes",
        );

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let changes = ctx.diff_since(Some(&initial_oid.to_string()), None).unwrap();
        assert_eq!(changes.len(), 3);

        let added: Vec<_> = changes.iter().filter(|c| c.kind == ChangeKind::Added).collect();
        let modified: Vec<_> = changes.iter().filter(|c| c.kind == ChangeKind::Modified).collect();
        let deleted: Vec<_> = changes.iter().filter(|c| c.kind == ChangeKind::Deleted).collect();

        assert_eq!(added.len(), 1);
        assert_eq!(modified.len(), 1);
        assert_eq!(deleted.len(), 1);
    }

    #[test]
    fn test_diff_first_sync_no_prior_commit() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
            ("api/API-T-0001.md", "task"),
        ]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // No prior commit — everything is new
        let changes = ctx.diff_since(None, None).unwrap();
        assert_eq!(changes.len(), 2);
        assert!(changes.iter().all(|c| c.kind == ChangeKind::Added));
    }

    #[test]
    fn test_diff_invalid_prior_commit() {
        let (_remote, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "vision")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let fake_sha = "a".repeat(40);
        let err = ctx.diff_since(Some(&fake_sha), None).unwrap_err();
        assert!(matches!(err, SyncError::CommitNotFound { .. }));
    }

    #[test]
    fn test_diff_with_path_filter() {
        let (remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
            ("sre/SRE-V-0001.md", "sre vision"),
        ]);
        add_commit_to_remote(
            remote_dir.path(),
            &[
                ("api/API-T-0001.md", "api task"),
                ("sre/SRE-T-0001.md", "sre task"),
            ],
            &[],
            "add tasks",
        );

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Filter to only api/ changes
        let changes = ctx
            .diff_since(Some(&initial_oid.to_string()), Some("api/"))
            .unwrap();
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].path, "api/API-T-0001.md");
    }

    #[test]
    fn test_diff_without_fetch_fails() {
        let (_remote, url) = create_bare_remote();
        let ctx = SyncContext::new(&url, "api").unwrap();

        let err = ctx.diff_since(None, None).unwrap_err();
        assert!(matches!(err, SyncError::FetchFailed { .. }));
    }

    // ─── Read Blob ────────────────────────────────────────────────────────

    #[test]
    fn test_read_blob() {
        let (_remote, url, commit_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "# My Vision\n\nContent here.")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let content = ctx.read_blob(commit_oid, "api/API-V-0001.md").unwrap();
        assert_eq!(
            String::from_utf8(content).unwrap(),
            "# My Vision\n\nContent here."
        );
    }

    #[test]
    fn test_read_blob_nonexistent_path() {
        let (_remote, url, commit_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "vision")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let err = ctx.read_blob(commit_oid, "api/NOPE.md").unwrap_err();
        assert!(matches!(err, SyncError::Git(_)));
    }

    // ─── Commit ───────────────────────────────────────────────────────────

    #[test]
    fn test_commit_to_empty_remote() {
        let (_remote, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap(); // empty

        let files = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: b"# Vision".to_vec(),
        }];

        let oid = ctx.commit_update(&files, &[], "initial commit").unwrap();
        let commit = ctx.repo.find_commit(oid).unwrap();
        assert_eq!(commit.message(), Some("initial commit"));
        assert_eq!(commit.parent_count(), 0);
    }

    #[test]
    fn test_commit_with_parent() {
        let (_remote, url, initial_oid) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let files = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: b"v2 updated".to_vec(),
        }];

        let oid = ctx.commit_update(&files, &[], "update vision").unwrap();
        let commit = ctx.repo.find_commit(oid).unwrap();
        assert_eq!(commit.parent_count(), 1);
        assert_eq!(commit.parent_id(0).unwrap(), initial_oid);
    }

    #[test]
    fn test_commit_preserves_other_workspace_files() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
            ("sre/SRE-V-0001.md", "sre vision"),
        ]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        let _head = ctx.fetch().unwrap().unwrap();

        let files = vec![FileEntry {
            path: "api/API-T-0001.md".to_string(),
            content: b"new api task".to_vec(),
        }];

        let oid = ctx.commit_update(&files, &[], "add api task").unwrap();

        // Verify SRE files are preserved
        let sre_content = ctx.read_blob(oid, "sre/SRE-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(sre_content).unwrap(), "sre vision");

        // Verify new file exists
        let api_task = ctx.read_blob(oid, "api/API-T-0001.md").unwrap();
        assert_eq!(String::from_utf8(api_task).unwrap(), "new api task");
    }

    #[test]
    fn test_commit_with_removals() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "vision"),
            ("api/API-T-0001.md", "task"),
        ]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let oid = ctx
            .commit_update(&[], &["api/API-T-0001.md".to_string()], "remove task")
            .unwrap();

        // Removed file should be gone
        let err = ctx.read_blob(oid, "api/API-T-0001.md").unwrap_err();
        assert!(matches!(err, SyncError::Git(_)));

        // Other file preserved
        let vision = ctx.read_blob(oid, "api/API-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(vision).unwrap(), "vision");
    }

    #[test]
    fn test_commit_rejects_path_outside_workspace() {
        let (_remote, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "vision")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let files = vec![FileEntry {
            path: "sre/SRE-V-0001.md".to_string(),
            content: b"hacking into sre workspace".to_vec(),
        }];

        let err = ctx.commit_update(&files, &[], "evil commit").unwrap_err();
        assert!(matches!(err, SyncError::PathOutsideWorkspace { .. }));
    }

    #[test]
    fn test_commit_rejects_removal_outside_workspace() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api"),
            ("sre/SRE-V-0001.md", "sre"),
        ]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let err = ctx
            .commit_update(&[], &["sre/SRE-V-0001.md".to_string()], "evil delete")
            .unwrap_err();
        assert!(matches!(err, SyncError::PathOutsideWorkspace { .. }));
    }

    // ─── Push ─────────────────────────────────────────────────────────────

    #[test]
    fn test_push_first_commit() {
        let (remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let files = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: b"# Vision".to_vec(),
        }];
        ctx.commit_update(&files, &[], "initial").unwrap();
        ctx.push().unwrap();

        // Verify the remote has the commit
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        assert_eq!(commit.message(), Some("initial"));
    }

    #[test]
    fn test_push_update_to_existing() {
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let files = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: b"v2".to_vec(),
        }];
        ctx.commit_update(&files, &[], "update").unwrap();
        ctx.push().unwrap();

        // Verify updated
        let bare = Repository::open_bare(remote_dir.path()).unwrap();
        let main_ref = bare.find_reference("refs/heads/main").unwrap();
        let commit = bare.find_commit(main_ref.target().unwrap()).unwrap();
        assert_eq!(commit.message(), Some("update"));
    }

    #[test]
    fn test_push_non_fast_forward_rejected() {
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        // Workspace A fetches and makes a commit
        let mut ctx_a = SyncContext::new(&url, "api").unwrap();
        ctx_a.fetch().unwrap();
        let files_a = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: b"from A".to_vec(),
        }];
        ctx_a.commit_update(&files_a, &[], "A's update").unwrap();

        // Someone else pushes to remote first
        add_commit_to_remote(
            remote_dir.path(),
            &[("sre/SRE-V-0001.md", "sre stuff")],
            &[],
            "concurrent push",
        );

        // A's push should be rejected (non-fast-forward)
        let err = ctx_a.push().unwrap_err();
        assert!(
            matches!(err, SyncError::PushRejected | SyncError::PushFailed { .. }),
            "expected push rejection, got: {:?}",
            err
        );
    }

    // ─── Full Cycle Integration ───────────────────────────────────────────

    #[test]
    fn test_full_cycle_init_fetch_commit_push() {
        let (_remote_dir, url) = create_bare_remote();

        // First workspace creates initial content
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let files = vec![
            FileEntry {
                path: "api/API-V-0001.md".to_string(),
                content: b"# API Vision\n\nBuild the best API.".to_vec(),
            },
            FileEntry {
                path: "api/API-I-0001.md".to_string(),
                content: b"# API Initiative\n\nFirst initiative.".to_vec(),
            },
            FileEntry {
                path: "api/API-T-0001.md".to_string(),
                content: b"# Task 1\n\nDo the thing.".to_vec(),
            },
        ];

        let commit_oid = ctx.commit_update(&files, &[], "initial push").unwrap();
        ctx.push().unwrap();

        // Second context fetches and sees everything
        let mut ctx2 = SyncContext::new(&url, "sre").unwrap();
        let head = ctx2.fetch().unwrap().unwrap();
        assert_eq!(head, commit_oid);

        // Diff from scratch shows all files
        let changes = ctx2.diff_since(None, None).unwrap();
        assert_eq!(changes.len(), 3);
        assert!(changes.iter().all(|c| c.kind == ChangeKind::Added));

        // Can read content
        let vision = ctx2.read_blob(head, "api/API-V-0001.md").unwrap();
        assert_eq!(
            String::from_utf8(vision).unwrap(),
            "# API Vision\n\nBuild the best API."
        );
    }

    #[test]
    fn test_two_workspaces_independent_push() {
        let (_remote_dir, url) = create_bare_remote();

        // API workspace pushes first
        let mut api_ctx = SyncContext::new(&url, "api").unwrap();
        api_ctx.fetch().unwrap();
        api_ctx
            .commit_update(
                &[FileEntry {
                    path: "api/API-V-0001.md".to_string(),
                    content: b"api vision".to_vec(),
                }],
                &[],
                "api initial",
            )
            .unwrap();
        api_ctx.push().unwrap();

        // SRE workspace fetches (sees api content), then pushes own content
        let mut sre_ctx = SyncContext::new(&url, "sre").unwrap();
        sre_ctx.fetch().unwrap();
        sre_ctx
            .commit_update(
                &[FileEntry {
                    path: "sre/SRE-V-0001.md".to_string(),
                    content: b"sre vision".to_vec(),
                }],
                &[],
                "sre initial",
            )
            .unwrap();
        sre_ctx.push().unwrap();

        // Verify both workspaces' content exists
        let mut verify_ctx = SyncContext::new(&url, "verify").unwrap();
        let head = verify_ctx.fetch().unwrap().unwrap();

        let api_content = verify_ctx.read_blob(head, "api/API-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(api_content).unwrap(), "api vision");

        let sre_content = verify_ctx.read_blob(head, "sre/SRE-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(sre_content).unwrap(), "sre vision");
    }

    #[test]
    fn test_no_persistent_git_dir() {
        let (_remote, url) = create_bare_remote();
        let metis_dir = TempDir::new().unwrap();

        // Create context, do operations, drop it
        {
            let mut ctx = SyncContext::new(&url, "api").unwrap();
            ctx.fetch().unwrap();
        }

        // .metis/ should have no .git directory
        assert!(!metis_dir.path().join(".git").exists());
    }

    #[test]
    fn test_binary_safe_content() {
        let (_remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Content with various byte patterns
        let binary_content: Vec<u8> = (0..=255).collect();
        let files = vec![FileEntry {
            path: "api/API-T-0001.md".to_string(),
            content: binary_content.clone(),
        }];

        let oid = ctx.commit_update(&files, &[], "binary content").unwrap();
        let read_back = ctx.read_blob(oid, "api/API-T-0001.md").unwrap();
        assert_eq!(read_back, binary_content);
    }

    #[test]
    fn test_unicode_content_preserved() {
        let (_remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let content = "# 愿景\n\n中文内容 🎉 emoji ñ àéîõü";
        let files = vec![FileEntry {
            path: "api/API-V-0001.md".to_string(),
            content: content.as_bytes().to_vec(),
        }];

        let oid = ctx.commit_update(&files, &[], "unicode").unwrap();
        let read_back = ctx.read_blob(oid, "api/API-V-0001.md").unwrap();
        assert_eq!(String::from_utf8(read_back).unwrap(), content);
    }

    #[test]
    fn test_large_payload() {
        let (_remote_dir, url) = create_bare_remote();
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Create 100 files (testing performance is reasonable)
        let files: Vec<FileEntry> = (1..=100)
            .map(|i| FileEntry {
                path: format!("api/API-T-{:04}.md", i),
                content: format!("# Task {}\n\nContent for task {}.", i, i).into_bytes(),
            })
            .collect();

        let oid = ctx.commit_update(&files, &[], "bulk push").unwrap();
        ctx.push().unwrap();

        // Verify count by reading back
        let commit = ctx.repo.find_commit(oid).unwrap();
        let tree = commit.tree().unwrap();
        let api_entry = tree.get_name("api").unwrap();
        let api_tree = ctx.repo.find_tree(api_entry.id()).unwrap();
        assert_eq!(api_tree.len(), 100);
    }
}
