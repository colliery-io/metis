---
id: consolidate-phase-transition
level: task
title: "Consolidate phase transition validation logic"
short_code: "METIS-T-0056"
created_at: 2026-01-12T13:13:49.387803+00:00
updated_at: 2026-01-12T17:33:38.770114+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Consolidate phase transition validation logic

## Objective

Eliminate duplicate phase transition validation logic by having a single source of truth for valid transitions.

## Problem

Phase transition rules are currently defined in two places:

1. **`PhaseTransitionService::get_valid_transitions()`** - `crates/metis-docs-core/src/application/services/workspace/transition.rs`
2. **Each document's `can_transition_to()` trait impl** - e.g., `crates/metis-docs-core/src/domain/documents/vision/mod.rs`

This caused a bug where the service claimed backward transitions were valid, but the document-level validation rejected them. We fixed this by aligning both to forward-only, but the duplication remains.

## Proposed Solutions

### Option A: Service delegates to documents
Have `get_valid_transitions()` call the document's `can_transition_to()` for each possible phase instead of maintaining its own list.

### Option B: Documents delegate to service
Remove validation from document's `transition_phase()` and have it trust the service's validation.

### Option C: Shared transition rules
Extract transition rules to a single location (e.g., constants or a dedicated module) that both service and documents reference.

## Files Involved

- `crates/metis-docs-core/src/application/services/workspace/transition.rs`
- `crates/metis-docs-core/src/domain/documents/vision/mod.rs`
- `crates/metis-docs-core/src/domain/documents/strategy/mod.rs`
- `crates/metis-docs-core/src/domain/documents/initiative/mod.rs`
- `crates/metis-docs-core/src/domain/documents/task/mod.rs`
- `crates/metis-docs-core/src/domain/documents/adr/mod.rs`

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Single source of truth for phase transition rules
- [x] No duplicate transition validation logic
- [x] All existing tests pass
- [x] Documentation updated if API changes

## Resolution

Implemented Option C (shared transition rules) by adding methods to `DocumentType`:
- `valid_transitions_from(phase)` - returns valid target phases
- `can_transition(from, to)` - checks if transition is valid
- `next_phase(current)` - gets next phase in sequence
- `phase_sequence()` - returns ordered phases for display

Updated all consumers to use these methods:
- `PhaseTransitionService::get_valid_transitions()` now delegates to `DocumentType`
- `PhaseTransitionService::get_next_phase()` now delegates to `DocumentType`
- All document `can_transition_to()` implementations now delegate to `DocumentType`
- MCP `transition_phase.rs` now uses `DocumentType::phase_sequence()`