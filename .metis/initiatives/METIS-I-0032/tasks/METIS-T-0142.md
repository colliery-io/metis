---
id: desktop-target-dioxus-desktop
level: task
title: "Desktop target (dioxus-desktop); retire Tauri GUI"
short_code: "METIS-T-0142"
created_at: 2026-06-12T20:00:17.107966+00:00
updated_at: 2026-06-12T20:00:17.107966+00:00
parent: METIS-I-0032
blocked_by: ["METIS-T-0140"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# Desktop target (dioxus-desktop); retire Tauri GUI

## Parent Initiative

[[METIS-I-0032]]

## Objective

Ship the same Dioxus UI as a native desktop app via `dioxus-desktop` (wry/tao webview) — the payoff of the all-Rust choice — and retire the 2.x Tauri GUI.

## Acceptance Criteria

- [ ] `metis-web` gains a `dioxus-desktop` target (feature/bin) that renders the same components in a native window
- [ ] Desktop app takes a configurable server URL + token (not same-origin like the web build); launches, logs in, shows the board
- [ ] Components are shared with the web target — only the entrypoint/renderer differs (cfg per target), no forked UI
- [ ] `crates/metis-docs-gui` (2.x Tauri GUI) is retired — removed from the workspace (or clearly marked frozen/removed) and dropped from the GUI CI job; docs updated
- [ ] Build instructions for the desktop artifact

## Implementation Notes

### Technical Approach
- The web build is same-origin (no base URL); desktop needs an explicit server URL + token entry/config. Abstract the REST client's base URL so both targets share it.
- `cfg(target_arch = "wasm32")` vs desktop entrypoint; the component tree is identical.

### Dependencies
- Blocked by METIS-T-0140 (the shared component set). Can run in parallel with METIS-T-0141.

### Risk Considerations
- Retiring `metis-docs-gui` touches the 2.x GUI CI (`ci.yml` builds the Tauri app on `main`); coordinate so `main`'s 2.x build isn't broken — the retirement lands on the 3.0 line, and 2.x on `main` keeps its GUI until 3.0 releases.

## Status Updates

*To be added during implementation*