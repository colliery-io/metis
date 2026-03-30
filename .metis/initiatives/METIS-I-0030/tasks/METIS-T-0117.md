---
id: add-git2-dependency-and-implement
level: task
title: "Add git2 dependency and implement branch detection"
short_code: "METIS-T-0117"
created_at: 2026-03-29T23:01:24.988042+00:00
updated_at: 2026-03-30T00:34:47.658760+00:00
parent: METIS-I-0030
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0030
---

# Add git2 dependency and implement branch detection

## Parent Initiative

[[METIS-I-0030]]

## Objective

Add the `git2` crate to `metis-docs-core` and implement utility functions to detect whether the repo is a git repo, find the main/master branch, and determine the current branch.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `git2` added to `metis-docs-core/Cargo.toml`
- [ ] Function to open the git repo from a workspace path (returns `Option` — `None` if not a git repo)
- [ ] Function to resolve the main branch name (`main` or `master`, whichever exists)
- [ ] Function to get the current HEAD branch name
- [ ] Function to determine if we're on main (`is_on_main()`)
- [ ] Unit tests for branch detection (using a temp git repo)

## Implementation Notes

- Place git utilities in a new `crates/metis-docs-core/src/dal/git/` module
- Use `Repository::discover()` to find the repo from any subdirectory
- Handle detached HEAD gracefully (treat as non-main)
- This is the foundation — no behavioral changes yet, just the detection layer

## Status Updates

- git2 v0.19 added to metis-docs-core/Cargo.toml
- Created `crates/metis-docs-core/src/dal/git/mod.rs` with `GitRepo` struct
- Functions: `discover()`, `main_branch()`, `current_branch()`, `is_on_main()`, `main_ref()`, `repo()`
- `resolve_main_branch_name()` checks local then remote refs for main/master
- 9 unit tests all passing