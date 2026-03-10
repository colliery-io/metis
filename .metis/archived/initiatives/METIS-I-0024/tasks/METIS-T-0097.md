---
id: remove-strategy-from-mcp-tools
level: task
title: "Remove Strategy from MCP tools"
short_code: "METIS-T-0097"
created_at: 2026-03-03T19:10:52.293987+00:00
updated_at: 2026-03-04T01:18:01.395188+00:00
parent: METIS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0024
---

# Remove Strategy from MCP tools

## Parent Initiative

[[METIS-I-0024]]

## Objective

Remove strategy as a valid document type from all MCP tool parameters and simplify initiative parent handling in the create_document tool.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `"strategy"` removed from valid `document_type` values in `create_document.rs`
- [ ] `risk_level` parameter removed from create_document tool (strategy-only field)
- [ ] Strategy enablement validation removed from create_document
- [ ] Strategy creation branch removed from create_document execution logic
- [ ] Initiative parent handling simplified — no NULL strategy pattern, parent_id points directly to vision
- [ ] Tool descriptions updated — no mention of strategy in parameter docs
- [ ] Any other MCP tools referencing strategy (list, search, transition, etc.) updated
- [ ] MCP server instructions/description text updated to remove strategy references

## Implementation Notes

Depends on METIS-T-0091 (domain model) and METIS-T-0092 (services). The MCP layer is thin — mostly parameter validation and delegation to services.

## Status Updates

### Session 2 (continued from context compaction)

**All MCP strategy references removed.** Files modified:

**MCP crate (primary scope):**
- `crates/metis-docs-mcp/src/lib.rs` — Removed `strategies_enabled` references from `generate_operation_notes()`, updated config text (removed --strategies flag, "full" preset)
- `crates/metis-docs-mcp/src/tools/create_document.rs` — Major rewrite: removed Strategy match arm, `risk_level` field, `find_strategy_short_code_for_initiative()` method, `Database` import. Simplified `create_initiative_with_config(config, &flight_config)` (2 args) and `create_task_with_config(config, initiative_id, &flight_config)` (3 args)
- `crates/metis-docs-mcp/src/tools/transition_phase.rs` — Removed `Phase::Shaping` and `DocumentType::Strategy` from parsing
- `crates/metis-docs-mcp/src/tools/search_documents.rs` — Updated doc comment to remove "strategy" from valid types
- `crates/metis-docs-mcp/src/tools/list_documents.rs` — Removed "strategy" from doc type iteration and sort order
- `crates/metis-docs-mcp/instructions.md` — Removed Strategy row from table, Strategy phase sequence, "strategy" from short code types, tool parameter docs, preset references

**CLI crate (leftover fixes from T-0096):**
- `create/adr.rs`, `create/initiative.rs`, `create/task.rs` — Removed `risk_level: None` from DocumentCreationConfig
- `transition.rs` — Removed `Phase::Shaping` from parsing
- `cli.rs` — Fixed unused `OutputFormat` import

**GUI crate (discovered during workspace compile):**
- `services/document.rs` — Removed `find_strategy_short_code_for_initiative()`, removed `risk_level` from struct/config, removed "strategy" create branch, simplified initiative/task creation calls, fixed `get_available_parents` (initiatives now parent to visions not strategies), removed "strategy" from doc type list, fixed tests
- `services/project.rs` — Removed `strategies_enabled` from ProjectConfig, removed "full" preset
- `services/transition.rs` — Removed `Phase::Shaping`

**Verification:**
- `cargo check --workspace` — clean (only pre-existing warnings)
- `angreal test-core` — 184 tests pass
- `grep -r strateg` in MCP src — zero matches (only "strategic" adjective in instructions.md)