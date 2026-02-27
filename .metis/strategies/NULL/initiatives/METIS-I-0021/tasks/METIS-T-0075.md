---
id: implement-incremental-re-indexing
level: task
title: "Implement incremental re-indexing with content hash tracking"
short_code: "METIS-T-0075"
created_at: 2026-02-20T14:47:14.905539+00:00
updated_at: 2026-02-25T05:27:44.009221+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0021
---

# Implement incremental re-indexing with content hash tracking

## Parent Initiative
[[METIS-I-0021]]

## Objective

Add content hash tracking so `metis index --incremental` only re-parses changed files. Store hashes in `.metis/code-index-hashes.json`. On re-index, compare hashes, skip unchanged files, regenerate symbols for changed files, and trigger summary regeneration only for affected directories.

## Acceptance Criteria

## Acceptance Criteria

- [x] `.metis/code-index-hashes.json` created on first full index
- [x] Hash file maps file paths to content hashes (BLAKE3)
- [x] `metis index --incremental` skips files with unchanged hashes
- [x] Changed files are re-parsed and their symbols updated in the output
- [x] Deleted files are removed from the index
- [x] New files are added to the index
- [x] Stats report shows files skipped vs re-indexed
- [x] Tests covering add/modify/delete scenarios
- [x] `angreal test` passes

## Implementation Notes

Hash computation: use `sha2` crate or `blake3` for speed. Store as `{ "src/auth/handler.rs": "abc123...", ... }` in JSON. On incremental run:
1. Load existing hashes
2. Walk files, compute hashes
3. Compare: unchanged → skip, changed → re-parse, missing from disk → remove from index, new → parse and add
4. Write updated hashes
5. Regenerate markdown output

For Layer 2 summaries: flag directories with changed files so the skill knows which summaries to regenerate.

Blocked by: METIS-T-0071 (needs the full pipeline working first)

## Progress

### Session 1 — hasher.rs module
- Created `crates/metis-code-index/src/hasher.rs` with `HashManifest` (BLAKE3 hashing, load/save/diff/update) and `IncrementalDiff` structs
- Added `blake3 = "1"` to metis-code-index Cargo.toml
- 12 tests covering hash file, save/load, diff scenarios, manifest update, affected directories

### Session 2 — Full incremental wiring
- Added `SymbolCache` to hasher.rs (load/save/update/to_path_map/from_path_map) with 4 tests (total: 16 tests)
- Wired incremental logic into CLI `commands/index.rs`:
  - Full index now saves hash manifest + symbol cache for future incremental runs
  - `--incremental` loads manifest, computes diff, parses only changed files, uses cached symbols for unchanged
  - Reports stats: files re-indexed vs skipped
  - Early return with cached symbols when nothing changed
- Wired same logic into MCP `tools/index_code.rs` with incremental stats in response table
- 3 new CLI tests: hash file creation, incremental skips unchanged, incremental detects changes
- All tests pass via `angreal test`