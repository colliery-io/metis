---
id: update-plugin-documentation-for
level: task
title: "Update plugin documentation for Specification type"
short_code: "METIS-T-0106"
created_at: 2026-03-04T03:06:37.946662+00:00
updated_at: 2026-03-04T13:15:00.619710+00:00
parent: METIS-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0025
---

# Update plugin documentation for Specification type

## Parent Initiative

[[METIS-I-0025]]

## Objective

Add Specification document type to all plugin documentation — MCP server instructions, hooks, skills, agents, and commands.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MCP server instructions updated — Specification added to document type tables, phase tables, hierarchy descriptions, workflow examples
- [ ] Hook content updated to reference Specification type where appropriate (SessionStart hook context, etc.)
- [ ] Skill content updated — document-selection, phase-transitions, decomposition, project-patterns skills
- [ ] Agent descriptions updated — flight-levels agent
- [ ] Plugin `plugin.json` updated if Specification needs to appear in tool descriptions
- [ ] README.md updated with Specification in document type list

## Implementation Notes

Text-sweep task similar to METIS-T-0099. `grep -r "specification\|Specification" plugin/` to verify coverage. Add Specification alongside ADR references since both are attached document types.

Key additions needed:
- Document type table: Specification row with phases (discovery → drafting → review → published)
- Hierarchy description: show Specification as attached to Vision or Initiative
- Phase flow: Specification phase sequence
- Document selection: when to create a Specification vs other types
- Workflow examples: creating a Specification under a Vision

## Status Updates

### Session 1 — Completed
Updated 13 files across plugins, docs, and README:

**Plugin files updated (11 files, 31 specification references):**
- `agents/flight-levels.md` — Added Specification to document selection table, parent guidance, terminology mapping
- `skills/document-selection/SKILL.md` — Updated description, decision guide, type table, terminology mapping, "When to Create" section, edge cases
- `skills/document-selection/references/decision-trees.md` — Added Specification to quick reference, decision tree, detailed guidance, edge cases
- `skills/phase-transitions/SKILL.md` — Added Specification phase sequence and default phase
- `skills/phase-transitions/references/phase-flow.md` — Added Specification phase sequence with descriptions
- `skills/project-patterns/SKILL.md` — Added specification mentions in greenfield tips and feature design
- `skills/project-patterns/references/greenfield.md` — Added "Specifications for Key Systems" section
- `skills/project-patterns/references/preset-selection.md` — Note that specs are always available in all presets
- `skills/code-index/SKILL.md` — Added specifications to parenthetical list
- `hooks/session-start-hook.sh` — Updated create_document tool description
- `README.md` — Updated document-selection skill description

**Other files updated (2 files):**
- `README.md` (project root) — Added Specification to phase table and MCP tool description
- `docs/claude-code-plugin.md` — Updated document-selection skill description