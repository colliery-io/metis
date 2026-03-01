---
id: strategies-gate-test-coverage-gaps
level: task
title: "Strategies-gate test coverage gaps and consistency fixes"
short_code: "METIS-T-0090"
created_at: 2026-03-01T00:03:15.365033+00:00
updated_at: 2026-03-01T00:03:15.365033+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Strategies-gate test coverage gaps and consistency fixes

## Context

Commit `d5249b9` hard-gated `strategies_enabled = true` behind multi-workspace sync configuration across all entry points (CLI init, CLI config set, MCP recovery, GUI). A post-implementation review identified test coverage gaps and consistency issues that should be addressed.

## Objective

Close test coverage gaps in the strategies-gate feature and fix minor consistency issues in the implementation.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [ ] P1 - High
- [x] P2 - Medium (nice to have)

### Technical Debt Impact
- **Current Problems**: Several edge cases and integration paths lack test coverage; error messages vary across entry points; one minor defense-in-depth gap in initialization service
- **Benefits of Fixing**: Higher confidence in gate correctness, consistent UX, prevents regressions
- **Risk Assessment**: Low risk of current bugs, but untested paths could regress silently

## Acceptance Criteria

- [ ] All items below addressed or explicitly deferred with rationale
- [ ] `angreal test` passes

## 1. Correctness Fixes

### 1a. Remove redundant gate in `init.rs:execute()` (lines 68-74)
The gate at line 68 is unreachable — if `self.upstream.is_some()`, execution returns early via `execute_with_upstream()` at line 43. The real gate lives in `create_workspace()` at line 192. Remove the dead code.

### 1b. Add `enforce_strategies_require_sync()` in initialization service
**File:** `crates/metis-docs-core/src/application/services/workspace/initialization.rs` (~line 79)

The core initialization service creates a `ConfigFile` from whatever flight levels are in the database without validation. Add:
```rust
let mut config_file = ConfigFile::new(project_prefix, flight_levels)...;
config_file.enforce_strategies_require_sync();
```
Low risk since new workspaces default to streamlined, but completes the defense-in-depth pattern.

### 1c. Unify error messages across entry points
Current messages differ between CLI init (3 lines with streamlined fallback mention), CLI config set (2 lines, no fallback), MCP (different phrasing), and GUI (yet another variant). Extract a shared constant or at minimum align the wording.

## 2. Test Coverage Gaps

### 2a. Auto-downgrade integration test (HIGH priority)
**Missing:** No test that writes `strategies_enabled = true` without sync to config.toml, runs recovery, then verifies:
1. config.toml on disk now reads `strategies_enabled = false`
2. The next MCP/CLI operation sees streamlined config
3. Strategy creation fails with "disabled in streamlined mode"

**File:** `crates/metis-docs-core/tests/configuration_recovery_test.rs`

### 2b. Positive test for `config set --preset full` WITH sync
**Missing:** Only failure cases are tested. No test proves the success path works — that `config set --preset full` actually enables strategies when sync is already configured.

**File:** `crates/metis-docs-cli/src/commands/config.rs` (tests module)

### 2c. Workspace-without-sync and sync-without-workspace edge cases
**Missing:** `is_multi_workspace()` requires both `[workspace]` and `[sync]`, but only the "neither present" case is exercised. Add unit tests for:
- workspace set, sync missing → gate triggers
- sync set, workspace missing → gate triggers

**File:** `crates/metis-docs-core/src/domain/configuration.rs` (tests module)

### 2d. CLI init gate in integration tests
**Missing:** The comprehensive functional test only covers the success path (`--preset full --upstream`). Add a test that `--preset full` without `--upstream` fails with the expected error.

**File:** `crates/metis-docs-cli/tests/comprehensive_functional_test.rs`

### 2e. Config round-trip: full → streamlined → full
**Missing:** Only full → streamlined is tested. Add a test that switches back to full (with sync present) and verifies strategies work again and old strategy documents are still accessible.

**File:** `crates/metis-docs-mcp/tests/configuration_scenarios_test.rs`

## 3. Deferred (not in scope)

- **GUI-specific gate test**: Requires Tauri test harness, not worth the setup cost for a single gate check
- **Gate in `ConfigFile::new()` constructor**: Would be a stronger invariant but breaks existing test patterns that construct configs for validation testing. Consider in a future refactor.

## Implementation Notes

### Dependencies
- Depends on commit `d5249b9` (strategies-gate implementation)

### Risk Considerations
- Test-only changes (2a-2e) are zero-risk to production code
- Correctness fixes (1a-1c) are low-risk refactors — run `angreal test` after each

## Status Updates

### 2026-03-01: Correctness fixes completed
- [x] 1a. Removed redundant gate in `init.rs:execute()` — unreachable code since upstream check returns early
- [x] 1b. Added `enforce_strategies_require_sync()` in `initialization.rs` for defense-in-depth
- [x] 1c. Unified error messages: all now use 3-line format with streamlined fallback suggestion; fixed MCP `create_document.rs` bug where initiative remediation incorrectly suggested sync config instead of preset change
- All tests pass

### 2026-03-01: Test coverage gaps closed
- [x] 2a. Added `test_auto_downgrade_persists_to_disk_and_database` — writes full config without sync, runs sync, verifies config.toml and DB both show streamlined
- [x] 2b. Added `test_config_set_preset_full_with_sync_succeeds` — positive path: manually adds sync to config.toml, then `config set --preset full` succeeds
- [x] 2c. Added 4 unit tests for workspace/sync edge cases: `test_validate_workspace_without_sync_fails`, `test_validate_sync_without_workspace_fails`, `test_enforce_workspace_without_sync_downgrades`, `test_enforce_sync_without_workspace_downgrades`
- [x] 2d. Added `test_init_full_without_upstream_fails` — CLI integration test for gate
- [x] 2e. Added `test_configuration_round_trip_full_streamlined_full` — creates strategy in full, switches to streamlined (strategy creation blocked), switches back to full (old strategy readable, new strategies creatable)
- All tests pass (241 core + 74 CLI + 25 MCP)