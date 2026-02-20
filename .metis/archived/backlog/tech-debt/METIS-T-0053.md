---
id: skill-documentation-ambiguity-and
level: task
title: "Skill documentation ambiguity and missing interface descriptions"
short_code: "METIS-T-0053"
created_at: 2025-12-31T16:40:14.523160+00:00
updated_at: 2025-12-31T17:54:29.013968+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: true
strategy_id: NULL
initiative_id: NULL
---

# Skill documentation ambiguity and missing interface descriptions

## Objective

Fix ambiguous language and missing interface/process descriptions in the Metis skill documentation to prevent agent confusion and ensure consistent behavior.

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: Agents receive conflicting or incomplete guidance, leading to errors like creating documents under wrong parent states, not knowing about archive operations, or confusion about backlog items vs tasks
- **Benefits of Fixing**: Agents will have clear, unambiguous instructions; fewer user interventions needed; consistent document creation workflow
- **Risk Assessment**: Medium - agents currently work but make avoidable mistakes; fixing improves reliability

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### High Priority
- [x] **Parent state requirements table**: Add clear table showing document type → parent type → required parent phase (Vision=published, Strategy=active, Initiative=decompose/active)
- [x] **Clarify backlog terminology**: Make explicit that "Backlog Item" = `type="task"` with `backlog_category`, not a separate document type
- [x] **Add `blocked` phase**: Document the blocked phase for tasks in phase-transitions.md

### Medium Priority  
- [x] **Add archive operation**: Document `archive_document` tool usage in SKILL.md and anti-patterns.md
- [x] **Add list_documents usage**: Show how to check existing documents before creating new ones
- [x] **Preset change guidance**: Clarify that preset changes require CLI (`metis config set`), not available via MCP
- [x] **How to check current preset**: Document that MCP tool responses show current preset configuration

### Low Priority
- [x] **Fix test terminology**: Update `formatting.rs:353` test from "doing"/"done" to "active"/"completed"
- [x] **Strategy is preset-specific**: Add note that Strategy phases only apply to Full preset
- [x] **Add complexity parameter**: Include `complexity` in initiative creation examples
- [x] **Document exit_criteria_met field**: Explain this frontmatter field in phase-transitions.md

## Issues Detail

| # | Issue | Location | Severity |
|---|-------|----------|----------|
| 1 | Test uses "doing"/"done" not "active"/"completed" | `formatting.rs:353` | Low |
| 2 | No MCP tool for preset changes | `preset-selection.md:105` | Medium |
| 3 | Parent state requirements unclear | Multiple files | High |
| 4 | Missing `blocked` phase for tasks | `phase-transitions.md:36-47` | Medium |
| 5 | No archive operation docs | All skill files | Medium |
| 6 | "Backlog Item" vs "task with backlog_category" | Throughout | High |
| 7 | Strategy phases are Full-preset only | `phase-transitions.md` | Low |
| 8 | How to check current preset | Not documented | Medium |
| 9 | No parent requirements quick-ref table | `core-principles.md` | Medium |
| 10 | No list_documents guidance | All skill files | Medium |
| 11 | exit_criteria_met field undocumented | `phase-transitions.md` | Low |
| 12 | complexity parameter not in examples | Pattern files | Low |

## Implementation Notes

### Files to Modify
- `skill/AGENTS.md`
- `skill/skills/metis/SKILL.md`
- `skill/skills/metis/methodology/core-principles.md`
- `skill/skills/metis/methodology/phase-transitions.md`
- `skill/skills/metis/methodology/preset-selection.md`
- `skill/skills/metis/methodology/anti-patterns.md`
- `skill/skills/metis/patterns/feature-development.md`
- `skill/skills/metis/patterns/greenfield.md`
- `crates/metis-docs-mcp/src/formatting.rs` (test fix)

## Status Updates

- **2025-12-31**: Created from deep review of skill documentation. 12 issues identified across 9 files.
- **2025-12-31**: Fixed all 3 high priority issues:
  - Added parent requirements table to core-principles.md
  - Clarified backlog terminology in SKILL.md (Task (backlog) with footnote)
  - Documented blocked phase with transitions in phase-transitions.md
  - Also added references to upcoming `reassign_parent` tool (METIS-T-0054)
- **2025-12-31**: Completed all medium and low priority items:
  - Added archive_document example to SKILL.md Common Operations
  - Enhanced anti-patterns.md Stale Work section with archive_document reference
  - Added list_documents usage guidance to SKILL.md
  - Added CLI requirement note and `metis config show` to preset-selection.md
  - Fixed constants.rs TASK_DOING → TASK_ACTIVE, added TASK_BLOCKED
  - Updated formatting.rs test to use active/completed terminology
  - Updated GUI transition.rs comment for legacy compatibility
  - Added Strategy full-preset-only footnote to SKILL.md document types table
  - Added complexity parameter to greenfield.md and tech-debt.md examples
  - Documented exit_criteria_met field in phase-transitions.md