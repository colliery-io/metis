---
id: vscode-viewer-backend
level: task
title: "VSCode viewer backend"
short_code: "METIS-T-0110"
created_at: 2026-03-26T14:59:09.854055+00:00
updated_at: 2026-03-26T14:59:09.854055+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# VSCode viewer backend

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Implement the VSCode viewer backend that opens Metis documents in VSCode via the `code` CLI.

## Acceptance Criteria

- [ ] `VscodeViewer` struct implements `DocumentViewer` trait (both `open` and `is_open`)
- [ ] `is_open` checks if file is already open in a VSCode editor tab (e.g., via `code --status` or VSCode CLI introspection)
- [ ] Single file open via `code <file>` works, skipped if already open
- [ ] Multi-file open via `code <file1> <file2> ...` works (for include_children)
- [ ] Graceful error if `code` command is not found on PATH
- [ ] Opens files in the existing VSCode window associated with the project workspace if one exists
- [ ] Tests cover: single file, multi-file, missing `code` binary

## Implementation Notes

### Technical Approach
- Shell out via `std::process::Command::new("code")` with file path args
- Use `--reuse-window` flag to avoid spawning new windows when a project window exists
- Check `which code` or equivalent to detect availability before attempting
- For multi-file, pass all paths as args in one command

### Dependencies
- METIS-T-0107 (viewer trait)

## Status Updates

*To be added during implementation*