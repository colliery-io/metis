---
id: add-scrollable-project-list-in-gui
level: task
title: "Add scrollable project list in GUI sidebar"
short_code: "METIS-T-0061"
created_at: 2026-01-28T14:46:15.967145+00:00
updated_at: 2026-01-28T15:40:42.422315+00:00
parent: METIS-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0019
---

# Add scrollable project list in GUI sidebar

## Parent Initiative

[[METIS-I-0019]]

## Objective

Add a scrollbar to the project list in the left sidebar of the Metis GUI so users can scroll through projects when they have many open.

## Problem Statement

With many projects open, the project list in the left sidebar runs off the screen. There's no way to scroll to see projects that extend beyond the visible area. A simple scrollbar would solve this.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Project list container has overflow-y: auto or scroll
- [ ] Scrollbar appears when project list exceeds available height
- [ ] Scrollbar styling matches existing UI theme
- [ ] All projects remain accessible via scrolling

## Implementation Notes

### Location
- Left sidebar project list component in the Tauri GUI

### Technical Approach
- Add appropriate CSS overflow property to the project list container
- Ensure the container has a constrained height (likely flex-based)
- Style scrollbar to match the dark theme if needed

## Status Updates

### 2026-01-28: Completed

**Root cause**: Projects were limited to 8 with `.slice(0, 8)` on line 36.

**Fix**: Removed the slice limit in `ProjectSidebar.vue`:
```diff
- v-for="(project, index) in recentProjects.slice(0, 8)"
+ v-for="(project, index) in recentProjects"
```

**Scrolling already supported**: The container div (line 20) already has `flex-1 overflow-y-auto` which enables scrolling when content exceeds available height.

**File modified**: `crates/metis-docs-gui/src/components/ProjectSidebar.vue`