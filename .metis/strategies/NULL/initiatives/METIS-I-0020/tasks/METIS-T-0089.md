---
id: mcp-server-sync-integration-with
level: task
title: "MCP server sync integration with time-based debounce"
short_code: "METIS-T-0089"
created_at: 2026-02-28T18:29:03.984883+00:00
updated_at: 2026-02-28T18:29:03.984883+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# MCP server sync integration with time-based debounce

## Parent Initiative

[[METIS-I-0020]]

## Objective

Add git-based multi-workspace sync to the MCP server so Claude Code agents see remote workspace changes without requiring manual `metis sync` CLI calls. Currently `prepare_workspace()` only syncs the local filesystem to SQLite — it has no awareness of the upstream git repository.

## Context

- `WorkspaceDetectionService::prepare_workspace()` in `metis-docs-core` is called by every MCP tool before operating. It runs `app.sync_directory()` which syncs `.metis/` files to the SQLite DB.
- The `metis-sync` crate provides `orchestration::sync()` (full push+pull) and `orchestration::sync_pull_only()` (fetch+hydrate only).
- The GUI already has full sync integration via `services/sync.rs` Tauri commands.
- There is no sync MCP tool and no git sync in the MCP server path.

## Design

**Three behaviors based on operation type:**

1. **Read operations** (`list_documents`, `search_documents`, `read_document`): pull-only sync — fetch remote changes but don't push local changes
2. **Write operations** (`create_document`, `edit_document`, `transition_phase`, `archive_document`, `reassign_parent`): full sync — pull remote then push local changes
3. **Neutral operations** (`initialize_project`, `index_code`): no git sync needed

**Time-based debounce:** Only trigger git sync if it's been more than N seconds (e.g., 30-60s) since the last sync. This prevents hammering the remote on rapid successive tool calls. The debounce state should be tracked per-process (the MCP server is a long-running stdio process).

**Key files to modify:**
- `crates/metis-docs-core/src/application/services/workspace/detection.rs` — extend `prepare_workspace()` or add a new method that accepts a sync mode parameter
- `crates/metis-docs-mcp/src/server.rs` — add sync state (last sync timestamp) and pass sync mode to tool calls
- Individual MCP tool files in `crates/metis-docs-mcp/src/tools/` — pass sync mode through to prepare_workspace
- `crates/metis-docs-mcp/Cargo.toml` — add `metis-sync` dependency

**Sync mode enum:**
- `SyncMode::None` — filesystem-to-DB only (current behavior)
- `SyncMode::Pull` — git pull + filesystem-to-DB
- `SyncMode::Full` — git pull + filesystem-to-DB + git push

**Error handling:** Git sync failures should be non-fatal for read operations (log warning, fall back to local state). For write operations, sync errors should be surfaced but not block the write itself — the local filesystem is always the source of truth.

## Acceptance Criteria

- [ ] Read MCP tools (list, search, read) trigger pull-only git sync when upstream is configured
- [ ] Write MCP tools (create, edit, transition, archive, reassign) trigger full git sync when upstream is configured
- [ ] Time-based debounce prevents git sync more often than every N seconds
- [ ] No git sync when upstream is not configured (current behavior preserved)
- [ ] Git sync errors are non-fatal — operations fall back to local state
- [ ] All existing tests pass
- [ ] New tests for debounce logic

## Implementation Notes

### Key Dependencies
- `metis-sync` crate (already exists) — `orchestration::sync()` and `sync_pull_only()`
- `metis-core` — `WorkspaceDetectionService`, config.toml reading
- Config reading: `metis-core` already has config.toml parsing for upstream URL / workspace prefix

### Risk Considerations
- Network latency on every MCP call if debounce is too short
- Push conflicts during write operations — `metis-sync` already handles retry logic
- MCP server is long-running — need process-level state for debounce timestamps

## Status Updates

*To be added during implementation*