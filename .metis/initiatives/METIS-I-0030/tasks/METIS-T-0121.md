---
id: implement-find-markdown-files-and
level: task
title: "Implement find_markdown_files and file_exists against main tree + overlay"
short_code: "METIS-T-0121"
created_at: 2026-03-29T23:01:29.076423+00:00
updated_at: 2026-03-30T00:46:56.515768+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Implement find_markdown_files and file_exists against main tree + overlay

## Parent Initiative

[[METIS-I-0030]]

## Objective

Implement `find_markdown_files` and `file_exists` for the `GitOverlay` backend. Must produce a unified view by walking main's git tree and merging with overlay filesystem contents.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `find_markdown_files` walks main's tree via git2 `TreeWalkCallback`, collecting `.md` paths
- [ ] Overlay `.md` files are merged into the result (additions from overlay appear, tombstoned files excluded)
- [ ] `code-index.md` exclusion still applies
- [ ] `file_exists` checks overlay first (including tombstones), then main's tree
- [ ] Results use absolute paths matching current behavior (callers expect absolute paths)
- [ ] Tests: files only on main, files only in overlay, files in both, deleted via tombstone

## Implementation Notes

- git2 tree walking: `tree.walk(TreeWalkMode::PreOrder, callback)` to enumerate entries
- Need to filter to only `.metis/` subtree and only `.md` files
- Overlay walk uses existing `walkdir` logic on `.metis/.pending/`
- Merge: overlay additions are unioned, tombstones suppress main entries
- Blocked by: METIS-T-0120 (needs overlay infrastructure)

## Status Updates

- `find_markdown_files` on GitOverlay: walks main's tree via `list_markdown_files()`, merges overlay files via walkdir
- Tombstoned files excluded from main's results, overlay `.deleted` files skipped, `code-index.md` excluded
- Results are absolute paths (matching Local backend behavior), sorted for determinism
- Added path canonicalization (`canonical()` helper) to handle macOS `/var` vs `/private/var` symlink mismatch
- `to_tree_path()` and `to_overlay_path()` now canonicalize both sides before stripping prefixes
- 4 new overlay find tests: from main, with additions, excludes tombstoned, mixed scenario
- 191 total lib tests passing