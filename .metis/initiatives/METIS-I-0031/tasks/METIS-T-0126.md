---
id: logical-schema-and-metis-core
level: task
title: "Logical schema and metis-core crate skeleton"
short_code: "METIS-T-0126"
created_at: 2026-06-11T13:09:10.797577+00:00
updated_at: 2026-06-11T14:34:59.090079+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Logical schema and metis-core crate skeleton

## Parent Initiative

[[METIS-I-0031]]

## Objective

Stand up the `metis-core` crate with the full 3.0 logical schema on diesel-dualdb: logical DDL migrations, generated Postgres + SQLite migrations and unified `schema.rs` committed, a connection pool that works against both backends, and CI guarding against generated-output drift.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `schema/migrations/` contains logical DDL for all tables in METIS-I-0031 D1: projects, users, tokens, repos, work_items, work_item_links, work_item_repos, work_item_tags, exit_criteria, events, views, objects
- [x] `diesel-dualdb-schema` output (migrations-postgres, migrations-sqlite, schema.rs) is committed under `schema/generated/` and treated as read-only
- [x] CI step regenerates into a temp dir and fails on diff (`scripts/check-schema-drift.sh`, wired into `ci-3.0.yml`)
- [x] `crates/metis-core` exists in the workspace, compiles, and exposes `Pool::connect` working for both `:memory:`/file SQLite and a Postgres URL
- [x] Generated migrations run green on both backends in CI (Postgres service container added to `ci-3.0.yml`); locally SQLite arm green, PG arm self-skips
- [x] angreal `build`/`test`/`check` tasks cover the new crate

## Implementation Notes

### Technical Approach
- Follow `../diesel-dualdb/docs/how-to/generate-schema-and-migrations.md`: logical column types (UUID, TIMESTAMP, JSON, BYTES) in `up.sql`, generator emits both backends + `schema.rs`; `include!` the generated schema
- Portable types: `types::Uuid`, `types::Timestamp`, `types::Json<T>`, `types::Bytes` for `objects.value`
- FK constraints in the DDL give `joinable!` wiring for free
- **FTS tables are deliberately NOT in the logical schema** — FTS5/tsvector DDL is backend-specific and lands with the search/query task as hand-written per-backend migrations
- Decide the dependency form for diesel-dualdb (path dep on `../diesel-dualdb` for development; needs a git or crates.io reference before any release)

### Dependencies
None — this is the first task of the initiative. T-0128 (objects table) and T-0129 (DAL) build on it.

### Risk Considerations
- diesel-dualdb is sync-only v1 — fine at this layer; the async boundary is handled in the server task via `spawn_blocking`
- Required diesel feature `returning_clauses_for_sqlite_3_35` (already used by 2.x, so the bundled libsqlite3 is new enough)

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0126-logical-schema` off `3.0`)

Stood up `crates/metis-core` with the full 3.0 logical schema on diesel-dualdb. Both test arms green, drift check passing, frozen 2.x crates rebuild under the diesel 2.3 bump.

**Decisions / deviations:**
- **New schema, not a DAL port** (as designed). DB is the store; 2.x `filepath`/`file_hash`/`frontmatter_json` are gone.
- **Inline `REFERENCES` only where single & non-self-referential**, because the generator emits `joinable!` and diesel rejects duplicate/self joinables. `parent_id`, `assignee`, `created_by`, link endpoints, and `actor_user` are plain UUID columns — integrity enforced in the service layer (T-0129). SQLite doesn't enforce FKs by default regardless.
- Reserved-word renames: `type`→`item_type`; object cols → `object_key`/`content`/`byte_size`; `position`→`ordinal`; `query`→`query_json`.
- **Migration runner intentionally un-versioned** (no schema-migrations table) — fine for one init migration on a fresh DB; revisit at the 2nd migration.

**Release blocker (carried):** diesel-dualdb is a path dep on `../../../diesel-dualdb`; must become a git/crates.io ref before 3.0 ships. `ci-3.0.yml` checks it out as a sibling as a stopgap.