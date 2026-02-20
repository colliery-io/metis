---
id: redesign-ticket-detail-view-with
level: task
title: "Redesign ticket detail view with read-first mode and slide-out panel"
short_code: "METIS-T-0062"
created_at: 2026-01-28T14:46:16.031339+00:00
updated_at: 2026-01-28T15:45:17.012368+00:00
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

# Redesign ticket detail view with read-first mode and slide-out panel

## Parent Initiative

[[METIS-I-0019]]

## Objective

Redesign how ticket details are displayed in the GUI. Change from current behavior to a read-first mode on first click, with edit mode requiring a second action. Also explore replacing the current modal with a slide-out panel on the right side.

## Problem Statement

The current ticket detail view doesn't "feel" right. The interaction model and visual presentation need improvement:
1. Clicking a ticket should show a read-only view first (not edit mode)
2. A second click/action should enable editing
3. The current modal window approach may not be optimal - a fold-out/slide-out panel on the right could work better

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] First click on a ticket opens read-only view mode
- [ ] Explicit action (button/second click) required to enter edit mode
- [ ] Implement slide-out panel from right side instead of modal (or improve modal)
- [ ] Smooth animation for panel open/close
- [ ] Panel feels natural and integrated with the main view
- [ ] Edit mode clearly distinguished from view mode

## Design Considerations

### View Mode
- Clean, readable presentation of document content
- Clear visual hierarchy
- Obvious path to edit mode (edit button/icon)

### Slide-out Panel
- Slides in from right edge
- Appropriate width (maybe 40-50% of screen?)
- Doesn't completely obscure the main document list
- Can be dismissed easily (click outside, escape key, close button)

## Implementation Notes

### Components Affected
- Document detail/modal component
- Document list click handlers
- Overall layout structure

## Status Updates

### 2026-01-28: Completed

**File modified**: `crates/metis-docs-gui/src/components/DocumentViewer.vue`

**Changes made:**

1. **Slide-out panel instead of centered modal**
   - Changed from `fixed inset-0` centered modal to right-side slide-out panel
   - Panel width: 50vw (min 500px, max 800px)
   - Left border accent with shadow for depth
   - Backdrop darkens main content area

2. **Read-first mode**
   - Changed `isEditing = ref(true)` to `ref(false)`
   - Documents now open in read-only view by default
   - Added Edit button in header to toggle to edit mode
   - Button shows "✏️ Edit" in read mode, "✓ Editing" in edit mode

3. **Smooth animations**
   - Added Vue `<Transition>` with slide animation for panel (translateX)
   - Added backdrop fade transition
   - 0.3s ease timing for both

4. **UX improvements**
   - Edit mode resets to read mode when closing panel
   - Clicking backdrop closes panel
   - Clear visual distinction between modes

**Build**: Verified successful with `npm run build`