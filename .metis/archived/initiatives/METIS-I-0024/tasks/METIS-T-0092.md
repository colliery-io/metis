---
id: remove-strategy-from-services
level: task
title: "Remove Strategy from services"
short_code: "METIS-T-0092"
created_at: 2026-03-03T19:10:48.284964+00:00
updated_at: 2026-03-03T21:00:07.463407+00:00
parent: METIS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0024
---

# Remove Strategy from services

## Parent Initiative

[[METIS-I-0024]]

## Objective

Remove all strategy handling from the application service layer. This includes creation, discovery, transition, and archive services. The NULL strategy pattern for initiative parenting is eliminated here.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `create_strategy()` method deleted from `creation.rs` (done in T-0091)
- [x] `create_initiative_with_config()` simplified — no `strategy_id` parameter, no NULL pattern (done in T-0091)
- [x] `create_task_with_config()` updated to remove strategy_id handling (done in T-0091)
- [x] `DocumentType::Strategy` match arm removed from `find_document_of_type()` in `discovery.rs` (done in T-0091)
- [x] `"S"` → Strategy mapping removed from `document_type_from_short_code()` — removed from `configuration_repository.rs`
- [x] Strategy path construction logic removed from discovery (done in T-0091)
- [x] `DocumentType::Strategy` match arm removed from `get_current_phase()` and `perform_transition()` in `transition.rs` (done in T-0091)
- [x] `DocumentType::Strategy` match arm removed from `mark_as_archived_helper()` in `archive.rs` (done in T-0091)
- [x] All strategy-related path logic in services uses new flat layout (see METIS-T-0093)

## Implementation Notes

Depends on METIS-T-0091 (domain model removal). This task and METIS-T-0093 (filesystem layout) are tightly coupled — the new initiative paths need to be decided before service code can be updated. Coordinate with the filesystem task.

## Status Updates

### Session 1 — 2026-03-03

**All acceptance criteria met. Most work was already completed during T-0091.**

#### New changes in this task
- `error/conversions.rs` — removed "strategy" from valid document types error message
- `dal/database/configuration_repository.rs` — removed `"strategy" => "S"` mapping from `generate_short_code()`

#### Already completed during T-0091 (same crate, required for compilation)
- `create_strategy()` deleted from `creation.rs`
- `create_initiative_with_config()` / `create_task_with_config()` simplified (no `strategy_id`)
- `DocumentType::Strategy` match arms removed from discovery, transition, archive services
- Strategy path construction logic removed from discovery and synchronization
- DAL models, schema, and repository cleaned of strategy references

#### Remaining "strategy" references in .rs files (all appropriate to keep)
- Test fixtures using `level: strategy` — these test "wrong level" error handling
- `"Testing Strategy"` — generic English in markdown section headers
- `#strategy` tag parsing test — validates label tags, not document types
- `configuration.rs` comment — documents the deprecated `strategies_enabled` field

#### Verification
- `cargo check -p metis-docs-core` — compiles cleanly
- `angreal test-core` — 174 tests pass (147 unit + 27 integration)