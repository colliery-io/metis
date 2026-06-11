---
id: objectstore-trait-with-db-blob-and
level: task
title: "ObjectStore trait with db-blob and filesystem backends"
short_code: "METIS-T-0128"
created_at: 2026-06-11T13:09:26.508688+00:00
updated_at: 2026-06-11T13:09:26.508688+00:00
parent: METIS-I-0031
blocked_by: ["METIS-T-0126"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# ObjectStore trait with db-blob and filesystem backends

## Parent Initiative

[[METIS-I-0031]]

## Objective

Implement the content-addressed `ObjectStore` abstraction from the initiative design (D1): the trait, the db-blob and filesystem backends, a shared conformance test suite, and the GC sweep for unreferenced objects. The s3 backend is a later task.

## Acceptance Criteria

- [ ] `trait ObjectStore { get, put, delete, list }` in `metis-core`; `put` computes and returns the sha256 hex key and is idempotent (same content → same key, no duplicate work)
- [ ] db-blob backend over the `objects` table, working on both database backends; participates in the caller's transaction where one exists
- [ ] Filesystem backend under a configured root with sharded layout (`ab/cdef…`), atomic writes (temp file + rename)
- [ ] One conformance test suite (round-trip, idempotent put, missing-key get, delete, list) run against both backends
- [ ] GC function: deletes objects referenced by neither any `work_items.content_key` nor any content key recorded in `events` payloads; dry-run mode; covered by tests
- [ ] Write-ordering contract (object written before row commit; orphans are harmless and GC-able) documented on the trait

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

*To be added during implementation*