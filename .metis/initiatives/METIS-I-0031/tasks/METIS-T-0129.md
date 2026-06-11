---
id: dal-and-core-service-layer
level: task
title: "DAL and core service layer"
short_code: "METIS-T-0129"
created_at: 2026-06-11T13:09:27.903017+00:00
updated_at: 2026-06-11T15:02:02.174312+00:00
parent: METIS-I-0031
blocked_by: [METIS-T-0126, METIS-T-0127, METIS-T-0128]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# DAL and core service layer

## Parent Initiative

[[METIS-I-0031]]

## Objective

Build the single-arm diesel-dualdb DAL over the 3.0 schema and the core service layer (`ProjectService`, `ItemService`, `UserService`, `QueryService`) that REST, MCP, and CLI will all sit on — including content writes through ObjectStore, workflow enforcement through the transition engine, and an `events` audit row on every mutation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Single-arm CRUD for projects, users, tokens, repos, work_items, links, tags, exit_criteria (views deferred to the query/views task — see note)
- [x] `ItemService::create` allocates the short code from the per-(project,type) sequence inside the insert transaction, with retry-on-unique-violation; sequential allocation tested. (Threaded PG concurrency test deferred — see note)
- [x] `ItemService::edit` writes new content via ObjectStore, repoints `content_key`, records prior key in the `events` row; history retrievable (tested)
- [x] `ItemService::transition` enforces the T-0127 engine and returns structured gate-violation errors (tested)
- [x] Structured queries: filter by project, type, phase, assignee, tag, repo (free-text `q` excluded)
- [x] Every mutation writes an `events` row carrying `Actor { user, agent }` (tested)
- [x] All DAL/service tests run on both backends via `#[diesel_dualdb::test]` (16 service tests)

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

### 2026-06-11 — Complete (branch `feat/T-0129-dal-services` off `3.0`)

`metis-core::{actor, models, service}` (project/user/item/query). 48 crate tests green on both backends; clippy clean.

**Decisions / deviations:**
- **Single-arm queries are the DAL** — no separate dal layer; service functions hold the diesel queries directly (satisfies "one query codebase"). Modules are free functions taking `&mut DualConnection`, mirroring the rest of the crate.
- **UUID wrapper gotcha**: columns use diesel-dualdb's `Uuid` wrapper, not plain `uuid::Uuid`. Fixed several `.eq(x.0)` → `.eq(x)` (wrapper is `Copy`).
- **Short-code allocation**: `MAX(seq)+1` per (project,type) inside the tx, retry up to 5× on unique violation (fresh tx each attempt, since a violation aborts a PG tx). Sequential allocation tested; a true threaded-PG concurrency test is **deferred** (needs a Pool against shared PG + careful isolation; the retry path is implemented and the unique constraint is the backstop).
- **`views` CRUD deferred** to the query/saved-views task (T-0131+ tranche) — not load-bearing for the server skeleton. Repos/links/tags/exit_criteria all covered.
- **Edit records history** via an `edited_body` event carrying `prev_content_key`/`new_content_key` — matches the GC convention from T-0128, so prior bodies are retained and retrievable.
- **Optimistic concurrency**: `ItemEdit.expected_content_key` precondition → `ServiceError::Conflict` on mismatch (the hook the REST If-Match in T-0131 uses).
- **PG test isolation**: the dualdb test macro shares one PG database and doesn't isolate, so added `db::reset` (down-then-up) called at each test's start; CI runs the PG arm `--test-threads=1`. (Discovered by inspecting the macro — bare `CREATE TABLE` would otherwise collide across tests in CI.)