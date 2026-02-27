---
id: implement-markdown-output
level: task
title: "Implement markdown output formatter for code-index.md"
short_code: "METIS-T-0070"
created_at: 2026-02-20T14:47:09.169842+00:00
updated_at: 2026-02-25T02:10:14.426363+00:00
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

# Implement markdown output formatter for code-index.md

## Parent Initiative
[[METIS-I-0021]]

## Objective

Write the formatter that takes the parsed symbols and file tree from the walker/extractors and outputs a flat markdown file at `.metis/code-index.md` with three sections: Project Structure (tree), Module Summaries (placeholder for Layer 2), and Key Symbols (signatures grouped by file).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Generates markdown file at `.metis/code-index.md`
- [ ] Header with generation timestamp, file count, detected languages
- [ ] `## Project Structure` section with ASCII file tree (directories and source files)
- [ ] `## Module Summaries` section with placeholder text (populated by Layer 2 skill)
- [ ] `## Key Symbols` section with symbols grouped by file path, showing signatures
- [ ] Public symbols prioritized over private
- [ ] Output is readable and navigable by an AI agent
- [ ] Tests with fixture data producing expected markdown
- [ ] `angreal test` passes

## Implementation Notes

Takes two inputs: file tree from walker (METIS-T-0069) and extracted symbols from parsers. Formats into markdown. Use a simple tree-drawing algorithm for the project structure section (`├──`, `└──`, `│`). Group symbols by file path, show `pub` items first with their full signatures.

Blocked by: METIS-T-0066, METIS-T-0069

## Progress

### Session 1 (2026-02-24)
- Created `crates/metis-code-index/src/formatter.rs` with full markdown output formatter
- `format_index()` produces complete markdown with three sections:
  - `## Project Structure` — ASCII tree with box-drawing characters (├──, └──, │)
  - `## Module Summaries` — placeholder for AI agent skill (Layer 2)
  - `## Key Symbols` — symbols grouped by file, public symbols sorted first
- `write_index_file()` convenience function writes formatted output to disk
- Header includes generation timestamp, file count, and detected languages
- Tree rendering via `build_tree()` → `insert_path()` → `render_tree()`
- Updated `lib.rs` with formatter module and re-exports
- 10 new formatter tests covering all sections, edge cases, and multi-language projects
- 82 total tests pass, clippy clean, fmt clean
- All acceptance criteria met