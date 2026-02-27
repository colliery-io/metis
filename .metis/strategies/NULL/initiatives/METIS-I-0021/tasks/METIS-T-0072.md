---
id: add-index-code-mcp-tool
level: task
title: "Add index_code MCP tool"
short_code: "METIS-T-0072"
created_at: 2026-02-20T14:47:11.391775+00:00
updated_at: 2026-02-25T05:12:16.684103+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0021
---

# Add index_code MCP tool

## Parent Initiative
[[METIS-I-0021]]

## Objective

Expose code indexing as an MCP tool so AI agents can trigger index generation programmatically. Parameters: `project_path`, `structure_only` (bool), `incremental` (bool). Calls the same pipeline as `metis index`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `index_code` MCP tool registered in metis-docs-mcp
- [ ] Parameters: `project_path` (required), `structure_only` (optional bool), `incremental` (optional bool)
- [ ] Calls the same indexing pipeline as `metis index` CLI
- [ ] Returns success with stats (files indexed, languages, time)
- [ ] Returns clear error if `.metis/` doesn't exist
- [ ] Tool description in MCP schema is clear for AI consumption
- [ ] `angreal test` passes

## Implementation Notes

Add a new tool handler in `metis-docs-mcp/src/tools/`. Follow the pattern of existing tools (e.g., `create_document`). The tool calls into metis-code-index crate's public API.

Blocked by: METIS-T-0071 (CLI command wires up the pipeline first)

## Progress

### Session 1 (2026-02-24)
- Added `metis-code-index` as dependency of `metis-docs-mcp` in Cargo.toml
- Created `crates/metis-docs-mcp/src/tools/index_code.rs` with `IndexCodeTool`
  - Parameters: `project_path` (required), `structure_only` (optional bool), `incremental` (optional bool)
  - Validates `.metis/` directory exists, returns clear error if not
  - Full pipeline: walk files → parse → extract symbols → write `.metis/code-index.md`
  - Returns stats table: files indexed, symbols extracted, time, output path, parse errors, mode
  - Returns languages detected with file counts
  - MCP tool hints: idempotent, not destructive, not read-only
- Registered in `tools/mod.rs`, `tools/all_tools.rs` (tool_box! macro), and `server.rs` (dispatch match)
- All tests pass via `angreal test`, formatting clean
- All acceptance criteria met