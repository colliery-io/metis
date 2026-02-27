---
id: hydration-central-to-local
level: task
title: "Hydration: central to local document sync"
short_code: "METIS-T-0079"
created_at: 2026-02-26T01:32:06.954531+00:00
updated_at: 2026-02-26T16:40:10.071706+00:00
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

# Hydration: central to local document sync

## Objective

Implement hydration: fetching all workspace folders from central and writing remote documents to local `.metis/<prefix>/` subfolders. After hydration, every workspace's documents are available locally as regular files on disk.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Fetch all workspace folders from central repo (full fetch — read is open)
- [ ] Write each remote workspace's documents into `.metis/<prefix>/` as flat `.md` files
- [ ] Owned workspace folder is skipped during hydration (don't overwrite local state with central's copy)
- [ ] Documents that no longer exist in central are removed from local hydrated folders
- [ ] Existing local hydrated files are overwritten with the latest from central (central is source of truth for remote workspaces)
- [ ] First sync (no prior hydrated folders) creates all remote workspace folders
- [ ] Hydration is idempotent — running twice with same central state produces same local state

## Implementation Notes

### Technical Approach

After the git fetch (METIS-T-0078), walk the fetched tree and for each workspace folder that is NOT the owned workspace:

1. List all `.md` files in the remote workspace folder
2. Write each file to `.metis/<prefix>/<SHORT-CODE>.md`
3. Remove any local `.metis/<prefix>/*.md` files that don't exist in the fetched tree (handles deletions/archives)
4. Skip non-`.md` files (config, caches, etc.)

The hydrated files are just regular files on disk. The projection cache (METIS-T-0083) will index them alongside owned documents.

### Edge Cases

- New workspace appears in central → create the local folder
- Workspace removed from central → remove the local folder and all its files
- Central is empty (first workspace to push) → no remote folders to hydrate

### Dependencies

- METIS-T-0077 (flat document layout — defines the central ↔ local mapping)
- METIS-T-0078 (libgit2 — provides the fetched git tree)

## Test Scenarios

### Unit Tests — Hydration Core

1. **Single remote workspace**: central has `strat/` with 3 docs → local `.metis/strat/` created with 3 `.md` files
2. **Multiple remote workspaces**: central has `strat/`, `alpha/`, `sre/` → local gets all three folders with correct files
3. **Owned workspace skipped**: central has `api/` (owned) and `strat/` (remote) → only `strat/` is hydrated, `api/` files untouched
4. **New file in remote**: central has a new doc in `strat/` that doesn't exist locally → file created
5. **Updated file in remote**: central has newer version of `strat/STRAT-V-0001.md` → local file overwritten with central version
6. **Deleted file in remote**: local has `strat/STRAT-T-0005.md` but central doesn't → local file removed
7. **Deleted workspace in remote**: local has `.metis/alpha/` folder but `alpha/` no longer in central → entire folder removed
8. **New workspace appears**: central has new workspace `sre/` not seen before → `.metis/sre/` folder created, files written
9. **First sync — no prior hydrated folders**: no remote folders exist locally → all remote workspace folders created from scratch
10. **Empty remote workspace**: central has `alpha/` folder but no files in it → `.metis/alpha/` created as empty folder (or not created — define behavior)

### Unit Tests — Idempotency

11. **Double hydration**: hydrate same central state twice → local state identical after both runs, no extra files, no errors
12. **Hydration after no remote changes**: central unchanged since last hydration → local files unchanged (compare timestamps or content)
13. **Hydration preserves file content exactly**: byte-for-byte comparison of central file content and hydrated local file → identical

### Unit Tests — File Filtering

14. **Non-md files in central ignored**: central has `strat/config.toml` or `strat/notes.txt` → not copied to local
15. **Hidden files in central ignored**: central has `strat/.gitkeep` → not copied to local
16. **Only .md files hydrated**: central workspace folder has `.md`, `.json`, `.toml` files → only `.md` copied

### Integration Tests — With Flatten/Unflatten

17. **End-to-end: push then hydrate**: workspace A dehydrates (pushes) → workspace B hydrates → B sees A's documents correctly
18. **Cross-workspace references resolved**: workspace B hydrates strat/ and alpha/ → documents referencing each other across workspaces have correct parent links
19. **Sequential syncs**: hydrate once, central gets new commits from multiple workspaces, hydrate again → all new and updated documents appear locally

### Edge Cases

20. **Large workspace hydration**: remote workspace with 500+ documents → completes in reasonable time (<5s)
21. **Disk full during hydration**: disk runs out mid-write → partial state is recoverable (re-hydration from scratch fixes it), no corrupted files left
22. **Permission error on write**: cannot write to `.metis/strat/` → clear error message, other workspaces still hydrated
23. **Symlink in central**: central has a symlink in a workspace folder → not followed (security), skipped with warning
24. **Concurrent hydration**: two processes try to hydrate simultaneously → no corrupted files (last writer wins is acceptable, but no partial files)
25. **File with same name as folder**: edge case in filesystem handling → appropriate error
26. **Central workspace prefix collides with reserved names**: workspace named `config` or `archived` → handled correctly (should be caught at init time, but defensive here)

### Gitignore Management

27. **New remote workspace adds gitignore entry**: hydrating `strat/` for first time → `.metis/.gitignore` updated to include `strat/`
28. **Multiple remotes in gitignore**: hydrating `strat/`, `alpha/`, `sre/` → all three listed in `.metis/.gitignore`
29. **Gitignore preserved**: existing gitignore entries for `metis.db`, `code-index-hashes.json` etc. → not removed when adding new remote entries
30. **Removed remote cleaned from gitignore**: workspace disappears from central → gitignore entry removed (or left as harmless no-op — define behavior)

## Status Updates

### Session 1 — Implementation Complete

**New module**: `crates/metis-sync/src/hydration.rs` — hydration logic for central → local sync.

**New methods on SyncContext** (in `lib.rs`):
- `list_workspace_folders()` — enumerates top-level directories in fetched HEAD tree (workspace prefixes)
- `list_workspace_files(prefix)` — lists all `.md` files in a workspace folder, returning `(filename, content_bytes)` pairs

**Hydration API** (`hydration::hydrate(ctx, metis_dir, owned_prefix)`):
- Walks all workspace folders in central
- Skips the owned workspace (no overwriting local state)
- For each remote workspace: writes all `.md` files to `.metis/<prefix>/`, removes stale `.md` files
- Removes stale workspace folders that no longer exist in central (with safety checks — only removes dirs that contain only `.md` files)
- Updates `.metis/.gitignore` with hydrated workspace entries (deduplicated, preserves existing entries)
- Returns `HydrationResult` with stats: workspaces hydrated, files written/removed, errors

**Safety features**:
- Reserved directory names (`archived`, `strategies`, `adrs`, `backlog`, `templates`, etc.) are never removed
- Hidden directories (`.foo`) are never removed
- Only `.md` files are hydrated from central
- Non-fatal errors: if one workspace fails, others still process

**Tests**: 20 hydration tests covering:
- Core: single remote, multiple remotes, owned workspace skipped, new/updated/deleted files, deleted/new workspaces, first sync, empty remote
- Idempotency: double hydration produces identical state, content preserved byte-for-byte
- File filtering: non-md files ignored
- Integration: push-then-hydrate end-to-end, sequential syncs with updates
- Edge cases: 100-file workspace, reserved directory safety
- Gitignore: entries created, existing entries preserved, no duplicates

**Results**: All 56 tests pass (36 lib + 20 hydration). Workspace compiles cleanly. Zero warnings from metis-sync.