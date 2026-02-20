---
id: reassign-parent-tool-not-appearing
level: task
title: "reassign_parent tool not appearing in MCP server"
short_code: "METIS-T-0065"
created_at: 2026-01-28T14:46:16.234025+00:00
updated_at: 2026-01-28T14:58:51.925304+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# reassign_parent tool not appearing in MCP server

## Objective

Fix the reassign_parent tool which is not appearing/exposed in the MCP server.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [ ] P1 - High (important for user experience)

### Impact Assessment
- **Affected Users**: All users trying to reassign tasks between initiatives or move to/from backlog
- **Reproduction Steps**: 
  1. Connect to Metis MCP server
  2. List available tools
  3. Observe that reassign_parent is missing from the tool list
- **Expected vs Actual**: 
  - Expected: reassign_parent tool should be listed and callable
  - Actual: Tool does not appear in the MCP server's tool list

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] reassign_parent tool appears in MCP server tool listing
- [ ] Tool is callable and functions correctly
- [ ] Tool documentation/schema is properly exposed

## Investigation Notes

The reassign_parent functionality exists in the core library (METIS-T-0054 was completed), but the MCP server may not be exposing it. Need to check:
- MCP server tool registration
- Whether the tool handler was implemented
- Any errors during server initialization

## Implementation Notes

### Likely Locations
- `metis-mcp-server/src/` - MCP server implementation
- Tool registration code
- Handler implementations

## Status Updates

### 2026-01-28: Investigation Complete

**Finding: NOT A CODE BUG**

The `reassign_parent` tool IS correctly implemented and exposed:

1. **Code check**: Tool is properly registered in `all_tools.rs` (line 22) and `mod.rs` (lines 8, 19)
2. **Build check**: `cargo check -p metis-docs-mcp` passes with no errors
3. **Runtime check**: Querying `metis mcp` with `tools/list` returns `reassign_parent` as the 9th tool with correct schema
4. **Version check**: Installed `metis` (1.0.11) matches repo version

**Likely cause**: Claude Code session caching or MCP server connection refresh issue. The tool exists under two namespaces:
- `mcp__plugin_metis_metis__reassign_parent` (plugin MCP)
- `mcp__metis__reassign_parent` (may require session refresh)

**Resolution**: No code changes needed. If tool doesn't appear, restart Claude Code session or refresh MCP server connection.