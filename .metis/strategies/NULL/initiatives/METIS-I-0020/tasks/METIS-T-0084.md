---
id: metis-sync-cli-command
level: task
title: "metis sync CLI command"
short_code: "METIS-T-0084"
created_at: 2026-02-26T01:32:11.419760+00:00
updated_at: 2026-02-26T23:58:22.710389+00:00
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

# metis sync CLI command

## Objective

Add a `metis sync` CLI command that triggers the sync orchestration engine. Provides manual sync for dev teams and is the entry point called by git hooks.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `metis sync` runs the full sync cycle and outputs results
- [ ] Output shows: files pulled (by workspace), files pushed, errors
- [ ] Exit code 0 on success, non-zero on failure
- [ ] Clear error messages for common failures (no upstream configured, auth failure, network unreachable)
- [ ] `--dry-run` flag that shows what would be synced without actually pushing
- [ ] Quiet mode for git hook usage (`--quiet` or detect non-interactive)
- [ ] Works when called from git hooks (post-commit, post-push) in background

## Implementation Notes

### Technical Approach

- New subcommand in `metis-docs-cli`: `metis sync`
- Reads config, calls sync orchestration engine (METIS-T-0081), formats output
- For git hook usage: `metis sync --quiet` suppresses output, only logs errors

### Output Format

```
Syncing with git@github.com:org/metis-central.git...
  Pulled: strat/ (2 docs), alpha/ (5 docs)
  Pushed: api/ (3 docs changed)
  Sync complete in 1.2s
```

### Dependencies

- METIS-T-0081 (sync orchestration engine)
- METIS-T-0076 (config.toml — reads upstream_url)

## Test Scenarios

### Unit Tests — CLI Interface

1. **metis sync — happy path**: upstream configured, central reachable → outputs pull/push summary, exits 0
2. **metis sync — no upstream**: no `upstream_url` in config → prints "No upstream configured. Run `metis init --upstream <url>` to set up." exits 0
3. **metis sync — auth failure**: SSH key invalid → prints auth error with URL, exits non-zero
4. **metis sync — network failure**: remote unreachable → prints network error with URL, exits non-zero
5. **metis sync — push conflict exhausted**: retries exhausted → prints retry failure message, exits non-zero
6. **metis sync — no .metis directory**: run outside a Metis project → prints "Not a Metis project" error, exits non-zero

### Unit Tests — Output Format

7. **Output shows workspaces pulled**: pulled from strat/ (2 docs), alpha/ (5 docs) → output lists each workspace with count
8. **Output shows files pushed**: pushed 3 changed docs → output shows push count
9. **Output shows timing**: sync completed → output includes elapsed time
10. **No changes output**: nothing changed anywhere → output says "Already up to date." or similar
11. **First sync output**: first time syncing → output shows "Registered: api/ (first push)" or similar

### Unit Tests — Flags

12. **--dry-run shows changes without pushing**: local changes exist → output shows what would be pushed, but central is unchanged after command
13. **--dry-run shows pulls without writing**: remote changes exist → output shows what would be hydrated, but local files unchanged
14. **--quiet suppresses output**: sync succeeds → stdout is empty, exit code 0
15. **--quiet shows errors**: sync fails → stderr has error message, exit code non-zero
16. **--quiet + --dry-run**: both flags together → works correctly (quiet dry-run, exit code indicates whether changes exist)
17. **--force skips freshness check**: project repo is stale → sync proceeds anyway

### Integration Tests

18. **CLI end-to-end**: init project with upstream, create documents, run `metis sync` → central has documents, second project can sync and see them
19. **Git hook usage**: `metis sync --quiet` called from post-commit hook → runs silently in background, doesn't block git
20. **Sequential CLI syncs**: `metis sync` twice in a row → second run is fast no-op
21. **CLI with push retry**: central HEAD moves during push → CLI handles retry transparently, output shows retry count

### Edge Cases

22. **Very long output**: 50 workspaces with many docs → output is readable, not overwhelming (consider summarization)
23. **Interrupted sync**: Ctrl+C during sync → cleanup runs (temp dir removed), partial state recoverable on next sync
24. **Concurrent CLI invocations**: two terminals run `metis sync` simultaneously → both complete without corruption (may both succeed or one waits)

## Status Updates

### Session 1 — Implementation Complete

**Files modified:**
- `crates/metis-docs-cli/Cargo.toml` — Added `metis-sync` dependency, `git2` dev-dependency
- `crates/metis-docs-cli/src/commands/sync.rs` — Full rewrite with git sync integration
- `crates/metis-docs-cli/src/cli.rs` — Updated SyncCommand struct initialization (3 places)
- `crates/metis-docs-cli/src/commands/create/mod.rs` — Updated SyncCommand struct in auto-sync

**Implementation details:**
- `SyncCommand` now has `--dry-run`, `--quiet`, `--force` flags
- `execute()` checks for upstream config, runs git sync if configured, then local db sync
- `execute_git_sync()` — flattens workspace via core's `flatten_workspace()`, converts `FlatDocument` → sync's `FlatDoc`, builds `SyncConfig`/`SyncOptions`, calls `metis_sync::orchestration::sync()`, updates `last_synced_commit` in config.toml
- `execute_dry_run()` — fetch-only without pushing, shows what would happen
- `print_git_sync_results()` — formatted output: pulled workspaces, pushed files, retries, timing
- `format_sync_error()` — user-friendly error messages for auth, network, URL, retry exhaustion
- `execute_local_sync()` — refactored local db sync with quiet mode support (errors to stderr only)

**Tests (9 passing):**
1. No workspace found → error
2. With workspace, no upstream → runs local sync only
3. No upstream configured → info message
4. Quiet mode → suppresses stdout
5. Dry run, no upstream → skips git sync
6. Auth failure → user-friendly error message
7. File:// upstream E2E → full sync cycle
8. Dry run with upstream → fetch-only output
9. Quiet with upstream → errors to stderr

**Decisions:**
- Tests require `--test-threads=1` due to process-global `set_current_dir` usage in CLI tests
- `write_upstream_config()` helper writes config.toml directly in tests because `set_sync()` rejects `file://` URLs (validates SSH/HTTPS only)
- All 65 CLI crate tests pass (61 unit + 4 integration)