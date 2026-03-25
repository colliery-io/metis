# How to Transition Document Phases

Every Metis document moves through a defined sequence of phases. This guide shows how to advance documents through their lifecycle.

## Basic Transition

Specify the target phase explicitly:

```bash
metis transition PROJ-T-0001 active
```

Or auto-advance to the next phase:

```bash
metis transition PROJ-T-0001
```

Auto-advance moves the document to the next phase in its sequence. For a task in `todo`, this transitions to `active`.

## Common Examples

### Advance a task through its lifecycle

```bash
metis transition PROJ-T-0001 active      # Todo → Active
metis transition PROJ-T-0001 completed   # Active → Completed
```

### Block and unblock a task

```bash
metis transition PROJ-T-0001 blocked     # Active → Blocked
metis transition PROJ-T-0001 active      # Blocked → Active (unblock)
```

Tasks are the only document type that supports moving backward (from `blocked` to `active` or `todo`).

### Advance an initiative

```bash
metis transition PROJ-I-0001 design      # Discovery → Design
metis transition PROJ-I-0001 ready       # Design → Ready
metis transition PROJ-I-0001 decompose   # Ready → Decompose
```

See [Phase Lifecycle Reference](../reference/phase-lifecycle.md) for the complete phase sequences for all five document types.

## Transition Rules

**You cannot skip phases.** Transitions must be to adjacent phases. For example, `todo → completed` is invalid — you must go `todo → active → completed`.

**Phase names are case-insensitive.** `Active`, `active`, and `ACTIVE` all work.

**The current phase is stored in frontmatter tags.** When you transition, Metis updates the `#phase/` tag in the document's frontmatter.

## Via MCP (Claude Code)

Ask Claude to transition documents:

> "Move task PROJ-T-0001 to active"

Or auto-advance:

> "Advance PROJ-I-0001 to the next phase"

Claude uses the `transition_phase` MCP tool, which validates the transition and returns the from/to phases.

## Check Current Phase

```bash
metis status                              # Shows phase for all documents
metis list -t task -p active              # Filter by phase
```

## Handling Blocked Tasks

When a task is blocked by another document, transition it to `blocked`:

```bash
metis transition PROJ-T-0003 blocked
```

Edit the task's frontmatter to add the blocking document:

```yaml
blocked_by:
  - PROJ-T-0001
```

Once the blocker is resolved, unblock:

```bash
metis transition PROJ-T-0003 active
```

See [Phase Lifecycle Reference](../reference/phase-lifecycle.md) for complete transition rules and exit criteria.
