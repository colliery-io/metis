---
id: backlog-management-system
level: initiative
title: "Backlog Management System"
created_at: 2025-07-31T21:18:38.020096+00:00
updated_at: 2025-07-31T21:18:38.020096+00:00
parent: user-experience-enhancement
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
---

# Backlog Management System Initiative

## Context

The Metis TUI needs a dedicated backlog management system to track bugs, technical debt, and feature requests that exist outside of active initiatives. Currently, such items must be immediately assigned to initiatives or exist only in external systems, making it difficult to maintain a buffer of work that can be pulled into active development as capacity allows.

This initiative addresses the need for a flexible backlog system that integrates seamlessly with the existing Metis document hierarchy while providing a specialized TUI interface for managing different types of work items.

## Goals & Non-Goals

**Goals:**
- Create a dedicated backlog folder structure within Metis projects for managing work items outside active initiatives
- Implement specialized task templates for bugs, tech debt, and feature requests with appropriate metadata
- Develop a TUI interface with columnar views (Bugs, Tech Debt, Feature Requests, Active) for backlog management
- Enable seamless transition of backlog items into active initiatives while maintaining traceability
- Provide mechanisms to associate backlog items with existing initiatives when pulled into active work
- Support prioritization and filtering within each backlog category
- Maintain integration with existing Metis document hierarchy and phase management

**Non-Goals:**
- Replacing the existing initiative/task structure for planned work
- Creating external integrations with third-party issue tracking systems
- Implementing complex workflow automation or business rules
- Adding time tracking or estimation features beyond basic complexity markers
- Creating notification systems or email alerts
- Supporting multi-project backlog aggregation in the initial version

## Detailed Design

### Backlog Folder Structure

The backlog system will be implemented as a dedicated folder structure within each Metis project:

```
.metis/
├── backlog/
│   ├── bugs/
│   │   ├── bug-001-parsing-error.md
│   │   └── bug-002-ui-rendering.md
│   ├── tech-debt/
│   │   ├── td-001-refactor-parser.md
│   │   └── td-002-update-dependencies.md
│   ├── feature-requests/
│   │   ├── fr-001-dark-mode.md
│   │   └── fr-002-export-function.md
│   └── archived/
│       └── {completed items moved here}
```

### Backlog Item Templates

#### Bug Template
```yaml
---
id: bug-{sequence}
level: task
type: bug
title: "{Bug Title}"
created_at: {timestamp}
updated_at: {timestamp}
parent: backlog
priority: {high|medium|low}
severity: {critical|major|minor|trivial}
status: backlog
reporter: "{person}"
assignee: "{person|unassigned}"
related_initiative: "{initiative-id|none}"
blocked_by: []
archived: false

tags:
  - "#backlog"
  - "#bug"
  - "#phase/backlog"
---

# {Bug Title}

## Description
{Detailed description of the bug}

## Steps to Reproduce
1. {Step 1}
2. {Step 2}
3. {Step 3}

## Expected Behavior
{What should happen}

## Actual Behavior
{What actually happens}

## Environment
- OS: {operating system}
- Version: {software version}
- Configuration: {relevant config}

## Additional Context
{Screenshots, logs, related issues}
```

#### Tech Debt Template
```yaml
---
id: td-{sequence}
level: task
type: tech-debt
title: "{Tech Debt Title}"
created_at: {timestamp}
updated_at: {timestamp}
parent: backlog
priority: {high|medium|low}
impact: {performance|maintainability|security|scalability}
effort: {xs|s|m|l|xl}
status: backlog
assignee: "{person|unassigned}"
related_initiative: "{initiative-id|none}"
blocked_by: []
archived: false

tags:
  - "#backlog"
  - "#tech-debt"
  - "#phase/backlog"
---

# {Tech Debt Title}

## Problem Statement
{Description of the technical debt issue}

## Current Impact
{How this debt affects the system now}

## Proposed Solution
{High-level approach to addressing the debt}

## Benefits
{Expected improvements after resolution}

## Risks of Not Addressing
{Consequences of leaving this unresolved}
```

#### Feature Request Template
```yaml
---
id: fr-{sequence}
level: task
type: feature-request
title: "{Feature Title}"
created_at: {timestamp}
updated_at: {timestamp}
parent: backlog
priority: {high|medium|low}
complexity: {xs|s|m|l|xl}
status: backlog
requestor: "{person}"
assignee: "{person|unassigned}"
related_initiative: "{initiative-id|none}"
blocked_by: []
archived: false

tags:
  - "#backlog"
  - "#feature-request"
  - "#phase/backlog"
---

# {Feature Title}

## Feature Description
{What the feature should do}

## User Story
As a {user type}, I want {goal} so that {benefit}.

## Acceptance Criteria
- [ ] {Criterion 1}
- [ ] {Criterion 2}
- [ ] {Criterion 3}

## Use Cases
{Specific scenarios where this feature would be used}

## Business Value
{Why this feature is valuable}
```

### TUI Interface Design

The backlog view will be a separate screen accessible from the main TUI:

**Backlog View:**
```
┌─ Backlog Management ─────────────────────────────────────────────────────┐
│ [B]ugs (5)              [T]ech Debt (3)         [F]eature Requests (8)   │
├────────────────────────┬────────────────────────┬───────────────────────┤
│ HIGH                   │ HIGH                   │ HIGH                  │
│ • parsing error [→]    │ • refactor parser     │ • dark mode          │
│                        │                        │ • export function    │
│ MEDIUM                 │ MEDIUM                 │                       │
│ • ui render bug        │ • update deps         │ MEDIUM                │
│ • validation issue     │                        │ • advanced search    │
│                        │ LOW                    │ • custom filters     │
│ LOW                    │ • logging cleanup     │                       │
│ • tooltip position     │                        │ LOW                   │
│                        │                        │ • custom tooltips    │
│                        │                        │ • additional themes  │
└────────────────────────┴────────────────────────┴───────────────────────┘
│ Navigation: ←→ columns, ↑↓ items, Enter: details, Space: activate       │
│ Actions: [n]ew, [e]dit, [d]elete, [p]riority, [a]ctivate, [q]uit       │
└──────────────────────────────────────────────────────────────────────────┘
```

**Integration with Main Kanban Board:**

When a backlog item is activated:
1. Its phase changes from "backlog" to "todo"
2. It appears in the main Task kanban board under the "Todo" column
3. It follows the normal task workflow: Todo → Doing → Done
4. The item retains its `type` field (bug/tech-debt/feature-request) for filtering
5. A special indicator (e.g., 🐛 for bugs, 🔧 for tech debt, ✨ for features) shows its origin

Main Kanban Board with activated backlog items:
```
┌─ Tasks ──────────────────────────────────────────────────────────────────┐
│ Todo (3)                │ Doing (2)              │ Done (4)             │
├─────────────────────────┼────────────────────────┼──────────────────────┤
│ • Implement auth       │ • 🐛 parsing error     │ • Deploy to staging  │
│ • 🔧 refactor parser   │ • Write docs          │ • Setup CI/CD        │
│ • ✨ dark mode         │                        │ • 🐛 ui render bug   │
│                        │                        │ • Update README      │
└─────────────────────────┴────────────────────────┴──────────────────────┘
```

### Transition Workflows

#### Activating Backlog Items
1. User selects item from backlog view
2. Press 'a' or Space to activate
3. System optionally prompts for initiative association
4. Item phase changes from "backlog" to "todo"
5. Item now appears in main Task kanban board
6. Original file remains in backlog folder but is tracked as active

#### Working on Active Items
1. Once in the main kanban board, items follow normal task workflow
2. Move from Todo → Doing → Done using existing kanban controls
3. Backlog type indicator remains visible for context
4. All standard task operations apply (editing, blocking, etc.)

#### Archiving Completed Items
1. When item reaches "Done" status, it can be archived
2. Archive operation moves file to `backlog/archived/` folder
3. Item is removed from kanban board view
4. Archived items remain searchable and can be restored if needed

### Database and State Management

The system will extend existing task management with:
- Additional `type` field to distinguish bug/tech-debt/feature-request
- `parent: backlog` to identify backlog items
- Special handling in kanban board to show type indicators
- Filter options to show/hide backlog items in main board
- Backlog view state management separate from main kanban state

## Description
{Detailed description of the bug}

## Steps to Reproduce
1. {Step 1}
2. {Step 2}
3. {Step 3}

## Expected Behavior
{What should happen}

## Actual Behavior
{What actually happens}

## Environment
- OS: {operating system}
- Version: {software version}
- Configuration: {relevant config}

## Additional Context
{Screenshots, logs, related issues}
```

#### Tech Debt Template
```yaml
---
id: td-{sequence}
level: backlog-item
type: tech-debt
title: "{Tech Debt Title}"
created_at: {timestamp}
updated_at: {timestamp}
priority: {high|medium|low}
impact: {performance|maintainability|security|scalability}
effort: {xs|s|m|l|xl}
status: {identified|planned|in-progress|resolved}
assignee: "{person|unassigned}"
related_initiative: "{initiative-id|none}"
blocked_by: []
archived: false

tags:
  - "#backlog"
  - "#tech-debt"
  - "#phase/{status}"
---

# {Tech Debt Title}

## Problem Statement
{Description of the technical debt issue}

## Current Impact
{How this debt affects the system now}

## Proposed Solution
{High-level approach to addressing the debt}

## Benefits
{Expected improvements after resolution}

## Risks of Not Addressing
{Consequences of leaving this unresolved}
```

#### Feature Request Template
```yaml
---
id: fr-{sequence}
level: backlog-item
type: feature-request
title: "{Feature Title}"
created_at: {timestamp}
updated_at: {timestamp}
priority: {high|medium|low}
complexity: {xs|s|m|l|xl}
status: {requested|reviewed|approved|in-progress|completed}
requestor: "{person}"
assignee: "{person|unassigned}"
related_initiative: "{initiative-id|none}"
blocked_by: []
archived: false

tags:
  - "#backlog"
  - "#feature-request"
  - "#phase/{status}"
---

# {Feature Title}

## Feature Description
{What the feature should do}

## User Story
As a {user type}, I want {goal} so that {benefit}.

## Acceptance Criteria
- [ ] {Criterion 1}
- [ ] {Criterion 2}
- [ ] {Criterion 3}

## Use Cases
{Specific scenarios where this feature would be used}

## Business Value
{Why this feature is valuable}
```

### TUI Interface Design

The backlog management interface will be accessible via a dedicated view in the Metis TUI:

**Layout:**
```
┌─ Backlog Management ─────────────────────────────────────────────────────┐
│ [B]ugs (5)    [T]ech Debt (3)    [F]eature Req (8)    [A]ctive (2)      │
├──────────────┬──────────────┬──────────────┬──────────────────────────────┤
│ HIGH         │ HIGH         │ HIGH         │ IN PROGRESS                  │
│ • parsing    │ • refactor   │ • dark mode  │ • parsing-error              │
│   error      │   parser     │ • export     │   (from bugs)                │
│              │              │   function   │ • dark-mode                  │
│ MEDIUM       │ MEDIUM       │              │   (from features)            │
│ • ui render  │ • update     │ MEDIUM       │                              │
│ • validation │   deps       │ • search     │ COMPLETED                    │
│              │              │ • filters    │ • None                       │
│ LOW          │ LOW          │              │                              │
│ • tooltip    │ • None       │ LOW          │                              │
│              │              │ • tooltips   │                              │
│              │              │ • themes     │                              │
└──────────────┴──────────────┴──────────────┴──────────────────────────────┘
│ Navigation: ←→ columns, ↑↓ items, Enter: details, Space: move to active  │
│ Actions: [n]ew, [e]dit, [d]elete, [p]riority, [m]ove, [q]uit             │
└───────────────────────────────────────────────────────────────────────────┘
```

### Transition Workflows

#### Moving Items to Active
1. User selects item from any backlog column
2. Press Space or 'm' to move to active
3. System prompts for optional initiative association
4. Item is moved to `backlog/active/` folder
5. If associated with initiative, `related_initiative` field is updated
6. Status changes to `in-progress`

#### Moving Items to Initiatives
1. From active backlog, user can promote item to full initiative task
2. System creates proper task under selected initiative
3. Original backlog item is archived with reference to new task
4. Full traceability maintained through metadata

#### Completion and Archival
1. When backlog item work is completed, status updates to `resolved`/`completed`
2. Item can be archived manually or automatically after configured period
3. Archived items move to `backlog/archived/` with timestamp

## Alternatives Considered

### Alternative 1: External Issue Tracker Integration
**Approach:** Integrate with external systems like GitHub Issues, Jira, or Linear
**Rejected because:**
- Adds external dependencies and complexity
- Breaks the self-contained nature of Metis projects
- Requires network connectivity and authentication management
- Would fragment the user experience between Metis and external tools

### Alternative 2: Extend Existing Task System
**Approach:** Add backlog categories to the existing task structure within initiatives
**Rejected because:**
- Would pollute the initiative-focused task structure
- Lacks the flexibility needed for different backlog item types
- Makes it harder to manage items that aren't yet assigned to initiatives
- Doesn't provide the specialized metadata needed for bugs vs tech debt vs features

### Alternative 3: Single Backlog File Approach
**Approach:** Maintain all backlog items in a single large markdown file
**Rejected because:**
- Poor scalability as backlog grows
- Difficult to manage concurrent edits
- Lacks individual item metadata and version history
- Would require complex parsing for TUI interface
- No integration with existing Metis document management

### Alternative 4: Database-backed Solution
**Approach:** Store backlog items in SQLite or similar database
**Rejected because:**
- Breaks consistency with Metis file-based architecture
- Adds complexity for backup, version control, and portability
- Would require data migration tooling
- Doesn't leverage existing document management infrastructure

## Implementation Plan

### Phase 1: Core Infrastructure (2-3 weeks)
**Deliverables:**
- Extend Metis core library to support backlog document types
- Create backlog folder structure and initialization logic
- Implement backlog item templates (bug, tech-debt, feature-request)
- Add backlog item validation and parsing

**Tasks:**
- Extend document type enum to include backlog-item types
- Create template files for each backlog item type
- Implement backlog folder creation in project initialization
- Add validation rules for backlog item metadata
- Update document parsing to handle backlog-specific fields

### Phase 2: Backend Operations (2-3 weeks)
**Deliverables:**
- CRUD operations for backlog items
- Item transition logic (backlog → active → initiative)
- Search and filtering capabilities for backlog items
- Integration with existing MCP server

**Tasks:**
- Implement create/read/update/delete for backlog items
- Add move operations between backlog folders
- Implement status transition workflows
- Add backlog-specific search and filter functions
- Extend MCP server with backlog management tools

### Phase 3: TUI Interface (3-4 weeks)
**Deliverables:**
- Backlog management screen with columnar layout
- Navigation and selection mechanisms
- Item creation, editing, and deletion interfaces
- Keyboard shortcuts and action menus

**Tasks:**
- Design and implement backlog TUI layout
- Add column-based navigation with priority grouping
- Implement item detail views and edit forms
- Add keyboard shortcuts for common operations
- Create new item creation wizards for each type

### Phase 4: Advanced Features (2-3 weeks)
**Deliverables:**
- Priority and status management
- Initiative association workflows
- Bulk operations and filtering
- Archive management

**Tasks:**
- Implement priority change operations
- Add initiative selection and association dialogs
- Create bulk action capabilities (multi-select, batch operations)
- Implement archive viewing and restoration
- Add advanced filtering (by assignee, date, priority, etc.)

### Phase 5: Integration & Polish (1-2 weeks)
**Deliverables:**
- Integration with main TUI navigation
- Performance optimization
- Documentation and help system
- User testing and feedback incorporation

**Tasks:**
- Add backlog access to main TUI menu
- Optimize rendering performance for large backlogs
- Create help screens and keyboard shortcut documentation
- Conduct user testing with development team
- Address feedback and polish user experience

## Testing Strategy

### Unit Testing
**Core Library Testing:**
- Test backlog item creation, validation, and parsing
- Test folder structure creation and management
- Test item transition workflows and status changes
- Test search and filtering functionality
- Test integration with existing document management

**Coverage Targets:**
- 90%+ coverage for all backlog-related core functionality
- 100% coverage for critical workflows (item creation, transitions)

### Integration Testing
**MCP Server Integration:**
- Test all backlog management MCP tools
- Validate proper error handling and response formats
- Test concurrent access and file locking scenarios
- Verify integration with existing MCP functionality

**File System Integration:**
- Test backlog folder creation and maintenance
- Verify proper file naming and organization
- Test archive operations and cleanup
- Validate cross-platform file operations

### User Interface Testing
**TUI Interface Testing:**
- Test keyboard navigation across all columns and views
- Verify proper rendering on different terminal sizes
- Test responsive layout behavior
- Validate accessibility features and screen reader compatibility

**User Workflow Testing:**
- End-to-end testing of complete backlog management workflows
- Test item creation through completion lifecycle
- Verify proper state management and UI updates
- Test error scenarios and recovery mechanisms

### Performance Testing
**Scalability Testing:**
- Test TUI performance with large backlogs (100+ items per category)
- Measure rendering time and memory usage
- Test search and filter performance with large datasets
- Validate responsive UI with high item throughput

**Load Testing:**
- Test concurrent operations on backlog items
- Verify file system performance under load
- Test memory usage patterns during extended use

### User Acceptance Testing
**Development Team Testing:**
- Internal dogfooding with actual project backlogs
- Feedback collection on workflow efficiency
- Usability testing with different user personas
- Validation of productivity improvements

**Success Criteria:**
- Users can create and manage backlog items 50% faster than current external tools
- 95% of common backlog operations can be completed without consulting documentation
- Zero data loss during item transitions and operations
- TUI remains responsive with backlogs up to 500 items per category

### Validation Scenarios
**Core Workflows:**
1. Create bug report from TUI → verify proper template and metadata
2. Move feature request to active → verify status updates and file location
3. Associate active item with initiative → verify metadata and traceability
4. Complete item and archive → verify cleanup and historical tracking
5. Search and filter operations → verify accuracy and performance
6. Bulk operations → verify consistency and atomicity

**Error Scenarios:**
1. Invalid metadata formats → verify graceful error handling
2. File system permissions issues → verify appropriate error messages
3. Concurrent edit conflicts → verify conflict resolution
4. Corrupted backlog items → verify recovery mechanisms