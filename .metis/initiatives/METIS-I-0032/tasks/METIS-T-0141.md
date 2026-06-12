---
id: item-create-edit-search-and-saved
level: task
title: "Item create/edit, search, and saved views in the UI"
short_code: "METIS-T-0141"
created_at: 2026-06-12T20:00:15.759355+00:00
updated_at: 2026-06-12T20:00:15.759355+00:00
parent: METIS-I-0032
blocked_by: ["METIS-T-0140"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# Item create/edit, search, and saved views in the UI

## Parent Initiative

[[METIS-I-0032]]

## Objective

Make the UI write, not just read: create/edit items (markdown editor with live preview), transition phases, run a free-text search, and use saved views. Rounds the MVP into a usable tracker.

## Acceptance Criteria

- [ ] Create item: form (project, type from config, title, body, optional parent, tags) → `POST /items`; lands on the new item
- [ ] Edit item: title/body/assignee/tags/exit-criteria → `PATCH /items/:short_code` with `If-Match` (the item's `content_key`); a stale edit surfaces the 409 as a friendly "changed under you" message
- [ ] Markdown editor: textarea + live rendered preview (reuses the renderer from METIS-T-0140)
- [ ] Phase transition control on item detail → `POST .../transition`; a phase-gate 409 is shown clearly (which gate failed)
- [ ] Search box → `GET /items?q=`; renders a results list
- [ ] Saved views: list + create + run (`GET`/`POST /projects/:slug/views`, `GET /views/:id/items`); running a view shows its items
- [ ] All mutations reflect immediately (refetch/optimistic update)

## Implementation Notes

### Technical Approach
- Reuse the REST client + DTOs; surface `ApiError` codes (phase_gate, conflict, validation) as readable messages.
- The `If-Match` flow mirrors the server's optimistic-concurrency contract (T-0131); hold the item's `content_key` from the last read.

### Dependencies
- Blocked by METIS-T-0140 (board + detail views, markdown renderer).

### Risk Considerations
- Conflict UX: don't silently overwrite — on 409, re-fetch and let the user reconcile.

## Status Updates

*To be added during implementation*