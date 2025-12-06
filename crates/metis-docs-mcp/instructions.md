# Metis - Flight Levels Work Management

Metis organizes work hierarchically using Flight Levels methodology: Vision (strategic) -> Initiative (projects) -> Task (work items). Work flows down through phases; feedback flows up.

## Document Types & Phases

| Type | Purpose | Phases | Parent Required |
|------|---------|--------|-----------------|
| **Vision** | Strategic direction (6mo-2yr) | draft -> review -> published | No |
| **Initiative** | Concrete projects (1-6mo) | discovery -> design -> ready -> decompose -> active -> completed | Vision (published) |
| **Task** | Individual work (1-14 days) | todo -> doing -> completed | Initiative (decompose/active) |
| **Backlog** | Standalone bugs/features/debt | backlog -> todo -> doing -> completed | No (use `backlog_category`) |
| **ADR** | Architecture decisions | draft -> discussion -> decided -> superseded | No |

**Note**: Configuration may disable some document types. The current project shows enabled types in tool responses.

## Short Codes

All documents get unique IDs: `PREFIX-TYPE-NNNN` (e.g., `PROJ-V-0001`, `ACME-T-0042`)
- **V**=Vision, **I**=Initiative, **T**=Task, **A**=ADR
- Use short codes to reference documents in all operations

## Tools Reference

### initialize_project
Create a new Metis workspace.
```
project_path: string (required) - Path where .metis/ will be created
prefix: string (optional) - Short code prefix, max 6 chars (default: "PROJ")
```

### list_documents
List all documents in the project.
```
project_path: string (required) - Path to .metis folder
include_archived: bool (optional) - Include archived docs (default: false)
```

### search_documents
Full-text search across documents.
```
project_path: string (required) - Path to .metis folder
query: string (required) - Search text
document_type: string (optional) - Filter: vision, initiative, task, adr
limit: number (optional) - Max results
include_archived: bool (optional) - Include archived docs (default: false)
```

### read_document
Get full document content and metadata.
```
project_path: string (required) - Path to .metis folder
short_code: string (required) - Document ID (e.g., PROJ-I-0001)
```

### create_document
Create a new document.
```
project_path: string (required) - Path to .metis folder
document_type: string (required) - vision, initiative, task, adr
title: string (required) - Document title
parent_id: string (optional) - Parent short code (required for initiative/task)
complexity: string (optional) - For initiatives: xs, s, m, l, xl
decision_maker: string (optional) - For ADRs
backlog_category: string (optional) - For backlog items: bug, feature, tech-debt
```

### edit_document
Search-and-replace edit on document content.
```
project_path: string (required) - Path to .metis folder
short_code: string (required) - Document ID
search: string (required) - Text to find
replace: string (required) - Replacement text
replace_all: bool (optional) - Replace all occurrences (default: false)
```

### transition_phase
Move document to next phase or specific phase.
```
project_path: string (required) - Path to .metis folder
short_code: string (required) - Document ID
phase: string (optional) - Target phase (omit for auto-advance)
force: bool (optional) - Skip exit criteria validation
```
**Best practice**: Omit `phase` to auto-advance. Only specify phase for non-linear transitions like marking tasks "blocked".

### archive_document
Archive a document and all its children.
```
project_path: string (required) - Path to .metis folder
short_code: string (required) - Document ID
```

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

## Key Principles

- **Read before edit**: Always `read_document` before `edit_document`
- **Auto-transition**: Omit phase parameter to follow natural workflow
- **Hierarchy matters**: Tasks need initiatives, initiatives need visions
- **Short codes everywhere**: Reference documents by ID, not title
- **Archive completed work**: Use `archive_document` to clean up finished trees
