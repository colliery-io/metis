---
id: project-config-model-and-workflow
level: task
title: "Project config model and workflow engine"
short_code: "METIS-T-0127"
created_at: 2026-06-11T13:09:25.078162+00:00
updated_at: 2026-06-11T13:09:25.078162+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Project config model and workflow engine

## Parent Initiative

[[METIS-I-0031]]

## Objective

Define the `projects.config` JSON model (work item types, workflows, short-code letters, templates) and the pure-domain workflow engine that enforces it — closing the "exact JSON schema for projects.config" open item from the initiative design. Flight Levels ships as the default config.

## Acceptance Criteria

- [ ] Serde model for project config: a map of work item types, each with display name, short-code letter, ordered phase list, template body, and flags for special semantics (e.g. ADR-style `superseded` terminal phase, parent-type constraints)
- [ ] Config validation with actionable errors: duplicate type letters, empty phase lists, unknown parent types, reserved names all rejected
- [ ] Built-in Flight Levels default config reproducing today's five types and their phase sequences (vision, initiative, task, adr, specification), plus general-purpose `bug`/`feature`/`chore` task-like types
- [ ] Transition engine: adjacent-phase rule, `force` semantics, exit-criteria gating (can't complete with unmet criteria unless forced) — pure functions, no I/O
- [ ] Short-code formatting/parsing: `{PROJECT}-{LETTER}-{NNNN}` round-trips; parser rejects malformed codes
- [ ] Unit tests cover legal and illegal transitions for the default config, including ADR supersede flow
- [ ] Config format documented (rustdoc on the model is sufficient for now)

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

*To be added during implementation*