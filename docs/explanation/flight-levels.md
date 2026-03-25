# Flight Levels Methodology

Metis organizes work using a hierarchical model inspired by the Flight Levels framework. This document explains why the hierarchy exists, how the layers relate, and when to use each document type.

## The Problem

Software projects generate many kinds of work artifacts: strategic goals, project plans, individual tasks, architecture decisions, specifications. Without structure, these blur together — a task description drifts into strategic justification, a spec becomes a TODO list, and nobody knows which document is the source of truth for what.

Metis addresses this by giving each kind of artifact a distinct document type with a defined lifecycle, clear parent-child relationships, and explicit phase transitions.

## The Hierarchy

```
Vision          — Where are we going? (6mo-2yr)
  └── Initiative  — How will we get there? (1-6mo)
        └── Task    — What do we do today? (1-14 days)
```

Each layer answers a different question at a different time horizon. Work flows down through decomposition; feedback flows up through status.

### Vision

The vision is the top of the hierarchy. Every project has exactly one. It defines strategic direction, success criteria, principles, and constraints. Think of it as the document you'd give a new team member to explain *why* the project exists and where it's headed.

Visions change slowly. Once published, they're rarely modified. If your vision needs frequent updates, it's probably too tactical — push those details down to initiatives.

### Initiative

An initiative is a concrete project aligned with the vision. It has a defined scope, timeline, and complexity estimate. Initiatives go through a longer lifecycle (discovery → design → ready → decompose → active → completed) because they require planning before execution.

The `decompose` phase is key: this is where you break the initiative into tasks. Decomposition happens after design is complete, ensuring tasks are well-defined before work begins.

### Task

A task is an actionable piece of work that one person can complete in 1-14 days. Tasks have the simplest lifecycle (todo → active → completed) because they're about *doing*, not *planning*.

Tasks belong to an initiative (in streamlined mode) or directly to the vision (in direct mode). They can be blocked by other tasks, and they support bidirectional transitions to and from the `blocked` state.

### Supporting Documents

Two document types sit alongside the hierarchy rather than within it:

**ADRs (Architecture Decision Records)** capture decisions and their rationale. They're standalone documents that don't require a parent. An ADR might be triggered by work on an initiative, but the decision itself is independent — it outlives the initiative that prompted it.

**Specifications** capture system-level design. They attach to a vision or initiative as "living documents" that remain editable even after publishing. Specifications don't have children — they're reference material, not work items.

## Why Not Just Use Tasks?

A flat list of tasks doesn't capture:

- **Strategic context:** Why are we doing this? What's the bigger picture?
- **Planning phases:** Is this initiative ready for execution, or still being designed?
- **Decomposition timing:** Should we break this down now, or after the design is complete?
- **Decision tracking:** What architectural choices did we make, and why?

The hierarchy separates *what we're trying to achieve* (vision), *how we'll achieve it* (initiative), and *what we're doing right now* (task). Each layer has its own lifecycle because the work at each layer moves at different speeds.

## Presets: Streamlined vs. Direct

Metis offers two presets to match different project needs:

### Streamlined (Default)

```
Vision → Initiative → Task
```

Use streamlined when:
- Multiple people are working on the project
- Work spans more than 2 weeks
- You need to coordinate multiple workstreams
- Strategic planning matters (design reviews, decomposition)

### Direct

```
Vision → Task
```

Use direct when:
- You're working solo
- The project is small or short-lived
- You don't need initiative-level planning
- You want minimal overhead

Switching presets doesn't delete existing documents — it only changes what's available for new documents.

## Phase Transitions as Workflow Gates

Phases aren't just status labels — they're workflow gates. Each phase represents a meaningful state in the work lifecycle, and transitions between phases are intentional acts.

For initiatives, the phase sequence enforces a planning discipline:

1. **Discovery** — Understand the problem before solving it
2. **Design** — Plan the approach before committing to it
3. **Ready** — Confirm the design is reviewed before decomposing
4. **Decompose** — Break down into tasks before starting execution
5. **Active** — Execute the planned tasks
6. **Completed** — Deliver the results

You can't skip from discovery to active. This isn't bureaucracy — it's a forcing function that ensures you've thought through the work before doing it.

For tasks, the phases are simpler because tasks are meant to be *done*, not planned extensively. The `blocked` state provides an escape valve for dependencies without disrupting the forward-only flow.

## Backlog Items

Not all work fits neatly into the initiative hierarchy. Bug reports, feature requests, and tech debt items often arrive independently. Metis handles these as **backlog items** — tasks with no parent initiative, categorized by type:

- **Bug** — Something that's broken
- **Feature** — Something users want
- **Tech Debt** — Something that needs cleanup

Backlog items start in the `backlog` phase and can be promoted to `todo` when you're ready to work on them. They can also be assigned to an initiative using the `reassign_parent` operation.

## The Flight Levels Connection

The name "Flight Levels" comes from aviation — the idea that different altitudes give different views of the terrain below. At 30,000 feet (vision), you see the landscape. At 10,000 feet (initiative), you see the roads and buildings. At ground level (task), you see the individual steps.

Metis adapts this metaphor for software project management, focusing on three practical levels rather than the full Flight Levels framework (which includes additional organizational layers). The key insight is the same: work at each level requires different tools, different time horizons, and different decision-making processes.
