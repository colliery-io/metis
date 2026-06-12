---
id: server-resource-guardrails-noisy
level: task
title: "Server resource guardrails (noisy-neighbor hardening)"
short_code: "METIS-T-0137"
created_at: 2026-06-12T12:39:55.375338+00:00
updated_at: 2026-06-12T12:55:31.043368+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Server resource guardrails (noisy-neighbor hardening)

## Parent Initiative

[[METIS-I-0031]]

## Objective

Contain "noisy-neighbor" load before adding the web UI: a clumsy/heavy client (human via the UI, or an agent) must not be able to wedge the shared server/DB for everyone. The risk lives in the shared process + connection pool, not in any frontend; these are cheap, standard guardrails that turn "could hang the server" into "gets a 4xx/timeout while everyone else is fine."

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] **Result-size cap**: `query::list` clamps `limit` to `MAX_LIMIT` (500) and `offset >= 0` regardless of request — covers REST `/items` + MCP `list_items`
- [x] **Request body limit**: `DefaultBodyLimit::max(1 MiB)` on the router → `413` on oversize (tested)
- [x] **Pool acquire timeout**: `db::connect` sets a 10s `connection_timeout`; Postgres `max_size(16)`, SQLite stays `max_size(1)`
- [x] **Team-on-SQLite guard**: `build_state` logs a warning when team mode runs on a SQLite URL
- [x] Tests: limit clamped above max (both backends); over-limit body → 413; full suite green (84)
- [x] Docs: "Operations / limits" section in the `metis-server` crate docs (caps + `statement_timeout` + two-process isolation recommendations)

## Non-Goals (this task)

- Per-token rate limiting / quotas — add only if real abuse appears.
- Postgres `statement_timeout` enforced in code — the diesel-dualdb pool doesn't expose an r2d2 connection customizer; recommend setting it via the `DATABASE_URL` (`?options=-c statement_timeout=...`) or server config. Documented, not coded here.
- Multi-process UI/API isolation — available for free architecturally (stateless server, run two `metis serve` against one DB); documented as the escape hatch, not built now.

## Implementation Notes

### Technical Approach
- `metis-core::query`: a `MAX_LIMIT` constant; clamp `filter.limit`/`offset` in `list` (covers REST `/items` and the MCP `list_items` tool, both of which route through it).
- `metis-server`: `DefaultBodyLimit::max(...)` layer on the router; `db::connect` uses `Pool::builder().connection_timeout(..).max_size(..)` (SQLite stays `max_size(1)`); a startup check in `serve`/`build_state` logs the team-on-SQLite warning via `detect_backend`.

### Dependencies
- METIS-T-0136 and earlier (query layer, server, pool). No new external deps.

### Risk Considerations
- Clamping `limit` changes observable API behavior for a client asking for more than the cap — acceptable and documented; the cap is generous (hundreds).
- Don't over-restrict the body limit: item bodies are markdown and can be sizable; 1 MiB is comfortably large for prose but bounds abuse.

## Status Updates

### 2026-06-12 — Complete (branch `feat/T-0137-hardening` off `initiative/METIS-I-0031`)

Guardrails landed; full suite (84) green, clippy clean. This was the agreed prerequisite to the web UI (noisy-neighbor concern). The risk is shared-process/pool, not frontend — so these apply regardless of the Dioxus UI choice.

**Decisions / deviations:**
- `MAX_LIMIT = 500` (generous; covers REST + MCP via the shared `query::list`).
- Body limit 1 MiB via `DefaultBodyLimit`; verified a 2 MiB body → 413.
- Pool: 10s acquire timeout both backends; Postgres `max_size(16)`, SQLite `max_size(1)` (single-writer).
- Team-on-SQLite is a **warning, not a hard error** — small teams may knowingly run SQLite; we inform rather than block.
- **`statement_timeout` deferred to deployment** (the diesel-dualdb pool exposes no r2d2 connection customizer); documented to set via `DATABASE_URL ?options=-c statement_timeout=...`. The strongest isolation (separate UI vs agent `metis serve` processes on one DB) is documented as the architectural escape hatch.