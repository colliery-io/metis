---
id: gui-viewer-backend
level: task
title: "GUI viewer backend"
short_code: "METIS-T-0114"
created_at: 2026-03-26T14:59:09.953384+00:00
updated_at: 2026-03-26T14:59:09.953384+00:00
parent: METIS-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0028
---

# GUI viewer backend

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Implement the GUI viewer backend in the MCP server that opens documents in the Metis Tauri app via the `metis://` URL scheme.

## Acceptance Criteria

- [ ] `GuiViewer` struct implements `DocumentViewer` trait (both `open` and `is_open`)
- [ ] `is_open` queries the Tauri app state to check if the document is already displayed
- [ ] Opens documents by invoking `metis://open/<short-code>` URL
- [ ] Supports `include_children` via URL query param
- [ ] Works on macOS and Linux
- [ ] Returns a clear error if the Metis GUI app is not installed or URL scheme is not registered — dispatcher will fallback to sys_editor
- [ ] On success, returns after dispatching the URL (fire-and-forget — no need to confirm the GUI handled it)

## Implementation Notes

### Technical Approach
- Use `open::that()` crate (or `std::process::Command` with `open`/`xdg-open`) to invoke the `metis://` URL
- Construct URL from short code and params: `metis://open/{short_code}?children={include_children}`
- The OS routes the URL to the Tauri app (which handles it via METIS-T-0113)
- No need to detect if the app is running — the URL scheme handles both cases

### Dependencies
- METIS-T-0107 (viewer trait)
- METIS-T-0113 (GUI deep-link URL scheme) — the Tauri app must handle `metis://` URLs

## Status Updates

*To be added during implementation*