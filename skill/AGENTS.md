# Metis Flight Levels Agent

This agent configuration teaches Flight Levels methodology for work management using Metis.

## Overview

Metis implements Flight Levels - a Kanban-based work management system with hierarchical organization:

```
Vision (FL 3)     - North star objectives and values
Strategy (FL 2)   - Coordinated approaches to achieve vision
Initiative (FL 1) - Capability increments (projects)
Task (FL 0)       - Atomic units of work
```

## Agent Capabilities

When using Metis tools, this agent understands:

### Methodology
- Pull-based flow (work is pulled, never pushed)
- Value alignment (all work traces back to vision)
- Phase-gated progress with exit criteria
- Scope-based sizing (capability increments, not time estimates)

### Document Selection
- **Vision**: North star, objectives, values - rarely changes
- **Strategy**: Coordinated approaches - when multi-team coordination needed
- **Initiative**: Fundamental capability increment - the main unit of project work
- **Task**: Atomic work unit - belongs to an initiative
- **Backlog**: Ad-hoc bugs/features/tech-debt - standalone or feeds into initiatives
- **ADR**: Architectural decisions - when "why did we do it this way?" matters

### Terminology Mapping

When users request tickets using common terms, create backlog items:

| User Says | Create |
|-----------|--------|
| "bug ticket", "bug", "defect" | `create_document(type="task", backlog_category="bug", ...)` |
| "feature ticket", "feature request" | `create_document(type="task", backlog_category="feature", ...)` |
| "tech debt ticket", "tech debt", "debt" | `create_document(type="task", backlog_category="tech-debt", ...)` |

These are standalone tasks not tied to an initiative. They live in the backlog until prioritized.

### Patterns
- Greenfield projects: Vision first, then initiatives
- Tech debt campaigns: Backlog items → grouped initiatives
- Incident response: Backlog items for tracking, initiatives for systemic fixes
- Feature development: Initiative with discovery → design → decompose → execute

### Anti-Patterns to Avoid
- Shadow work (untracked effort)
- Shadow backlogs (secret lists outside the system)
- Too many active items (WIP overload)
- Skipping phases (leads to rework)
- Premature decomposition (tasks before design is clear)

## Key Behaviors

### When Creating Work

**CRITICAL: Documents must be completed after creation, not left as templates.**

The workflow for every document creation:
1. `create_document` - Creates document from template
2. `read_document` - Read the created document to see template structure
3. `edit_document` - Fill in ALL required sections with actual content
4. `edit_document` - Delete any optional sections that don't apply (remove placeholders)

**Never leave a document with template placeholders like `{Describe the context}` or `{Primary objective 1}`.** Every document should have real content before moving on.

Additional creation guidelines:
- Check alignment to vision/strategy/initiative
- Choose appropriate document type based on scope
- Set parent relationships correctly
- Define clear acceptance criteria

### When Transitioning Phases
1. Verify exit criteria are met
2. Use auto-advance (no phase parameter) for normal flow
3. Specify phase only for non-linear transitions (e.g., blocked)
4. Don't force unless consciously accepting the risk

### When Backlog is Low
1. Look UP to the next level
2. Pull work down through decomposition
3. Don't start new initiatives without capacity

## Reference Documentation

Detailed methodology is available in:
- `skill/methodology/` - Core principles, decomposition, phases, anti-patterns
- `skill/patterns/` - Project-type-specific guidance
- `skill/decision-trees/` - Document type and ADR decision guides

The Metis MCP server description provides tool parameters and operational details.
