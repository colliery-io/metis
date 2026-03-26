---
id: gui-deep-link-url-scheme
level: task
title: "GUI deep-link URL scheme"
short_code: "METIS-T-0113"
created_at: 2026-03-26T14:59:09.929134+00:00
updated_at: 2026-03-26T14:59:09.929134+00:00
parent: METIS-I-0028
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0028
---

# GUI deep-link URL scheme

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Register a `metis://` custom URL scheme in the Tauri app so external processes can deep-link to specific documents (e.g., `metis://open/METIS-I-0027`).

## Acceptance Criteria

- [ ] Tauri app registers `metis://` custom URL scheme on macOS (via Info.plist) and Linux (via .desktop file)
- [ ] `metis://open/<short-code>` navigates to the specified document in the app
- [ ] `metis://open/<short-code>?children=true` is accepted but opens only the parent document (GUI limitation: one document at a time). Children parameter is acknowledged but not acted on.
- [ ] Works when app is not running (launches app, then navigates)
- [ ] Works when app is already running (brings to front, navigates)
- [ ] Invalid short codes show a user-friendly error in the GUI
- [ ] URL scheme includes project path context to handle multiple `.metis` projects

## Implementation Notes

### Technical Approach
- Use Tauri's deep-link plugin (`tauri-plugin-deep-link`) for URL scheme registration
- On macOS: registers via `CFBundleURLTypes` in Info.plist (handled by Tauri plugin)
- On Linux: registers via `.desktop` file with `MimeType` entry
- App listens for deep-link events on startup and while running
- Parse the URL to extract short code and params, then navigate the frontend router to the document view
- Multi-project disambiguation: include project path in URL (e.g., `metis://open/<short-code>?project=/path/to/.metis`) or use the currently-open project

### Dependencies
- None for the Tauri-side work, but METIS-T-0114 (GUI viewer backend) depends on this

### Risk Considerations
- Custom URL scheme registration on macOS requires app to be installed (not just run from build dir) — may need installer/DMG for full functionality
- Linux URL scheme registration varies by desktop environment
- Security: validate that incoming URLs are well-formed and short codes are sanitized

## Status Updates

*To be added during implementation*