---
id: 1-0-release-readiness-code-quality
level: task
title: "1.0 Release Readiness - Code Quality Fixes"
short_code: "METIS-T-0020"
created_at: 2025-11-25T11:57:59.361676+00:00
updated_at: 2025-11-25T12:23:41.482571+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# 1.0 Release Readiness - Code Quality Fixes

Address code quality issues identified in comprehensive crate review before 1.0 release.

## Objective

Fix all critical and high-priority code quality issues across all four crates (core, mcp, cli, gui) to ensure production readiness for 1.0 release.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

## Review Summary

### Test Coverage
| Crate | Unit Tests | Integration Tests | Status |
|-------|------------|-------------------|--------|
| Core | 132 pass, 1 fail | 19 pass | Fix 1 test |
| MCP | 5 pass | 14 pass | Good |
| CLI | Comprehensive | Full workflow | Good |
| GUI | Minimal | None | Could improve |

### What's Working Well
- Clean layered architecture in core (application/domain/dal/error)
- Comprehensive error handling with user-friendly messages
- Complete MCP tool coverage for Flight Levels workflow
- Excellent MCP documentation in instructions.md
- CLI covers all operations (create, list, search, transition, archive, sync, validate)
- GUI has multi-board Kanban with drag-drop, theming, and CLI auto-install

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Critical (Must Fix)
- [x] Remove emojis from CLI output (sync.rs, status.rs, init.rs) - replace with ASCII indicators
- [x] Fix MCP version string in lib.rs:167 (hardcoded "0.4.2" -> env!("CARGO_PKG_VERSION"))
- [x] Replace alert() with toast in GUI (KanbanBoard.vue:358, ProjectSidebar.vue:172)
- [x] Fix failing test_sync_directory test in synchronization.rs:1077

### High Priority (Should Fix)
- [x] Fix config.rs:156-160 - return Err() instead of exit(1)
- [x] Remove or implement unused document_type param in transition.rs:19
- [x] Remove unused deleteDocument API in tauri-api.ts:256-283
- [x] Remove debug println from transition.rs:368 (replaced emoji with [+])
- [x] Apply clippy fixes in core (initialization.rs:80,82 - use function reference)
- [x] Complete or remove ProjectBrowser placeholder in App.vue:115-117
- [x] Fix silent error catches in KanbanBoard.vue:218-220 - show toast on error

### Medium Priority (Nice to Have)
- [ ] Consolidate truncate_string() to utils (duplicated in list.rs, status.rs, search.rs)
- [ ] Add module-level docs to core (application/mod.rs, domain/mod.rs, dal/mod.rs)
- [ ] Replace TypeScript `any` types in GUI components (8 instances)
- [ ] Remove console.log statements from GUI (4 instances)
- [ ] Remove main.rs from metis-docs-core (library-only crate)

## Implementation Notes

### Critical Issues Detail

#### 1. CLI Emoji Usage
**Files**: 
- `crates/metis-docs-cli/src/commands/sync.rs:43-93`
- `crates/metis-docs-cli/src/commands/status.rs:244-251`
- `crates/metis-docs-cli/src/commands/init.rs:85-88`

**Current**: Uses emojis like checkmark, x, warning, clipboard, arrows
**Fix**: Replace with `[+]`, `[-]`, `[!]`, `[*]`, `[>]`

#### 2. MCP Version Mismatch
**File**: `crates/metis-docs-mcp/src/lib.rs:167`
```rust
version: "0.4.2".to_string(),  // WRONG
```
**Fix**: `version: env!("CARGO_PKG_VERSION").to_string(),`

#### 3. GUI Alert Usage
**Files**:
- `crates/metis-docs-gui/src/components/KanbanBoard.vue:358`
- `crates/metis-docs-gui/src/components/ProjectSidebar.vue:172`

**Fix**: Use existing toast system from App.vue

#### 4. Failing Test
**File**: `crates/metis-docs-core/src/application/services/synchronization.rs:1077`
**Test**: `test_sync_directory`
**Error**: `ValidationFailed { message: "Database path not set" }`
**Fix**: Add `.with_workspace_dir(temp_dir.path())` to test setup

### High Priority Issues Detail

#### Config Exit Code
**File**: `crates/metis-docs-cli/src/commands/config.rs:156-160`
```rust
None => {
    eprintln!("Configuration key '{}' not found", key);
    std::process::exit(1);  // BAD
}
```
**Fix**: `return Err(anyhow::anyhow!("Configuration key '{}' not found", key));`

#### Unused Parameter
**File**: `crates/metis-docs-cli/src/commands/transition.rs:19`
`document_type: Option<String>` is parsed but never used - confuses users

#### Dead API Code
**File**: `crates/metis-docs-gui/src/lib/tauri-api.ts:256-283`
`deleteDocument()` defined but never called, no backend command exists

## Status Updates

2025-11-25: Task created from comprehensive 1.0 release readiness review
2025-11-25: Completed all critical and high priority fixes:
  - Replaced all emojis in CLI with ASCII indicators ([+], [-], [!], etc.)
  - Fixed MCP version to use env!("CARGO_PKG_VERSION")
  - Replaced alert() with toast notifications in GUI
  - Fixed test_sync_directory test with proper database setup
  - Fixed config.rs to return Err instead of exit(1)
  - Removed unused document_type param from transition command
  - Removed dead deleteDocument API from tauri-api.ts
  - Applied clippy fixes (function references)
  - Removed ProjectBrowser placeholder code
  - Added toast notifications for silent error catches