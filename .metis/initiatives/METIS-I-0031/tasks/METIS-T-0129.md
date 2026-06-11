---
id: dal-and-core-service-layer
level: task
title: "DAL and core service layer"
short_code: "METIS-T-0129"
created_at: 2026-06-11T13:09:27.903017+00:00
updated_at: 2026-06-11T13:09:27.903017+00:00
parent: METIS-I-0031
blocked_by: ["METIS-T-0126", "METIS-T-0127", "METIS-T-0128"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# DAL and core service layer

## Parent Initiative

[[METIS-I-0031]]

## Objective

Build the single-arm diesel-dualdb DAL over the 3.0 schema and the core service layer (`ProjectService`, `ItemService`, `UserService`, `QueryService`) that REST, MCP, and CLI will all sit on — including content writes through ObjectStore, workflow enforcement through the transition engine, and an `events` audit row on every mutation.

## Acceptance Criteria

- [ ] Single-arm CRUD for projects, users, tokens, repos, work_items, links, tags, exit_criteria, and views — no per-backend `match` outside the sanctioned FTS divergence (which is NOT in this task)
- [ ] `ItemService::create` allocates the short code from the per-project+type sequence inside the insert transaction; concurrent creates never collide (test with threads on Postgres)
- [ ] `ItemService::edit` writes new content via ObjectStore (write-before-commit ordering), repoints `content_key`, and records the prior key in the `events` row — body history is retrievable
- [ ] `ItemService::transition` enforces the METIS-T-0127 engine (adjacent phases, force, exit-criteria gating) and returns structured gate-violation errors
- [ ] Structured queries: filter by project, type, phase, assignee, tag, repo (free-text `q` excluded — lands with the FTS task)
- [ ] Every mutation writes an `events` row carrying `Actor { user, agent }`
- [ ] All DAL/service tests run on both backends via `#[diesel_dualdb::test]`

## Implementation Notes

### Technical Approach
- Services are sync (dualdb v1); the async boundary belongs to `metis-server` (`spawn_blocking`), not here
- Service methods take an `Actor` parameter — auth resolution happens in the server, services just record attribution
- Hierarchy: `parent_id` column with parent-type constraint checks from project config; links table for blocks/relates/supersedes with both-endpoints-same-project validation (cross-project links are out of scope day one)
- Keep DTOs (`ItemSummary`, `ItemDetail`) in `metis-core` so REST and MCP serialize the same shapes

### Dependencies
- METIS-T-0126 (schema + crate), METIS-T-0127 (workflow engine), METIS-T-0128 (ObjectStore)

### Risk Considerations
- Short-code allocation under concurrency is the classic race — sequence-in-transaction with a uniqueness backstop on `(project_id, type, seq)`
- Watch transaction scope around ObjectStore: db-blob can share the transaction, fs cannot — the service must use the write-before-commit pattern uniformly so behavior doesn't depend on backend

## Status Updates

*To be added during implementation*