---
id: flatten-filesystem-layout
level: task
title: "Flatten filesystem layout"
short_code: "METIS-T-0093"
created_at: 2026-03-03T19:10:49.173963+00:00
updated_at: 2026-03-03T21:58:47.016377+00:00
parent: METIS-I-0024
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0024
---

# Flatten filesystem layout

## Parent Initiative

[[METIS-I-0024]]

## Objective

Change the `.metis/` directory layout from the current `strategies/{id}/initiatives/` nesting to a flat `initiatives/` structure. Implement a v1→v2 filesystem migration that runs automatically on workspace open.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] New layout: `.metis/initiatives/{INITIATIVE_SHORT_CODE}/initiative.md`
- [x] Tasks remain nested under their initiative: `.metis/initiatives/{INIT}/tasks/{TASK}.md`
- [x] Backlog remains at `.metis/backlog/`
- [x] Archived remains at `.metis/archived/`
- [x] Vision remains at `.metis/vision.md`
- [x] ADRs remain at `.metis/adrs/`
- [x] Migration function: detect `strategies/` directory on workspace open, move `strategies/*/initiatives/*` up to `initiatives/`, delete `strategies/` directory, delete any `strategy.md` files
- [x] Migration is idempotent — running on an already-migrated workspace is a no-op
- [x] Migration logs what it moved for auditability

## Implementation Notes

This migration runs once when a v1 workspace is opened by v2 code. The detection is simple: if `.metis/strategies/` exists, migrate. The migration must handle:
- `strategies/NULL/initiatives/*` (streamlined/direct presets — the common case)
- `strategies/{SHORT_CODE}/initiatives/*` (full preset — rare but possible)
- `strategies/{SHORT_CODE}/strategy.md` files — delete these, the documents are gone

Coordinate with METIS-T-0092 (services) since path construction logic changes.

## Status Updates

### Session 1 — 2026-03-03

**All acceptance criteria met. All 182 tests pass (155 unit + 27 integration).**

#### Changes

**initialization.rs** — Removed `strategies/` directory creation from workspace init. Updated doc comment and test assertion.

**migration.rs** (NEW) — Created `WorkspaceMigrationService` with `migrate()` function:
- Detects v1 workspaces by checking for `strategies/` directory
- Moves `strategies/*/initiatives/*` up to `initiatives/`
- Handles NULL strategy pattern (streamlined/direct presets)
- Handles named strategy patterns (full preset)
- Deletes `strategy.md` files and strategy directories
- Skips conflicts (preserves existing v2 initiatives)
- Returns `MigrationReport` with moved/deleted items for auditability
- Idempotent — no-op on already-migrated workspaces
- 8 unit tests covering all scenarios

**detection.rs** — Wired migration into `prepare_workspace()` so it runs automatically before sync on every workspace open.

**mod.rs** — Registered migration module and exported `MigrationReport` and `WorkspaceMigrationService`.

**reassignment_test.rs** (integration test) — Rewrote all 8 tests to use v2 flat layout paths. Removed `strategy_id` from test frontmatter. Renamed `test_reassign_across_strategies` to `test_reassign_across_initiatives`.

#### V2 Layout
```
.metis/
├── metis.db
├── config.toml
├── vision.md
├── initiatives/
│   └── {SHORT_CODE}/
│       ├── initiative.md
│       └── tasks/
│           └── {SHORT_CODE}.md
├── backlog/
├── adrs/
└── archived/
```

#### Verification
- `cargo check -p metis-docs-core` — compiles cleanly
- `angreal test-core` — 182 tests pass (155 unit + 27 integration)