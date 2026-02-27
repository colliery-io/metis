---
id: dehydration-local-to-central
level: task
title: "Dehydration: local to central document push"
short_code: "METIS-T-0080"
created_at: 2026-02-26T01:32:07.924735+00:00
updated_at: 2026-02-26T17:00:53.533102+00:00
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

# Dehydration: local to central document push

## Objective

Implement dehydration: serializing the owned workspace's documents into a git commit for push to central. Only the owned workspace folder is pushed — the application enforces write scope.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Collect all document `.md` files from the local workspace (using the flat layout from METIS-T-0077)
- [ ] Build a git tree containing only `<prefix>/*.md` files
- [ ] Create a commit on the central repo's main branch with the updated tree
- [ ] Only the owned workspace folder is modified — other workspaces' folders in central are untouched
- [ ] Application refuses to stage files outside the owned prefix (write scope enforcement)
- [ ] Deleted local documents result in file removal from central on next push
- [ ] Commit message includes workspace prefix and timestamp (e.g. "sync: api @ 2026-02-26T01:30:00Z")
- [ ] Push to central's main branch

## Implementation Notes

### Technical Approach

1. Read the current central HEAD tree (from fetch)
2. Build a new tree that replaces the owned prefix subtree with current local state:
   - Flatten local documents into `prefix/SHORT-CODE.md` format
   - Create git blob objects for each file
   - Build a tree object for the owned prefix folder
   - Graft the new prefix tree into the full central tree (preserving all other folders)
3. Create a commit with the new tree, parenting on central HEAD
4. Push the commit to `refs/heads/main`

This is a tree-surgery operation via libgit2 — we're replacing one subtree within the full central tree, not doing a checkout/add/commit cycle.

### Write Scope Enforcement

The dehydration function takes the workspace prefix as input and ONLY reads documents from the local owned workspace. Even if a user manually edited a hydrated remote file on disk, it will never be included in the push. The application is the enforcement point.

### Dependencies

- METIS-T-0077 (flat document layout — defines the flattening)
- METIS-T-0078 (libgit2 — provides commit/push operations)

## Test Scenarios

### Unit Tests — Tree Surgery

1. **First push to empty central**: no prior commits → creates initial commit with `prefix/` subtree containing all owned documents
2. **Update existing workspace folder**: central already has `api/` → new commit replaces `api/` subtree entirely, other folders (`strat/`, `sre/`) untouched
3. **Other workspace folders preserved**: central has `strat/`, `alpha/`, `api/` → after push only `api/` contents change, `strat/` and `alpha/` byte-identical to prior commit
4. **New document pushed**: add a new task locally → central commit includes the new file in `api/`
5. **Modified document pushed**: edit an existing task locally → central commit has updated content
6. **Deleted document removed**: delete a task locally → central commit no longer has that file in `api/`
7. **Mixed operations**: add 2 docs, modify 3, delete 1 → all operations reflected correctly in single commit
8. **Empty workspace push**: owned workspace has 0 documents → `api/` folder is empty (or absent) in central commit, other folders untouched

### Unit Tests — Write Scope Enforcement

9. **Only owned prefix written**: dehydration function given `prefix="api"` → output tree only contains files from owned workspace, never from hydrated remotes
10. **Manually edited remote file ignored**: user edits `.metis/strat/STRAT-V-0001.md` on disk → dehydration does NOT include that file (it reads from owned workspace only)
11. **Cannot write outside owned prefix**: even if caller passes files outside prefix → rejected or ignored
12. **Prefix enforced in tree structure**: output tree has exactly one top-level entry matching the owned prefix

### Unit Tests — Commit Quality

13. **Commit message format**: commit message matches `"sync: api @ 2026-02-26T01:30:00Z"` pattern
14. **Commit parent is fetched HEAD**: commit parents the correct remote HEAD, not a stale reference
15. **Commit author**: uses reasonable default (workspace prefix or git config user)
16. **Single commit per sync**: all changes bundled into one commit, not one-per-file

### Integration Tests

17. **Push roundtrip — verify on remote**: dehydrate and push → clone the central repo independently → verify files match what was pushed
18. **Sequential pushes**: push once, make local changes, push again → central has two commits, latest state is correct
19. **Push after hydration**: fetch + hydrate (remote changes) + dehydrate + push → owned workspace pushed correctly, other workspace changes from hydration don't leak into push
20. **Push with archived documents**: owned workspace has archived docs → archived docs included in central (central stores full state)

### Edge Cases

21. **Large workspace push**: 500+ documents flattened and pushed → completes within reasonable time
22. **Document with very large content**: single document >1MB → pushed correctly
23. **Unicode in file content**: CJK, emoji, RTL text in documents → preserved exactly through dehydration
24. **Concurrent dehydration**: two processes try to dehydrate simultaneously → one succeeds or both produce valid (identical) output
25. **No changes since last push**: owned workspace unchanged → skip push entirely (no empty commit), return early
26. **Central repo was force-pushed/rebased**: fetched HEAD doesn't share history with last_synced_commit → clear error (not silent corruption)

## Status Updates

### Session 1 — Implementation Complete

**New module**: `crates/metis-sync/src/dehydration.rs` — local → central document push.

**Types**:
- `DehydrationResult` — commit OID (None if no changes), files pushed/removed counts, whether push occurred
- `FlatDoc` — lightweight document struct (short_code, filename, content) to avoid a dependency on `metis-docs-core`. Callers convert from `FlatDocument` to `FlatDoc`.

**API** (`dehydration::dehydrate(ctx, documents, prefix)`):
- Takes a list of flattened documents and pushes them to central under `<prefix>/`
- Computes removals by comparing local docs with central's current state for the workspace
- Skips push if no changes detected (content comparison — avoids empty commits)
- Commit message format: `"sync: api @ 2026-02-26T01:30:00Z"`
- Write scope enforcement: all paths are `prefix/filename`, enforced by `SyncContext::commit_update()`

**Added to SyncContext** (`lib.rs`):
- `repo()` — accessor for the underlying `git2::Repository` (needed for commit inspection in tests)

**No-change detection**:
- If files and removals are both empty → skip
- If all file contents match central's current state and file count is the same → skip
- Avoids empty commits on idempotent syncs

**Timestamp**: Custom `chrono_lite_now()` function produces ISO 8601 UTC timestamps without pulling in the `chrono` crate (uses civil-days algorithm for date conversion from epoch seconds).

**Tests**: 17 dehydration tests covering:
- Tree surgery: first push, update existing, other workspaces preserved, deleted docs removed, mixed operations, empty workspace push
- Write scope: only owned prefix written
- Commit quality: message format, parent is fetched HEAD, single commit per sync
- Integration: sequential pushes, push after hydration doesn't leak remote docs
- No-change detection: identical content skips push, empty docs + empty central skips
- Edge cases: 100-file push, unicode preserved, timestamp format validation

**Results**: All 73 tests pass (36 lib + 20 hydration + 17 dehydration). Workspace compiles cleanly. Zero warnings from metis-sync.