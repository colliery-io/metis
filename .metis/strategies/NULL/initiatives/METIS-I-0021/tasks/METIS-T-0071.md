---
id: add-metis-index-cli-command
level: task
title: "Add metis index CLI command"
short_code: "METIS-T-0071"
created_at: 2026-02-20T14:47:10.286707+00:00
updated_at: 2026-02-25T05:02:58.757831+00:00
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

# Add metis index CLI command

## Parent Initiative
[[METIS-I-0021]]

## Objective

Add `metis index` subcommand to `metis-docs-cli` that orchestrates the full indexing pipeline: walk files, parse with tree-sitter, extract symbols, write `.metis/code-index.md`. Support `--structure-only` and `--incremental` flags.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `metis index` subcommand added to CLI
- [ ] `--structure-only` flag skips Layer 2 summary placeholder
- [ ] `--incremental` flag supported (no-op until METIS-T-0075 implements hash tracking)
- [ ] Orchestrates: walk files → parse → extract symbols → write markdown
- [ ] Validates `.metis/` directory exists before indexing
- [ ] Reports stats on completion (files indexed, languages detected, time taken)
- [ ] `angreal test` passes

## Implementation Notes

Wire up the metis-code-index crate as a dependency of metis-docs-cli. Add an `index` subcommand using clap. The command should:
1. Resolve project root (find `.metis/` directory)
2. Call the walker to get file list
3. Parse each file, extract symbols
4. Call the markdown formatter to write `.metis/code-index.md`

Blocked by: METIS-T-0066, METIS-T-0069, METIS-T-0070

## Progress

### Session 1 (2026-02-24)
- Added `metis-code-index` as dependency of `metis-docs-cli` in Cargo.toml
- Created `crates/metis-docs-cli/src/commands/index.rs` with `IndexCommand`
  - `--structure-only` flag skips symbol extraction, generates tree only
  - `--incremental` flag accepted (no-op, prints note about future implementation)
  - Validates `.metis/` workspace exists before indexing
  - Full pipeline: walk files → parse with tree-sitter → extract symbols → write markdown
  - Reports stats: file count, languages detected, symbol count, parse errors, elapsed time
  - Dispatches to correct extractor per language (Rust, Python, TypeScript, JavaScript, Go)
- Registered command in `commands/mod.rs` and `cli.rs` (Commands enum + execute match)
- Fixed compilation issues: Result return types, TypeScript extractor takes Language param, used ParsedFile to avoid direct tree_sitter::Tree reference
- 4 new tests: no_workspace error, generates_file (full pipeline), structure_only, incremental_flag_accepted
- All tests pass via `angreal test`, formatting clean
- All acceptance criteria met