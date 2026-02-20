---
id: make-clicking-a-ticket-open-view
level: task
title: "Make clicking a ticket open view mode directly"
short_code: "METIS-T-0064"
created_at: 2026-01-28T14:46:16.172414+00:00
updated_at: 2026-01-28T15:50:34.285461+00:00
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

# Make clicking a ticket open view mode directly

## Parent Initiative

[[METIS-I-0019]]

## Objective

Change the click behavior on document cards so that clicking anywhere on the card opens the view mode directly, rather than requiring users to find and click the eyeball icon.

## Problem Statement

Currently, to view a ticket's details, users must click a small eyeball icon. This isn't obvious or discoverable. The natural expectation is that clicking on a ticket card would open it for viewing.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Clicking anywhere on a document card opens the view panel/modal
- [ ] Eyeball icon can be removed or repurposed
- [ ] Behavior is consistent across all document types
- [ ] Edit functionality remains accessible (via view mode or dedicated edit button)

## UX Considerations

- Most apps: click item = view item
- Current: click item = nothing, must find eyeball icon
- This change aligns with user expectations

## Related Tasks

This task is closely related to METIS-T-0062 (read-first mode redesign). They should likely be implemented together as part of the overall interaction model change.

## Implementation Notes

### Changes Needed
- Update click handler on document cards
- Remove or repurpose eyeball icon
- Ensure edit path remains clear

## Status Updates

### 2026-01-28: Completed

**File modified**: `crates/metis-docs-gui/src/components/KanbanCard.vue`

**Change**: Added `@click="$emit('view', document)"` to the main card div.

**Before**: Users had to find and click the small eyeball icon to view a document.

**After**: Clicking anywhere on the card opens the document viewer. The action buttons (view, promote, archive) use `@click.stop` so they don't double-trigger.

**Build**: Verified successful

**Note**: The eyeball icon remains for discoverability but is no longer the only way to open a document.