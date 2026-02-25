---
id: implement-incremental-re-indexing
level: task
title: "Implement incremental re-indexing with content hash tracking"
short_code: "METIS-T-0075"
created_at: 2026-02-20T14:47:14.905539+00:00
updated_at: 2026-02-20T14:47:14.905539+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

- [ ] `.metis/code-index-hashes.json` created on first full index
- [ ] Hash file maps file paths to content hashes (SHA-256 or similar)
- [ ] `metis index --incremental` skips files with unchanged hashes
- [ ] Changed files are re-parsed and their symbols updated in the output
- [ ] Deleted files are removed from the index
- [ ] New files are added to the index
- [ ] Stats report shows files skipped vs re-indexed
- [ ] Tests covering add/modify/delete scenarios
- [ ] `angreal test` passes

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

*Updated during implementation*