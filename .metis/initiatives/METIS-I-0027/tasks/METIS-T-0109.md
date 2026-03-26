---
id: open-document-mcp-tool
level: task
title: "open_document MCP tool"
short_code: "METIS-T-0109"
created_at: 2026-03-26T14:59:09.821963+00:00
updated_at: 2026-03-26T14:59:09.821963+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# open_document MCP tool

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Implement the `open_document` MCP tool that resolves short codes to file paths and dispatches to the configured viewer backend. This is the primary interface agents and users interact with to open documents externally.

## Acceptance Criteria

- [ ] New `open_document` tool registered in the MCP server
- [ ] Accepts `short_code` (required), `include_children` (bool, default false), `viewer` (optional override)
- [ ] Resolves short code to file path via existing document index
- [ ] When `include_children` is true, gathers all child document paths (tasks under an initiative)
- [ ] Reads `viewer` param or falls back to `default_viewer` from arawn.toml config, then `$EDITOR`
- [ ] Dispatches to the resolved `DocumentViewer` backend
- [ ] Returns success with list of opened file paths, or clear error if short code not found / viewer unavailable
- [ ] Tool description is clear enough for agents to use it correctly without additional prompting

## Implementation Notes

### Technical Approach
- Add tool handler in metis-mcp alongside existing tools
- Reuse existing short code resolution logic from `read_document`
- For `include_children`, query the document index for children of the given short code
- Call viewer dispatcher from METIS-T-0107

### Dependencies
- METIS-T-0107 (viewer config and trait abstraction) — needs the trait and dispatcher

## Status Updates

*To be added during implementation*