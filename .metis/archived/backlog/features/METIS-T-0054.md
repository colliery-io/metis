---
id: add-reassign-parent-mcp-tool-for
level: task
title: "Add reassign_parent MCP tool for parent assignment"
short_code: "METIS-T-0054"
created_at: 2025-12-31T16:51:51.628299+00:00
updated_at: 2025-12-31T17:36:23.303467+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#feature"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Add reassign_document MCP tool for parent assignment

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Objective

Add a `reassign_parent` MCP tool that allows moving documents between parents (e.g., assigning a backlog item to an initiative, or moving a task between initiatives).

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement  

### Priority
- [x] P2 - Medium (nice to have)

### Business Justification
- **User Value**: Currently, reassigning a backlog item to an initiative requires knowing the filesystem structure and manually moving files. A dedicated tool makes this discoverable and simple.
- **Business Value**: Improves workflow for managing backlog items through their lifecycle
- **Effort Estimate**: M (Medium)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Tool accepts `short_code` of document to move and `new_parent_id` (short code of target parent, or empty to move to backlog)
- [ ] Backlog item → Initiative: moves file from `backlog/` to `strategies/NULL/initiatives/{id}/tasks/`
- [ ] Task → Different Initiative: moves file between initiative task folders
- [ ] Task → Backlog: moves file from initiative tasks to appropriate `backlog/{category}/` folder
- [ ] Validates target parent exists and is in appropriate phase (decompose/active for initiatives)
- [ ] Preserves document short_code during move
- [ ] Updates MCP server instructions to document the new tool
- [ ] Updates skill documentation with reassignment workflow

## Implementation Notes

### Technical Approach
1. Create `reassign_parent.rs` in `crates/metis-docs-mcp/src/tools/`
2. Tool parameters:
   - `project_path`: Path to .metis folder
   - `short_code`: Document to reassign
   - `new_parent_id`: Target parent short code (or null/empty for backlog)
   - `backlog_category`: Required when moving to backlog (bug/feature/tech-debt)
3. Implementation:
   - Resolve source document path via short_code
   - Validate target parent exists and is in valid phase
   - Compute destination path based on target parent's location
   - Move file using `std::fs::rename`
   - Sync will auto-detect the move and update DB lineage

### Dependencies
- Sync service already handles move detection via short_code matching
- No core library changes needed - filesystem is source of truth

### Edge Cases
- Moving to backlog requires specifying category (bug/feature/tech-debt)
- Cannot reassign Vision, Strategy, Initiative, or ADR documents
- Cannot reassign to a parent that isn't in decompose/active phase

## Status Updates

- 2025-12-31: Created ticket after identifying need during skill documentation review
- 2025-12-31: Implemented `reassign_parent` tool:
  - Created `ReassignmentService` in `metis-docs-core/src/application/services/workspace/reassignment.rs`
  - Created thin MCP wrapper in `metis-docs-mcp/src/tools/reassign_parent.rs`
  - Added comprehensive integration tests in `tests/reassignment_test.rs` (6 tests)
  - Updated MCP server instructions with tool documentation and workflows