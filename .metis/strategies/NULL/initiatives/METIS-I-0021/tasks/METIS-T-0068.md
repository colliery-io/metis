---
id: add-go-tree-sitter-extractor
level: task
title: "Add Go tree-sitter extractor"
short_code: "METIS-T-0068"
created_at: 2026-02-20T14:47:06.846831+00:00
updated_at: 2026-02-25T01:22:43.619375+00:00
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

# Add Go tree-sitter extractor

## Parent Initiative
[[METIS-I-0021]]

## Objective

Add Go language support to the code index crate. Create a Go extractor using `tree-sitter-go` following the same pattern as existing extractors.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `tree-sitter-go` dependency added
- [ ] `Language` enum extended with `Go` variant
- [ ] File extensions: `.go`
- [ ] `GoExtractor` implemented in `lang/go.rs`
- [ ] Extracts: function signatures, struct definitions, interface definitions, type aliases, method receivers, imports
- [ ] Tree-sitter query file: `queries/go_symbols.scm`
- [ ] Tests for Go symbol extraction
- [ ] `angreal test` passes

## Implementation Notes

Follow the pattern from `lang/rust.rs`:
- Inline compiled queries via `OnceLock`
- `extract_symbols()`, `extract_imports()` methods
- Handle Go-specific patterns: method receivers (`func (s *Server) Start()`), embedded interfaces, multiple return values

Blocked by: METIS-T-0066 (crate must exist first)

## Progress

### Implementation Complete (2026-02-24)

**All acceptance criteria met:**

- [x] `tree-sitter-go` v0.25.0 dependency added to Cargo.toml
- [x] `Language` enum extended with `Go` variant
- [x] File extensions: `.go` mapped to `Language::Go`
- [x] `GoExtractor` implemented in `lang/go.rs`
- [x] Extracts: functions, methods with receivers, structs, interfaces, type definitions, constants, variables, imports
- [x] Tree-sitter query file: `queries/go_symbols.scm`
- [x] 12 tests for Go symbol extraction (all passing)
- [x] `angreal test` passes (full workspace: 62 code-index tests, all workspace tests green)

**Go-specific features implemented:**
- Visibility based on Go naming convention (capitalized = public, lowercase = private)
- Method receiver extraction in signatures (e.g., `func (s *Server) Start()`)
- Function/method signatures with params and return types
- Go doc comment extraction (`//` lines preceding declarations)
- Deduplication: structs/interfaces prioritized over generic type definitions
- Import extraction with alias support (named imports, `_`, `.`)

**Files modified:**
- `crates/metis-code-index/Cargo.toml` - added tree-sitter-go dep
- `crates/metis-code-index/src/parser.rs` - Go language variant + parser tests
- `crates/metis-code-index/src/queries/go_symbols.scm` - NEW
- `crates/metis-code-index/src/lang/go.rs` - NEW (~350 lines + tests)
- `crates/metis-code-index/src/lang/mod.rs` - added go module
- `crates/metis-code-index/src/lib.rs` - re-export GoExtractor

**Quality:** clippy clean, fmt clean, all 62 code-index tests pass