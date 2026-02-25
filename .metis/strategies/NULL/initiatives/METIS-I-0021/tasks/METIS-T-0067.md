---
id: add-typescript-javascript-tree
level: task
title: "Add TypeScript/JavaScript tree-sitter extractor"
short_code: "METIS-T-0067"
created_at: 2026-02-20T14:47:06.004700+00:00
updated_at: 2026-02-24T22:12:11.383032+00:00
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

# Add TypeScript/JavaScript tree-sitter extractor

## Parent Initiative
[[METIS-I-0021]]

## Objective

Add TypeScript and JavaScript language support to the code index crate. Following the same pattern as the vendored Rust and Python extractors, create a TS/JS extractor using `tree-sitter-typescript` and `tree-sitter-javascript`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tree-sitter-typescript` and `tree-sitter-javascript` dependencies added
- [ ] `Language` enum extended with `TypeScript` and `JavaScript` variants
- [ ] File extensions: `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`
- [ ] `TypeScriptExtractor` implemented in `lang/typescript.rs`
- [ ] Extracts: function/method signatures, class/interface definitions, type aliases, enums, imports
- [ ] Tree-sitter query file: `queries/typescript_symbols.scm`
- [ ] Tests for TS/JS symbol extraction
- [ ] `angreal test` passes

## Implementation Notes

Follow the pattern from `lang/rust.rs` and `lang/python.rs`:
- Inline compiled queries via `OnceLock`
- `extract_symbols()`, `extract_imports()` methods
- Handle both TS and JS with the same extractor (TypeScript grammar parses JS too)
- TSX/JSX: extract component function signatures but skip JSX template content

Blocked by: METIS-T-0066 (crate must exist first)

## Progress

### Session 1 (2026-02-21)
- Added `tree-sitter-typescript` (0.23) and `tree-sitter-javascript` (0.25) dependencies
- Extended `Language` enum with `TypeScript` and `JavaScript` variants
- TypeScript uses `LANGUAGE_TSX` grammar (superset, handles both .ts and .tsx)
- JavaScript uses `tree_sitter_javascript::LANGUAGE`
- File extensions mapped: `.ts`, `.tsx` -> TypeScript; `.js`, `.jsx`, `.mjs`, `.cjs` -> JavaScript
- Created `lang/typescript.rs` with `TypeScriptExtractor`:
  - Separate compiled query sets for TS and JS (different grammars, different node types)
  - `extract_symbols()` handles both TS and JS via language parameter
  - `extract_imports()` handles both TS and JS
  - TS extracts: functions, classes, interfaces, type aliases, enums, methods, arrow functions
  - JS extracts: functions, classes, methods, arrow functions
  - Export statements detected for visibility (Public vs Private)
  - Arrow functions assigned to variables detected as Function symbols
  - JSDoc comment extraction for classes
  - Deduplication logic for exported declarations that match both patterns
- Created query files: `typescript_symbols.scm`, `javascript_symbols.scm`
- Updated `lang/mod.rs` and `lib.rs` to export `TypeScriptExtractor`
- Updated parser tests for new extensions and parsing
- 15 new tests: TS function, class, interface, type alias, enum, arrow fn, imports, TSX component; JS function, class, arrow fn, imports, mixed
- Fixed clippy issues: single-match and never-loop warnings
- All checks pass: clippy (-D warnings), fmt, 50/50 tests pass
- `angreal test` passes for full workspace