---
id: implement-overlay-write-path-and
level: task
title: "Implement overlay write path and merged read resolution"
short_code: "METIS-T-0120"
created_at: 2026-03-29T23:01:28.255531+00:00
updated_at: 2026-03-30T00:44:09.284032+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Implement overlay write path and merged read resolution

## Parent Initiative

[[METIS-I-0030]]

## Objective

Implement the overlay write path and merged read resolution. Writes go to `.metis/.pending/` preserving relative directory structure. Reads check overlay first (local changes take precedence), then fall back to main's git tree.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `write_file` on `GitOverlay` writes to `.metis/.pending/<relative_path>` creating parent dirs as needed
- [ ] `read_file` checks `.metis/.pending/<relative_path>` first; if exists, returns overlay content
- [ ] `read_file` falls back to git2 blob lookup (from T-0119) if not in overlay
- [ ] `delete_file` creates a tombstone marker in overlay (e.g., `.metis/.pending/<path>.deleted`)
- [ ] `file_exists` checks overlay (including tombstones) before git tree
- [ ] Write → read round-trip works immediately without any git commit
- [ ] Tests: write on feature branch, read back from overlay; read unmodified file from main

## Implementation Notes

- Overlay directory structure mirrors `.metis/` structure: `.metis/.pending/initiatives/METIS-I-0001/initiative.md`
- Tombstones needed for delete operations — a file deleted in overlay shouldn't fall through to main's tree
- The overlay is ephemeral — cleared after post-commit hook flushes to main (T-0122)
- Blocked by: METIS-T-0119 (needs git2 read path for fallback)

## Status Updates

- `write_file` on GitOverlay writes to `.metis/.pending/<relative_path>`, creates parent dirs, removes any tombstone
- `read_file` checks overlay first (tombstone → error, overlay file → return, fallback → git2 blob)
- `delete_file` removes overlay file if present, creates tombstone (`.md.deleted` extension)
- `file_exists` checks tombstone (false), overlay (true), git tree (true/false)
- `is_tombstoned()` helper checks for `.deleted` extension tombstone files
- 6 overlay tests: read from main, write-then-read, write new file, delete tombstone, write-after-delete, exists checks both
- 187 total lib tests passing