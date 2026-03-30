---
id: implement-git2-read-path-blob
level: task
title: "Implement git2 read path (blob lookup from main's tree)"
short_code: "METIS-T-0119"
created_at: 2026-03-29T23:01:27.223212+00:00
updated_at: 2026-03-30T00:42:18.730816+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Implement git2 read path (blob lookup from main's tree)

## Parent Initiative

[[METIS-I-0030]]

## Objective

Implement the git2-based read path for the `GitOverlay` backend. `read_file` resolves blobs from main's committed tree — equivalent to `git show main:.metis/path/to/file`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `read_file` on `GitOverlay` backend resolves the path relative to `.metis/` in main's tree
- [ ] Uses `repo.find_reference(main_ref) → peel to commit → tree → get_path → blob`
- [ ] Returns blob content as UTF-8 string
- [ ] Returns appropriate error if file doesn't exist in main's tree
- [ ] `compute_file_hash` works by hashing the blob content directly
- [ ] `get_file_mtime` returns main's HEAD commit time (best available proxy)
- [ ] Tests: read a file committed on main from a feature branch

## Implementation Notes

- This is the pure git2 read path — overlay merging comes in T-0120
- Path resolution: callers pass absolute paths like `/project/.metis/initiatives/FOO/initiative.md`, need to strip prefix to get tree-relative path `.metis/initiatives/FOO/initiative.md`
- Blocked by: METIS-T-0118 (needs StorageBackend infrastructure)

## Status Updates

- Added `read_blob()`, `blob_exists()`, `list_markdown_files()`, `main_head_commit_time()`, `workdir()` to `GitRepo`
- `FilesystemService::read_file` on GitOverlay resolves absolute path → tree-relative path via `to_tree_path()`, then calls `git_repo.read_blob()`
- `file_exists` on GitOverlay checks `blob_exists()` on main's tree
- `get_file_mtime` on GitOverlay returns `main_head_commit_time()`
- `compute_file_hash` delegates to `read_file` + content hash (works for both backends)
- Added `to_tree_path()` and `to_overlay_path()` helpers for path resolution
- Added `StorageBackend::GitOverlay.workspace_dir` field for path stripping
- 14 git tests + 181 total lib tests passing