---
id: gui-file-watching-replace-manual
level: task
title: "GUI file watching (replace manual refresh)"
short_code: "METIS-T-0112"
created_at: 2026-03-26T14:59:09.904342+00:00
updated_at: 2026-03-26T14:59:09.904342+00:00
parent: METIS-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0028
---

# GUI file watching (replace manual refresh)

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Replace the manual refresh button in the Metis Tauri GUI with automatic file watching so the UI updates reactively when documents change on disk.

## Acceptance Criteria

- [ ] `notify` crate (or equivalent) watches the `.metis` directory recursively for file changes
- [ ] On file change, the GUI reloads the affected document without full page refresh
- [ ] Tiptap editor reloads content without losing scroll position (cursor position loss is acceptable if the file changed externally)
- [ ] Debounce rapid changes (e.g., agent making multiple quick edits) — don't reload on every write event
- [ ] Manual refresh button removed or demoted to a fallback
- [ ] File watcher starts on app launch and stops on app close
- [ ] No performance issues watching large `.metis` directories

## Implementation Notes

### Technical Approach
- Use Tauri's `tauri-plugin-fs-watch` or the `notify` crate via a Tauri command/event
- Emit Tauri events on file change, frontend listens and reloads affected document
- Debounce with ~500ms window to batch rapid edits
- Filter events to only `.md` files within `.metis`
- Frontend: on reload event, re-fetch document content and update tiptap editor state

### Dependencies
- None — standalone GUI improvement, no dependency on MCP-side tasks

## Status Updates

*To be added during implementation*