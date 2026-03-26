---
id: system-editor-viewer-backend
level: task
title: "System editor viewer backend"
short_code: "METIS-T-0111"
created_at: 2026-03-26T14:59:09.879883+00:00
updated_at: 2026-03-26T17:06:42.954775+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# System editor viewer backend

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Implement the system editor viewer backend that opens Metis documents using `$EDITOR` or the OS default handler (`open` on macOS, `xdg-open` on Linux).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `SysEditorViewer` struct implements `DocumentViewer` trait (both `open` and `is_open`)
- [ ] `is_open` tracks spawned editor PIDs or session state to detect if a file is already being edited
- [ ] Reads `$EDITOR` env var and shells out to it with file path(s)
- [ ] Falls back to `open` (macOS) / `xdg-open` (Linux) if `$EDITOR` is not set
- [ ] Handles TUI editors (vim, nano) — detects if editor needs a terminal and warns or spawns appropriately
- [ ] Multi-file open works (sequentially for TUI editors, all-at-once for GUI editors)
- [ ] Graceful error with clear message if no editor can be resolved

## Implementation Notes

### Technical Approach
- Read `$EDITOR` via `std::env::var("EDITOR")`
- If not set, use `open` on macOS (`cfg!(target_os = "macos")`) or `xdg-open` on Linux
- For TUI editors (heuristic: vim, nvim, nano, emacs -nw, etc.), may need to spawn in a new terminal window — investigate `open -a Terminal` on macOS or equivalent
- For GUI editors, spawn detached process

### Dependencies
- METIS-T-0107 (viewer trait)

### Risk Considerations
- TUI editors in a non-interactive MCP server context are tricky — may need to document this as a known limitation or require a terminal spawning approach

## Status Updates

- **2026-03-26**: Implemented. `SysEditorViewer` reads `$EDITOR`, falls back to `open` (macOS) / `xdg-open` (Linux). Detects TUI editors (vim, nvim, nano, etc.) and spawns them in a terminal window. Always reports as available (OS default handler exists). 4 unit tests including TUI editor detection. All 25 MCP tests pass.