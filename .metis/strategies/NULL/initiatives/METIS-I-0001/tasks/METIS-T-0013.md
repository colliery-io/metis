---
id: add-document-type-navigation
level: task
title: "Add document type navigation (Vision, Initiative, Task, ADR)"
short_code: "METIS-T-0013"
created_at: 2025-10-08T11:29:08.659964+00:00
updated_at: 2025-10-09T22:27:56.916726+00:00
parent: METIS-I-0001
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: 
initiative_id: METIS-I-0001
---

# Add document type navigation (Vision, Initiative, Task, ADR)

## Parent Initiative

[[METIS-I-0001]] - Multi-Project GUI Application

## Objective

Create dedicated kanban-style boards for each document type (Vision, Initiative, Task, ADR, Backlog) with phase-based columns, similar to the TUI interface. This allows users to navigate between document type boards and see documents organized in workflow columns within each board.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Board navigation allows switching between document type boards (Vision, Initiative, Task, ADR, Backlog)
- [ ] Each board displays documents of that type organized in phase-based columns
- [ ] Vision board shows columns for draft/review/published phases
- [ ] Initiative board shows columns for discovery/design/ready/decompose/active/completed phases
- [ ] Task board shows columns for todo/doing/completed phases
- [ ] ADR board shows columns for draft/discussion/decided/superseded phases
- [ ] Backlog board shows columns for different categories (General, Bug, Feature, Tech Debt)
- [ ] Current board selection is visually indicated (active tab/button)
- [ ] Empty columns show appropriate guidance messages
- [ ] Document cards within columns show essential information (title, short code, etc.)
- [ ] Clicking document cards navigates to detailed view (placeholder for future)



## Implementation Notes

### Technical Approach
1. Replace current DocumentBoard with KanbanBoard component that supports multiple board types
2. Create BoardNavigation component with tabs/buttons for each document type
3. Build KanbanColumn component for phase-based organization within each board
4. Add board state management to track current board selection
5. Filter documents by type and organize into appropriate phase columns
6. Implement board-specific phase definitions matching TUI patterns

### Components to Create
- `KanbanBoard` - Main board interface supporting multiple document types
- `BoardNavigation` - Tab/button navigation between board types
- `KanbanColumn` - Phase-based columns within each board
- `BoardEmptyState` - Guidance for empty boards/columns
- `DocumentKanbanCard` - Document cards optimized for column layout

### Dependencies
- METIS-T-0012 (Document board structure) - COMPLETED
- Existing Tauri backend commands for document loading
- Phase definitions matching TUI workflow patterns
- Document type and phase enumeration from backend

### Risk Considerations
- Different document types have different phase workflows
- Column width management with varying numbers of phases
- Performance impact of rendering multiple large boards
- State management complexity with multiple board views
- Responsive design challenges with columnar layouts

## Status Updates **[REQUIRED]**

### Completed - 2025-10-09

**Acceptance Criteria Completed:**
- ✅ Board navigation allows switching between document type boards (Vision, Initiative, Task, ADR, Backlog)
- ✅ Each board displays documents of that type organized in phase-based columns
- ✅ Vision board shows columns for draft/review/published phases
- ✅ Initiative board shows columns for discovery/design/ready/decompose/active/completed phases
- ✅ Task board shows columns for todo/doing/completed phases
- ✅ ADR board shows columns for draft/discussion/decided/superseded phases
- ✅ Backlog board shows columns for different categories (General, Bug, Feature, Tech Debt)
- ✅ Current board selection is visually indicated (active tab/button)
- ✅ Empty columns show appropriate guidance messages
- ✅ Document cards within columns show essential information (title, short code, etc.)
- ✅ Clicking document cards navigates to detailed view (placeholder for future)

**Implementation Details:**
- Created complete kanban board system matching TUI's 5-board architecture
- Built BoardNavigation component with tabs and document counts for all board types
- Implemented KanbanBoard component replacing the simple DocumentBoard
- Created KanbanColumn component with phase-based organization and color coding
- Added centralized board-config system for phase definitions and document filtering
- Integrated horizontal scrolling for boards with varying column counts

**Components Created:**
- `BoardNavigation.tsx`: Tab interface for switching between 5 board types (90+ lines)
- `KanbanBoard.tsx`: Main kanban interface with project header and board management (130+ lines)
- `KanbanColumn.tsx`: Individual phase columns with document cards and empty states (85+ lines)
- `board-config.ts`: Centralized phase definitions and filtering logic (200+ lines)

**Files Modified:**
- `App.tsx`: Updated to use KanbanBoard instead of DocumentBoard
- Preserved existing DocumentBoard, DocumentCard, DocumentTypeFilter components for future use

**Features Delivered:**
- 5 distinct kanban boards: Vision, Initiative, Task, ADR, Backlog
- Phase-specific workflows matching TUI patterns exactly
- Visual board navigation with document counts
- Color-coded columns indicating workflow stages
- Empty state guidance tailored to each phase
- Responsive layout supporting different screen sizes
- Complete replacement of simple list view with sophisticated kanban interface

**Technical Architecture:**
- Type-safe board and phase definitions
- Centralized configuration system for easy maintenance
- Document filtering and categorization by type and phase
- Preserved existing error handling and loading states
- Maintained integration with Tauri backend commands