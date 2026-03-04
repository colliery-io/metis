---
id: remove-strategy-layer-from-metis
level: initiative
title: "Remove Strategy Layer from Metis"
short_code: "METIS-I-0024"
created_at: 2026-03-03T17:39:39.082072+00:00
updated_at: 2026-03-03T19:17:43.862514+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: remove-strategy-layer-from-metis
---

# Remove Strategy Layer from Metis

## Context

The Strategy document type in Metis was inherited from Flight Levels methodology as a coordination layer between Vision and Initiative. ADR-0007 (METIS-A-0007) established that multi-team and cross-repo coordination is out of scope for Metis — the tool is operationally coupled to the repository and that's a strength, not a limitation. Strategy was the last vestige of that organizational coordination concept in the document model.

In practice, across 20+ projects over 6+ month cycles with dozens of initiatives and hundreds of tasks, the Strategy layer has never been meaningfully used. When created, strategies became static props that never changed — they added a hierarchy level without adding information or decision points. The layer exists in the codebase but serves no purpose at the repo-scoped level where Metis operates.

The `S` short code prefix and the hierarchy slot will be repurposed for a new "Specification" document type (a separate follow-on initiative) that captures system-level design: PRDs, requirements, system context, and architecture framing. This initiative handles the prerequisite — cleanly removing the vestigial Strategy type so the codebase is clean and the slot is open.

## Goals & Non-Goals

**Goals:**
- Remove the Strategy document type entirely from the domain model, CLI, MCP tools, templates, database schema, and configuration
- Reclaim the `S` short code prefix for the upcoming Specification type
- Clean up the NULL strategy directory pattern used for parentless initiatives in the streamlined/direct presets
- Ensure all existing presets (full, streamlined, direct) work correctly without strategies
- All tests pass after removal

**Non-Goals:**
- Building the Specification type — that's a separate follow-on initiative
- Migrating existing strategy documents — no known Metis deployments have real strategy documents in use
- Changing the Vision → Initiative → Task hierarchy — initiatives will parent directly under Vision (as they already do in streamlined preset)

## Detailed Design

### Codebase Inventory (60+ files, ~40 direct struct references)

**Domain model — complete removal:**
- `crates/metis-docs-core/src/domain/documents/strategy/mod.rs` — `Strategy` struct, `RiskLevel` enum, phase sequence (Shaping→Design→Ready→Active→Completed), Document trait impl. Delete entire module.
- `crates/metis-docs-core/src/domain/documents/types.rs` — `DocumentType::Strategy` enum variant, `Phase::Shaping` definition, strategy phase sequence in `phase_sequence()`, valid transitions. Remove variant and all strategy-specific phases.
- `crates/metis-docs-core/src/domain/documents/factory.rs` — `DocumentFactory::from_file()` and `from_content()` strategy match arms. Remove.
- `crates/metis-docs-core/src/domain/documents/traits.rs` — `DocumentCore::strategy_id: Option<DocumentId>` field. Remove field.
- `crates/metis-docs-core/src/domain/documents/mod.rs` — `pub mod strategy;` declaration. Remove.
- `crates/metis-docs-core/src/lib.rs` — exports `strategy::{RiskLevel, Strategy}`. Remove.

**Templates — delete:**
- `crates/metis-docs-core/src/domain/documents/strategy/frontmatter.yaml`
- `crates/metis-docs-core/src/domain/documents/strategy/content.md`
- `crates/metis-docs-core/src/templates/strategy/content.md`
- `crates/metis-docs-core/src/templates/strategy/postmatter.md`
- `crates/metis-docs-core/src/templates/strategy/frontmatter.yaml`

**Constants — remove:**
- `crates/metis-docs-core/src/constants.rs` — `STRATEGY_DIR`, `STRATEGY_TEMPLATE`, five phase constants (`STRATEGY_SHAPING` through `STRATEGY_COMPLETED`), risk level constants

**Configuration — simplify:**
- `crates/metis-docs-core/src/domain/configuration.rs` — Remove `strategies_enabled` from `FlightLevelConfig`. The "full" preset becomes identical to "streamlined" (both are now Vision→Initiative→Task). Decide whether to keep or collapse presets.

**Services — remove strategy match arms:**
- `creation.rs` — Delete `create_strategy()` method. Simplify `create_initiative_with_config()` to stop handling `strategy_id` and the NULL pattern. Initiatives go directly under `.metis/initiatives/`.
- `discovery.rs` — Remove `DocumentType::Strategy` from `find_document_of_type()`. Remove strategy path logic. Remove `"S"` → Strategy mapping from `document_type_from_short_code()`.
- `transition.rs` — Remove `DocumentType::Strategy` match arm from `get_current_phase()` and `perform_transition()`.
- `archive.rs` — Remove `DocumentType::Strategy` match arm from `mark_as_archived_helper()`.

**CLI — remove command:**
- Delete `crates/metis-docs-cli/src/commands/create/strategy.rs`
- Remove `CreateCommands::Strategy` variant from `crates/metis-docs-cli/src/commands/create/mod.rs`
- Update initiative creation in `crates/metis-docs-cli/src/commands/create/initiative.rs` to remove strategy parent handling

**MCP — remove from tool parameters:**
- `crates/metis-docs-mcp/src/tools/create_document.rs` — Remove "strategy" from valid `document_type` values. Remove `risk_level` parameter. Remove strategy enablement validation. Remove strategy creation logic. Simplify initiative parent handling to remove NULL pattern.

**Database — migration:**
- New migration `XXX_remove_strategy_columns/up.sql`:
  - Recreate documents table without `strategy_id` column
  - Drop `idx_documents_strategy_id` and `idx_documents_lineage` indexes
  - Copy data from old table, dropping strategy rows and clearing strategy_id
- `down.sql`: reverse (add column back)
- Update `schema.rs` — remove `strategy_id` field from table definition
- Update `models.rs` — remove `strategy_id` from `Document` and `NewDocument`

**Tests — update or remove:**
- `crates/metis-docs-core/tests/reassignment_test.rs`
- `crates/metis-docs-core/tests/collision_resolution_test.rs`
- `crates/metis-docs-core/tests/database_reconstruction_test.rs`
- `crates/metis-docs-core/tests/id_path_consistency_test.rs`
- `crates/metis-docs-cli/tests/comprehensive_functional_test.rs`
- `crates/metis-docs-mcp/tests/functional_test.rs`
- `crates/metis-docs-mcp/tests/comprehensive_functional_test.rs`
- `crates/metis-docs-mcp/tests/configuration_scenarios_test.rs`
- Plus inline tests in configuration.rs, types.rs, factory.rs, strategy/mod.rs

**Plugin — update documentation:**
- MCP server instructions reference strategies throughout (phase tables, hierarchy descriptions, preset docs)
- Hook/skill content mentioning strategies

### Filesystem Migration

Current layout nests initiatives under `strategies/{strategy_id}/initiatives/`. New layout:

```
.metis/
├── initiatives/
│   ├── METIS-I-0001/
│   │   ├── initiative.md
│   │   └── tasks/
│   │       ├── METIS-T-0001/task.md
│   │       └── METIS-T-0002/task.md
│   └── METIS-I-0002/
│       └── ...
├── backlog/
│   └── ...
├── archived/
│   └── ...
└── vision.md
```

Migration path: on workspace open, if `strategies/` directory exists, move all contents of `strategies/*/initiatives/` up to `initiatives/`. Delete `strategies/` directory. Archive/delete any `strategy.md` files found. This is a one-time filesystem migration triggered by the new version.

### Database Migration

New diesel migration removes `strategy_id` column. Any rows with `level = 'strategy'` are deleted (or archived). The `strategy_id` value on initiatives is cleared. The lineage index is simplified to just `initiative_id`.

### Configuration Migration

`config.toml` files with `strategies_enabled = true/false` need handling. The field is simply removed from the config struct. Old config files with the field should parse without error (ignore unknown fields) or the parser should strip it on read. The "full" preset either becomes an alias for "streamlined" or is removed. Two presets remain: **streamlined** (Vision→Initiative→Task) and **direct** (Initiative→Task).

### Breaking Changes (2.0.0)

- `Strategy` type removed from public API
- `strategy_id` removed from all document structs
- `create strategy` CLI command removed
- `"strategy"` no longer valid in MCP `document_type` parameter
- `strategies_enabled` config key removed
- `"full"` preset removed or aliased to "streamlined"
- Filesystem layout changes (initiatives move out of `strategies/` nesting)
- `RiskLevel` enum removed
- `Phase::Shaping` removed
- `"S"` short code prefix temporarily unavailable (reserved for Specification)

## Alternatives Considered

**Keep Strategy but rename to Specification** — Rejected. Strategy's phases (shaping → design → ready → active → completed), template, and semantics are wrong for a Specification. Cleaner to remove and build fresh than to mutate in place.

**Deprecate but don't remove** — Rejected. Dead code creates confusion, especially for an AI-agent-driven tool where the LLM sees document types in MCP tool descriptions. A deprecated-but-present Strategy type would leak into every prompt.

## Implementation Plan

1. Explore the codebase to map every touchpoint (domain, CLI, MCP, templates, DB, config, tests)
2. Remove strategy from domain model and document factory
3. Remove strategy CLI command
4. Remove strategy from MCP tool parameters and instructions
5. Update configuration/presets to remove `strategies_enabled`
6. Refactor filesystem layout — eliminate `strategies/` nesting
7. Update database schema
8. Update and fix all tests
9. Clean up plugin documentation and hook content