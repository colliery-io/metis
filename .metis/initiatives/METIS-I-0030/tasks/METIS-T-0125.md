---
id: route-documentcreationservice
level: task
title: "Route DocumentCreationService through FilesystemService"
short_code: "METIS-T-0125"
created_at: 2026-03-31T15:23:15.284457+00:00
updated_at: 2026-03-31T15:31:50.947781+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Route DocumentCreationService through FilesystemService

## Parent Initiative

[[METIS-I-0030]]

## Objective

`DocumentCreationService` (`creation.rs`) uses raw `std::fs` for all file operations — `exists()`, `create_dir_all()`, `fs::write` (via templates). On a feature branch with GitOverlay, this means:
- `file_path.exists()` returns false for files that exist in main's git tree (wrong "not found" for parent initiatives, wrong "doesn't exist" allowing duplicate creates)
- `fs::create_dir_all` creates directories on disk instead of in overlay
- Template writes go to disk instead of overlay
- Parent initiative validation at line 264 checks disk, not overlay/git tree

This breaks the core create-task-under-initiative flow on feature branches.

## Acceptance Criteria

## Acceptance Criteria

- [ ] `DocumentCreationService` holds a `FilesystemService` instance
- [ ] All `file_path.exists()` calls routed through `self.fs.file_exists()`
- [ ] All `fs::create_dir_all()` calls routed through `self.fs` (add `create_dir_all` method if needed)
- [ ] All file writes (template rendering output) routed through `self.fs.write_file()`
- [ ] Parent initiative validation (line 264) uses `self.fs.file_exists()` instead of `initiative_file.exists()`
- [ ] Constructor updated: `DocumentCreationService::new(workspace_dir)` creates `FilesystemService::new(workspace_dir)`
- [ ] Existing tests pass
- [ ] Test: create a task under an initiative that exists only in main's git tree (from a feature branch)

## Implementation Notes

- ~15 direct `std::fs` call sites in creation.rs production code
- Need to add `create_dir_all` to `FilesystemService` — on `Local` delegates to `std::fs::create_dir_all`, on `GitOverlay` it's a no-op (directories are created implicitly by `write_file`)
- Template rendering currently returns content as a string, then `fs::write` writes it — just replace with `self.fs.write_file()`
- The `exists()` checks serve two purposes: "does this already exist?" (reject duplicate) and "does parent exist?" (validate parent). Both need overlay-awareness.

## Status Updates

- Added `fs: FilesystemService` field to `DocumentCreationService`, auto-detected in `new()`
- Replaced all `file_path.exists()` → `self.fs.file_exists()` (6 production call sites)
- Replaced all `fs::create_dir_all()` → `self.fs.create_dir_all()` (5 call sites)
- Added `create_dir_all()` to `FilesystemService` — Local delegates to `std::fs`, GitOverlay is no-op
- Replaced all `doc.to_file(path)` → `doc.to_content()` + `self.fs.write_file()` (5 document types)
- Replaced `get_next_adr_number` `fs::read_dir` → `self.fs.find_markdown_files()` for overlay awareness
- Fixed Send/Sync issue: `StorageBackend::GitOverlay` no longer holds `GitRepo` directly, repo is opened lazily via `open_git_repo()` per-operation
- `to_tree_path()` now takes `&GitRepo` parameter instead of reading from enum
- Removed `use std::fs` from production imports (kept in test module)
- 191 lib tests pass, full workspace compiles clean (including MCP async contexts)