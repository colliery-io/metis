---
id: libgit2-integration-for-in-memory
level: task
title: "libgit2 integration for in-memory git operations"
short_code: "METIS-T-0078"
created_at: 2026-02-26T01:32:05.982008+00:00
updated_at: 2026-02-26T16:31:54.761208+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# libgit2 integration for in-memory git operations

## Objective

Integrate the `git2` crate (Rust bindings for libgit2) to provide in-memory git operations for sync. No persistent `.git/` directory inside `.metis/` — git context is created transiently during sync and torn down after.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `git2` crate added as dependency to a new `metis-sync` crate (or module in `metis-docs-core`)
- [ ] Can clone/fetch a remote repository into a temporary location
- [ ] Can create commits with arbitrary tree contents (for dehydration push)
- [ ] Can push commits to a remote (owned folder only)
- [ ] Can diff between two commits (for CDC — what changed since last sync)
- [ ] Authentication delegates to git's existing mechanisms (SSH agent, credential helpers)
- [ ] No persistent `.git/` directory left in `.metis/` after operations complete
- [ ] Works with both SSH and HTTPS remote URLs
- [ ] Temporary git state is cleaned up even on error (RAII / drop semantics)

## Implementation Notes

### Technical Approach

- New crate: `metis-sync` (or `metis-docs-core::sync` module) — keeps sync concerns separate from document management
- Use `git2::Repository::init()` with a temp directory for transient operations
- Fetch via `git2::Remote::fetch()` with callbacks for auth
- Auth callback chain: SSH agent → SSH key files → credential helper → fail with clear error
- Diff via `git2::Diff::tree_to_tree()` between `last_synced_commit` and fetched HEAD
- Push via `git2::Remote::push()` with refspec targeting owned folder
- Temp directory cleaned up via `Drop` impl on a wrapper struct

### Key API Surface

```rust
pub struct SyncContext {
    repo: git2::Repository,
    temp_dir: TempDir,
    remote_url: String,
    workspace_prefix: String,
}

impl SyncContext {
    pub fn new(remote_url: &str, prefix: &str) -> Result<Self>;
    pub fn fetch(&self) -> Result<git2::Oid>;          // returns fetched HEAD
    pub fn diff_since(&self, since: git2::Oid) -> Result<Vec<FileChange>>;
    pub fn push(&self, tree: git2::Tree) -> Result<git2::Oid>;
}
```

### Dependencies

- METIS-T-0076 (config.toml — needs `upstream_url` and `workspace_prefix`)

### Risk Considerations

- libgit2 SSH support can be tricky across platforms (especially macOS keychain integration). Test early on all target platforms.
- Credential callback ordering matters — SSH agent should be tried first.

## Test Scenarios

### Unit Tests — SyncContext Lifecycle

1. **Create context — valid URL**: SSH URL → SyncContext created, temp dir exists
2. **Create context — HTTPS URL**: HTTPS URL → SyncContext created
3. **Create context — invalid URL**: garbage string → clear error, no temp dir leaked
4. **Drop cleanup**: create SyncContext, drop it → temp directory is removed from disk
5. **Drop cleanup on error**: create SyncContext, simulate error mid-operation, drop → temp dir still cleaned up (RAII)
6. **Multiple concurrent contexts**: create two SyncContexts for different remotes → both work independently, no cross-contamination

### Unit Tests — Fetch

7. **Fetch from populated remote**: remote has commits → fetch returns HEAD OID
8. **Fetch from empty remote**: remote exists but has no commits → appropriate error or empty result
9. **Fetch updates after remote changes**: fetch once, remote gets new commits, fetch again → returns new HEAD
10. **Fetch with SSH agent auth**: SSH agent has key for remote → fetch succeeds
11. **Fetch with SSH key file auth**: no agent, but key file exists → fetch succeeds
12. **Fetch with HTTPS credential helper**: HTTPS URL, credential helper configured → fetch succeeds
13. **Fetch with no valid auth**: no SSH agent, no key files, no credential helper → clear auth error message, not a generic git error
14. **Fetch with network failure**: remote unreachable → error message includes URL and "cannot reach"
15. **Fetch timeout**: remote hangs indefinitely → operation times out (not hangs forever)

### Unit Tests — Diff

16. **Diff — no changes**: diff between same commit → empty changeset
17. **Diff — new files added**: files added between commits → listed as additions
18. **Diff — files modified**: files changed between commits → listed as modifications
19. **Diff — files deleted**: files removed between commits → listed as deletions
20. **Diff — mixed operations**: adds + modifies + deletes in same diff → all categorized correctly
21. **Diff — first sync (no prior commit)**: `last_synced_commit` is None → diff returns all files as new
22. **Diff — invalid prior commit**: `last_synced_commit` SHA doesn't exist in repo → appropriate error (repo was force-pushed or rebased)
23. **Diff — only other workspace changed**: changes in `strat/` folder, owned workspace is `api/` → diff still shows changes (hydration needs them)

### Unit Tests — Push

24. **Push — first commit to empty remote**: no prior commits → push creates initial commit
25. **Push — update to existing remote**: prior commits exist → push adds new commit with correct parent
26. **Push — non-fast-forward rejection**: remote HEAD moved since fetch → push fails with identifiable rejection error (not generic)
27. **Push — auth failure on push**: read auth works but push auth fails → clear error distinguishing read vs write access

### Integration Tests

28. **Full cycle — init, fetch, diff, push**: create remote (bare repo), create SyncContext, fetch, make changes, push → remote has the new commit
29. **Concurrent operations**: two SyncContexts for same remote, both fetch and push → at most one push succeeds, other gets rejection (no corruption)
30. **Large payload**: push 500+ files in a single commit → succeeds within reasonable time
31. **Binary-safe**: push `.md` files containing unusual byte sequences → content preserved exactly

### Platform-Specific Tests

32. **macOS SSH agent**: test with macOS keychain-based SSH agent
33. **Linux SSH agent**: test with ssh-agent on Linux
34. **HTTPS with token auth**: GitHub personal access token via credential helper → works

### Cleanup & Safety

35. **No persistent .git in .metis**: after any SyncContext operation and drop → `.metis/` has no `.git` directory
36. **Disk space recovery**: after context drop → temp directory fully removed, disk space freed
37. **Crash simulation**: kill process mid-fetch → temp dir is an orphan but in system temp, not in `.metis/` (OS will clean it up eventually)

## Status Updates

### Session 1 — Implementation Complete

**New crate**: `crates/metis-sync/` — separate from `metis-docs-core` to isolate git2/libgit2 dependency.

**Core types**:
- `SyncContext` — RAII wrapper around a temporary git repository. Creates temp dir on `new()`, cleans up on `Drop`. Holds `git2::Repository`, `TempDir`, remote URL, and workspace prefix.
- `SyncError` — typed error enum with variants for auth, fetch, push rejection, path-outside-workspace, etc.
- `FileChange` / `ChangeKind` — diff results (Added/Modified/Deleted)
- `FileEntry` — file path + content bytes for commit operations

**Key API**:
- `SyncContext::new(remote_url, workspace_prefix)` — creates temp repo, configures remote, no network call
- `SyncContext::fetch()` — fetches from remote, resolves HEAD (tries main, master, then any branch). Returns `Option<Oid>` (None if empty remote)
- `SyncContext::diff_since(since_sha, path_filter)` — diffs between prior commit and fetched HEAD. Supports optional path filter for workspace-scoped diffs. First sync (since=None) returns all files as Added.
- `SyncContext::read_blob(commit_oid, path)` — reads file content from a specific commit
- `SyncContext::commit_update(files, removals, message)` — creates a commit with file additions/modifications and removals. **Enforces workspace prefix** — rejects any paths not under `workspace_prefix/`. Preserves all other workspace files in the tree.
- `SyncContext::push()` — pushes local HEAD to remote's default branch

**Auth chain**: SSH agent → SSH key files (ed25519, rsa, ecdsa) → credential helper → default credentials. Prevents infinite loops (max 10 attempts). Clear error messages distinguish auth failures from network failures.

**Safety**:
- All paths validated against workspace prefix before commit (prevents cross-workspace writes)
- RAII cleanup via TempDir — temp dir removed on drop even if errors occur
- No persistent .git directory in .metis/

**Tests**: 36 unit tests covering:
- Lifecycle: create with file/SSH/HTTPS URLs, empty URL rejection, drop cleanup, concurrent contexts
- Fetch: populated remote, empty remote, updates after remote changes, unreachable remote
- Diff: no changes, additions, modifications, deletions, mixed operations, first sync, invalid prior commit, path filter, fetch-required check
- Read blob: success and nonexistent path
- Commit: empty remote (initial), with parent, preserves other workspaces, removals, rejects outside-workspace paths/removals
- Push: first commit, update existing, non-fast-forward rejection
- Integration: full cycle (init→fetch→commit→push), two independent workspaces, no persistent git dir, binary-safe content, unicode, large payload (100 files)

**Results**: All 36 tests pass. Workspace compiles cleanly. Zero regressions.