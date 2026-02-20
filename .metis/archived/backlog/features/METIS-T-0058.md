---
id: enhanced-beads-inspired-hooks-and
level: task
title: "Enhanced Beads-inspired hooks and CLI alignment"
short_code: "METIS-T-0058"
created_at: 2026-01-25T20:25:32.761937+00:00
updated_at: 2026-01-25T21:01:31.832437+00:00
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

# Enhanced Beads-inspired hooks and CLI alignment

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Context

This is a standalone backlog feature inspired by research into Beads (https://github.com/steveyegge/beads) - a task tracking system for AI agents that has gained significant community adoption.

## Objective

Improve Metis hooks and CLI based on research into Beads (https://github.com/steveyegge/beads) patterns. Make Metis more prescriptive in guiding Claude Code behavior and align CLI output with MCP tool output.

## Research Findings: Beads vs Metis

### What Beads Does Well

1. **Prescriptive Rules**: "Do NOT use TodoWrite - use `bd` instead"
2. **Dynamic Context via `bd prime`**: Adapts output based on current state (ready tasks, auto-sync status, ephemeral branches)
3. **Session Close Protocol ("Land the Plane")**: Mandatory git push, status update, work filing
4. **Ready-to-Use Commands**: Concrete examples, not just suggestions

### Current Metis Gaps

| Aspect | Metis Current | Beads Approach |
|--------|--------------|----------------|
| Tool guidance | Suggestive | Prescriptive |
| Context injection | Static | Dynamic |
| Session discipline | None | Mandatory |
| CLI/MCP alignment | Inconsistent | N/A (CLI only) |

### Full CLI vs MCP Comparison

#### Commands/Tools Mapping

| Operation | CLI Command | MCP Tool | Status |
|-----------|-------------|----------|--------|
| Initialize | `init` | `initialize_project` | ✅ Both exist |
| Create | `create strategy/initiative/task/adr` | `create_document` | ✅ Both exist |
| List | `list` | `list_documents` | ⚠️ Output differs |
| Search | `search` | `search_documents` | ⚠️ Output differs |
| Status | `status` | - | ❌ MCP missing |
| Read | - | `read_document` | ❌ CLI missing |
| Edit | - | `edit_document` | ❌ CLI missing |
| Transition | `transition` | `transition_phase` | ✅ Both exist |
| Archive | `archive` | `archive_document` | ✅ Both exist |
| Reassign | - | `reassign_parent` | ❌ CLI missing |
| Sync | `sync` | - | ❌ MCP missing (by design) |
| Config | `config show/set/get` | - | ❌ MCP missing (by design) |
| Validate | `validate` | - | ❌ MCP missing |
| MCP Server | `mcp` | - | N/A (starts server) |

#### Output Format Inconsistencies

**`list` vs `list_documents`:**
| Aspect | CLI | MCP |
|--------|-----|-----|
| ID | Slug truncated (`add-depend...`) | Short code (`METIS-T-0058`) |
| Columns | TYPE, TITLE, PHASE, ID, UPDATED | Type, Code, Title, Phase |
| Sort | Updated descending | Type order, then short code |
| Updated | Shows timestamp | Not shown |

**`search` vs `search_documents`:**
| Aspect | CLI | MCP |
|--------|-----|-----|
| ID | Slug | Short code |
| Columns | ID, Type, Path | Code, Title, Type |
| Path | Shows full path | Not shown |

**`status` (CLI only):**
- Shows: TITLE, TYPE, PHASE, BLOCKED BY, UPDATED
- Uses slug ID (not short code)
- Priority sorted (actionable first)
- Human-friendly relative time ("2 weeks ago")

**`create` output:**
- CLI: Shows ID (slug), Short Code, Title, Parent
- MCP: Shows Short Code, Title, Type, Parent

### Git Branch Handling (Already Solved)

Investigated Beads' hash-based IDs for merge-conflict-free branching. Found Metis **already handles this** via `resolve_short_code_collisions()` in synchronization.rs - automatically renumbers conflicting short codes on sync.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

### Part 1: CLI Output Alignment
- [ ] CLI `list` shows short codes instead of slugs
- [ ] CLI `list` columns: Type, Code, Title, Phase (match MCP order)
- [ ] CLI `list` supports `--format` option (table, compact, json)
- [ ] CLI `search` shows short codes instead of slugs  
- [ ] CLI `search` columns: Code, Title, Type (match MCP)
- [ ] CLI `status` shows short codes instead of slugs
- [ ] CLI `create` output shows short code prominently (already does, just verify consistency)

### Part 2: Missing CLI Commands (Optional - Lower Priority)
- [ ] Add `metis read <short-code>` command (mirrors `read_document`)
- [ ] Add `metis edit <short-code>` command (mirrors `edit_document`)
- [ ] Add `metis reassign <short-code>` command (mirrors `reassign_parent`)

### Part 3: Global Format Option
- [ ] Add global `--format` flag to CLI (table, compact, json)
- [ ] Compact format: one line per document for scripts/hooks
- [ ] JSON format: machine-readable for programmatic use

### Part 4: Enhanced Hooks
- [ ] SessionStart hook includes "Do NOT use TodoWrite" rule
- [ ] SessionStart hook shows dynamic state via `metis status --format=compact`
- [ ] SessionStart hook includes ready-to-use MCP command examples with actual project_path
- [ ] PreCompact hook re-injects same enhanced context
- [ ] Consider session close discipline (optional - may be too prescriptive)

## Implementation Notes

### Files to Modify

**Part 1: CLI Output Alignment**
| File | Changes |
|------|---------|
| `commands/list.rs` | Replace `doc.id` with `doc.short_code`, reorder columns to Type/Code/Title/Phase |
| `commands/search.rs` | Replace ID column with Code, reorder to Code/Title/Type |
| `commands/status.rs` | Add short_code column, keep BLOCKED BY and relative time |
| `commands/create/*.rs` | Verify short code is shown consistently |

**Part 2: Format Option (shared module)**
| File | Changes |
|------|---------|
| `commands/mod.rs` | Add shared `OutputFormat` enum and formatting utilities |
| `commands/list.rs` | Add `--format` flag, implement table/compact/json outputs |
| `commands/search.rs` | Add `--format` flag, implement table/compact/json outputs |
| `commands/status.rs` | Add `--format` flag, implement table/compact/json outputs |

**Part 3: New CLI Commands (Optional)**
| File | Changes |
|------|---------|
| `commands/read.rs` | New file - read document by short code |
| `commands/edit.rs` | New file - search/replace edit by short code |
| `commands/reassign.rs` | New file - reassign task parent |
| `cli.rs` | Register new subcommands |

**Part 4: Enhanced Hooks**
| File | Changes |
|------|---------|
| `hooks/session-start-hook.sh` | Add prescriptive rules, dynamic state, MCP examples |
| `hooks/pre-compact-hook.sh` | Mirror session-start enhancements |

### Technical Approach

1. **Shared OutputFormat**:
```rust
#[derive(Debug, Clone, ValueEnum)]
pub enum OutputFormat {
    Table,   // Human-readable table (default)
    Compact, // One line per doc: CODE PHASE TITLE
    Json,    // Full JSON for programmatic use
}
```

2. **Compact Format Example**:
```
METIS-T-0058 backlog Enhanced Beads-inspired hooks and CLI alignment
METIS-I-0018 active Enhanced Search Capabilities
METIS-T-0018 todo Test task for UAT
```

3. **JSON Format Example**:
```json
[{"code":"METIS-T-0058","type":"task","title":"...","phase":"backlog"}]
```

4. **Hook Dynamic State**:
```bash
# In session-start-hook.sh
ACTIVE_WORK=$(metis status --format=compact 2>/dev/null | head -5)
```

### Key Reference Files
- `crates/metis-docs-mcp/src/tools/list_documents.rs` - MCP output format to match
- `crates/metis-docs-mcp/src/tools/search_documents.rs` - MCP search output to match
- `crates/metis-docs-mcp/src/formatting.rs` - ToolOutput patterns for consistency

## Status Updates

### 2026-01-25 - Research Complete
- Investigated Beads repository and documentation
- Identified key patterns: `bd prime`, prescriptive rules, dynamic context, session close protocol
- Reviewed all Metis CLI commands (list, search, status, etc.)
- Compared CLI output to MCP tool output - found inconsistencies
- Confirmed git branch handling is already solved via collision detection
- Created this task to preserve research before context compaction

### 2026-01-25 - Full CLI Analysis Complete
- Mapped all 11 CLI commands to 9 MCP tools
- Identified 3 missing CLI commands (read, edit, reassign)
- Identified 3 CLI-only commands (sync, config, validate) - intentional
- Documented all output format inconsistencies between CLI and MCP
- Defined shared OutputFormat enum for table/compact/json
- Updated acceptance criteria with prioritized work items

### 2026-01-25 - Implementation Complete
**Part 1: CLI Output Alignment - DONE**
- [x] `list.rs`: Now shows short codes, columns match MCP (Type, Code, Title, Phase)
- [x] `search.rs`: Now shows short codes, columns match MCP (Code, Title, Type)  
- [x] `status.rs`: Now shows short codes, added `--format` option
- [x] `create/task.rs`: Fixed to show short code prominently (was showing UUID)

**Part 3: Format Option - DONE**
- [x] Created shared `OutputFormat` enum (Table, Compact, Json) in `list.rs`
- [x] Exported via `mod.rs` for reuse across commands
- [x] Added `--format` flag to `list`, `search`, and `status` commands
- [x] Compact format: `CODE PHASE TITLE` - perfect for hooks
- [x] JSON format: Machine-readable for programmatic use

**Part 4: Enhanced Hooks - DONE**
- [x] `session-start-hook.sh`: Added "Do NOT use TodoWrite" rule
- [x] `session-start-hook.sh`: Dynamic state via `metis status --format compact`
- [x] `session-start-hook.sh`: Ready-to-use MCP tool examples with descriptions
- [x] `pre-compact-hook.sh`: Mirrors session-start enhancements
- [x] Both hooks show current project state (blocked/active/todo counts)
- [x] Both hooks list actionable work items

**Files Modified:**
- `crates/metis-docs-cli/src/commands/list.rs`
- `crates/metis-docs-cli/src/commands/search.rs`
- `crates/metis-docs-cli/src/commands/status.rs`
- `crates/metis-docs-cli/src/commands/mod.rs`
- `crates/metis-docs-cli/src/commands/create/task.rs`
- `crates/metis-docs-cli/src/cli.rs` (test updates)
- `plugins/metis/hooks/session-start-hook.sh`
- `plugins/metis/hooks/pre-compact-hook.sh`

**Tests:**
- All relevant unit tests pass (list, status, search tests)
- Hook scripts tested with both old and new metis binary versions

**Not Implemented (Optional):**
- Part 2: Missing CLI commands (read, edit, reassign) - deferred for future work