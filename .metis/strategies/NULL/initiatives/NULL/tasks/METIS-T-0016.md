---
id: standardized-tool-output-formatting
level: task
title: "Standardized Tool Output Formatting"
short_code: "METIS-T-0016"
created_at: 2025-11-25T02:33:01.340238+00:00
updated_at: 2025-11-25T02:33:01.340238+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Standardized Tool Output Formatting

**As a** consumer of MCP tool results  
**I want** consistent, readable formatting across tool responses  
**So that** output is scannable and structured in terminal contexts

## Context

MCP tool results render as markdown in Claude Code. Raw JSON dumps and unstructured text are hard to parse visually. A thin formatting layer can standardize output across tools without coupling to specific rendering implementations.

## Acceptance Criteria

- [ ] Text results use markdown structure (headers, code blocks, rules) for visual hierarchy
- [ ] Structured data (dicts, lists) renders in fenced code blocks with language hints
- [ ] Tabular data uses ASCII tables when row count is reasonable (< ~20 rows)
- [ ] Status/state indicators use unicode symbols (check, x, warning, filled/empty circles)
- [ ] Errors and warnings are visually distinct from success output
- [ ] Formatting is centralized (single module/utility, not scattered per-tool)

## Out of Scope

- Rich UI rendering (images, HTML, interactive elements)
- Client-specific formatting branches
- Color/ANSI escape codes (terminal compatibility issues)

## Implementation Notes

Keep it simple - a few utility functions that tool handlers call. Don't over-abstract; formatting needs will vary by tool and can be extended as patterns emerge.

## Output Mockups

### create_document

```
## Document Created

METIS-T-0016 created successfully

| Field       | Value                                |
|-------------|--------------------------------------|
| Title       | Standardized Tool Output Formatting  |
| Type        | task                                 |
| Phase       | todo                                 |
| Parent      | -                                    |

Path: `strategies/NULL/initiatives/NULL/tasks/METIS-T-0016.md`
```

### list_documents

```
## Documents (7 total)

### Vision
| Code        | Title            | Phase     |
|-------------|------------------|-----------|
| METIS-V-0001 | Metis Platform  | published |

### Tasks
| Code         | Title                          | Phase |
|--------------|--------------------------------|-------|
| METIS-T-0014 | Fix counter skip bug           | done  |
| METIS-T-0015 | Improve error messages         | doing |
| METIS-T-0016 | Standardized Tool Formatting   | todo  |

### ADRs
| Code        | Title                    | Phase   |
|-------------|--------------------------|---------|
| METIS-A-0001 | Use SQLite for storage  | decided |
```

### read_document

```
## METIS-T-0016: Standardized Tool Output Formatting

| Field    | Value      |
|----------|------------|
| Type     | task       |
| Phase    | todo       |
| Created  | 2025-11-25 |
| Archived | No         |

### Exit Criteria (0/6)
○ Text results use markdown structure
○ Structured data renders in code blocks
○ Tabular data uses ASCII tables
○ Status indicators use unicode symbols
○ Errors visually distinct from success
○ Formatting is centralized

---

**As a** consumer of MCP tool results
**I want** consistent, readable formatting across tool responses
**So that** output is scannable and structured in terminal contexts

[... content continues ...]
```

### transition_phase

```
## Phase Transition

METIS-T-0016: todo -> doing

● todo -> ● doing -> ○ done
```

### edit_document (single replacement)

```
## Document Updated

METIS-T-0016 modified

### Change
` ``diff
- - [ ] Status indicators use unicode symbols (check, x, warning)
+ - [ ] Status indicators use unicode symbols (✓, ✗, ⚠, ●, ○)
` ``

Section: Acceptance Criteria
```

### edit_document (replace_all: true)

```
## Document Updated

METIS-T-0016 modified

### Changes (3 replacements)
` ``diff
- DocumentType::Task
+ DocumentType::WorkItem
` ``

Locations:
  - Line 12 (imports)
  - Line 45 (match arm)
  - Line 89 (return type)
```

### edit_document (multiline replacement)

```
## Document Updated

METIS-A-0001 modified

### Change
` ``diff
- ## Status
- 
- Proposed
+ ## Status
+ 
+ Decided
+ 
+ **Decision Date**: 2025-11-25
+ **Decision Maker**: @dstorey
` ``

Section: Status
```

### search_documents

```
## Search Results for "formatting"

Found 2 matches

| Code         | Title                        | Match Context              |
|--------------|------------------------------|----------------------------|
| METIS-T-0016 | Standardized Tool Formatting | "...formatting layer can..." |
| METIS-A-0003 | Output Format Standards      | "...JSON formatting..."      |
```

### archive_document

```
## Document Archived

METIS-T-0012 and 3 children archived

Archived:
  ✓ METIS-T-0012 (task)
  ✓ METIS-T-0013 (task)
  ✓ METIS-T-0014 (task)
  ✓ METIS-T-0015 (task)

Moved to: `archived/2025-11/`
```

### Error case

```
## Error

✗ Document not found: METIS-T-9999

No document with short code "METIS-T-9999" exists in this project.

Hint: Use `list_documents` to see available documents.
```

### edit_document (no match - error)

```
## Error

✗ No match found in METIS-T-0016

Search text not found:
` ``
- [ ] This criteria doesn't exist
` ``

Hint: Use `read_document` to view current content.
```

## Design Patterns

- **Header** with tool action as H2
- **Short code prominently displayed** (it's the primary identifier)
- **Tables** for structured metadata and lists
- **Progress indicators** (●/○ for phases, ✓/✗ for status)
- **Diff blocks** for edit operations
- **Contextual hints** on errors
- **Minimal prose** - scannable, not chatty

## Status Updates

*To be added during implementation*