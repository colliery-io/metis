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
1. Check alignment to vision/strategy/initiative
2. Choose appropriate document type based on scope
3. Set parent relationships correctly
4. Define clear acceptance criteria

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
