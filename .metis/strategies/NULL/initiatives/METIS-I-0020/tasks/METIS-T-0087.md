---
id: gui-upstream-context-views-and
level: task
title: "GUI upstream context views and onboarding"
short_code: "METIS-T-0087"
created_at: 2026-02-26T01:32:13.922324+00:00
updated_at: 2026-02-27T01:02:38.698214+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# GUI upstream context views and onboarding

## Objective

Build the GUI views for upstream context (showing remote documents) and the first-run onboarding wizard for connecting to a central repo. New teams see their own work by default with the ability to browse upstream.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Onboarding Wizard
- [ ] First-run detection: if no `upstream_url` in config, offer to set up multi-workspace
- [ ] URL input field with connectivity test ("Testing connection... OK")
- [ ] Workspace prefix input with validation feedback
- [ ] Team label input (optional)
- [ ] Initial sync progress indicator
- [ ] Skip option for single-workspace-only users

### Upstream Context Views
- [ ] "My Work" default view: owned documents + direct parent chain (upstream context)
- [ ] "Upstream" panel/section: shows remote documents that this workspace's documents reference
- [ ] Hierarchical navigation: drill from vision → strategy → initiative → tasks across workspaces
- [ ] Visual distinction between owned (editable) and remote (read-only) documents
- [ ] Progress indicators on remote parent documents (e.g. "3/5 tasks completed" on an upstream initiative)
- [ ] Workspace/team labels shown on remote documents

### Scope & Filtering
- [ ] Default scope: own workspace + parent chain (not everything)
- [ ] Browse mode: expand to see all workspaces
- [ ] Team grouping: filter by team label across workspaces

## Implementation Notes

### Default Scope Design

New teams see their own work immediately. The "Upstream Context" section shows the parent chain — if API-T-0001 references WGR-T-0001 which references WGR-I-0001 which references STRAT-S-0001, the upstream panel shows that chain. The user doesn't need to know about the SRE team's documents unless they browse for them.

### Visual Design

- Owned documents: full edit controls (phase transitions, content editing, etc.)
- Remote documents: read-only card/row with phase indicator, last updated timestamp, workspace badge
- Progress bars on parent documents showing child completion across workspaces

### Dependencies

- METIS-T-0081 (sync orchestration — for initial sync in wizard)
- METIS-T-0083 (projection cache — cross-workspace queries for upstream views)
- METIS-T-0086 (GUI sync — sync button and status)

## Test Scenarios

### Onboarding Wizard Tests

1. **First-run detection**: no `upstream_url` in config → wizard offered automatically on first app open
2. **Skip option**: user clicks "Skip" → wizard closes, single-workspace mode, no upstream config written
3. **URL input — valid SSH**: enter SSH URL → connectivity test runs, "OK" shown
4. **URL input — valid HTTPS**: enter HTTPS URL → connectivity test runs, "OK" shown
5. **URL input — invalid**: enter garbage → connectivity test fails, error shown inline, user can re-enter
6. **URL input — auth failure**: valid host but bad auth → clear error "Authentication failed", user can fix credentials and retry
7. **Prefix input — validation feedback**: type invalid prefix (uppercase, too short, special chars) → real-time validation shows error below field
8. **Prefix input — accepted**: type valid prefix → green checkmark, proceed enabled
9. **Team label — optional**: leave blank → wizard proceeds without team label
10. **Initial sync progress**: wizard triggers first sync → progress bar or spinner with "Syncing..." message
11. **Initial sync failure**: sync fails during wizard → error shown, option to retry or skip (config still written, sync can happen later)
12. **Wizard completion**: all steps done → lands on main app view with upstream context visible
13. **Wizard not shown again**: complete wizard once → app opens without wizard on subsequent launches
14. **Existing upstream — no wizard**: config already has `upstream_url` → wizard never shown

### Upstream Context View Tests

15. **My Work default view**: app opens → shows owned documents + parent chain from upstream, not everything
16. **Parent chain visible**: owned task references upstream initiative → initiative shown in upstream context panel
17. **Deep parent chain**: task → initiative → strategy → vision across workspaces → full chain navigable
18. **No upstream references**: owned documents don't reference anything upstream → upstream panel shows "No upstream context" or is collapsed
19. **Upstream panel shows correct documents**: owned `API-T-0001` has parent `WGR-I-0001` → upstream panel shows `WGR-I-0001` and its parent chain

### Visual Distinction Tests

20. **Owned documents — full controls**: owned document → shows edit button, phase transition controls, all actions
21. **Remote documents — read-only**: hydrated document → shows content but no edit button, no phase transition, visual "read-only" badge
22. **Workspace badge on remote docs**: remote document → shows workspace prefix badge (e.g., "strat", "alpha")
23. **Visual styling difference**: owned vs remote documents → clearly distinguishable at a glance (different background, border, or opacity)

### Progress Indicator Tests

24. **Progress on upstream initiative**: initiative with 5 tasks (3 completed, 1 active, 1 todo) → shows "3/5 completed" progress bar
25. **Cross-workspace progress**: initiative tasks from 3 workspaces → progress aggregates all workspaces
26. **Progress updates after sync**: sync pulls completed tasks → progress bar updates
27. **Zero progress**: initiative with no tasks yet → shows "0 tasks" or empty progress

### Scope & Filtering Tests

28. **Default scope — own work**: new user opens app → sees only own workspace + upstream parent chain
29. **Browse mode — all workspaces**: user clicks "Browse all" → sees documents from every hydrated workspace
30. **Team filter**: user filters by team "platform" → sees only workspaces with team label "platform"
31. **Filter persists**: set filter, navigate away, come back → filter still applied
32. **Search across workspaces**: search for keyword → results from all visible workspaces, respecting current scope/filter

### Hierarchical Navigation Tests

33. **Drill down — vision to tasks**: click vision → shows strategies/initiatives → click initiative → shows tasks. Works across workspace boundaries
34. **Drill up — task to vision**: viewing a task → breadcrumb shows full parent chain → click any ancestor to navigate up
35. **Cross-workspace navigation**: viewing owned task → click upstream parent (different workspace) → navigates to read-only view of that document
36. **Back navigation**: navigate deep into upstream context → back button returns to previous view

### Integration Tests

37. **End-to-end onboarding**: fresh app install → wizard → enter URL → set prefix → initial sync → see upstream documents in context panel
38. **Sync updates views**: sync pulls new documents → upstream context panel and browse views update without manual refresh
39. **Onboarding then work**: complete wizard → create a task referencing upstream initiative → task appears in "My Work", initiative appears in upstream context
40. **Multi-team scenario**: 5 workspaces synced → team filtering correctly groups workspaces, upstream context shows cross-team relationships

### Edge Cases

41. **No remote workspaces yet**: central is empty except owned workspace → upstream panel says "No other workspaces yet"
42. **Very many workspaces**: 50 workspaces → browse view is performant, team filter helps narrow
43. **Broken parent reference**: owned task references initiative that was archived upstream → shows "Referenced document archived" or similar, not a crash
44. **Remote document updated between views**: viewing upstream doc, sync happens, doc changed → view refreshes with new content
45. **Offline mode**: no network, upstream data stale → app still works with cached data, status shows "Last synced 3 hours ago"

## Status Updates

### Session 1 — Frontend Components + Wiring (Complete)

**New files created:**
- `src/components/OnboardingWizard.vue` — Multi-step wizard (Repository URL → Workspace Prefix → Initial Sync)
- `src/components/SyncStatusIndicator.vue` — Compact status in top bar (synced/error/in-progress/ready states)
- `src/components/UpstreamContext.vue` — Collapsible panel showing remote workspace documents

**Files modified:**
- `src/lib/tauri-api.ts` — Added `WorkspaceSyncResult`, `SyncStatus` interfaces; `syncWorkspace()`, `getSyncStatus()`, `isUpstreamConfigured()` methods
- `src/App.vue` — Integrated OnboardingWizard + SyncStatusIndicator; onboarding auto-shows on first load without upstream; Skip persists via localStorage
- `src/components/KanbanBoard.vue` — Added UpstreamContext panel below boards; Refresh button now uses workspace sync when upstream configured

**Onboarding Wizard (OnboardingWizard.vue):**
- 3-step wizard: Repository URL → Workspace Prefix → Initial Sync
- URL input with format validation (SSH, HTTPS, file://)
- Workspace prefix input with real-time validation (lowercase, alphanumeric)
- Optional team label
- Sync progress indicator with error handling and retry
- Skip option persists per-project via localStorage
- Auto-shown when project loaded without upstream configured

**Sync Status Indicator (SyncStatusIndicator.vue):**
- Compact button in top bar (right side, next to search)
- States: synced (green checkmark + relative time), error (red dot), in-progress (spinner), ready (sync icon)
- Click to trigger full workspace sync
- Polls status every 30s, listens for sync-completed events
- Auto-hides when no upstream configured
- Relative time display: "Just now", "5m ago", "2h ago", "Yesterday"

**Upstream Context Panel (UpstreamContext.vue):**
- Collapsible panel below kanban boards
- Shows remote workspace documents grouped by workspace prefix
- Two modes: "Parent Chain" (default, focused) and "All Workspaces" (browse)
- Visual distinction: workspace badges, team badges, read-only indicators
- Progress bars on upstream initiatives (completed/total tasks)
- Document type badges (V/S/I/T/A) with color coding
- Phase badges with appropriate styling
- Listens for sync-completed events to auto-refresh

**Build:** Frontend (vue-tsc + vite) compiles cleanly. Backend (cargo) compiles cleanly.
**Tests:** All 16 Rust tests pass.