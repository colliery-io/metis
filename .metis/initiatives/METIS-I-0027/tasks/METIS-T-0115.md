---
id: proactive-open-on-create-edit
level: task
title: "Proactive open on create/edit"
short_code: "METIS-T-0115"
created_at: 2026-03-26T14:59:09.976053+00:00
updated_at: 2026-03-26T17:18:22.651730+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# Proactive open on create/edit

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Wire `create_document` and `edit_document` to proactively open documents in the configured viewer after successful operations, controlled by the `suppress_proactive_ticket_opening` config flag.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `create_document` automatically opens the new document in the configured viewer after creation
- [ ] `edit_document` automatically opens the document if not already open in this session
- [ ] Documents already open in the viewer are not re-opened (delegates to backend `is_open` check)
- [ ] Proactive opening is suppressed when `suppress_proactive_ticket_opening = true` in arawn.toml
- [ ] `open_document` explicit calls always work regardless of the suppress flag
- [ ] Opening failures (e.g., viewer not available) are logged but don't fail the create/edit operation

## Implementation Notes

### Technical Approach
- After successful `create_document`: check suppress flag → call viewer dispatcher (which checks `is_open` before opening)
- After successful `edit_document`: same flow — dispatcher's look-before-you-leap handles dedup
- Opening is best-effort — if the viewer fails, log a warning and return the create/edit result normally
- The `open_document` tool bypasses suppress flag and opened set (always opens)

### Dependencies
- METIS-T-0107 (viewer config and trait)
- METIS-T-0109 (open_document tool — for the dispatcher)
- At least one viewer backend (METIS-T-0110 or METIS-T-0111)

## Status Updates

- **2026-03-26**: Implemented. `create_document` and `edit_document` now accept a `ViewerDispatcher` and proactively open documents after successful operations. Opening is best-effort (failures logged via `warn!`, don't fail the operation). Suppressed when `suppress_proactive_ticket_opening = true`. Dispatcher's `is_open` check prevents re-opening already-open files. All 25 MCP tests pass.