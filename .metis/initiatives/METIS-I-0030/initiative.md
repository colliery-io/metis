---
id: branch-independent-metis-storage
level: initiative
title: "Branch-independent .metis storage via git-backed filesystem layer"
short_code: "METIS-I-0030"
created_at: 2026-03-29T22:59:38.797801+00:00
updated_at: 2026-03-29T23:01:20.554158+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
initiative_id: branch-independent-metis-storage
---

# Branch-independent .metis storage via git-backed filesystem layer Initiative

## Context

Metis documents live in `.metis/` alongside code, so they follow whatever branch is checked out. This causes real problems:
- Task updates, backlog notes, and tech debt items get stranded on feature branches
- Switching branches loses visibility into active work
- Notes collected during feature work don't make it back to main

## Goals & Non-Goals

**Goals:**
- All `.metis/` reads and writes operate against main/master's state, regardless of current branch
- Changes accumulate as pending (not committed per-operation)
- Pending `.metis/` changes flush to a single commit on main via `post-commit` git hook — when you commit code, `.metis/` changes silently land on main
- Zero overhead when already on main (normal filesystem ops)
- Transparent to all consumers: CLI, MCP server, GUI, direct file editing

**Non-Goals:**
- Metis does not become git-aware beyond the filesystem layer — no branch UI, no merge logic
- No changes to the SQLite sync model
- No changes to document structure or frontmatter

## Architecture

### Approach: Git2 plumbing reads + filesystem overlay writes, post-commit flush

**On main/master:** Normal filesystem operations, no change from current behavior.

**On a non-main branch:**

1. **Reads** — `FilesystemService::read_file` uses `git2` to read blobs from main's tree, overlaid with any pending local changes from the write overlay
2. **Writes** — `FilesystemService::write_file` writes to a local overlay directory (e.g., `.metis/.pending/`) rather than the tracked `.metis/` paths
3. **find_markdown_files** — walks main's tree via `git2`, merged with overlay contents
4. **file_exists** — checks main's tree + overlay
5. **Hashing** — computed from content (works the same regardless of source)
6. **Flush** — a `post-commit` git hook detects dirty overlay, builds a single commit on main using git2 plumbing (hash-object → tree-builder → commit → update-ref), clears the overlay

### Read resolution order
1. Check overlay (`.metis/.pending/`) — if file exists there, return it (local write takes precedence)
2. Fall back to `git show main:.metis/<path>` via git2 blob lookup

### Key design decisions
- **Why not a worktree?** Can't have main checked out in two places simultaneously. Git2 plumbing avoids this.
- **Why an overlay instead of direct git writes?** Avoids a commit per operation. Changes batch naturally until the next `git commit`.
- **Why post-commit hook?** Natural batch point — you're already committing code, `.metis/` changes ride along as a separate commit on main. No extra workflow.

## Detailed Design

### Changes to `FilesystemService` (metis-docs-core)
- Add `git2` dependency to `metis-docs-core`
- Introduce a `StorageBackend` enum: `Local` (on main) vs `GitOverlay { repo, main_ref, overlay_dir }`
- `FilesystemService` becomes stateful (holds the backend) or receives it as a parameter
- Each method dispatches based on backend
- Initialization detects current branch and configures the backend

### Post-commit hook
- Shell script in `.git/hooks/post-commit` (or installable via `metis init`)
- Checks if `.metis/.pending/` has files
- If yes: uses `git` plumbing commands to create a commit on main with the overlay contents
- Clears the overlay after successful commit

### Direct `std::fs` call sites to address
- `synchronization.rs` lines 247, 474, 781 — direct reads/renames bypassing FilesystemService
- `reassignment.rs` lines 240, 247 — `fs::create_dir_all` and `fs::rename`
- `migration.rs` — extensive direct fs usage (but migration is a one-time main-branch operation, likely fine as-is)

### What doesn't change
- SQLite database and sync logic — it syncs from whatever FilesystemService returns
- Code-index generation — operates on source code, not `.metis/` documents
- MCP server, CLI, GUI — all go through the core library

## Alternatives Considered

1. **Symlink to persistent worktree of main** — Simple path remapping, but git can't have the same branch in two worktrees. Would need a shadow branch, adding complexity.
2. **Git hooks only (no Metis changes)** — post-checkout copies main's `.metis/`, pre-commit strips `.metis/` from feature commits. Fragile, doesn't handle reads from main's tree, race conditions on branch switch.
3. **Commit per operation via git2** — Clean but extremely noisy git history. Every `edit_document` call = a commit on main.
4. **Deferred worktree writes with batch commit** — Similar to chosen approach but using a real worktree. Blocked by git's same-branch-two-worktrees restriction.

## Implementation Plan

1. Add `git2` to `metis-docs-core`, implement branch detection and main-ref resolution
2. Introduce `StorageBackend` abstraction and refactor `FilesystemService` to be stateful
3. Implement git2-based read path (blob lookup from main's tree)
4. Implement overlay write path (`.metis/.pending/` directory)
5. Implement merged read resolution (overlay + main tree)
6. Implement `find_markdown_files` against main's tree + overlay
7. Build post-commit hook (flush overlay → commit on main)
8. Add `metis init` support for installing the hook
9. Route remaining direct `std::fs` calls through `FilesystemService`
10. Testing: unit tests for each backend path, integration test for full round-trip