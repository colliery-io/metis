---
id: full-text-search-and-saved-views
level: task
title: "Full-text search and saved views"
short_code: "METIS-T-0136"
created_at: 2026-06-12T10:47:05.433923+00:00
updated_at: 2026-06-12T12:01:37.127212+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Full-text search and saved views

## Parent Initiative

[[METIS-I-0031]]

## Objective

The query layer's two missing pieces: free-text **search** over work items (the one sanctioned per-backend divergence — SQLite FTS5 vs Postgres `tsvector`) and **saved views** (named, reusable item queries, the part of a tracker people use daily). Search is fed at write time and exposed through `/items?q=` and the `list_items` MCP tool; views are REST CRUD that a UI/agent can run.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `work_item_search` (short_code + content) — hand-written migration `0002` (SQLite FTS5 virtual table; Postgres table + `tsvector` + GIN), outside the generated tree; `ensure_migrated` applies it incrementally via per-migration probe tables
- [x] Fed at write time by `item::create`/`item::edit`; `service::search::{index,search}` run per-backend SQL (FTS5 `MATCH` / `@@ plainto_tsquery`) behind one API; ranked short codes
- [x] `query::list` gains `q` — FTS-restricted, rank-ordered, structured filters still apply; exposed via `GET /api/v1/items?q=` and the `list_items` MCP tool
- [x] `service::view` create/list/get/delete/run; stores owner/name/query(`ItemFilter`)/shared
- [x] REST: `POST`/`GET /api/v1/projects/:slug/views`, `GET`/`DELETE /api/v1/views/:id`, `GET /api/v1/views/:id/items`
- [x] Tests (both backends): FTS hit/non-match/archived-excluded/edit-reflected/structured-filter; views CRUD+run; REST search + views; live curl smoke

## Non-Goals (this task)

- Search ranking/relevance tuning, highlighting, prefix/fuzzy — basic FTS only.
- View tools in MCP (views are REST/UI; agents get `q` in `list_items`). May add later.
- A migration-version table — `ensure_migrated` gets per-migration probes for now (a real version table is its own task when migrations multiply).

## Implementation Notes

### Technical Approach
- **Migration 0002** is hand-written per backend (the generator only emits portable DDL): `schema/manual/0002_fts/{postgres,sqlite}/{up,down}.sql`, `include_str!`'d into `db.rs`. `Migration` gains a `probe_table`; `ensure_migrated` runs each migration whose probe table is absent (raw `SELECT 1 FROM <t> LIMIT 1`), so 0002 applies to an existing 0001 DB.
- **`service::search`**: `index(conn, short_code, title, body)` (delete-then-insert upsert) and `search(...)` via `diesel::sql_query` with per-backend SQL chosen by the live connection's backend (`?`/FTS5 `MATCH` vs `$n`/`@@`). Returns ranked `short_code`s; `list` loads + filters + reorders.
- **`service::view`**: rows in the existing `views` table; query stored as `Json<ItemFilter>`-shaped value. Run = deserialize → `query::list`.

### Dependencies
- METIS-T-0129 (services, work_items/views tables, ItemFilter), T-0131 (REST), T-0132/0135 (MCP list tool). Object store for body text.

### Risk Considerations
- The FTS divergence is the whole point — the dual-backend test must pass on SQLite (FTS5) and Postgres (tsvector). PG arm verified in CI; SQLite locally.
- FTS5 query syntax can choke on punctuation in the user string; keep the query simple (pass through; harden later). Don't let a search error 500 the list endpoint.
- Index maintenance is best-effort/eventually-consistent with the body; fed synchronously at write for now (no triggers).

## Status Updates

### 2026-06-12 — Complete (branch `feat/T-0136-fts-views` off `initiative/METIS-I-0031`)

`metis-core::service::{search,view}` + migration 0002 + REST/MCP wiring. 8 dual-backend core tests (search+views) + REST search/views + live smoke; full suite (81) green, clippy clean.

**Decisions / deviations:**
- **FTS keyed by `short_code`, not `id`** — avoids the dual uuid representation (BLOB on SQLite vs uuid on PG); short codes are identical TEXT on both, so results map straight back to `work_items`.
- **`ItemFilter.item_type` now serializes as `type`** — so a saved view's query and `GET /items?type=` use the same field name (caught by the views test where `{"type":"bug"}` was being ignored).
- **`ensure_migrated` gained per-migration probe tables** (`SELECT 1 FROM <t> LIMIT 1`) so 0002 applies to an already-initialized 0001 DB without a version table (deferred). Manual migrations live in `schema/manual/` (outside the generated tree, so the drift check is unaffected — verified).
- **SQLite query sanitized** to bare alphanumeric terms before FTS5 `MATCH` (punctuation is FTS5 syntax); Postgres uses `plainto_tsquery` which is already injection/“syntax”-safe. Search failures never 500 the list (q empty → no matches).
- **Search fed synchronously at write** (`item::create`/`edit` → `search::index`), best-effort (logged, never fails the write); archived items are excluded by the `work_items` filter, not by de-indexing.
- **Views are REST-only** (no MCP tool); agents get search via `q` on `list_items`. View tools can be added later if needed.