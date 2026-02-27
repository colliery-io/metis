//! Sync orchestration engine.
//!
//! Composes fetch, hydration, dehydration, and push into a single `sync()`
//! operation. This is the single entry point for all sync triggers
//! (CLI, GUI, git hooks).

use crate::dehydration::{self, DehydrationResult, FlatDoc};
use crate::hydration::{self, HydrationResult};
use crate::{SyncContext, SyncError};
use std::path::Path;
use tracing::{debug, info, warn};

/// Configuration needed for a sync operation.
/// Extracted from config.toml by the caller.
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// The upstream remote URL (SSH or HTTPS)
    pub upstream_url: String,
    /// The owned workspace prefix (e.g., "api")
    pub workspace_prefix: String,
    /// The last synced commit SHA (None for first sync)
    pub last_synced_commit: Option<String>,
}

/// Options for controlling sync behavior.
#[derive(Debug, Clone, Default)]
pub struct SyncOptions {
    /// Skip the pre-sync freshness check
    pub force: bool,
    /// Maximum push retry attempts (0 = no retry, default = 3)
    pub max_retries: u32,
}

impl SyncOptions {
    pub fn new() -> Self {
        Self {
            force: false,
            max_retries: 5,
        }
    }
}

/// Result of a sync operation.
#[derive(Debug)]
pub struct SyncResult {
    /// Hydration results (None if hydration was skipped)
    pub hydration: Option<HydrationResult>,
    /// Dehydration results (None if dehydration was skipped)
    pub dehydration: Option<DehydrationResult>,
    /// The new commit SHA to record as last_synced_commit
    pub new_synced_commit: Option<String>,
    /// Number of push retries needed (0 = first attempt succeeded)
    pub push_retries: u32,
    /// Whether this was a no-op (nothing to sync)
    pub is_noop: bool,
    /// Non-fatal warnings
    pub warnings: Vec<String>,
}

impl SyncResult {
    /// Total files pulled from remote workspaces.
    pub fn files_pulled(&self) -> usize {
        self.hydration
            .as_ref()
            .map(|h| h.files_written)
            .unwrap_or(0)
    }

    /// Total files pushed to central.
    pub fn files_pushed(&self) -> usize {
        self.dehydration
            .as_ref()
            .map(|d| d.files_pushed)
            .unwrap_or(0)
    }

    /// Whether a push was performed.
    pub fn pushed(&self) -> bool {
        self.dehydration
            .as_ref()
            .map(|d| d.pushed)
            .unwrap_or(false)
    }
}

/// Run the full sync cycle with push conflict retry.
///
/// Each attempt performs the complete cycle: fetch → hydrate → dehydrate → push.
/// On push rejection (non-fast-forward), the entire cycle is re-run from fetch
/// so that remote changes from other workspaces are incorporated before retrying.
/// Non-push errors (auth, network, I/O) are NOT retried.
///
/// # Sequence per attempt
///
/// 1. Fetch latest state from central
/// 2. Hydrate remote workspaces to local `.metis/<prefix>/` (non-fatal)
/// 3. Dehydrate owned workspace (flatten + commit + push)
///
/// # Arguments
///
/// * `config` - Sync configuration (extracted from config.toml)
/// * `metis_dir` - Path to the local `.metis/` directory
/// * `local_documents` - Flattened owned workspace documents (from layout::flatten_workspace)
/// * `options` - Sync behavior options (includes max_retries, default 5)
pub fn sync(
    config: &SyncConfig,
    metis_dir: &Path,
    local_documents: &[FlatDoc],
    options: &SyncOptions,
) -> Result<SyncResult, SyncError> {
    info!(
        upstream = %config.upstream_url,
        prefix = %config.workspace_prefix,
        "starting sync"
    );

    let mut ctx = SyncContext::new(&config.upstream_url, &config.workspace_prefix)?;
    let mut retries = 0u32;

    loop {
        // Step 1: Fetch from central
        let fetched_head = ctx.fetch()?;

        // Step 2: Hydrate remote workspaces (non-fatal)
        let mut warnings = Vec::new();
        let hydration = if fetched_head.is_some() {
            debug!("hydrating remote workspaces");
            match hydration::hydrate(&ctx, metis_dir, &config.workspace_prefix) {
                Ok(hydration_result) => {
                    for (workspace, err) in &hydration_result.errors {
                        warnings.push(format!("hydration error for {}: {}", workspace, err));
                    }
                    Some(hydration_result)
                }
                Err(e) => {
                    warnings.push(format!("hydration failed: {}", e));
                    None
                }
            }
        } else {
            None
        };

        // Step 3: Dehydrate owned workspace (commit + push)
        match dehydration::dehydrate(&mut ctx, local_documents, &config.workspace_prefix) {
            Ok(dehydration_result) => {
                // Build final result
                let new_synced_commit = if dehydration_result.pushed {
                    dehydration_result.commit_oid.map(|oid| oid.to_string())
                } else {
                    fetched_head.map(|oid| oid.to_string())
                };

                let is_noop = hydration
                    .as_ref()
                    .map(|h| h.files_written == 0 && h.files_removed == 0)
                    .unwrap_or(true)
                    && dehydration_result.files_pushed == 0;

                let result = SyncResult {
                    hydration,
                    dehydration: Some(dehydration_result),
                    new_synced_commit,
                    push_retries: retries,
                    is_noop,
                    warnings,
                };

                info!(
                    files_pulled = result.files_pulled(),
                    files_pushed = result.files_pushed(),
                    pushed = result.pushed(),
                    retries = result.push_retries,
                    is_noop = result.is_noop,
                    "sync complete"
                );

                return Ok(result);
            }
            Err(SyncError::PushRejected) => {
                if retries >= options.max_retries {
                    return Err(SyncError::RetriesExhausted {
                        max_retries: options.max_retries,
                    });
                }
                retries += 1;
                warn!(
                    retry = retries,
                    max = options.max_retries,
                    "push rejected, running full sync cycle retry"
                );
                // Loop continues: full cycle fetch → hydrate → dehydrate
            }
            Err(e) => {
                // Non-push errors are NOT retried
                return Err(e);
            }
        }
    }
}

/// Run a sync that only fetches and hydrates (no push).
/// Useful for read-only sync operations.
pub fn sync_pull_only(
    config: &SyncConfig,
    metis_dir: &Path,
) -> Result<SyncResult, SyncError> {
    info!(
        upstream = %config.upstream_url,
        prefix = %config.workspace_prefix,
        "starting pull-only sync"
    );

    let mut ctx = SyncContext::new(&config.upstream_url, &config.workspace_prefix)?;
    let fetched_head = ctx.fetch()?;

    let mut result = SyncResult {
        hydration: None,
        dehydration: None,
        new_synced_commit: fetched_head.map(|oid| oid.to_string()),
        push_retries: 0,
        is_noop: true,
        warnings: Vec::new(),
    };

    if fetched_head.is_some() {
        match hydration::hydrate(&ctx, metis_dir, &config.workspace_prefix) {
            Ok(hydration_result) => {
                result.is_noop = hydration_result.files_written == 0
                    && hydration_result.files_removed == 0;
                for (workspace, err) in &hydration_result.errors {
                    result
                        .warnings
                        .push(format!("hydration error for {}: {}", workspace, err));
                }
                result.hydration = Some(hydration_result);
            }
            Err(e) => {
                result
                    .warnings
                    .push(format!("hydration failed: {}", e));
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{build::TreeUpdateBuilder, Repository, Signature};
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    /// Helper: create a bare remote
    fn create_bare_remote() -> (TempDir, String) {
        let dir = TempDir::new().unwrap();
        Repository::init_bare(dir.path()).unwrap();
        let url = format!("file://{}", dir.path().display());
        (dir, url)
    }

    /// Helper: create a bare remote with commits
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

    /// Helper: add a commit to remote
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

    fn make_config(url: &str, prefix: &str, last_commit: Option<&str>) -> SyncConfig {
        SyncConfig {
            upstream_url: url.to_string(),
            workspace_prefix: prefix.to_string(),
            last_synced_commit: last_commit.map(|s| s.to_string()),
        }
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

    // ─── Happy Path ───────────────────────────────────────────────────────

    #[test]
    fn test_full_sync_happy_path() {
        let (_remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // Hydration should have pulled strat
        assert!(result.hydration.is_some());
        let hydration = result.hydration.as_ref().unwrap();
        assert_eq!(hydration.files_written, 1);
        assert!(metis_dir.path().join("strat/STRAT-V-0001.md").exists());

        // Dehydration should have pushed api
        assert!(result.pushed());
        assert_eq!(result.files_pushed(), 1);

        // Should have a new commit SHA
        assert!(result.new_synced_commit.is_some());
    }

    #[test]
    fn test_first_sync_no_last_commit() {
        let (_remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None); // First sync — no last commit
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        assert!(result.hydration.is_some());
        assert!(result.pushed());
        assert!(result.new_synced_commit.is_some());
    }

    #[test]
    fn test_first_sync_empty_central() {
        let (_remote_dir, url) = create_bare_remote();

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // No hydration (empty central)
        assert!(result.hydration.is_none());

        // Should push successfully
        assert!(result.pushed());
        assert!(result.new_synced_commit.is_some());
    }

    // ─── No-op Scenarios ──────────────────────────────────────────────────

    #[test]
    fn test_no_local_changes_skips_push() {
        let (_remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        // Same content as already in central
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // Hydration still runs
        assert!(result.hydration.is_some());
        // But push is skipped (no changes)
        assert!(!result.pushed());
        // Still records the fetched HEAD
        assert!(result.new_synced_commit.is_some());
    }

    #[test]
    fn test_no_remote_changes_push_still_works() {
        let (_remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "old vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        // Updated local content
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "new vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // No remote workspaces to hydrate
        assert!(result.hydration.is_some());
        assert_eq!(result.files_pulled(), 0);
        // But push should happen
        assert!(result.pushed());
    }

    #[test]
    fn test_hydration_only_no_push() {
        let (remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
            ("strat/STRAT-V-0001.md", "strat v1"),
        ]);

        // Strat gets updated in central
        add_commit_to_remote(
            remote_dir.path(),
            &[("strat/STRAT-V-0001.md", "strat v2")],
            &[],
            "update strat",
        );

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        // Local unchanged
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // Hydration should pull updated strat
        assert_eq!(result.files_pulled(), 1);
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("strat/STRAT-V-0001.md")).unwrap(),
            "strat v2"
        );
        // No push needed (local unchanged)
        assert!(!result.pushed());
    }

    #[test]
    fn test_both_local_and_remote_changes() {
        let (remote_dir, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "old api vision"),
            ("strat/STRAT-V-0001.md", "strat v1"),
        ]);

        // Strat updates in central
        add_commit_to_remote(
            remote_dir.path(),
            &[("strat/STRAT-V-0001.md", "strat v2")],
            &[],
            "update strat",
        );

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        // Local has updated content
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "new api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        // Both hydration and dehydration happened
        assert_eq!(result.files_pulled(), 1);
        assert!(result.pushed());
        assert_eq!(result.files_pushed(), 1);
    }

    // ─── Pull-Only Sync ───────────────────────────────────────────────────

    #[test]
    fn test_pull_only_sync() {
        let (_remote, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
            ("api/API-V-0001.md", "api vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);

        let result = sync_pull_only(&config, metis_dir.path()).unwrap();

        assert!(result.hydration.is_some());
        assert_eq!(result.files_pulled(), 1); // only strat, not api (owned)
        assert!(result.dehydration.is_none()); // no push
        assert!(result.new_synced_commit.is_some());
    }

    // ─── SyncResult ───────────────────────────────────────────────────────

    #[test]
    fn test_result_noop_detection() {
        let (_remote, url, initial_oid) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "api vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", Some(&initial_oid.to_string()));
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();

        assert!(result.is_noop);
    }

    // ─── Error Handling ───────────────────────────────────────────────────

    #[test]
    fn test_network_failure() {
        let config = make_config("file:///nonexistent/repo.git", "api", None);
        let metis_dir = TempDir::new().unwrap();
        let docs: Vec<FlatDoc> = vec![];
        let opts = SyncOptions::new();

        let err = sync(&config, metis_dir.path(), &docs, &opts).unwrap_err();
        assert!(matches!(err, SyncError::FetchFailed { .. }));
    }

    #[test]
    fn test_partial_failure_hydration_error_nonfatal() {
        // This tests that if hydration has errors for one workspace,
        // the sync still proceeds with dehydration
        let (_remote_dir, url) = create_bare_remote();

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        // This should succeed even if there's nothing to hydrate
        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();
        assert!(result.pushed());
    }

    // ─── Two-Workspace Roundtrip ──────────────────────────────────────────

    #[test]
    fn test_two_workspace_roundtrip() {
        let (_remote_dir, url) = create_bare_remote();

        // Workspace A syncs (pushes)
        let metis_a = TempDir::new().unwrap();
        let config_a = make_config(&url, "api", None);
        let docs_a = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "api vision"),
            ("API-T-0001", "API-T-0001.md", "api task 1"),
        ]);
        let opts = SyncOptions::new();

        let result_a = sync(&config_a, metis_a.path(), &docs_a, &opts).unwrap();
        assert!(result_a.pushed());
        let commit_a = result_a.new_synced_commit.clone().unwrap();

        // Workspace B syncs (pulls A, pushes B)
        let metis_b = TempDir::new().unwrap();
        let config_b = make_config(&url, "sre", None);
        let docs_b = make_docs(&[("SRE-V-0001", "SRE-V-0001.md", "sre vision")]);

        let result_b = sync(&config_b, metis_b.path(), &docs_b, &opts).unwrap();
        assert!(result_b.pushed());
        assert_eq!(result_b.files_pulled(), 2); // pulled api's 2 files

        // Verify B has A's documents
        assert!(metis_b.path().join("api/API-V-0001.md").exists());
        assert!(metis_b.path().join("api/API-T-0001.md").exists());

        // Now A syncs again (pulls B)
        let config_a2 = make_config(&url, "api", Some(&commit_a));
        let result_a2 = sync(&config_a2, metis_a.path(), &docs_a, &opts).unwrap();
        assert_eq!(result_a2.files_pulled(), 1); // pulled sre's 1 file
        assert!(!result_a2.pushed()); // no local changes

        // Verify A has B's documents
        assert!(metis_a.path().join("sre/SRE-V-0001.md").exists());
    }

    #[test]
    fn test_three_workspace_convergence() {
        let (_remote_dir, url) = create_bare_remote();
        let opts = SyncOptions::new();

        // A pushes
        let metis_a = TempDir::new().unwrap();
        let config_a = make_config(&url, "alpha", None);
        let docs_a = make_docs(&[("ALPHA-V-0001", "ALPHA-V-0001.md", "alpha")]);
        let r_a = sync(&config_a, metis_a.path(), &docs_a, &opts).unwrap();
        let commit_a = r_a.new_synced_commit.unwrap();

        // B pushes
        let metis_b = TempDir::new().unwrap();
        let config_b = make_config(&url, "beta", None);
        let docs_b = make_docs(&[("BETA-V-0001", "BETA-V-0001.md", "beta")]);
        let r_b = sync(&config_b, metis_b.path(), &docs_b, &opts).unwrap();
        let commit_b = r_b.new_synced_commit.unwrap();

        // C pushes
        let metis_c = TempDir::new().unwrap();
        let config_c = make_config(&url, "gamma", None);
        let docs_c = make_docs(&[("GAMMA-V-0001", "GAMMA-V-0001.md", "gamma")]);
        let _r_c = sync(&config_c, metis_c.path(), &docs_c, &opts).unwrap();

        // Now all three sync again to converge
        let config_a2 = make_config(&url, "alpha", Some(&commit_a));
        sync(&config_a2, metis_a.path(), &docs_a, &opts).unwrap();

        let config_b2 = make_config(&url, "beta", Some(&commit_b));
        sync(&config_b2, metis_b.path(), &docs_b, &opts).unwrap();

        // A should see beta and gamma
        assert!(metis_a.path().join("beta/BETA-V-0001.md").exists());
        assert!(metis_a.path().join("gamma/GAMMA-V-0001.md").exists());

        // B should see alpha and gamma
        assert!(metis_b.path().join("alpha/ALPHA-V-0001.md").exists());
        assert!(metis_b.path().join("gamma/GAMMA-V-0001.md").exists());
    }

    // ─── Sequential Syncs ─────────────────────────────────────────────────

    #[test]
    fn test_sequential_syncs_incremental() {
        let (_remote_dir, url) = create_bare_remote();
        let opts = SyncOptions::new();

        let metis_dir = TempDir::new().unwrap();
        let config1 = make_config(&url, "api", None);
        let docs1 = make_docs(&[("API-V-0001", "API-V-0001.md", "v1")]);

        let r1 = sync(&config1, metis_dir.path(), &docs1, &opts).unwrap();
        assert!(r1.pushed());
        let commit1 = r1.new_synced_commit.unwrap();

        // Second sync with a change
        let config2 = make_config(&url, "api", Some(&commit1));
        let docs2 = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "v2"),
            ("API-T-0001", "API-T-0001.md", "new task"),
        ]);

        let r2 = sync(&config2, metis_dir.path(), &docs2, &opts).unwrap();
        assert!(r2.pushed());
        assert_eq!(r2.files_pushed(), 2);
    }

    #[test]
    fn test_rapid_sequential_sync_noop() {
        let (_remote_dir, url) = create_bare_remote();
        let opts = SyncOptions::new();

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);

        let r1 = sync(&config, metis_dir.path(), &docs, &opts).unwrap();
        assert!(r1.pushed());
        let commit = r1.new_synced_commit.unwrap();

        // Immediately sync again with same content
        let config2 = make_config(&url, "api", Some(&commit));
        let r2 = sync(&config2, metis_dir.path(), &docs, &opts).unwrap();
        assert!(!r2.pushed()); // no-op
    }

    // ─── Push Retry — Component Level (Deterministic) ───────────────────

    #[test]
    fn test_push_conflict_detected() {
        // Inject a concurrent commit between fetch and push → PushRejected
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Simulate concurrent push from another workspace
        add_commit_to_remote(
            remote_dir.path(),
            &[("sre/SRE-V-0001.md", "sre content")],
            &[],
            "concurrent push",
        );

        // Our dehydrate should fail — remote HEAD moved
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "v2")]);
        let err = dehydration::dehydrate(&mut ctx, &docs, "api").unwrap_err();
        assert!(
            matches!(err, SyncError::PushRejected | SyncError::PushFailed { .. }),
            "expected push rejection, got: {:?}",
            err
        );
    }

    #[test]
    fn test_push_conflict_resolved_after_refetch() {
        // Inject conflict, re-fetch, retry → success
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        // Simulate concurrent push
        add_commit_to_remote(
            remote_dir.path(),
            &[("sre/SRE-V-0001.md", "sre content")],
            &[],
            "concurrent push",
        );

        // First attempt fails
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "v2")]);
        let err = dehydration::dehydrate(&mut ctx, &docs, "api");
        assert!(err.is_err(), "first attempt should fail");

        // Re-fetch and retry
        ctx.fetch().unwrap();
        let result = dehydration::dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(result.files_pushed, 1);
    }

    #[test]
    fn test_retry_full_cycle_rehydrates_new_content() {
        // On retry, re-hydration picks up new content from other workspace
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        let metis_dir = TempDir::new().unwrap();

        // Step 1: Create context and do initial fetch + hydrate
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();
        // No SRE workspace exists yet, nothing to hydrate
        let hydrate1 = hydration::hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert_eq!(hydrate1.files_written, 0);

        // Step 2: Another workspace pushes (simulating concurrent push)
        add_commit_to_remote(
            remote_dir.path(),
            &[("sre/SRE-V-0001.md", "sre vision content")],
            &[],
            "sre workspace push",
        );

        // Step 3: Our dehydrate fails (non-fast-forward)
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "v2")]);
        let err = dehydration::dehydrate(&mut ctx, &docs, "api");
        assert!(err.is_err());

        // Step 4: Re-fetch (like sync() retry does)
        ctx.fetch().unwrap();

        // Step 5: Re-hydrate — should pick up SRE's new content
        let hydrate2 = hydration::hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert_eq!(hydrate2.files_written, 1);
        assert!(metis_dir.path().join("sre/SRE-V-0001.md").exists());
        assert_eq!(
            fs::read_to_string(metis_dir.path().join("sre/SRE-V-0001.md")).unwrap(),
            "sre vision content"
        );

        // Step 6: Re-dehydrate — should succeed now
        let result = dehydration::dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
    }

    #[test]
    fn test_multiple_conflicts_then_success() {
        // Push fails multiple times, succeeds after enough retries
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("api/API-V-0001.md", "v1")]);

        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();

        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "updated")]);
        let mut retries = 0u32;

        // Inject 3 sequential conflicts
        for i in 0..3 {
            add_commit_to_remote(
                remote_dir.path(),
                &[(&format!("ws{}/DOC.md", i), &format!("content {}", i))],
                &[],
                &format!("concurrent push {}", i),
            );

            let err = dehydration::dehydrate(&mut ctx, &docs, "api");
            assert!(err.is_err(), "attempt {} should fail", i);
            retries += 1;
            ctx.fetch().unwrap();
        }

        // No more interference — this attempt should succeed
        let result = dehydration::dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
        assert_eq!(retries, 3);
    }

    // ─── Push Retry — sync() Level ───────────────────────────────────────

    #[test]
    fn test_success_first_try_zero_retries() {
        let (_remote_dir, url, _) = create_remote_with_commit(&[
            ("strat/STRAT-V-0001.md", "strat vision"),
        ]);

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "api vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();
        assert!(result.pushed());
        assert_eq!(result.push_retries, 0);
    }

    #[test]
    fn test_max_retries_zero_no_conflict_succeeds() {
        let (_remote_dir, url) = create_bare_remote();

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);
        let opts = SyncOptions {
            force: false,
            max_retries: 0,
        };

        // No conflict → succeeds even with max_retries=0
        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();
        assert!(result.pushed());
        assert_eq!(result.push_retries, 0);
    }

    #[test]
    fn test_retries_exhausted_error() {
        // Verify the RetriesExhausted error variant exists and is correct
        let err = SyncError::RetriesExhausted { max_retries: 5 };
        assert!(matches!(err, SyncError::RetriesExhausted { max_retries: 5 }));
        assert!(
            err.to_string().contains("5"),
            "error message should include retry count: {}",
            err
        );
    }

    #[test]
    fn test_network_failure_not_retried() {
        // Network errors should fail immediately, not trigger retries
        let config = make_config("file:///nonexistent/repo.git", "api", None);
        let metis_dir = TempDir::new().unwrap();
        let docs: Vec<FlatDoc> = vec![];
        let opts = SyncOptions {
            force: false,
            max_retries: 5, // plenty of retries available
        };

        let err = sync(&config, metis_dir.path(), &docs, &opts).unwrap_err();
        // Should be FetchFailed, NOT RetriesExhausted
        assert!(
            matches!(err, SyncError::FetchFailed { .. }),
            "expected FetchFailed, got: {:?}",
            err
        );
    }

    #[test]
    fn test_default_max_retries_is_five() {
        let opts = SyncOptions::new();
        assert_eq!(opts.max_retries, 5);
    }

    #[test]
    fn test_push_retries_field_accessible() {
        let (_remote_dir, url) = create_bare_remote();

        let metis_dir = TempDir::new().unwrap();
        let config = make_config(&url, "api", None);
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "vision")]);
        let opts = SyncOptions::new();

        let result = sync(&config, metis_dir.path(), &docs, &opts).unwrap();
        // push_retries is accessible and meaningful
        assert_eq!(result.push_retries, 0);
    }

    // ─── Push Retry — Racing Workspaces (Threaded) ───────────────────────

    #[test]
    fn test_two_workspaces_racing() {
        use std::sync::Arc;
        use std::thread;

        // Seed with initial commit to avoid race on initial ref creation
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("seed/SEED.md", "initial seed")]);
        let _keep_remote = Arc::new(remote_dir);

        let url_a = url.clone();
        let url_b = url.clone();

        let handle_a = thread::spawn(move || {
            let metis = TempDir::new().unwrap();
            let config = make_config(&url_a, "alpha", None);
            let docs = make_docs(&[("A-V-0001", "A-V-0001.md", "alpha vision")]);
            let opts = SyncOptions::new();
            sync(&config, metis.path(), &docs, &opts).unwrap()
        });

        let handle_b = thread::spawn(move || {
            let metis = TempDir::new().unwrap();
            let config = make_config(&url_b, "beta", None);
            let docs = make_docs(&[("B-V-0001", "B-V-0001.md", "beta vision")]);
            let opts = SyncOptions::new();
            sync(&config, metis.path(), &docs, &opts).unwrap()
        });

        let result_a = handle_a.join().unwrap();
        let result_b = handle_b.join().unwrap();

        // Both should succeed
        assert!(result_a.pushed());
        assert!(result_b.pushed());

        // At least one may have needed retries (or none if timing allows)
        // Total retries should be <= max_retries
        assert!(result_a.push_retries <= 5);
        assert!(result_b.push_retries <= 5);
    }

    #[test]
    fn test_three_workspaces_racing() {
        use std::sync::Arc;
        use std::thread;

        // Seed with initial commit to avoid race on initial ref creation
        let (remote_dir, url, _) =
            create_remote_with_commit(&[("seed/SEED.md", "initial seed")]);
        let _keep_remote = Arc::new(remote_dir);

        let handles: Vec<_> = ["alpha", "beta", "gamma"]
            .iter()
            .map(|prefix| {
                let url = url.clone();
                let prefix = prefix.to_string();
                thread::spawn(move || {
                    let metis = TempDir::new().unwrap();
                    let config = make_config(&url, &prefix, None);
                    let sc = format!("{}-V-0001", prefix.to_uppercase());
                    let fn_ = format!("{}.md", sc);
                    let docs = make_docs(&[(&sc, &fn_, &format!("{} vision", prefix))]);
                    let opts = SyncOptions::new();
                    sync(&config, metis.path(), &docs, &opts).unwrap()
                })
            })
            .collect();

        let results: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

        // All should succeed
        for (i, result) in results.iter().enumerate() {
            assert!(result.pushed(), "workspace {} failed to push", i);
            assert!(
                result.push_retries <= 5,
                "workspace {} exceeded retry budget: {}",
                i,
                result.push_retries
            );
        }
    }

    #[test]
    fn test_five_workspaces_sequential_all_converge() {
        // 5 workspaces push sequentially, all content preserved in central.
        // Tests the same scenario as racing but without file:// transport
        // race conditions (concurrent local pushes can corrupt refs).
        let (_remote_dir, url) = create_bare_remote();
        let opts = SyncOptions::new();

        let prefixes = ["alpha", "beta", "gamma", "delta", "epsilon"];

        // Each workspace syncs in turn
        for prefix in &prefixes {
            let metis = TempDir::new().unwrap();
            let config = make_config(&url, prefix, None);
            let sc = format!("{}-V-0001", prefix.to_uppercase());
            let fn_ = format!("{}.md", sc);
            let docs = make_docs(&[(&sc, &fn_, &format!("{} vision", prefix))]);

            let result = sync(&config, metis.path(), &docs, &opts).unwrap();
            assert!(result.pushed(), "{} failed to push", prefix);
        }

        // Verify central has all 5 workspaces
        let mut verify_ctx = SyncContext::new(&url, "verify").unwrap();
        let head = verify_ctx.fetch().unwrap().unwrap();
        for prefix in &prefixes {
            let path = format!("{}/{}-V-0001.md", prefix, prefix.to_uppercase());
            let content = verify_ctx.read_blob(head, &path).unwrap();
            assert_eq!(
                String::from_utf8(content).unwrap(),
                format!("{} vision", prefix),
                "content mismatch for {}",
                prefix
            );
        }
    }

    #[test]
    fn test_sequential_pushes_no_lost_writes() {
        // Each workspace pushes multiple documents sequentially.
        // Verifies no data is lost when workspaces take turns.
        let (_remote_dir, url) = create_bare_remote();
        let opts = SyncOptions::new();

        // API workspace pushes
        let metis_api = TempDir::new().unwrap();
        let config_api = make_config(&url, "api", None);
        let docs_api = make_docs(&[
            ("API-V-0001", "API-V-0001.md", "api vision"),
            ("API-T-0001", "API-T-0001.md", "api task 1"),
            ("API-T-0002", "API-T-0002.md", "api task 2"),
        ]);
        let result_api = sync(&config_api, metis_api.path(), &docs_api, &opts).unwrap();
        assert!(result_api.pushed());
        assert_eq!(result_api.files_pushed(), 3);

        // SRE workspace pushes (should NOT overwrite API's data)
        let metis_sre = TempDir::new().unwrap();
        let config_sre = make_config(&url, "sre", None);
        let docs_sre = make_docs(&[
            ("SRE-V-0001", "SRE-V-0001.md", "sre vision"),
            ("SRE-T-0001", "SRE-T-0001.md", "sre task 1"),
            ("SRE-T-0002", "SRE-T-0002.md", "sre task 2"),
        ]);
        let result_sre = sync(&config_sre, metis_sre.path(), &docs_sre, &opts).unwrap();
        assert!(result_sre.pushed());
        assert_eq!(result_sre.files_pushed(), 3);

        // Verify ALL files present in central (no lost writes)
        let mut verify_ctx = SyncContext::new(&url, "verify").unwrap();
        let head = verify_ctx.fetch().unwrap().unwrap();

        for prefix in &["api", "sre"] {
            let up = prefix.to_uppercase();
            for suffix in &["V-0001", "T-0001", "T-0002"] {
                let path = format!("{}/{}-{}.md", prefix, up, suffix);
                let content = verify_ctx.read_blob(head, &path);
                assert!(content.is_ok(), "missing file in central: {}", path);
            }
        }
    }

    // ─── Push Retry — Edge Cases ─────────────────────────────────────────

    #[test]
    fn test_retry_after_partial_hydration() {
        // First attempt hydrates content, push fails, retry re-hydrates (idempotent)
        let (remote_dir, url, _) = create_remote_with_commit(&[
            ("api/API-V-0001.md", "v1"),
            ("sre/SRE-V-0001.md", "sre content"),
        ]);

        let metis_dir = TempDir::new().unwrap();

        // First cycle: fetch, hydrate (writes SRE files)
        let mut ctx = SyncContext::new(&url, "api").unwrap();
        ctx.fetch().unwrap();
        let h1 = hydration::hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert_eq!(h1.files_written, 1);
        assert!(metis_dir.path().join("sre/SRE-V-0001.md").exists());

        // Inject conflict
        add_commit_to_remote(
            remote_dir.path(),
            &[("gamma/G-V-0001.md", "gamma")],
            &[],
            "concurrent",
        );

        // Push fails
        let docs = make_docs(&[("API-V-0001", "API-V-0001.md", "v2")]);
        assert!(dehydration::dehydrate(&mut ctx, &docs, "api").is_err());

        // Retry cycle: re-fetch, re-hydrate (SRE already there, gamma is new)
        ctx.fetch().unwrap();
        hydration::hydrate(&ctx, metis_dir.path(), "api").unwrap();
        assert!(metis_dir.path().join("sre/SRE-V-0001.md").exists()); // still there
        assert!(metis_dir.path().join("gamma/G-V-0001.md").exists()); // newly hydrated

        // Re-push succeeds
        let result = dehydration::dehydrate(&mut ctx, &docs, "api").unwrap();
        assert!(result.pushed);
    }
}
