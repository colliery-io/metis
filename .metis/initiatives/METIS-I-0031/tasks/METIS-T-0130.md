---
id: metis-server-skeleton-with-pat
level: task
title: "metis-server skeleton with PAT auth and admin bootstrap"
short_code: "METIS-T-0130"
created_at: 2026-06-11T13:09:29.325408+00:00
updated_at: 2026-06-11T13:09:29.325408+00:00
parent: METIS-I-0031
blocked_by: ["METIS-T-0129"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# metis-server skeleton with PAT auth and admin bootstrap

## Parent Initiative

[[METIS-I-0031]]

## Objective

Stand up the `metis-server` crate: an axum binary with `serve`, `serve --local`, and `admin` commands, configuration loading, PAT auth middleware resolving tokens to `Actor { user, agent }`, and a docker-compose deployment that boots green.

## Acceptance Criteria

- [ ] `metis serve` starts from config/env (`DATABASE_URL` Postgres or SQLite path, object-store selection), runs pending migrations on startup, serves `/healthz`
- [ ] Auth middleware: `Authorization: Bearer mtk_…` resolved via sha256 lookup to `Actor { user, agent: Option<String> }`; missing/invalid/revoked token → 401; `last_used_at` updated
- [ ] `metis admin create-user <username> [--admin]` and `metis admin create-token --user <u> [--agent-name <a>]` work directly against the database; the plaintext token is printed exactly once
- [ ] `metis serve --local` binds localhost only, SQLite + db-blob defaults, auth disabled with an implicit single user
- [ ] All DAL access from handlers goes through `spawn_blocking` with a connection pool — no sync diesel calls on the async runtime
- [ ] `docker-compose.yml` (metis-server + postgres) boots and passes a smoke test (healthz, create user, create token, authed request)
- [ ] Structured logging via `tracing` with request spans

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

*To be added during implementation*