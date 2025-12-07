# Document Type Selection

A decision framework for choosing the right document type.

## Quick Reference

| Document Type | Use When | Parent Required |
|--------------|----------|-----------------|
| Vision | Defining north star objectives | No |
| Strategy | Coordinating approaches to vision | Vision |
| Initiative | Delivering a capability increment | Strategy/Vision |
| Task | Atomic unit of work | Initiative |
| Backlog Item | Ad-hoc work (bug/feature/debt) | No |
| ADR | Recording architectural decisions | No |

## Decision Tree

### Start Here

**Is this work, or is it a decision?**
- Decision about architecture/approach → **ADR**
- Work to be done → Continue below

### For Work Items

**Does this define WHY the project exists?**
- Yes → **Vision**
- No → Continue

**Does this coordinate multiple capability increments?**
- Yes → **Strategy** (if enabled) or just organize initiatives
- No → Continue

**Does this create a fundamental capability increment?**
- Yes → **Initiative**
- No → Continue

**Is this a discrete, completable piece of work?**
- Yes, and it belongs to an initiative → **Task**
- Yes, but it's standalone (bug/feature/debt) → **Backlog Item**
- No → Probably needs to be broken down further

## Detailed Guidance

### When to Create a Vision

Create a vision when:
- Starting a new project
- Redefining project direction
- Current vision no longer represents objectives

**Not a vision:**
- "Build feature X" (that's an initiative)
- "Fix bugs" (that's operational work)
- "Q1 goals" (that's potentially a strategy or just initiatives)

### When to Create a Strategy

Create a strategy when (Full preset only):
- Multiple teams need coordination
- There are competing approaches to pursue
- Resource allocation needs explicit decisions
- Strategic trade-offs should be documented

**Not a strategy:**
- A single project (that's an initiative)
- A decision (that's an ADR)
- A wish list (that's a backlog)

### When to Create an Initiative

Create an initiative when:
- Work delivers a meaningful capability increment
- Multiple tasks are needed
- Discovery/design phases are valuable
- You want to track it as a distinct project

**Not an initiative:**
- A single task (just make it a task)
- Ongoing operations (use backlog)
- An aspiration without commitment (keep in backlog until ready)

### When to Create a Task

Create a task when:
- Work has a clear parent initiative
- It's a discrete, completable unit
- One person can own it
- Done criteria are clear

**Not a task:**
- Work with no parent (use backlog item)
- Work too large to be atomic (break it down or make it an initiative)

### When to Create a Backlog Item

Create a backlog item when:
- It's a bug discovered in production
- It's a feature request not yet tied to an initiative
- It's tech debt to address when capacity allows
- It's operational/maintenance work

**Backlog categories:**
- `bug` - Something broken
- `feature` - Enhancement request
- `tech-debt` - Code quality improvement

### When to Create an ADR

Create an ADR when:
- Making a significant architectural decision
- Choosing between meaningful alternatives
- Decision will affect multiple initiatives
- Future developers will wonder "why did we do it this way?"

**Not an ADR:**
- Trivial decisions (just decide)
- Work to be done (that's a task/initiative)
- Meeting notes (that's documentation)

## Common Mistakes

### Task vs Initiative
**Wrong**: Creating a "task" that takes months
**Right**: If it has subtasks, it's probably an initiative

### Initiative vs Backlog
**Wrong**: Creating initiatives for every idea
**Right**: Backlog items can be promoted to initiatives when committed

### Strategy vs Initiative
**Wrong**: Using strategy for a single project
**Right**: Strategy coordinates multiple initiatives toward vision

### ADR vs Task
**Wrong**: "Implement decision X" as an ADR
**Right**: ADR records the decision; tasks implement it

## Edge Cases

### "This could be either..."

When unclear between levels:
- **Task vs Initiative**: Does it need discovery/design phases? If yes, initiative.
- **Initiative vs Backlog**: Are you committing to it now? If no, backlog.
- **Backlog vs Task**: Does it have a parent? If no, backlog.

### Cross-Cutting Work

Work that touches multiple areas:
- Create initiative under most relevant parent
- Tasks can reference other initiatives in their documentation
- Consider if it should be multiple initiatives

### Maintenance/Operations

Ongoing operational work:
- Regular maintenance → Backlog items
- Significant improvement → Initiative
- Process change → Maybe an ADR for the decision
