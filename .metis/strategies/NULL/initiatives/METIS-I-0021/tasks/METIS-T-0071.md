---
id: add-metis-index-cli-command
level: task
title: "Add metis index CLI command"
short_code: "METIS-T-0071"
created_at: 2026-02-20T14:47:10.286707+00:00
updated_at: 2026-02-20T14:47:10.286707+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


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

*Updated during implementation*