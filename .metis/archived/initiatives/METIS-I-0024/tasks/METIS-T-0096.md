---
id: remove-strategy-cli-command
level: task
title: "Remove Strategy CLI command"
short_code: "METIS-T-0096"
created_at: 2026-03-03T19:10:51.419103+00:00
updated_at: 2026-03-04T01:09:32.785807+00:00
parent: METIS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0024
---

# Remove Strategy CLI command

## Parent Initiative

[[METIS-I-0024]]

## Objective

Remove the `metis create strategy` CLI subcommand and update initiative creation to remove strategy parent handling.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `crates/metis-docs-cli/src/commands/create/strategy.rs` deleted
- [x] `CreateCommands::Strategy` variant removed from `create/mod.rs`
- [x] Initiative creation in `create/initiative.rs` simplified — no strategy parent lookup, initiatives parent directly to vision
- [x] `metis create --help` no longer shows `strategy` subcommand
- [x] Any strategy references in `list.rs`, `status.rs`, or other CLI commands removed

## Implementation Notes

Depends on METIS-T-0091 (domain model) and METIS-T-0092 (services). Straightforward deletion once the underlying types and services are gone.

## Status Updates

### Session 1 (2026-03-03)
- Deleted `strategy.rs` CLI command file
- Rewrote `create/mod.rs`: removed `Strategy` variant, changed `Initiative` to take `--vision` parent instead of `--strategy`
- Rewrote `create/initiative.rs`: replaced `find_strategy()` with `find_vision()`, initiatives now created directly under visions via `create_initiative()` (no strategy_id param)
- Rewrote `create/task.rs`: removed `find_strategy_for_initiative()`, task creation uses flat layout `initiatives/{short_code}/initiative.md`
- Rewrote `config.rs`: removed `--strategies` flag from `ConfigAction::Set`, removed "full" preset, only offers streamlined/direct
- Updated `init.rs`: removed `--strategies` flag from `InitCommand`, simplified `determine_flight_config()`, removed "full" preset
- Updated `list.rs`: removed "strategy" from document type list and sort order
- Updated `status.rs`: removed "strategy" from `get_document_types()`, removed `strategy_id` from test mock
- Updated `transition.rs`: removed strategy transition test, rewrote initiative/task tests to use vision-parented hierarchy
- Updated `sync.rs`: removed strategy document test fixture
- Rewrote `cli.rs` integration test: full workflow now creates initiative→task under vision (no strategies)
- Removed `strategies: None` from all InitCommand test structs across all files (archive.rs, validate.rs, adr.rs, index.rs, config.rs)
- Zero strategy references remain in CLI crate (`grep -r strateg` returns 0 matches, excluding "strategic" in about text)
- Core tests pass (184 tests). CLI won't compile until T-0097 (MCP crate) is done (MCP dependency blocks `cargo check`)
- **Blocked**: CLI tests cannot run until MCP crate is updated in T-0097