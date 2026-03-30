---
id: introduce-storagebackend-and-make
level: task
title: "Introduce StorageBackend and make FilesystemService stateful"
short_code: "METIS-T-0118"
created_at: 2026-03-29T23:01:26.306989+00:00
updated_at: 2026-03-30T00:39:35.991451+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Introduce StorageBackend and make FilesystemService stateful

## Parent Initiative

[[METIS-I-0030]]

## Objective

Refactor `FilesystemService` from a stateless struct with static methods into a stateful service that dispatches through a `StorageBackend` enum. On main → `Local` backend (current behavior, zero change). On non-main → `GitOverlay` backend (implemented in later tasks).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `StorageBackend` enum defined: `Local` and `GitOverlay { repo, main_ref, overlay_dir }`
- [ ] `FilesystemService` holds a `StorageBackend` instance
- [ ] All static methods converted to instance methods
- [ ] `Local` backend delegates to current `std::fs` implementations (no behavioral change)
- [ ] `GitOverlay` variant exists but can stub/panic for now (filled in by T-0119/T-0120)
- [ ] All 23 production call sites updated to use the instance
- [ ] Factory method: `FilesystemService::new(workspace_path)` that auto-detects backend via git module from T-0117
- [ ] All existing tests pass unchanged

## Implementation Notes

- This is the biggest refactoring task — touches every caller of `FilesystemService`
- The `SyncService` and workspace services will need to receive or construct a `FilesystemService` instance
- Consider whether `FilesystemService` should be passed through or stored in a shared context
- Blocked by: METIS-T-0117 (needs branch detection to choose backend)

## Status Updates

- `StorageBackend` enum with `Local` and `GitOverlay { git_repo, overlay_dir }` variants
- `FilesystemService` now holds a `StorageBackend` field
- `new(workspace_path)` factory auto-detects git repo and branch, selects backend
- `local()` constructor for tests and non-git contexts
- All static methods converted to `&self` instance methods (except `compute_content_hash` which stays static)
- `SyncService` now holds a `fs: FilesystemService` field, auto-created in `with_workspace_dir()`
- All 23+ production call sites in synchronization.rs updated from `FilesystemService::method()` to `self.fs.method()`
- Test helper calls updated to use `FilesystemService::local()`
- 176 lib tests pass, full workspace compiles clean