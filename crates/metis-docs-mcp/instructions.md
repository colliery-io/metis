# Metis - Flight Levels Work Management

Metis organizes work hierarchically using Flight Levels methodology: Vision (strategic) -> Initiative (projects) -> Task (work items). Work flows down through phases; feedback flows up.

## Document Types & Phases

| Type | Purpose | Phases | Parent Required |
|------|---------|--------|-----------------|
| **Vision** | Strategic direction (6mo-2yr) | draft → review → published | No |
| **Strategy** | Coordinated approaches (Full preset only) | shaping → design → ready → active → completed | Vision (published) |
| **Initiative** | Concrete projects (1-6mo) | discovery → design → ready → decompose → active → completed | Strategy or Vision (published) |
| **Task** | Individual work (1-14 days) | todo → active → completed | Initiative (decompose/active) |
| **Backlog** | Standalone bugs/features/debt | backlog → todo → active → completed | No (use `backlog_category`) |
| **ADR** | Architecture decisions | draft → discussion → decided → superseded | No |

**Note**: Configuration may disable some document types. The current project shows enabled types in tool responses.

## Phase Transition Rules

**IMPORTANT**: Phase transitions are forward-only. You cannot skip phases or go backward.

### Valid Transitions by Document Type

**Vision**: `draft → review → published`
- draft → review
- review → published
- published → (terminal)

**Strategy** (Full preset only): `shaping → design → ready → active → completed`
- shaping → design
- design → ready
- ready → active
- active → completed
- completed → (terminal)

**Initiative**: `discovery → design → ready → decompose → active → completed`
- discovery → design
- design → ready
- ready → decompose
- decompose → active
- active → completed
- completed → (terminal)

**Task**: `backlog → todo → active → completed` (with blocked as alternate state)
- backlog → todo
- todo → active OR blocked
- active → completed OR blocked
- blocked → todo OR active
- completed → (terminal)

**ADR**: `draft → discussion → decided → superseded`
- draft → discussion
- discussion → decided
- decided → superseded
- superseded → (terminal)

### What This Means

- **Cannot skip phases**: A task in "todo" cannot go directly to "completed" - it must go through "active" first
- **Cannot skip phases**: An initiative in "discovery" cannot jump to "active" - it must progress through design, ready, decompose
- **Forward-only**: Phases progress forward; use blocked state for tasks that are stuck
- **Use auto-advance**: Omit the `phase` parameter to automatically move to the next phase in sequence

## Short Codes

All documents get unique IDs: `PREFIX-TYPE-NNNN` (e.g., `PROJ-V-0001`, `ACME-T-0042`)
- **V**=Vision, **S**=Strategy, **I**=Initiative, **T**=Task, **A**=ADR
- Use short codes to reference documents in all operations

## CRITICAL: project_path Format

All tools require a `project_path` parameter. This **MUST** be the path to the `.metis` directory, NOT the project root.

- **Correct**: `/Users/me/my-project/.metis`
- **WRONG**: `/Users/me/my-project`

The server will attempt to auto-correct if you pass the project root, but always include `.metis` in the path.

## Tools Reference

### initialize_project
Create a new Metis workspace.
```
project_path: string (required) - Path where .metis/ will be created
prefix: string (optional) - Short code prefix, 2-8 uppercase letters (default: "PROJ")
```

### list_documents
List all documents in the project.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
include_archived: bool (optional) - Include archived docs (default: false)
```

### search_documents
Full-text search across documents.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
query: string (required) - Search text
document_type: string (optional) - Filter: vision, strategy, initiative, task, adr
limit: number (optional) - Max results
include_archived: bool (optional) - Include archived docs (default: false)
```

### read_document
Get full document content and metadata.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
short_code: string (required) - Document ID (e.g., PROJ-I-0001)
```

### create_document
Create a new document.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
document_type: string (required) - vision, strategy, initiative, task, adr
title: string (required) - Document title
parent_id: string (optional) - Parent short code (required for strategy/initiative/task)
risk_level: string (optional) - For strategies: low, medium, high, critical
complexity: string (optional) - For initiatives: xs, s, m, l, xl
decision_maker: string (optional) - For ADRs
backlog_category: string (optional) - For backlog items: bug, feature, tech-debt
```
**CRITICAL**: Creating a document is only the first step. You MUST immediately follow up with `read_document` then `edit_document` to populate the content sections with actual information. A document with only template placeholders is incomplete and useless.

### edit_document
Search-and-replace edit on document content.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
short_code: string (required) - Document ID
search: string (required) - Text to find
replace: string (required) - Replacement text
replace_all: bool (optional) - Replace all occurrences (default: false)
```

### transition_phase
Advance document to its next phase or transition to a valid adjacent phase.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
short_code: string (required) - Document ID
phase: string (optional) - Target phase (must be a valid adjacent phase - see Phase Transition Rules)
force: bool (optional) - Skip exit criteria validation
```
**IMPORTANT**: You cannot skip phases. See "Phase Transition Rules" section for valid transitions from each phase.
**Best practice**: Omit `phase` to auto-advance to the next sequential phase. Only specify phase for:
- Moving to blocked state (tasks only)
- Returning from blocked to todo or active (tasks only)

**For initiatives and strategies**: ALWAYS check in with the human before transitioning phases. Summarize current state and get explicit approval to proceed.

### archive_document
Archive a document and all its children.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
short_code: string (required) - Document ID
```

### reassign_parent
Move a task to a different parent initiative or to/from the backlog.
```
project_path: string (required) - Path to the .metis folder (e.g., "/path/to/project/.metis")
short_code: string (required) - Task short code to reassign
new_parent_id: string (optional) - Target initiative short code. Omit to move to backlog.
backlog_category: string (optional) - Required when moving to backlog: bug, feature, tech-debt
```
**Note**: Only tasks can be reassigned. Target initiative must be in `decompose` or `active` phase.

## Common Workflows

### Starting a Project
1. `initialize_project` - Create workspace
2. `create_document` type=vision - Define strategic direction
3. `transition_phase` - Move vision through draft -> review -> published
4. `create_document` type=initiative parent_id=PROJ-V-0001 - Create initiatives under vision

### Managing Work
1. `list_documents` - See all active work
2. `read_document` - Check document details and exit criteria
3. `transition_phase` - Advance work through phases
4. `edit_document` - Update content, add notes, mark blockers

### Creating Backlog Items
For standalone bugs, features, or tech debt not tied to initiatives:
```
create_document:
  document_type: "task"
  title: "Fix login timeout"
  backlog_category: "bug"  # or "feature" or "tech-debt"
```

### Decomposing Initiatives
1. Transition initiative to "decompose" phase
2. Create tasks with parent_id pointing to the initiative
3. Transition initiative to "active" when ready to execute

### Assigning Backlog Items to Initiatives
To move a standalone backlog item into an initiative:
```
reassign_parent:
  short_code: "PROJ-T-0042"
  new_parent_id: "PROJ-I-0005"
```

To move a task back to the backlog:
```
reassign_parent:
  short_code: "PROJ-T-0042"
  backlog_category: "tech-debt"
```

## Key Principles

- **ALWAYS populate document content**: Creating a document is NOT complete until you edit it with real content. The workflow is: `create_document` → `read_document` → `edit_document` with actual information. Never leave template placeholders.
- **Read before edit**: Always `read_document` before `edit_document`
- **Delete unused sections**: Templates contain optional sections. If a section doesn't apply to your document, delete it entirely rather than leaving it empty or with placeholder text
- **Auto-transition**: Omit phase parameter to follow natural workflow
- **Hierarchy matters**: Tasks need initiatives, initiatives need strategies (Full) or visions (Streamlined)
- **Short codes everywhere**: Reference documents by ID, not title
- **Archive completed work**: Use `archive_document` to clean up finished trees

## Human-in-the-Loop for Strategic Work

**CRITICAL**: Initiatives and strategies represent higher-level strategic decisions that require human oversight. Agents should guide and support, but humans must remain in control of strategic direction.

### When to Check In With Humans

**ALWAYS pause and consult the human before:**
- Transitioning an initiative or strategy to a new phase
- Making architectural or design decisions
- Decomposing an initiative into tasks
- Any action that commits significant resources or direction

### Required Behaviors for Initiatives/Strategies

1. **Discovery Phase**: Ask clarifying questions about scope, priorities, and constraints. Do NOT assume you understand the full context.

2. **Design Phase**: Present multiple options with trade-offs. Let the human choose the approach rather than deciding unilaterally.

3. **Before Decomposition**: Review the proposed task breakdown with the human. Get explicit approval before creating tasks.

4. **Phase Transitions**: Summarize current state, what was accomplished, and what the next phase entails. Ask for approval to proceed.

### How to Check In

When working on initiatives or strategies:
```
"Here's the current state of [INITIATIVE-CODE]:
- Completed: [summary]
- Current phase: [phase]
- Proposed next steps: [what you plan to do]

Do you want me to proceed, or would you like to adjust the direction?"
```

### What NOT to Do

- Do NOT autonomously transition initiatives through multiple phases
- Do NOT create large numbers of tasks without human review
- Do NOT make strategic assumptions - ask instead
- Do NOT skip the check-in because "it seems obvious"

Agents are powerful assistants for strategic work, but the human must drive the decisions. When in doubt, ask.

## Using Active Tasks as Working Memory

**CRITICAL**: Active tasks and initiatives serve as persistent working memory. While a task is in the `active` phase, you MUST regularly update it with progress, findings, and plan changes as you work.

### Why This Matters
- Long-running tasks may experience context compaction (memory loss)
- Documents persist across sessions and context windows
- Future work can reference past decisions and discoveries
- Other agents/humans can pick up where you left off

### What to Record in Active Tasks
Update frequently during active work:
- **Progress**: What you've completed, files modified, tests run
- **Findings**: Unexpected discoveries, code patterns found, blockers encountered
- **Decisions**: Why you chose approach A over B, trade-offs considered
- **Plan changes**: If original approach didn't work, document what changed and why
- **Next steps**: What remains to be done if work is interrupted

### How Often to Update
- After completing each significant step
- When you discover something unexpected
- When your approach changes from the original plan
- Every few tool calls during long operations
- Before ending a session with incomplete work

### Example Update Pattern
```
edit_document:
  short_code: "PROJ-T-0042"
  search: "## Progress"
  replace: "## Progress\n\n### Session 1\n- Investigated auth module, found rate limiter at src/auth/limiter.rs\n- Original plan to modify middleware won't work - limiter is applied earlier\n- New approach: add bypass flag to limiter config\n- Modified: src/auth/limiter.rs, src/config/auth.yaml\n- Tests passing locally, need integration test"
```

This ensures no work is lost even if context is compacted or the session ends unexpectedly.

## Common Mistakes to Avoid

**Creating documents without content**: This is the #1 mistake. You MUST populate documents with real information immediately after creation. The correct workflow is:
1. `create_document` - Creates document with template
2. `read_document` - Read the template to see what sections exist
3. `edit_document` - Replace placeholder text with actual content

Do NOT move on to other tasks until the document has meaningful content. A document full of `{placeholder text}` is worthless.

**Phase skipping will fail**: These transitions are INVALID and will error:
- `todo → completed` (must go todo → active → completed)
- `discovery → active` (must progress through all intermediate phases)
- `draft → published` (must go draft → review → published)

**Backward transitions are not supported**: Phases only move forward. Use the blocked state for tasks that are stuck.

**To complete a task**, call `transition_phase` twice:
1. First call: todo → active (start working)
2. Second call: active → completed (finish work)

**To publish a vision**, call `transition_phase` twice:
1. First call: draft → review
2. Second call: review → published
