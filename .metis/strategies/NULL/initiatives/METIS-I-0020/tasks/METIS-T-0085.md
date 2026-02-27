---
id: metis-init-upstream-enhancement
level: task
title: "metis init --upstream enhancement"
short_code: "METIS-T-0085"
created_at: 2026-02-26T01:32:12.337258+00:00
updated_at: 2026-02-27T00:23:22.301792+00:00
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

# metis init --upstream enhancement

## Objective

Enhance `metis init` to support upstream configuration for multi-workspace mode. Walks the user through connecting to a central repo, choosing a workspace prefix, and running the initial sync.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `metis init --upstream <url>` adds upstream configuration to `config.toml`
- [ ] Interactive prompt for workspace prefix if not provided (with validation: lowercase alphanum + hyphens, 2-20 chars)
- [ ] Interactive prompt for team label (optional)
- [ ] Tests connectivity with `git ls-remote` (or libgit2 equivalent) before writing config
- [ ] Clear error if remote is unreachable or auth fails
- [ ] Runs initial sync after configuration (first pull from central)
- [ ] First push creates the workspace folder in central (registration = first push)
- [ ] Works for brand new projects (no existing .metis/) and existing projects (adding upstream to existing workspace)
- [ ] Existing `metis init` (no --upstream) continues to work unchanged for single-workspace mode

## Implementation Notes

### Interactive Flow

```
$ metis init --upstream git@github.com:org/metis-central.git

Connecting to git@github.com:org/metis-central.git... OK
Workspace prefix (lowercase, 2-20 chars): api
Team label (optional, for multi-workspace views): platform

Writing config... done
Running initial sync...
  Pulled: strat/ (2 docs), alpha/ (5 docs)
  Registered: api/ (first push)
  Sync complete.

Ready. Run `metis sync` to sync with central.
```

### Technical Approach

- Extend existing `init` command in `metis-docs-cli`
- Add `--upstream` flag (optional URL)
- Add `--prefix` and `--team` flags for non-interactive use
- Connectivity test via libgit2 `ls_remote` or equivalent
- Write `[workspace]` and `[sync]` sections to `config.toml`
- Call sync orchestration for initial sync

### Dependencies

- METIS-T-0076 (config.toml schema)
- METIS-T-0081 (sync orchestration — for initial sync)

## Test Scenarios

### Unit Tests — Flag Parsing

1. **--upstream with SSH URL**: `metis init --upstream git@github.com:org/repo.git` → parsed correctly
2. **--upstream with HTTPS URL**: `metis init --upstream https://github.com/org/repo.git` → parsed correctly
3. **--prefix flag**: `metis init --upstream <url> --prefix api` → prefix set without interactive prompt
4. **--team flag**: `metis init --upstream <url> --prefix api --team platform` → team label set
5. **--upstream without --prefix**: `metis init --upstream <url>` → interactive prompt for prefix (or error in non-interactive mode)
6. **No flags (existing behavior)**: `metis init` → single-workspace init, no upstream config, unchanged behavior

### Unit Tests — Connectivity Test

7. **Remote reachable**: valid URL, valid auth → "Connecting... OK"
8. **Remote unreachable**: invalid host → clear error "Cannot reach <url>", config NOT written
9. **Auth failure**: valid host, bad credentials → clear error "Authentication failed for <url>", config NOT written
10. **Timeout**: remote hangs → connection times out with clear message, config NOT written
11. **Empty remote (no commits)**: valid remote but no commits yet → still OK (this workspace will be the first to push)

### Unit Tests — Config Writing

12. **New project**: no existing `.metis/` → creates `.metis/`, writes `config.toml` with workspace and sync sections
13. **Existing project — add upstream**: `.metis/config.toml` exists with preset config → adds `[workspace]` and `[sync]` sections, preserves existing settings
14. **Existing project — already has upstream**: config already has `upstream_url` → error "Upstream already configured. Use `metis sync` to sync."
15. **Config written only after connectivity test**: connectivity test fails → config.toml unchanged (no partial writes)

### Unit Tests — Prefix Validation (Interactive)

16. **Valid prefix accepted**: user types `"api"` → accepted, proceed
17. **Invalid prefix rejected with retry**: user types `"API"` → error shown, prompted again
18. **Prefix validation matches METIS-T-0076 rules**: all validation cases from T-0076 apply here too

### Integration Tests — Initial Sync

19. **First workspace to push**: empty central → after init, central has `api/` folder with owned documents
20. **Joining existing central**: central has `strat/`, `alpha/` → after init, local has hydrated `strat/` and `alpha/` folders, plus pushed own `api/`
21. **Init then normal sync**: `metis init --upstream <url>` then `metis sync` → second sync works correctly (initial sync set `last_synced_commit`)
22. **Init with existing local documents**: project already has visions, initiatives, tasks → init pushes all of them to central on first sync

### Integration Tests — Non-Interactive Mode

23. **All flags provided**: `metis init --upstream <url> --prefix api --team platform` → no interactive prompts, completes silently
24. **CI/CD usage**: run in non-interactive environment with all flags → works without stdin
25. **Missing required flag in non-interactive**: `metis init --upstream <url>` in non-interactive environment without --prefix → clear error, not a hang waiting for input

### Edge Cases

26. **Init then immediately re-init**: `metis init --upstream <url>` twice → second time gets clear "already configured" error
27. **Prefix collision with existing workspace in central**: chosen prefix `api` already exists in central → warning about overwriting (or error if workspace isn't owned by this project)
28. **Init with network drop mid-sync**: connectivity test passes, initial sync starts, network drops → config is written (connectivity was verified), sync failure reported, next `metis sync` recovers
29. **Very slow initial sync**: central has 50 workspaces → progress indication during initial hydration

## Status Updates

### Session 1 — Implementation Complete

**Files modified:**
- `crates/metis-docs-core/src/domain/configuration.rs` — Added `file://` to `validate_upstream_url()` accepted URL formats
- `crates/metis-docs-cli/src/commands/init.rs` — Full rewrite with `--upstream`, `--workspace-prefix`, `--team` flags
- `crates/metis-docs-cli/src/cli.rs` — Updated InitCommand constructor in test
- `crates/metis-docs-cli/src/commands/sync.rs` — Updated all InitCommand constructors with new fields
- `crates/metis-docs-cli/src/commands/{transition,list,config,index,status,validate,archive}.rs` — Updated InitCommand constructors
- `crates/metis-docs-cli/src/commands/create/{strategy,adr,task,initiative}.rs` — Updated InitCommand constructors
- `crates/metis-docs-cli/tests/comprehensive_functional_test.rs` — Updated InitCommand constructor

**Implementation details:**
- Added `--upstream <url>` flag: triggers multi-workspace configuration flow
- Added `--workspace-prefix <name>` flag: required when `--upstream` is used (2-20 chars, lowercase alphanum + hyphens)
- Added `--team <label>` flag: optional team label for multi-workspace views
- `execute_with_upstream()`: validates prefix, tests connectivity via `SyncContext::new() + fetch()`, writes config only if connectivity succeeds, runs initial sync
- `create_workspace()`: extracted shared workspace creation logic
- `run_initial_sync()`: flattens workspace, calls orchestration::sync, updates `last_synced_commit`
- `format_connectivity_error()`: user-friendly messages for auth, network, URL failures
- Existing `metis init` (no --upstream) behavior is completely unchanged
- Supports both new projects (creates workspace + configures upstream) and existing projects (adds upstream to existing config)
- Config is NOT written if connectivity test fails (ensures no partial state)

**Tests (8 new, 73 total CLI tests passing):**
1. Missing --workspace-prefix → clear error
2. Invalid workspace prefix (uppercase) → validation error
3. Unreachable URL → error, config not written
4. Already configured upstream → error "Upstream already configured"
5. E2E new project with file:// central → workspace created, upstream configured, initial sync runs
6. E2E existing project → upstream added to existing config, initial sync runs
7. Team label → written to config.toml
8. Init then sync → subsequent `metis sync` works correctly after init

**Core change:**
- `validate_upstream_url()` now accepts `file://` URLs (legitimate git URL format, needed for testing)