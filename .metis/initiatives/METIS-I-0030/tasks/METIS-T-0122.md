---
id: build-post-commit-hook-to-flush
level: task
title: "Build post-commit hook to flush overlay to main"
short_code: "METIS-T-0122"
created_at: 2026-03-29T23:01:30.404678+00:00
updated_at: 2026-03-30T00:53:07.528186+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Build post-commit hook to flush overlay to main

## Parent Initiative

[[METIS-I-0030]]

## Objective

Build a `post-commit` git hook that detects pending `.metis/` changes in the overlay and flushes them as a single commit on main using git plumbing commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Shell script that checks if `.metis/.pending/` has any files
- [ ] If empty, exits silently (no-op)
- [ ] If on main already, moves overlay files into `.metis/`, stages, and amends/creates commit
- [ ] If on feature branch: uses git plumbing to commit to main without checkout:
  - `git hash-object -w` each overlay file to get blob OIDs
  - Read main's current tree, apply overlay changes to build new tree
  - `git commit-tree` with parent = main's HEAD
  - `git update-ref refs/heads/main <new-commit>`
- [ ] Handles tombstone files (`.deleted` markers) by removing entries from main's tree
- [ ] Clears `.metis/.pending/` after successful flush
- [ ] Commit message: `metis: sync document changes`
- [ ] Hook is idempotent — safe to run multiple times
- [ ] Does not interfere with the user's commit (runs after it completes)

## Implementation Notes

- Could be a shell script or a `metis flush` subcommand invoked by the hook
- A Rust implementation via `metis flush` using git2 would be more robust than shell plumbing
- Consider: what happens if main has diverged (remote has new commits)? The hook only updates the local ref — push handles remote sync
- Blocked by: METIS-T-0120 (needs overlay directory structure defined)

## Status Updates

- Implemented as `metis flush` CLI subcommand in `crates/metis-docs-cli/src/commands/flush.rs`
- Uses git2 to build a new tree from main's tree + overlay files - tombstones
- `FlushCommand::execute()`: finds .metis workspace, opens repo, resolves main branch, collects overlay contents, builds merged tree, commits to main's ref
- `build_merged_tree()`: reads all blobs from main's tree, applies overlay additions/updates, removes tombstoned entries
- `build_tree_from_entries()`: recursively builds nested git tree structure from flat path→content map
- Shell hook at `crates/metis-docs-cli/src/hooks/post-commit`: just calls `metis flush`, fails silently if metis not available
- Marker comment `METIS_POST_COMMIT_HOOK` for idempotent install detection
- Cleans up `.metis/.pending/` after successful flush
- git2 and walkdir added to metis-docs-cli dependencies
- Full workspace compiles clean