---
id: create-code-index-plugin-skill-for
level: task
title: "Create code-index plugin skill for AI-generated module summaries"
short_code: "METIS-T-0074"
created_at: 2026-02-20T14:47:13.836008+00:00
updated_at: 2026-02-25T05:27:09.456045+00:00
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

# Create code-index plugin skill for AI-generated module summaries

## Parent Initiative
[[METIS-I-0021]]

## Objective

Create a plugin skill that guides Claude through generating Layer 2 module summaries. The skill teaches Claude how to read the structural index + source code, write per-directory summaries (purpose, key files, dependencies), and append them to `.metis/code-index.md`. Default model: Sonnet.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Skill file at `plugins/metis/skills/code-index.md`
- [ ] Triggered by: "create a code index", "index this codebase", "update the code index", "generate code index"
- [ ] Guides Claude to: run `metis index --structure-only` first via MCP tool, then read the structural index, read source files per directory, write module summaries into the `## Module Summaries` section
- [ ] Summaries follow the format: Purpose, Key files, Dependencies per directory
- [ ] Skill specifies Sonnet as the default model for summary generation
- [ ] Instructions for when to regenerate (new project, after major refactors, significant new modules)

## Implementation Notes

Create `plugins/metis/skills/code-index.md` following the existing skill pattern (see `decomposition.md`, `phase-transitions.md`). The skill should be a progressive disclosure document -- start with the trigger and high-level flow, then detail the summary writing process.

Key guidance for Claude:
- Run structural index first (Layer 1) via the MCP tool
- Read `.metis/code-index.md` to see the file tree and symbols
- For each top-level source directory, read a few representative files
- Write a concise summary: what the module does, key files, what it depends on
- Edit the code-index.md to replace the placeholder Module Summaries section

No blockers -- can be done in parallel with Rust crate work. But the skill is most useful after the MCP tool exists (METIS-T-0072).

## Progress

### Session 1 (2026-02-24)
- Created `plugins/metis/skills/code-index/SKILL.md` following existing skill pattern
- Skill triggers on: "create a code index", "index this codebase", "update the code index", "generate code index", "build code index", "refresh module summaries"
- Guides Claude through two-layer process:
  - Layer 1: Run `index_code` MCP tool for structural index (file tree + symbols)
  - Layer 2: Read source files, write per-directory module summaries
- Summary format: Purpose (1 sentence), Key files (3-5), Dependencies
- Includes complete example with real module summaries
- Specifies Sonnet as default model for summary generation
- Guidelines for when to regenerate (new project, major refactors, new modules)
- Deployed to marketplace and cache directories
- All acceptance criteria met