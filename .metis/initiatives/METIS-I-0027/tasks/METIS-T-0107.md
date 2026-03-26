---
id: viewer-config-and-trait-abstraction
level: task
title: "Viewer config and trait abstraction"
short_code: "METIS-T-0107"
created_at: 2026-03-26T14:59:07.714919+00:00
updated_at: 2026-03-26T14:59:07.714919+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# Viewer config and trait abstraction

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Add the `[viewer]` configuration section to arawn.toml and define the `DocumentViewer` trait that all viewer backends will implement. This is the foundation that every other task in this initiative depends on.

## Acceptance Criteria

- [ ] arawn.toml schema supports `[viewer]` section with `default` field (`sys_editor` | `code` | `gui`) and `suppress_proactive_ticket_opening` (bool, default false)
- [ ] `default` falls back to `$EDITOR` environment variable when not configured
- [ ] `DocumentViewer` trait defined with `open(&self, paths: &[PathBuf]) -> Result<()>` and `is_open(&self, path: &PathBuf) -> Result<bool>`
- [ ] Viewer dispatcher reads config and routes to the correct backend (can use stub backends initially)
- [ ] Dispatcher implements fallback chain on failure: configured viewer → sys_editor → error. Failed viewer's error message is included in fallback notification.
- [ ] Existing tests pass, new unit tests for config parsing, dispatcher logic, and fallback behavior

## Implementation Notes

### Technical Approach
- Add config structs to arawn.toml parsing (likely in metis-docs-core or metis-mcp config module)
- Define `DocumentViewer` trait in a new `viewer` module with `open()` and `is_open()` methods
- Implement dispatcher function that reads config, resolves viewer, checks `is_open()` before calling `open()` (look before you leap)
- Dispatcher implements fallback chain: if the configured viewer fails, propagate to the next viable viewer (e.g., gui fails → try sys_editor). Error message from the failed viewer should be included so the user knows what happened.
- Stub backends can return `Ok(())` or `unimplemented!()` — real backends come in subsequent tasks

### Dependencies
- None — this is the first task in the chain

## Status Updates

*To be added during implementation*