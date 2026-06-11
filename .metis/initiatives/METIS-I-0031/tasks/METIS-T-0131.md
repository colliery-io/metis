---
id: rest-api-v1-projects-and-work-items
level: task
title: "REST API v1: projects and work items"
short_code: "METIS-T-0131"
created_at: 2026-06-11T13:09:30.045818+00:00
updated_at: 2026-06-11T15:17:49.017046+00:00
parent: METIS-I-0031
blocked_by: [METIS-T-0130]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# REST API v1: projects and work items

## Parent Initiative

[[METIS-I-0031]]

## Objective

Implement the `/api/v1` REST surface for projects and work items per the initiative's D3 sketch: CRUD, transitions, links, and archive — with structured errors that surface phase-gate violations machine-readably. (Repo registry, briefing, views, search `q`, and export land in later tasks.)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Projects: `POST`/`GET`/`PATCH /api/v1/projects[/:slug]` (config re-validated through the workflow engine on update)
- [x] Items: `GET /api/v1/items` (filters project/type/phase/assignee/tag/repo + limit/offset), `POST`, `GET`/`PATCH /items/:short_code` (title/body/assignee/tags/criteria)
- [x] `POST /items/:short_code/transition { phase?, force? }` — omitted phase advances to next
- [x] `POST`/`DELETE /items/:short_code/links` (blocks/relates/supersedes); `POST /items/:short_code/archive`
- [x] JSON error envelope `{error:{code,message,details}}`; phase-gate → 409 with gate detail; validation → 422; unknown → 404; bad input → 400
- [x] `If-Match` (content_key) optimistic-concurrency precondition on PATCH → 409 on mismatch
- [x] Router tests via `oneshot`: full lifecycle, auth failures, phase-gate 409, stale-If-Match 409, 404 (7 tests)

## Implementation Notes

### Technical Approach
- Handlers are thin: deserialize → `spawn_blocking(service call)` → serialize the shared `metis-core` DTOs
- The error envelope is the contract MCP tools will reuse in the next tranche — design it once here (`{ error: { code, message, details } }`)
- Pagination: limit/offset day one (cursors only if it ever matters)

### Dependencies
- METIS-T-0130 (server skeleton + auth)

### Risk Considerations
- This surface is the de-facto public API — names and shapes here propagate to MCP, UI, and integrations; review against D3 before merging
- The optimistic-concurrency check replaces the 2.x "read-before-edit" guard in spirit — without it, multi-agent teams will hit lost updates immediately

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0131-rest-api` off `3.0`)

`metis-server::routes` — full `/api/v1` surface over the service layer; 7 router tests green (total 7 server + 48 core).

**Decisions / deviations:**
- Handlers are thin adapters: deserialize → `AppState::blocking(service)` → serialize the shared `metis-core` DTOs. The `ApiError` envelope (built in T-0130) is reused unchanged — it's the contract MCP/clients will share.
- **If-Match** header carries the expected `content_key`; mapped to `ItemEdit.expected_content_key` → `ServiceError::Conflict` → 409. This is the server-side successor to 2.x read-before-edit.
- PATCH `assignee` uses a double-`Option` serde shim so JSON `null` (clear) is distinct from an absent field (leave unchanged).
- Status mapping: phase-gate 409 (`code: phase_gate`, with from/to/reason detail), conflict 409, validation/invalid-config 422, not-found 404, bad-request 400.
- Deferred to a later tranche (as scoped in the initiative): repo registry endpoints, `briefing`, saved `views`, free-text `q`/FTS, and `export`. `whoami` retained from T-0130.