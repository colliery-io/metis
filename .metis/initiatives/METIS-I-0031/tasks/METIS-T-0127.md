---
id: project-config-model-and-workflow
level: task
title: "Project config model and workflow engine"
short_code: "METIS-T-0127"
created_at: 2026-06-11T13:09:25.078162+00:00
updated_at: 2026-06-11T14:43:59.585861+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Project config model and workflow engine

## Parent Initiative

[[METIS-I-0031]]

## Objective

Define the `projects.config` JSON model (work item types, workflows, short-code letters, templates) and the pure-domain workflow engine that enforces it — closing the "exact JSON schema for projects.config" open item from the initiative design. Flight Levels ships as the default config.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Serde model for project config: a map of work item types, each with display name, short-code letter, ordered phase list, template body, and parent-type constraints (`allowed_parents` + `allow_root`)
- [x] Config validation with actionable errors: duplicate type letters, empty/duplicate phases, unknown parent types, unreachable types, bad names all rejected
- [x] Built-in Flight Levels default config reproducing today's five types and their phase sequences, plus general-purpose `bug`/`feature`/`chore`
- [x] Transition engine: forward-adjacent rule, `force` semantics, exit-criteria gating — pure functions, no I/O
- [x] Short-code formatting/parsing: `{PREFIX}-{LETTER}-{NNNN}` round-trips (hyphenated prefixes too); parser rejects malformed codes
- [x] Unit tests cover legal and illegal transitions for the default config, including ADR supersede flow (22 tests)
- [x] Config format documented (rustdoc on the model)

## Implementation Notes

### Technical Approach
- Lives in `metis-core::domain` — no database or ObjectStore dependency, so it can be developed in parallel with METIS-T-0126
- Port the phase-gate rules from `metis-docs-core` (`application/services/workspace/transition.rs` and `domain/documents/*`) rather than reinventing — the semantics agents rely on (adjacent-only transitions, force flag) must match
- Config is stored as `projects.config JSON` (see D1); this task defines the shape, validation, and engine that interprets it

### Dependencies
None hard. Consumed by METIS-T-0129 (service layer enforces transitions through this engine).

### Risk Considerations
- Over-generalizing workflows is the JIRA trap — keep the model to ordered phases + a terminal archive/supersede concept; resist conditional transitions, per-phase permissions, etc.
- Template bodies in config raise size questions; fine as JSON for day one, revisit if configs get unwieldy

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0127-workflow-engine` off `3.0`)

`metis-core::workflow` module (pure domain, no DB/ObjectStore), 22 unit tests green.

**Decisions / deviations:**
- **All default phase sequences are linear** (verified against 2.x `types.rs`), so the engine is forward-adjacent over an ordered list — no transition graph. ADR supersede is `decided → superseded` as the terminal step.
- **Exit-criteria gate applies to every non-forced forward transition** (2.x behavior generalized), not only completion. `force` overrides adjacency + criteria and allows backward moves.
- Engine takes `exit_criteria_met: bool` as input; the service layer (T-0129) computes it from the checklist. Engine stays I/O-free.
- Added `allow_root` to `ItemTypeConfig` so a type can be root-capable AND have allowed parents (task: both). "Reserved names" enforced as format validation (`[a-z][a-z0-9_]*` names/phases, `[A-Z]{1,2}` letters).
- Short codes parse right-to-left so hyphenated prefixes work. Templates are minimal per-type stubs for now (full 2.x parity deferred — not required).