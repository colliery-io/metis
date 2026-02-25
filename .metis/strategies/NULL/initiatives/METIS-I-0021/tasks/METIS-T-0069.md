---
id: implement-gitignore-aware-file
level: task
title: "Implement gitignore-aware file tree walker using ignore crate"
short_code: "METIS-T-0069"
created_at: 2026-02-20T14:47:08.759930+00:00
updated_at: 2026-02-20T14:47:08.759930+00:00
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

# Implement gitignore-aware file tree walker using ignore crate

## Parent Initiative
[[METIS-I-0021]]

## Objective

Replace the naive `std::fs::read_dir` directory walker from muninn with an `ignore::WalkBuilder`-based walker that respects `.gitignore`. Output a filtered file tree of source files and feed parsed files into the extractors.

## Acceptance Criteria

- [ ] `ignore::WalkBuilder` used for directory traversal
- [ ] Respects `.gitignore` (and `.git/info/exclude`, global gitignore)
- [ ] Filters to supported source file extensions (`.rs`, `.py`, `.ts`, `.tsx`, `.js`, `.jsx`, `.mjs`, `.cjs`, `.go`)
- [ ] Skips hidden directories, `target/`, `node_modules/`, `__pycache__/`
- [ ] Outputs a tree structure suitable for the markdown formatter
- [ ] Tests with a fixture directory containing gitignored files
- [ ] `angreal test` passes

## Implementation Notes

The `ignore` crate (v0.4) is already a dependency in muninn-graph. Use `WalkBuilder::new(root).git_ignore(true).hidden(true).build()` and filter entries by extension. Output a `Vec<PathBuf>` of source files plus a tree structure for display.

Blocked by: METIS-T-0066 (crate must exist first)

## Progress

*Updated during implementation*