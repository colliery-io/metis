---
id: improve-spacing-for-document
level: task
title: "Improve spacing for document metadata display in view mode"
short_code: "METIS-T-0063"
created_at: 2026-01-28T14:46:16.109369+00:00
updated_at: 2026-01-28T15:48:47.810676+00:00
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

# Improve spacing for document metadata display in view mode

## Parent Initiative

[[METIS-I-0019]]

## Objective

Improve the spacing and visual presentation of document metadata (short code, phase, document type) in the view mode. Currently these elements are cramped together.

## Problem Statement

In the view mode, metadata like:
- CLOACI-I-0008
- discovery
- INITIATIVE

...are displayed very cramped in the UI. They need room to breathe for better readability and visual appeal.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Adequate spacing between metadata elements (short code, phase, type)
- [ ] Each element is clearly readable and distinct
- [ ] Visual hierarchy makes it easy to scan
- [ ] Consistent with overall UI design language
- [ ] Works well at different content lengths (short vs long codes, various phases)

## Design Suggestions

- Add padding/margin between elements
- Consider vertical stacking vs horizontal layout
- Use visual separators or grouping if helpful
- Ensure text doesn't feel cramped even with longer values

## Implementation Notes

### Location
- Document view/detail component in the Tauri GUI
- Metadata display section

## Status Updates

### 2026-01-28: Completed

**File modified**: `crates/metis-docs-gui/src/components/DocumentViewer.vue`

**Changes made:**

1. **Container spacing**
   - Changed `space-x-3 mt-1` to `flex-wrap gap-3 mt-2`
   - Added `flex-wrap` to handle overflow gracefully
   - Increased vertical margin for breathing room

2. **Short code badge**
   - Added `px-3 py-1 rounded` padding
   - Added background color for visual weight
   - Made text bolder (`fontWeight: 600`)

3. **Phase badge**
   - Increased padding: `px-2 py-1` → `px-3 py-1.5`
   - Added `tracking-wide` for letter spacing
   - Changed to `font-semibold`
   - Slightly more opaque background (`20` → `25`)

4. **Document type badge**
   - Increased padding: `px-2 py-1` → `px-3 py-1.5`
   - Changed to use interactive primary color (more visually distinct)
   - Added `tracking-wide` letter spacing
   - Moved `uppercase` to class

**Result**: Metadata elements now have room to breathe, are more visually distinct, and wrap gracefully on narrow panels.

**Build**: Verified successful