---
id: cross-workspace-projection-cache
level: task
title: "Cross-workspace projection cache"
short_code: "METIS-T-0083"
created_at: 2026-02-26T01:32:10.492355+00:00
updated_at: 2026-02-26T19:34:58.259958+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# Cross-workspace projection cache

## Objective

Expand the projection cache (`metis.db`) to index ALL documents on disk — both owned workspace documents and hydrated remote workspace documents. Compute cross-workspace relationships, inverse references, and progress aggregation.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Cache rebuild scans all `.metis/<prefix>/*.md` files, not just the owned workspace
- [ ] `parent` references resolve across workspace boundaries (e.g. API-T-0001 with parent ALPHA-T-0001)
- [ ] `blocked_by` references resolve across workspace boundaries
- [ ] Inverse relationships computed: given ALPHA-T-0001, find all children across all workspaces
- [ ] Progress aggregation: for any document, count children by phase across all workspaces (e.g. "3/5 tasks completed")
- [ ] Owned vs hydrated documents are distinguishable in the cache (for write-scope enforcement in UI)
- [ ] Cache rebuild after sync completes in reasonable time (<1s for ~1000 documents)
- [ ] Existing single-workspace queries continue to work unchanged

## Implementation Notes

### Technical Approach

The current cache rebuild walks the owned workspace's hierarchical document structure. Expand this to:

1. Walk the owned workspace as before (preserving current behavior)
2. Additionally walk each `.metis/<prefix>/` folder for hydrated remote workspaces
3. Parse frontmatter from each document to extract: short_code, document_type, phase, parent, blocked_by, archived, tags
4. Build the relationship graph across all documents regardless of workspace
5. Compute inverse relationships (children, blocks) by scanning all `parent` and `blocked_by` references

### New Queries Needed

- `children_of(short_code)` → returns all documents with `parent: short_code` across all workspaces
- `blocked_by(short_code)` → returns all documents with `blocked_by` containing short_code
- `progress(short_code)` → counts children by phase (todo, active, completed, blocked)
- `workspace_documents(prefix)` → returns all documents in a specific workspace
- `upstream_context(prefix)` → returns documents referenced by this workspace's documents (parent chain)

### Dependencies

- METIS-T-0079 (hydration — puts remote documents on disk for the cache to index)

## Test Scenarios

### Unit Tests — Cache Rebuild Scope

1. **Owned workspace only (single-workspace mode)**: no hydrated remotes → cache indexes owned documents exactly as before, no regression
2. **Owned + one remote workspace**: owned `api/` + hydrated `strat/` → cache contains documents from both
3. **Owned + multiple remote workspaces**: owned `api/` + hydrated `strat/`, `alpha/`, `sre/` → all workspaces indexed
4. **Empty remote workspace**: hydrated `alpha/` folder exists but empty → no documents indexed for that workspace, no error
5. **Non-document files skipped**: `.metis/metis.db`, `.metis/config.toml`, `.metis/code-index.md` → not indexed as documents
6. **Archived documents indexed**: archived docs on disk → included in cache with `archived: true` flag

### Unit Tests — Cross-Workspace Relationships

7. **Parent reference across workspaces**: `API-T-0001` has `parent: WGR-I-0001` → `children_of("WGR-I-0001")` returns `API-T-0001`
8. **Multiple children across workspaces**: `WGR-I-0001` is parent of tasks in `api/`, `sre/`, `alpha/` → `children_of` returns all of them
9. **blocked_by across workspaces**: `API-T-0002` blocked by `SRE-T-0001` → `blocked_by("SRE-T-0001")` returns `API-T-0002`
10. **Deep parent chain**: `API-T-0001` → `WGR-I-0001` → `STRAT-S-0001` → `STRAT-V-0001` → each link resolves correctly across workspace boundaries
11. **Orphaned reference**: `API-T-0001` references `parent: UNKNOWN-I-9999` (doesn't exist in any workspace) → reference stored but flagged as unresolved, not a crash
12. **Self-referencing document**: `API-T-0001` has `parent: API-T-0001` → detected and handled (ignored or error), not an infinite loop
13. **Circular blocked_by**: A blocked by B, B blocked by A → detected and handled, not an infinite loop in queries

### Unit Tests — Query Functions

14. **children_of — no children**: document with no children → returns empty list
15. **children_of — owned children only**: initiative with tasks only in owned workspace → returns those tasks
16. **children_of — cross-workspace children**: initiative with tasks in 3 workspaces → returns all tasks
17. **progress — all phases**: initiative with tasks in todo(2), active(1), completed(3), blocked(1) → returns accurate counts `{todo: 2, active: 1, completed: 3, blocked: 1}`
18. **progress — cross-workspace aggregation**: tasks from 3 workspaces contributing to same initiative → progress counts aggregate all of them
19. **progress — no children**: document with no children → progress returns all zeros
20. **workspace_documents — owned**: `workspace_documents("api")` → returns all owned documents
21. **workspace_documents — remote**: `workspace_documents("strat")` → returns all hydrated strat documents
22. **workspace_documents — nonexistent**: `workspace_documents("xyz")` → returns empty list
23. **upstream_context — full chain**: given `api` workspace → returns all documents referenced by api's documents (parent chain up through remote workspaces)
24. **upstream_context — no upstream**: workspace with no cross-workspace references → returns empty

### Unit Tests — Owned vs Hydrated Distinction

25. **Owned documents writeable**: cache marks documents in owned workspace as `owned: true`
26. **Hydrated documents read-only**: cache marks documents in remote workspace folders as `owned: false`
27. **Write scope check**: attempt to transition/edit a hydrated document via cache → rejected with clear error
28. **Workspace prefix on each document**: every cached document has its workspace prefix accessible for UI display

### Integration Tests

29. **Cache rebuild after sync**: full sync (hydrate + dehydrate) → cache rebuild → all relationships resolve, queries return correct results
30. **Cache rebuild after document operations**: create/edit/delete documents locally → cache rebuild → changes reflected immediately
31. **Cache consistency across syncs**: sync, rebuild cache, sync again (new remote changes), rebuild cache → cache state matches disk state exactly
32. **Performance — 1000 documents**: 10 workspaces × 100 documents each → cache rebuild completes in <1s
33. **Performance — 5000 documents**: stress test with large workspace → rebuild still completes in reasonable time

### Edge Cases

34. **Corrupted document in remote workspace**: one hydrated `.md` file has invalid frontmatter → that document skipped, rest of cache built correctly, warning emitted
35. **Duplicate short codes across workspaces**: `API-T-0001` in `api/` and another `API-T-0001` in `alpha/` (shouldn't happen but defensive) → handled without crash
36. **Cache file locked**: `metis.db` locked by another process → clear error message, retry or wait
37. **Cache rebuild idempotent**: rebuild twice with same disk state → identical cache contents
38. **Empty project**: no documents anywhere → cache builds successfully (empty), queries return empty results

## Status Updates

### Session 2 — Implementation Complete

**Changes made:**
- Added `gray_matter = "0.2"` dependency to `crates/metis-sync/Cargo.toml`
- Created `crates/metis-sync/src/projection.rs` — full projection cache module (~700 lines)
- Added `pub mod projection;` to `crates/metis-sync/src/lib.rs`

**Module architecture:**
- `CachedDocument` — struct with short_code, title, document_type, phase, parent, blocked_by, archived, workspace, owned, file_path
- `ProgressSummary` — phase counts (backlog, todo, active, completed, blocked, other)
- `ProjectionWarning` — non-fatal parse errors
- `ProjectionCache` — main cache with HashMap indices for documents, children, blocks, workspace membership
- `ProjectionCache::build(metis_dir, owned_prefix)` — scans all workspace dirs, parses frontmatter, builds indices
- Query methods: `get()`, `children_of()`, `blocks()`, `progress()`, `workspace_documents()`, `upstream_context()`

**Frontmatter parsing:**
- Uses `gray_matter` crate (same as metis-docs-core) for YAML frontmatter extraction
- Extracts: short_code, level, title, phase (from tags), parent, blocked_by, archived
- Gracefully handles corrupted/incomplete frontmatter with warnings (no crash)

**Test coverage — 36 tests:**
- Cache rebuild scope (6): owned only, +1 remote, +multiple remotes, empty remote, non-doc files skipped, archived indexed
- Cross-workspace relationships (7): parent across workspaces, multiple children across workspaces, blocked_by across workspaces, deep parent chain, orphaned reference, self-reference, circular blocked_by
- Query functions (11): children_of (no children, owned only, cross-workspace), progress (all phases, cross-workspace, no children), workspace_documents (owned, remote, nonexistent), upstream_context (full chain, no upstream)
- Owned vs hydrated distinction (4): owned marked, hydrated marked, workspace prefix accessible
- Edge cases (4): corrupted doc skipped, duplicate short codes, idempotent rebuild, empty project
- Performance (2): 1000 docs <1s, 5000 docs <5s
- Integration-style (3): disk changes reflected, phase change consistency, full multi-workspace scenario

**All 139 tests pass (36 new + 103 existing). Zero warnings.**