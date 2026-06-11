---
id: rest-api-v1-projects-and-work-items
level: task
title: "REST API v1: projects and work items"
short_code: "METIS-T-0131"
created_at: 2026-06-11T13:09:30.045818+00:00
updated_at: 2026-06-11T13:09:30.045818+00:00
parent: METIS-I-0031
blocked_by: ["METIS-T-0130"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# REST API v1: projects and work items

## Parent Initiative

[[METIS-I-0031]]

## Objective

Implement the `/api/v1` REST surface for projects and work items per the initiative's D3 sketch: CRUD, transitions, links, and archive — with structured errors that surface phase-gate violations machine-readably. (Repo registry, briefing, views, search `q`, and export land in later tasks.)

## Acceptance Criteria

- [ ] Projects: `POST /api/v1/projects`, `GET /api/v1/projects/:slug`, `PATCH /api/v1/projects/:slug` (config updates re-validated through the workflow engine)
- [ ] Items: `GET /api/v1/items` (filters: project, type, phase, assignee, tag, repo; limit/offset pagination), `POST /api/v1/items`, `GET /api/v1/items/:short_code`, `PATCH /api/v1/items/:short_code` (title/body/assignee/tags/criteria)
- [ ] `POST /api/v1/items/:short_code/transition { phase?, force? }` — omitted phase advances to next, mirroring today's MCP semantics
- [ ] `POST` / `DELETE /api/v1/items/:short_code/links` for blocks/relates/supersedes; `POST /api/v1/items/:short_code/archive`
- [ ] Consistent JSON error envelope with machine-readable codes; phase-gate violations return 409 with the violated gate spelled out; validation 422; unknown short code 404
- [ ] Item edits accept an optimistic-concurrency precondition (current `content_key` as the If-Match value) → 409 on mismatch, so two agents can't silently clobber each other
- [ ] Router tests via `tower::ServiceExt::oneshot` for happy paths, auth failures, and every gate-violation case

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

*To be added during implementation*