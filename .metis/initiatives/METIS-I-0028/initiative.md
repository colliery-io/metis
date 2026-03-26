---
id: metis-gui-improvements
level: initiative
title: "Metis GUI Improvements"
short_code: "METIS-I-0028"
created_at: 2026-03-26T15:17:02.591408+00:00
updated_at: 2026-03-26T15:17:02.591408+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: metis-gui-improvements
---

# Metis GUI Improvements Initiative

## Context

The Metis Tauri GUI works but has several UX limitations that make it better suited as a teaching/visualization tool than a day-to-day workflow tool. This initiative collects GUI-specific improvements to address over time.

Spun out of METIS-I-0027 (External Document Viewer Integration) to keep the viewer initiative focused on VSCode/sys_editor as the primary paths.

## Goals & Non-Goals

**Goals:**
- Replace manual refresh with reactive file watching (notify-based)
- Add deep-link URL scheme (`metis://`) for external navigation into the GUI
- Implement GUI viewer backend for the `open_document` MCP tool
- Support multi-document viewing (currently limited to one document at a time)
- Debounce rapid file change events for smooth editor experience

**Non-Goals:**
- Replacing VSCode/sys_editor as the primary viewer — GUI is a complement
- Real-time collaborative editing

## Detailed Design

To be fleshed out when this initiative is prioritized. Existing task documents (migrated from METIS-I-0027) contain detailed acceptance criteria and implementation notes.

## Implementation Plan

Tasks migrated from METIS-I-0027:
- METIS-T-0112: GUI file watching (replace manual refresh)
- METIS-T-0113: GUI deep-link URL scheme
- METIS-T-0114: GUI viewer backend