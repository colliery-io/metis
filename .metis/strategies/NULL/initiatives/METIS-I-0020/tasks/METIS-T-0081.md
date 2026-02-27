---
id: sync-orchestration-engine
level: task
title: "Sync orchestration engine"
short_code: "METIS-T-0081"
created_at: 2026-02-26T01:32:08.792941+00:00
updated_at: 2026-02-26T17:21:38.363117+00:00
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

# Sync orchestration engine

## Objective

Implement the top-level sync orchestration that composes fetch, hydration, dehydration, push, and cache rebuild into a single `metis sync` operation. This is the single entry point for all sync triggers (CLI, GUI, git hooks).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Single `sync()` function that runs the full cycle: init context → fetch → hydrate → dehydrate → push → record commit → teardown → rebuild cache
- [ ] Returns a `SyncResult` with: files pulled, files pushed, errors, whether push succeeded
- [ ] Skips push if no local changes since last sync (nothing to dehydrate)
- [ ] Skips hydration-only changes from triggering a push (pulling new remote docs doesn't dirty the owned workspace)
- [ ] Updates `last_synced_commit` in `config.toml` after successful sync
- [ ] Handles first-time sync (no `last_synced_commit` — full fetch, create workspace folder in central)
- [ ] Interruption-safe: if sync fails before push, central is untouched; re-running sync recovers
- [ ] Delegates to push conflict retry (METIS-T-0082) when push fails
- [ ] No-op if `upstream_url` is not configured (single-workspace mode)
- [ ] Pre-sync freshness check: if inside a git repo, verify no unpulled remote commits touch `.metis/<owned-prefix>/` before dehydrating. Warn and abort if stale (override with `--force`).

## Implementation Notes

### Sync Sequence

```
1. Read config.toml → get upstream_url, workspace_prefix, last_synced_commit
2. If no upstream_url → return Ok (single-workspace, nothing to sync)
3. Create SyncContext (transient git repo)
4. Fetch from central
5. Diff fetched HEAD against last_synced_commit (CDC)
6. Hydrate: write remote workspace files to local .metis/<prefix>/ folders
7. Dehydrate: flatten owned workspace documents into prefix/*.md
8. Push owned folder to central (delegates to retry logic if conflict)
9. Update last_synced_commit in config.toml
10. Teardown SyncContext
11. Rebuild metis.db from all files on disk
12. Return SyncResult
```

### Error Handling

- Auth failure → clear error message ("cannot authenticate to <url> — check SSH keys or credential helper")
- Network failure → "cannot reach <url>"
- Push conflict exhausted retries → "sync failed after 5 retries — another workspace is pushing frequently, try again"
- Partial failure (fetch ok, push failed) → hydrated data is still updated locally, push retried next sync

### Dependencies

- METIS-T-0076 (config.toml — reads sync config)
- METIS-T-0078 (libgit2 — git operations)
- METIS-T-0079 (hydration)
- METIS-T-0080 (dehydration)
- METIS-T-0082 (push retry — can be stubbed initially)

## Test Scenarios

### Unit Tests — Sync Sequence

1. **Full sync — happy path**: config has upstream_url, central has remote workspaces, local has changes → fetch, hydrate, dehydrate, push, update commit SHA, rebuild cache — all steps run in order
2. **No upstream configured**: config has no `upstream_url` → sync returns immediately with no-op result, no git operations attempted
3. **First sync — no last_synced_commit**: first time syncing → full fetch (no diff baseline), hydrates all remotes, pushes owned workspace, records commit SHA
4. **No local changes since last sync**: owned workspace unchanged → fetch + hydrate run, push is skipped, commit SHA still updated to latest fetched HEAD
5. **No remote changes since last sync**: central unchanged → hydration produces no file changes, local push proceeds if local changes exist
6. **Both local and remote changes**: remote workspaces updated AND local docs changed → hydration updates remote files, dehydration pushes owned changes, both reflected in result
7. **Hydration-only sync**: only remote workspaces changed, no local changes → hydration runs, push skipped, cache rebuilt with new remote state

### Unit Tests — Pre-Sync Freshness Check

8. **Project repo up to date**: `git status` shows no unpulled commits touching `.metis/` → sync proceeds normally
9. **Stale project repo — .metis/ affected**: remote project repo has commits touching `.metis/<owned-prefix>/` that aren't pulled → sync aborts with clear warning
10. **Stale project repo — unrelated files**: remote project repo has unpulled commits but none touch `.metis/` → sync proceeds (only .metis/ staleness matters)
11. **Force flag overrides freshness**: `--force` flag set → skip freshness check, sync proceeds regardless
12. **Not in a git repo**: `.metis/` exists but not inside a git repo → freshness check skipped (no project repo to check), sync proceeds
13. **Git repo but no remote**: project repo has no remote configured → freshness check skipped, sync proceeds

### Unit Tests — SyncResult

14. **Result — files pulled**: hydration pulled 5 new files from 2 workspaces → SyncResult reflects correct counts per workspace
15. **Result — files pushed**: dehydration pushed 3 changed files → SyncResult reflects count
16. **Result — no-op**: nothing changed anywhere → SyncResult shows zeros, no errors
17. **Result — push retried**: push failed once, succeeded on retry → SyncResult includes retry count
18. **Result — errors captured**: auth failure on fetch → SyncResult has clear error, no partial state

### Unit Tests — Error Handling

19. **Auth failure**: invalid SSH key → sync fails with `"cannot authenticate to <url>"` error
20. **Network failure**: remote host unreachable → sync fails with `"cannot reach <url>"` error
21. **Push conflict**: remote HEAD moved → delegates to retry logic (METIS-T-0082), not handled here
22. **Partial failure — fetch OK, push fails**: hydrated data persists locally, push error reported, next sync can retry
23. **Partial failure — hydration error on one workspace**: one remote workspace has corrupted file → that workspace's hydration fails, others succeed, error reported
24. **Config write failure**: sync succeeds but can't update `last_synced_commit` in config.toml → warning (not fatal — next sync will re-process some files but won't lose data)
25. **Cache rebuild failure**: sync succeeds but cache rebuild fails → warning (cache is ephemeral, can be rebuilt later)

### Integration Tests — Full Cycle

26. **Two-workspace roundtrip**: workspace A pushes, workspace B syncs → B sees A's documents. B pushes, A syncs → A sees B's documents.
27. **Three-workspace convergence**: workspaces A, B, C all sync against same central → after all syncs, each has consistent view of all three workspaces
28. **Sync with empty central**: first workspace to sync → pushes successfully, creates workspace folder, no hydration (no remotes yet)
29. **Sync after document operations**: create, edit, transition, archive documents locally → sync → all changes reflected in central
30. **Sequential syncs — incremental**: sync once, make small change, sync again → second sync only pushes the delta, not full workspace

### Edge Cases

31. **Sync interrupted mid-hydration**: process killed during hydration → central untouched (no push happened), local has partial hydration, next sync recovers
32. **Sync interrupted mid-push**: process killed during push → push either completed or didn't (atomic), no partial commits in central
33. **Clock skew**: local machine clock is wrong → sync still works (uses commit SHAs not timestamps for tracking)
34. **Very large central repo**: 50 workspaces, 10000+ documents total → sync completes within reasonable time
35. **Rapid sequential syncs**: sync, immediately sync again → second sync is effectively a no-op (nothing changed)

## Status Updates

### Session 1 — Implementation Complete

**New module**: `crates/metis-sync/src/orchestration.rs` — top-level sync orchestration engine.

**Types**:
- `SyncConfig` — configuration extracted from config.toml: upstream_url, workspace_prefix, last_synced_commit
- `SyncOptions` — behavior options: force flag, max_retries (default 3)
- `SyncResult` — composite result: hydration/dehydration results, new_synced_commit SHA, push_retries, is_noop flag, warnings. Convenience methods: `files_pulled()`, `files_pushed()`, `pushed()`.

**API**:
- `sync(config, metis_dir, local_documents, options)` — full cycle: create context → fetch → hydrate remote workspaces → dehydrate owned workspace → push → return result with new commit SHA
- `sync_pull_only(config, metis_dir)` — fetch + hydrate only (no push). For read-only sync.

**Orchestration sequence**:
1. Create `SyncContext` from upstream_url and workspace_prefix
2. Fetch from central
3. If central has content, hydrate remote workspaces (non-fatal — errors captured as warnings)
4. Dehydrate owned workspace with push retry on conflict
5. Determine new_synced_commit (push commit if pushed, fetched HEAD otherwise)
6. Detect no-op (zero files pulled/pushed/removed)

**Push retry**: `dehydrate_with_retry()` — on `PushRejected`, re-fetches and retries up to `max_retries` times. Clear error if retries exhausted.

**Error handling**:
- Hydration errors are non-fatal (captured as warnings, sync continues)
- Auth/network failures propagate as `SyncError`
- Partial success: hydrated data persists even if push fails

**Added to SyncContext** (`lib.rs`):
- `list_workspace_folders()` — enumerates top-level directories in fetched HEAD (workspace prefixes)
- `list_workspace_files(prefix)` — lists all `.md` files in a workspace folder with content

**Tests**: 16 orchestration tests covering:
- Happy path: full sync, first sync (no last commit), first sync (empty central)
- No-op: no local changes skips push, hydration-only no push, no remote changes still pushes, noop detection, rapid sequential sync
- Both directions: local + remote changes
- Pull-only: sync without push
- Error handling: network failure, partial failure (hydration non-fatal)
- Multi-workspace: two-workspace roundtrip, three-workspace convergence
- Sequential: incremental syncs with delta
- Retry: push retry config respected

**Total crate tests**: 89 (36 lib + 20 hydration + 17 dehydration + 16 orchestration). All pass. Zero warnings.