---
name: document-selection
description: This skill should be used when the user asks "what document type should I create", "create a bug ticket", "create a feature request", "should this be a task or initiative", "when to use an ADR", "track this bug", "log this tech debt", or needs help choosing between vision, strategy, initiative, task, backlog item, or ADR document types.
---

# Document Type Selection

This skill helps choose the right Metis document type for different kinds of work.

## Quick Decision Guide

**Is this work, or is it a decision?**
- Decision about architecture/approach → **ADR**
- Work to be done → Continue below

**Does this define WHY the project exists?**
- Yes → **Vision**

**Does this coordinate multiple capability increments?**
- Yes → **Strategy** (if Full preset enabled)

**Does this create a fundamental capability increment?**
- Yes → **Initiative**

**Is this a discrete, completable piece of work?**
- Yes, belongs to an initiative → **Task**
- Yes, standalone (bug/feature/debt) → **Backlog Item**

## Document Types Reference

| Type | Purpose | Parent Required |
|------|---------|-----------------|
| Vision | North star objectives | No |
| Strategy | Coordinated approaches | Vision (published) |
| Initiative | Capability increments | Strategy/Vision (published) |
| Task | Atomic work units | Initiative (decompose/active phase) |
| Backlog Item | Ad-hoc bugs/features/debt | No |
| ADR | Architectural decisions | No |

**Parent phase guidance:**
- Initiatives are typically created under a published vision
- Tasks are typically created under an initiative in decompose or active phase
- `reassign_parent` enforces initiative phase (must be decompose/active)

## User Terminology Mapping

When users request work items using common terms, map to Metis document types:

| User Says | Create |
|-----------|--------|
| "bug ticket", "bug", "defect" | `create_document(type="task", backlog_category="bug", ...)` |
| "feature ticket", "feature request" | `create_document(type="task", backlog_category="feature", ...)` |
| "tech debt ticket", "tech debt" | `create_document(type="task", backlog_category="tech-debt", ...)` |
| "project", "epic", "feature work" | Initiative (with parent) |
| "work item", "ticket" | Task (if has parent) or Backlog Item (if standalone) |

## When to Create Each Type

### Vision
Create when:
- Starting a new project
- Redefining project direction
- Current vision no longer represents objectives

**Not a vision**: "Build feature X" (initiative), "Fix bugs" (operational), "Q1 goals" (strategy or initiatives)

### Strategy
Create when (Full preset only):
- Multiple teams need coordination
- Competing approaches to pursue
- Resource allocation needs decisions
- Strategic trade-offs should be documented

**Fields**:
- `stakeholders` - List of stakeholders involved
- `risk_level` - One of: `low`, `medium`, `high`, `critical` (defaults to medium)

**Not a strategy**: Single project (initiative), a decision (ADR), a wish list (backlog)

### Initiative
Create when:
- Work delivers meaningful capability increment
- Multiple tasks needed
- Discovery/design phases valuable
- Track as distinct project

**Not an initiative**: Single task, ongoing operations (backlog), aspiration without commitment (keep in backlog)

### Task
Create when:
- Clear parent initiative exists
- Discrete, completable unit
- One person can own it
- Done criteria are clear

**Not a task**: Work with no parent (backlog item), work too large (break down or make initiative)

### Backlog Item
Create when:
- Bug discovered in production
- Feature request not tied to initiative
- Tech debt to address when capacity allows
- Operational/maintenance work

**Categories**: `bug`, `feature`, `tech-debt`

**Moving backlog items**: Use `reassign_parent` to move a backlog item into an initiative, or move a task back to backlog.

### ADR
Create when:
- Making significant architectural decision
- Choosing between meaningful alternatives
- Decision affects multiple initiatives
- Future developers will wonder "why?"

**Not an ADR**: Trivial decisions, work to be done (task/initiative), meeting notes

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Task that takes months | Wrong granularity | If it has subtasks, make it an initiative |
| Initiative for every idea | Overhead | Use backlog items, promote when committed |
| Strategy for single project | Wrong level | Strategy coordinates multiple initiatives |
| ADR for implementation | Confusion | ADR records decision; tasks implement it |

## Edge Cases

**Task vs Initiative**: Does it need discovery/design phases? If yes, initiative.

**Initiative vs Backlog**: Committing to it now? If no, backlog.

**Backlog vs Task**: Does it have a parent? If no, backlog.

**Cross-cutting work**: Create initiative under most relevant parent; tasks can reference other initiatives.

## Additional Resources

For detailed decision trees and edge cases:
- **`references/decision-trees.md`** - Complete decision framework
- **`references/when-to-adr.md`** - ADR-specific guidance
