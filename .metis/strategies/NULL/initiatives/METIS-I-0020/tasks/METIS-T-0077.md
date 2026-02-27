---
id: flat-document-layout-for-central
level: task
title: "Flat document layout for central repo"
short_code: "METIS-T-0077"
created_at: 2026-02-26T01:32:05.055404+00:00
updated_at: 2026-02-26T02:13:23.185875+00:00
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

# Flat document layout for central repo

## Objective

Define and implement the flat-by-workspace document layout used in the central repo. Documents are stored as `prefix/SHORT-CODE.md` — one folder per workspace, flat within each folder.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Define the mapping: local `.metis/` hierarchical structure ↔ central flat `prefix/*.md` layout
- [ ] Implement serialization: given a local workspace's documents, produce the flat layout for central
- [ ] Implement deserialization: given a flat folder from central, map files into local `.metis/` structure
- [ ] Owned workspace folder contains all document types (visions, initiatives, tasks, ADRs) as flat `.md` files
- [ ] Short codes are the filenames (e.g. `api/API-T-0001.md`)
- [ ] Non-document files (`config.toml`, `metis.db`, `code-index.md`, etc.) are never included in the central layout
- [ ] Central `config.toml` (if it exists) is reserved and not overwritten by any workspace

## Implementation Notes

### Layout

```
central/.metis/
  config.toml              # central config (reserved)
  strat/
    STRAT-V-0001.md
    STRAT-S-0001.md
  api/
    API-V-0001.md
    API-I-0001.md
    API-T-0001.md
  sre/
    SRE-V-0001.md
    SRE-T-0001.md
```

Local Metis currently stores documents in a hierarchy (`strategies/X/initiatives/Y/tasks/Z.md`). The central layout flattens this — all documents for a workspace are peers in one folder. The hierarchical relationships are in `parent` frontmatter fields, not in filesystem nesting.

### Technical Approach

- New module in `metis-docs-core` for layout mapping (local ↔ central)
- Flatten: walk the local hierarchy, collect all `.md` document files, output as `prefix/SHORT-CODE.md`
- Unflatten: given a flat folder, read each `.md` file's frontmatter to determine document type and parent, place into local hierarchy
- The flattening/unflattening is used by hydration (METIS-T-0079) and dehydration (METIS-T-0080)

### Dependencies

- METIS-T-0076 (config.toml schema — needs `workspace_prefix`)

## Test Scenarios

### Unit Tests — Flatten (Local → Central)

1. **Single document**: one vision doc in local hierarchy → `prefix/PREFIX-V-0001.md` in flat output
2. **Multiple document types**: vision + initiative + tasks → all appear as flat files in `prefix/` folder, no subdirectories
3. **Nested hierarchy preserved in frontmatter**: task nested under initiative in local hierarchy → flat file has correct `parent: PREFIX-I-0001` in frontmatter
4. **ADR documents included**: ADRs (no parent) → flattened alongside other documents
5. **Backlog items included**: standalone backlog tasks → flattened with correct frontmatter
6. **Empty workspace**: no documents → empty prefix folder (folder still created)
7. **Non-document files excluded**: `config.toml`, `metis.db`, `code-index.md`, `code-index-hashes.json` → none appear in flat output
8. **Hidden files excluded**: `.gitignore`, `.index-dirty` → not included
9. **Archived documents**: archived docs in local hierarchy → included in flat output (central stores all state)
10. **Document with special characters in title**: titles with quotes, colons → filename is short code (safe), content preserved

### Unit Tests — Unflatten (Central → Local)

11. **Vision document**: `prefix/PREFIX-V-0001.md` → placed at correct location in local hierarchy
12. **Task with parent**: flat file with `parent: PREFIX-I-0001` → placed under the correct initiative in local hierarchy
13. **Orphaned task**: flat file with `parent: UNKNOWN-I-0001` (parent not in this workspace) → placed in a reasonable default location, not lost
14. **All document types roundtrip**: flatten then unflatten every document type → original hierarchy reconstructed
15. **Missing frontmatter**: `.md` file with no YAML frontmatter → skipped with warning (not a Metis document)
16. **Corrupted frontmatter**: `.md` file with invalid YAML → skipped with warning, other files processed normally
17. **Duplicate short codes**: two files with same short code → error reported, second file not silently dropped

### Integration Tests — Roundtrip

18. **Full roundtrip — simple project**: create project with vision + initiative + 3 tasks → flatten → unflatten → all documents identical (content, frontmatter, hierarchy)
19. **Full roundtrip — complex project**: multiple visions, initiatives, tasks, ADRs, backlog items, archived docs → flatten → unflatten → all documents preserved
20. **Incremental roundtrip**: flatten, add a new document locally, re-flatten → new document appears, existing documents unchanged
21. **Delete roundtrip**: flatten, delete a document locally, re-flatten → deleted document absent from flat output

### Edge Cases

22. **Large workspace**: 500+ documents → flattening completes without excessive memory or time
23. **Binary content in markdown**: document with embedded base64 images or binary-like content → preserved through flatten/unflatten
24. **Very long file paths**: deeply nested local hierarchy → flat path is always short (`prefix/SHORT-CODE.md`)
25. **Unicode in document content**: documents with CJK characters, emoji, RTL text → preserved through roundtrip
26. **Empty document**: `.md` file with frontmatter but no body content → handled correctly both directions
27. **Filename vs short code mismatch**: flat file named `PREFIX-T-0001.md` but frontmatter says `short_code: PREFIX-T-0002` → which wins? Define and test the rule

## Status Updates

### Session 1 — Implementation Complete

**New file**: `crates/metis-docs-core/src/application/services/layout.rs`
- Core types: `FlatDocument`, `FlattenResult`, `ReadFlatResult`
- `flatten_workspace(workspace_dir)` — walks local `.metis/` hierarchy, extracts `short_code` from YAML frontmatter via `gray_matter`, returns flat documents. Skips `archived/`, non-document files (`config.toml`, `metis.db`, `code-index.*`), and files without valid `short_code` frontmatter.
- `read_flat_documents(source_dir)` — reads flat `.md` files from a single directory level (no recursion). Extracts `short_code` from frontmatter.
- `write_flat_documents(documents, target_dir)` — writes flat documents to target directory, creating it if needed.
- `remove_stale_files(target_dir, current_documents)` — removes `.md` files not in the current set. Only removes `.md` files, preserves non-md files.
- `extract_short_code(content)` / `extract_level(content)` — frontmatter parsing helpers.
- Deduplication: BTreeMap keyed by short_code, first file wins, duplicates reported in errors.

**Modified file**: `crates/metis-docs-core/src/application/services/mod.rs`
- Registered `layout` module
- Re-exported: `extract_level`, `extract_short_code`, `flatten_workspace`, `read_flat_documents`, `remove_stale_files`, `write_flat_documents`, `FlatDocument`, `FlattenResult`, `ReadFlatResult`

**Tests**: 31 unit tests covering:
- Flatten: single doc, multiple types, skip non-documents, skip archived, skip hidden files, duplicate short codes
- Read flat: basic read, skip non-md, skip missing frontmatter
- Write flat: creates directory, writes files
- Stale removal: removes absent files, preserves non-md
- Roundtrip: flatten → write → read preserves all documents
- Edge cases: unicode content, empty workspace, extract helpers

**Results**: All 31 tests pass. Full workspace compiles (`cargo check --workspace` clean). Full `metis-docs-core` test suite passes with zero regressions.

**Design decisions**:
- Archived documents excluded from flatten (central stores active state only)
- Filename derived from short_code (e.g. `API-T-0001.md`), not from title
- Frontmatter `short_code` is authoritative — if filename disagrees with frontmatter, frontmatter wins
- `read_flat_documents` is non-recursive (one level only) since central layout is flat per workspace