---
id: update-plugin-documentation
level: task
title: "Update plugin documentation"
short_code: "METIS-T-0099"
created_at: 2026-03-03T19:10:54.160116+00:00
updated_at: 2026-03-03T19:10:54.160116+00:00
parent: METIS-I-0024
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0024
---

# Update plugin documentation

## Parent Initiative

[[METIS-I-0024]]

## Objective

Remove all strategy references from the Claude Code plugin — MCP server instructions, hooks, skills, agents, and commands.

## Acceptance Criteria

- [ ] MCP server instructions (`plugin/` directory) updated — strategy removed from document type tables, phase tables, hierarchy descriptions, preset documentation, workflow examples
- [ ] Hook content referencing strategies updated (SessionStart hook context, etc.)
- [ ] Skill content referencing strategies updated (decomposition, phase-transitions, document-selection, project-patterns skills)
- [ ] Agent descriptions referencing strategies updated (flight-levels agent)
- [ ] No remaining references to "strategy", "strategies", "shaping" (as a phase), "full preset" in plugin content
- [ ] Plugin `plugin.json` updated if strategy is referenced in tool descriptions

## Implementation Notes

This is a text-sweep task. `grep -r "strateg" plugin/` will find everything. Also search for "shaping" (strategy phase), "full" (preset name), and "risk_level" (strategy-specific field in MCP descriptions).

## Status Updates

### Session 1 (2026-03-03)

**All strategy references removed from plugin documentation.**

Files updated (18 files total):

**Skills - Phase Transitions:**
- `skills/phase-transitions/SKILL.md` — Removed Strategy phase section, shaping starting phase, "strategies" from blocked note
- `skills/phase-transitions/references/phase-flow.md` — Removed Strategy phase flow section

**Skills - Document Selection:**
- `skills/document-selection/SKILL.md` — Removed strategy from description, decision tree, document types table, entire Strategy section (with risk_level), common mistakes
- `skills/document-selection/references/decision-trees.md` — Removed Strategy rows, "When to Create a Strategy" section, "Strategy vs Initiative" section

**Skills - Project Patterns:**
- `skills/project-patterns/SKILL.md` — Removed Full preset, updated guidance
- `skills/project-patterns/references/preset-selection.md` — Major rewrite: removed Full preset entirely, updated to two presets only
- `skills/project-patterns/references/core-principles.md` — Extensive edits: removed Strategy from hierarchy, alignment chain, phases, short codes, parent tables, presets table
- `skills/project-patterns/references/greenfield.md` — Removed strategy/Full preset references
- `skills/project-patterns/references/anti-patterns.md` — Simplified parent requirement
- `skills/project-patterns/references/feature-development.md` — Removed STRATEGY-ID comment

**Skills - Decomposition:**
- `skills/decomposition/SKILL.md` — Removed Strategy from decomposition chain
- `skills/decomposition/references/decomposition-patterns.md` — Removed Strategy from chain and "Strategies: Coherent Approaches" section

**Agents:**
- `agents/flight-levels.md` — Removed Strategy from hierarchy, table, alignment check

**Hooks:**
- `hooks/session-start-hook.sh` — Updated human-in-the-loop heading and text
- `hooks/pre-compact-hook.sh` — Removed "/strategies" from guidance

**Other:**
- `README.md` — Removed "strategy" from document type list

**Intentionally kept** (generic English usage, not Metis document type):
- "strategic work/direction" in hooks and feature-development
- "Strategy impl" (design pattern) in code-index-summarizer
- "Strategies:" (meaning approaches) in tech-debt
- "the strategy implicitly" in preset-selection