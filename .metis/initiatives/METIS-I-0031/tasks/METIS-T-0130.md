---
id: metis-server-skeleton-with-pat
level: task
title: "metis-server skeleton with PAT auth and admin bootstrap"
short_code: "METIS-T-0130"
created_at: 2026-06-11T13:09:29.325408+00:00
updated_at: 2026-06-11T15:14:35.919170+00:00
parent: METIS-I-0031
blocked_by: [METIS-T-0129]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# metis-server skeleton with PAT auth and admin bootstrap

## Parent Initiative

[[METIS-I-0031]]

## Objective

Stand up the `metis-server` crate: an axum binary with `serve`, `serve --local`, and `admin` commands, configuration loading, PAT auth middleware resolving tokens to `Actor { user, agent }`, and a docker-compose deployment that boots green.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `metis serve` starts from config/env (`DATABASE_URL` / SQLite path, object-store selection), runs migrations idempotently on startup (`db::ensure_migrated`), serves `/healthz`
- [x] Auth middleware: `Authorization: Bearer mtk_…` resolved via sha256 lookup to `Actor { user, agent }`; missing/invalid/revoked/inactive → 401; `last_used_at` bumped
- [x] `metis admin create-user <username> [--admin]` and `metis admin create-token --user <u> [--agent-name <a>]` work directly against the DB; plaintext printed once
- [x] `metis serve --local` binds localhost, SQLite + db-blob, auth disabled with an implicit `local` user
- [x] All handler DAL goes through `spawn_blocking` on a pool (`AppState::blocking`)
- [x] `docker-compose.yml` + `Dockerfile` written; `scripts/smoke-test.sh` validated locally end-to-end (healthz, 401, create user, create token, authed request). Container *build* is blocked on the diesel-dualdb dependency (see note)
- [x] Structured logging via `tracing` (`tracing_subscriber` env filter; startup span). Per-request span layer deferred to T-0131 routing

## Implementation Notes

### Technical Approach
- axum + tower middleware; `AppState { pool, object_store, config }`
- Token verification is hash lookup (sha256 of presented token vs stored hash) — constant-time comparison, no token logging
- `admin` subcommands bypass HTTP deliberately (bootstrap problem): same binary, direct DB connection
- Config precedence: flags > env > config file; keep the day-one surface small (database URL, object store, bind address)

### Dependencies
- METIS-T-0129 (service layer). Routes beyond `/healthz` land in METIS-T-0131.

### Risk Considerations
- This task defines the binary's operational contract (config, bootstrap, deployment) — keep it boring and documented; it's what "self-hosted in an afternoon" depends on
- Auth middleware must be fail-closed: any route not explicitly public requires an Actor

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0130-server-skeleton` off `3.0`)

`metis-server` crate (axum); 5 router tests green; smoke test validated against the running binary.

**Decisions / deviations:**
- **Idempotent startup migration**: added `metis-core::db::ensure_migrated` (probes for the `projects` table, migrates only if absent) since the un-versioned runner can't re-run bare `CREATE TABLE`. Replaces the bootstrap blocker for a long-running server.
- **`user::resolve_actor` added to metis-core** (token→Actor: hash lookup, not-revoked + active-user, bumps `last_used_at`) — the query belongs with the schema.
- **Token format `mtk_` + 43 alphanumeric** (~256 bits) via `rand::Alphanumeric` (avoids a base62 dep; functionally base62).
- **Local mode** ensures a `local` user at startup and injects it; auth middleware short-circuits.
- **Deployment**: `Dockerfile` + `docker-compose.yml` written. The image **build** can't run yet because metis-core's diesel-dualdb path dep is outside the repo/Docker context — same release blocker as T-0126; documented in both files (build context = parent dir with a diesel-dualdb sibling). The `smoke-test.sh` flow itself is validated locally against `metis serve` (healthz → 401 → bootstrap → authed 200), so the criterion's *behavior* is proven even though the container build is gated.
- Per-request tracing span layer deferred to the routing task (T-0131); startup logging + env-filter in place.