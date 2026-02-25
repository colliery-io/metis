---
id: vendor-muninn-graph-parsing
level: task
title: "Vendor muninn-graph parsing infrastructure into new metis-code-index crate"
short_code: "METIS-T-0066"
created_at: 2026-02-20T14:47:04.951006+00:00
updated_at: 2026-02-24T22:12:10.770531+00:00
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

# Vendor muninn-graph parsing infrastructure into new metis-code-index crate

## Parent Initiative
[[METIS-I-0021]]

## Objective

Create a new `metis-code-index` crate in the workspace by vendoring the parsing infrastructure from `colliery-io/muninn/crates/muninn-graph`. This establishes the foundation for all code indexing work.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] New `crates/metis-code-index/` crate exists in the workspace
- [ ] Vendored modules: `parser.rs`, `symbols.rs`, `lang/rust.rs`, `lang/python.rs`, `lang/mod.rs`
- [ ] Vendored query files: `queries/rust_symbols.scm`, `queries/python_symbols.scm`
- [ ] Dropped modules: `store.rs`, `graph.rs`, `edges.rs`, `embeddings.rs`, `watcher.rs`
- [ ] Dependencies updated: `tree-sitter`, `tree-sitter-rust`, `tree-sitter-python`, `ignore`, `serde`, `thiserror`
- [ ] No `graphqlite` dependency
- [ ] Existing Rust and Python extractors compile and pass tests
- [ ] `angreal check` passes
- [ ] `angreal test` passes

## Implementation Notes

Source: `https://github.com/colliery-io/muninn/tree/main/crates/muninn-graph/`

Key changes from muninn:
- Remove all graphqlite/storage references
- Remove edges module (not needed for flat file output)
- Remove watcher module (incremental updates are a separate task)
- Keep `parser.rs` lazy language init pattern
- Keep `symbols.rs` types verbatim
- Keep Rust + Python extractors and their inline tree-sitter queries
- Add crate to workspace `Cargo.toml`

## Progress

### Session 1 (2026-02-20)
- Fetched all source files from `colliery-io/muninn/crates/muninn-graph/` via GitHub API
- Verified `lang/rust.rs` and `lang/python.rs` have no references to dropped modules (store, edges, watcher, builder)
- Created `crates/metis-code-index/` with full directory structure
- Wrote `Cargo.toml` with tree-sitter deps (Rust, Python), streaming-iterator, serde, thiserror. No graphqlite, notify, or ignore deps.
- Vendored and modified `parser.rs`: removed C/Cpp language variants (out of scope), kept Rust and Python only. Added `Language::all()` helper.
- Vendored `symbols.rs` as-is (clean, no storage refs)
- Vendored `lang/mod.rs`, `lang/rust.rs`, `lang/python.rs` as-is
- Vendored query files: `rust_symbols.scm`, `python_symbols.scm`
- Wrote new `lib.rs` exposing only parser, symbols, and lang modules
- Added crate to workspace `Cargo.toml`
- Fixed clippy `if_same_then_else` warning in Python visibility logic
- Applied rustfmt formatting
- All checks pass: `cargo clippy -p metis-code-index -- -D warnings`, `cargo fmt --check`, 35/35 tests pass
- `angreal test` passes for full workspace