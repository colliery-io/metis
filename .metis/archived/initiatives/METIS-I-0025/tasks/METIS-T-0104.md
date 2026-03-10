---
id: add-specification-to-mcp-tools
level: task
title: "Add Specification to MCP tools"
short_code: "METIS-T-0104"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T05:40:38.119566+00:00
parent: METIS-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0025
---

# Add Specification to MCP tools

## Parent Initiative

[[METIS-I-0025]]

## Objective

Add Specification support to all MCP tool implementations so AI agents can create, list, search, transition, and archive specifications.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **CreateDocumentTool** (`crates/metis-docs-mcp/src/tools/create_document.rs`):
  - Add `"specification"` as valid `document_type`
  - `parent_id` required when `document_type = "specification"`
  - Tool description updated to mention specification
  - Calls `DocumentCreationService::create_specification()` with parent validation
- [ ] **ListDocumentsTool** (`crates/metis-docs-mcp/src/tools/list_documents.rs`):
  - Add `"specification"` / `S` to `type_order_map` for sorting
  - Specifications appear in list output with correct formatting
- [ ] **SearchDocumentsTool** (`crates/metis-docs-mcp/src/tools/search_documents.rs`):
  - Add `"specification"` to valid `document_type` filter values in doc string
- [ ] **TransitionPhaseTool** (`crates/metis-docs-mcp/src/tools/transition_phase.rs`):
  - Add `"specification"` to `get_phase_sequence()` match — returns `["discovery", "drafting", "review", "published"]`
- [ ] **ArchiveDocumentTool** — No changes needed (generic archive works)
- [ ] **ReadDocumentTool** — No changes needed (generic read works)
- [ ] **EditDocumentTool** — No changes needed (generic edit works)
- [ ] MCP server instructions (`mcp_instructions()` or equivalent) updated to document Specification type

## Implementation Notes

The MCP layer is thin — most changes are adding "specification" to match arms and validation lists. Follow how ADR is handled in each tool.

Reference files:
- `crates/metis-docs-mcp/src/tools/create_document.rs` — ADR creation pattern
- `crates/metis-docs-mcp/src/tools/list_documents.rs` — type_order_map
- `crates/metis-docs-mcp/src/tools/transition_phase.rs` — get_phase_sequence

## Status Updates

### Session 1 (2026-03-04)

**All acceptance criteria met.** Build passes, all 316+ tests pass.

**CreateDocumentTool** (`create_document.rs`):
- Updated description to include "specification"
- Updated `document_type` doc to include "specification"
- Updated `parent_id` doc to note required for specification
- Replaced stub error with real implementation: validates parent_id required, calls `create_specification()`

**ListDocumentsTool** (`list_documents.rs`):
- Added `("specification", 1)` to `type_order_map` (between vision and initiative)
- Added `"specification"` to document type iteration list

**SearchDocumentsTool** (`search_documents.rs`):
- Updated `document_type` filter doc to include "specification"

**TransitionPhaseTool** (`transition_phase.rs`):
- Added `"drafting" => Ok(Phase::Drafting)` to `parse_phase()`
- Added `"specification" => Some(DocumentType::Specification)` to `get_phase_sequence()`

**MCP Instructions** (`instructions.md`):
- Added Specification row to Document Types & Phases table
- Added Specification phase transition rules section
- Added `S`=Specification to short codes legend
- Updated create_document tool reference: document_type and parent_id
- Updated search_documents tool reference: document_type filter