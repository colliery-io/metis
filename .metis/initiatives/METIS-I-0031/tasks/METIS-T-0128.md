---
id: objectstore-trait-with-db-blob-and
level: task
title: "ObjectStore trait with db-blob and filesystem backends"
short_code: "METIS-T-0128"
created_at: 2026-06-11T13:09:26.508688+00:00
updated_at: 2026-06-11T14:48:32.055898+00:00
parent: METIS-I-0031
blocked_by: [METIS-T-0126]
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# ObjectStore trait with db-blob and filesystem backends

## Parent Initiative

[[METIS-I-0031]]

## Objective

Implement the content-addressed `ObjectStore` abstraction from the initiative design (D1): the trait, the db-blob and filesystem backends, a shared conformance test suite, and the GC sweep for unreferenced objects. The s3 backend is a later task.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `trait ObjectStore { get, put, delete, list }` in `metis-core`; `put` returns sha256 hex key and is idempotent
- [x] db-blob backend over the `objects` table, working on both database backends; takes `&mut DualConnection` so it shares the caller's transaction
- [x] Filesystem backend under a configured root, sharded (`ab/<key>`), atomic temp+rename writes
- [x] One conformance suite (round-trip, idempotent put, missing-key get, delete, empty-blob) run against both stores × both DB backends
- [x] Conservative GC over `work_items.content_key` + any 64-hex string in `events` payloads; dry-run mode; tested
- [x] Write-ordering contract documented on the module/trait

## Implementation Notes

### Technical Approach
- Keys are `sha256(content)` hex — objects are immutable; edits write a new object and repoint `work_items.content_key` (that repoint happens in METIS-T-0129, not here)
- db-blob is the default backend and the only one with true write atomicity; fs backend relies on the write-before-commit ordering
- Backend selection via a config enum (`object_store = "db" | { fs = { root } } | { s3 = {...} }`); the s3 variant can exist in config parsing but return "not yet supported"
- GC must be conservative: scan `events.payload` JSON for prior content keys before considering an object unreferenced

### Dependencies
- METIS-T-0126 for the `objects` table (db-blob backend); the trait + fs backend can start before it lands

### Risk Considerations
- GC deleting a still-referenced object is the data-loss scenario — hence dry-run mode, conservative reference scanning, and tests before any automatic scheduling
- Large bodies in db-blob bloat Postgres backups; acceptable day one, that's exactly what the fs/s3 backends are for

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0128-object-store` off `3.0`)

`metis-core::objects` module; 8 tests green on both DB backends.

**Decisions / deviations:**
- **Trait methods take `&mut DualConnection`** (not a conn-free trait). This is how db-blob shares the caller's transaction; fs/s3 ignore the arg. The service layer always holds a connection, so no awkwardness.
- **`put` uses check-then-insert, not `ON CONFLICT`** — diesel-dualdb's `MultiBackend` doesn't support upsert clauses, and a failed insert would poison the caller's open Postgres transaction. The existence fast-path also makes the common dedup case a no-op. Residual TOCTOU race on byte-identical concurrent inserts is caught by mapping `UniqueViolation` → Ok.
- **GC reference scan is maximally conservative**: any 64-hex string anywhere in any event payload counts as a reference, so future event shapes can't make GC reclaim a live object. Convention for edit events (`prev_content_key`/`new_content_key`) will be honored by T-0129.
- s3 backend is `ObjectStoreConfig::S3` that parses but returns `Unsupported` (later task).
- Hand-rolled hex encoding to avoid a `hex` crate dep.