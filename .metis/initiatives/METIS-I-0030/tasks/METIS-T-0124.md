---
id: route-remaining-direct-std-fs
level: task
title: "Route remaining direct std::fs calls through FilesystemService"
short_code: "METIS-T-0124"
created_at: 2026-03-29T23:01:32.357677+00:00
updated_at: 2026-03-30T00:58:09.240421+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Route remaining direct std::fs calls through FilesystemService

## Parent Initiative

[[METIS-I-0030]]

## Objective

Route the remaining direct `std::fs` calls in production code through `FilesystemService` so the git overlay backend covers all document I/O paths.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `synchronization.rs:247` (`std::fs::read_to_string` in `extract_document_short_code`) → use `FilesystemService::read_file`
- [ ] `synchronization.rs:474` (`std::fs::rename` in `rename_file_for_number_change`) → add `rename_file` method to `FilesystemService`
- [ ] `synchronization.rs:781` (`std::fs::read_to_string` in `recover_id_counter`) → use `FilesystemService::read_file`
- [ ] `reassignment.rs:240,247` (`fs::create_dir_all` + `fs::rename`) → add `create_dir` and `rename_file` to `FilesystemService`
- [ ] `migration.rs` — assess and leave as-is if migration only runs on main (document decision)
- [ ] No production `std::fs` calls on `.metis/` document files remain outside `FilesystemService`
- [ ] Existing tests pass

## Implementation Notes

- `rename_file` on `GitOverlay` backend: read source from overlay/main, write to new path in overlay, tombstone old path
- `create_dir` on `GitOverlay`: no-op (git doesn't track directories, overlay dirs created on write)
- Migration code (`migration.rs`) is for one-time schema migrations that should only ever run on main — probably safe to leave as direct fs calls. Document this decision.
- Can be done in parallel with T-0122/T-0123 since it's independent refactoring

## Status Updates

- `synchronization.rs:extract_document_short_code` converted from static to `&self` method, now uses `self.fs.read_file()`
- `synchronization.rs:rename_file_for_number_change` now uses `self.fs.rename_file()`
- `synchronization.rs:recover_id_counter` now uses `self.fs.read_file()`
- Added `rename_file()` to `FilesystemService`: Local uses `fs::rename`, GitOverlay does read→write→tombstone
- `ReassignmentService`: added `fs: FilesystemService` field, constructor auto-detects backend, `move_file()` now uses `self.fs.rename_file()`
- `migration.rs`: left as-is — one-time schema migration that only runs on main (documented decision)
- Zero `std::fs` calls remaining in synchronization.rs or reassignment.rs production code
- 191 lib tests pass, full workspace compiles clean