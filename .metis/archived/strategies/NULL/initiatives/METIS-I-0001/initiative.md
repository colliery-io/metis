---
id: agent-skills
level: initiative
title: "Agent Skills"
short_code: "METIS-I-0001"
created_at: 2025-12-06T16:14:12.493463+00:00
updated_at: 2025-12-07T01:53:49.821708+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: agent-skills
---

# Metis Flight Levels Skill Initiative

## Context

Metis provides a robust MCP server with 11 tools for managing Flight Levels project documentation - creating documents, transitioning phases, validating exit criteria, etc. However, the tooling alone doesn't encode the *methodology* for effective work decomposition. Users connecting Claude (or other agents) to the Metis MCP get a capable operator but not an opinionated guide.

The broader agent ecosystem is fragmented around skill definition. Claude Code uses `SKILL.md`, Codex uses `AGENTS.md`, Gemini uses `gemini-extension.json`. HuggingFace recently validated a pattern where a single repository contains canonical methodology content with platform-specific adapter files at the root. This approach avoids lock-in to any single agent platform while maintaining a single source of truth.

A Metis skill would encode decision-making expertise: when to use Full vs Streamlined vs Direct presets, how to decompose work effectively, what constitutes good exit criteria, and patterns for common project types. Unlike HuggingFace's skills (which bundle scripts for submitting training jobs), the Metis skill is pure methodology - the capability layer already exists in the MCP server.

## Goals & Non-Goals

**Goals:**

- Create a canonical skill definition encoding Flight Levels methodology for Metis
- Support multiple agent platforms (Claude Code, Codex, Gemini CLI) via adapter files
- Improve agent decision-making when using Metis tools (measurable through qualitative testing)
- Establish a reusable pattern for methodology-only skills that depend on existing MCP servers
- Document the skill structure as a template others could follow

**Non-Goals:**

- Building a skill registry or discovery mechanism (git-native distribution is sufficient for v1)
- Adding executable scripts (Metis MCP tools are the capability layer)
- Supporting skill composition or inheritance
- Versioning/pinning infrastructure beyond git tags
- Modifying the Metis MCP server itself

## Detailed Design

### Repository Structure

```
skill/
├── SKILL.md                    # Claude Code assembler (references modules)
├── AGENTS.md                   # Codex assembler
├── gemini-extension.json       # Gemini CLI adapter
├── methodology/
│   ├── core-principles.md      # Flight Levels fundamentals, value flow
│   ├── preset-selection.md     # Full vs Streamlined vs Direct decision framework
│   ├── decomposition.md        # How to break work down effectively
│   ├── phase-transitions.md    # When/why to transition, exit criteria patterns
│   └── anti-patterns.md        # Common mistakes and corrections
├── patterns/
│   ├── greenfield.md           # New project pattern
│   ├── tech-debt.md            # Technical debt campaign pattern
│   ├── incident-response.md    # Urgent/reactive work pattern
│   └── feature-development.md  # Standard feature work pattern
└── decision-trees/
    ├── document-type.md        # Vision vs Strategy vs Initiative vs Task
    └── when-to-adr.md          # Architectural decision triggers
```

Note: The skill references `crates/metis-docs-mcp/instructions.md` for operational details (tool parameters, workflows). The skill content focuses on methodology and judgment.

### SKILL.md Structure

```yaml
---
name: metis-flight-levels
description: Opinionated methodology for decomposing work using Flight Levels
requires:
  mcp_servers:
    - metis
---
```

Followed by assembled methodology content organized as:

1. **Core Principles** - The "why" of Flight Levels, value flow direction
2. **Quick Reference** - Decision heuristics for common situations
3. **Detailed Guidance** - References to methodology files for deep dives
4. **Patterns** - Project-type-specific workflows
5. **Warnings** - Anti-patterns and how to recover

### Content Philosophy

The skill encodes *judgment*, not *procedure*. Rather than "call create_document with type=task", the skill teaches:

- "Tasks should be completable by one person in less than a week"
- "If you can't define exit criteria, the work needs more discovery"
- "Blocked items indicate missing decomposition or dependency issues"

The agent already knows *how* to call tools. The skill teaches *when* and *why*.

### Platform Adapters

**AGENTS.md** (Codex): Will contain the same core content as SKILL.md, formatted per Codex conventions.

**gemini-extension.json**: Minimal manifest pointing to SKILL.md content:

```json
{
  "name": "metis-flight-levels",
  "description": "Flight Levels methodology for Metis project management",
  "instructions": "./SKILL.md"
}
```

### Separate Repo vs Subfolder

Decision: **Subfolder** (`skill/` within main Metis repo)

Rationale:

- Simpler versioning - skill evolves with the tooling
- Single repo to maintain
- Users already have the repo if they have Metis
- Skill references MCP instructions.md - co-location makes this natural

The skill folder contains modular methodology content with platform-specific assemblers (SKILL.md, AGENTS.md, etc.) that compile the modules into each platform's expected format.

## Alternatives Considered

### MCP Resource Approach

Serve the skill as an MCP resource (`metis://skill/methodology`) from the existing server.

**Rejected because:** Ties distribution to a single protocol. Agents without MCP support couldn't access the skill. The methodology should be protocol-agnostic.

### Generated Platform Adapters

Build a compiler that generates AGENTS.md and gemini-extension.json from canonical source.

**Rejected because:** Premature optimization. The adapters are simple enough to maintain by hand. A build step adds complexity without clear benefit at this scale.

### Inline Everything in SKILL.md

Put all methodology in a single file rather than modular markdown files.

**Rejected because:** Harder to maintain, harder for humans to navigate. Modular files allow focused editing and review. SKILL.md can inline critical content and reference deeper material.

### Scripts for Workflow Automation

Include helper scripts like HuggingFace does.

**Rejected because:** Metis MCP already provides the capability layer. Scripts would duplicate functionality and create maintenance burden. Pure methodology is the cleaner architecture.

## Implementation Plan

### Phase 1: Content Development (Primary Work)

- Draft core-principles.md establishing Flight Levels fundamentals
- Draft preset-selection.md with decision framework
- Draft decomposition.md with breakdown heuristics
- Draft phase-transitions.md with exit criteria patterns
- Draft anti-patterns.md with common mistakes

### Phase 2: Pattern Library

- Draft greenfield.md pattern
- Draft tech-debt.md pattern
- Draft incident-response.md pattern
- Draft decision-trees for document type selection and ADR triggers

### Phase 3: Assembly & Adapters

- Assemble SKILL.md from modular content
- Create AGENTS.md for Codex
- Create gemini-extension.json for Gemini CLI
- Write README.md with installation instructions per platform

### Phase 4: Validation

- Test with Claude Code against real project scenarios
- Test with Codex (if available)
- Gather qualitative feedback on decision quality
- Iterate on content based on findings

### Phase 5: Release

- Create repository `colliery-io/metis-skill`
- Tag v1.0.0
- Update main Metis README to reference the skill
- Announce availability

## Testing Strategy

### Qualitative Scenario Testing

Define 5-7 realistic scenarios and evaluate agent behavior with/without the skill loaded:

1. **New project setup** - Does the agent ask good questions about preset selection?
2. **Work breakdown** - Given a vague goal, does the agent create appropriate document hierarchy?
3. **Phase transition** - Does the agent check exit criteria before recommending transitions?
4. **Scope creep** - When new requirements emerge, does the agent recommend new Initiative vs expanding existing?
5. **Blocked work** - Does the agent diagnose blocking issues and suggest resolution paths?
6. **ADR triggers** - Does the agent recognize when architectural decisions warrant documentation?
7. **Anti-pattern recognition** - Does the agent warn about too many active items or orphaned tasks?

### Success Criteria

- Agent asks clarifying questions aligned with methodology before acting
- Agent recommends appropriate document types for given work
- Agent references exit criteria when discussing phase transitions
- Agent warns about anti-patterns when they occur
- Qualitative assessment: "This feels like working with someone who understands Flight Levels"

### Iteration Process

After initial testing, identify gaps where agent behavior diverges from desired methodology. Update skill content to address gaps. Re-test. Repeat until satisfied with decision quality.