---
id: project-board-and-item-detail-views
level: task
title: "Project board and item detail views"
short_code: "METIS-T-0140"
created_at: 2026-06-12T20:00:14.228663+00:00
updated_at: 2026-06-12T20:00:14.228663+00:00
parent: METIS-I-0032
blocked_by: ["METIS-T-0139"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# Project board and item detail views

## Parent Initiative

[[METIS-I-0032]]

## Objective

The read-side core of the UI: a project board (items grouped by phase) and an item detail view (rendered markdown body, links, exit criteria). What makes the tool feel like a tracker.

## Acceptance Criteria

- [ ] Board at `/p/:project/board`: items grouped into columns by phase (phase set from the project's type/workflow config); cards show short code, title, assignee; reads via `GET /items` + `GET /projects/:slug` (for the phase config)
- [ ] Item detail at `/p/:project/i/:short_code`: title, type, phase, **rendered markdown body**, tags, repos, outgoing links, exit-criteria checklist; reads `GET /items/:short_code`
- [ ] Markdown rendered with a WASM-compatible renderer (`pulldown-cmark`/`comrak` → HTML)
- [ ] Deep links render the right view directly (shareable URLs); unknown short code → a clean not-found state
- [ ] Loading + error states (e.g. 401 → bounce to login)

## Implementation Notes

### Technical Approach
- Board columns come from `ProjectConfig` (the type's ordered phases); for a multi-type board, group by phase across the project or scope to a type — pick the simplest that's useful (document the choice).
- Markdown → HTML via a wasm md crate; inject with Dioxus (sanitize/escape as needed).
- Components stay target-agnostic (reused by the desktop target, METIS-T-0142).

### Dependencies
- Blocked by METIS-T-0139 (skeleton + REST client). Blocks METIS-T-0141 and METIS-T-0142.

### Risk Considerations
- Markdown rendering of untrusted content → guard against HTML injection (use the renderer's escaping; don't pass raw user HTML through unsanitized).

## Status Updates

*To be added during implementation*