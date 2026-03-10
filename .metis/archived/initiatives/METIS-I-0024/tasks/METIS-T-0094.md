---
id: database-migration-to-remove
level: task
title: "Database migration to remove strategy_id"
short_code: "METIS-T-0094"
created_at: 2026-03-03T19:10:50.072057+00:00
updated_at: 2026-03-04T00:42:18.531163+00:00
parent: METIS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0024
---

# Database migration to remove strategy_id

## Parent Initiative

[[METIS-I-0024]]

## Objective

Create a diesel migration that removes the `strategy_id` column from the documents table, drops strategy-related indexes, and deletes any strategy rows. Update the Rust schema and model definitions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] New diesel migration created: `008_remove_strategy_id`
- [x] `up.sql`: Recreate documents table without `strategy_id` column (SQLite requires table recreation for column removal)
- [x] `up.sql`: Delete any rows where `document_type = 'strategy'` (plus related tags, relationships, search entries)
- [x] `up.sql`: Drop `idx_documents_strategy_id` index
- [x] `up.sql`: Simplify `idx_documents_lineage` — removed entirely (was compound on strategy_id + initiative_id, now just `idx_documents_initiative_id`)
- [x] `down.sql`: Reverse migration (add `strategy_id` column back with NULL values)
- [x] `schema.rs` updated — already done in T-0091
- [x] `models.rs` updated — already done in T-0091
- [x] `repository.rs` updated — already done in T-0091
- [x] Migration runs cleanly on existing v1 databases

## Implementation Notes

SQLite doesn't support `ALTER TABLE DROP COLUMN` in older versions. The standard pattern is: create new table → copy data → drop old → rename. Check if the diesel migration framework handles this or if we need to write the SQL manually.

The `strategy_id` on initiative/task rows should be set to NULL or simply dropped during the copy — the data is meaningless once strategies are gone.

## Status Updates

### Session 1 — 2026-03-03

**All acceptance criteria met. All 182 tests pass.**

#### Created
- `migrations/008_remove_strategy_id/up.sql` — Deletes strategy documents from all tables (documents, tags, relationships, search), drops strategy indexes, recreates documents table without `strategy_id` column using SQLite table-recreation pattern, rebuilds indexes and FTS triggers
- `migrations/008_remove_strategy_id/down.sql` — Reverse migration that adds `strategy_id` column back (NULL for all rows)

#### Already done in T-0091
- `schema.rs` — `strategy_id` already removed from diesel table definition
- `models.rs` — `strategy_id` already removed from `Document` and `NewDocument` structs
- `repository.rs` — strategy query methods already removed

#### Notes
- Migration runs automatically via diesel's embedded migrations when `Database::new()` is called
- The compound `idx_documents_lineage` (strategy_id, initiative_id) was removed entirely; `idx_documents_initiative_id` remains as standalone index
- Migration safely handles v1 databases that have strategy documents by cleaning up related tables first

#### Verification
- `angreal test-core` — 182 tests pass (155 unit + 27 integration)