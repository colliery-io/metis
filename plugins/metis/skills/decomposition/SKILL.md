---
name: decomposition
description: This skill should be used when the user asks to "break down this initiative", "decompose into tasks", "create tasks from initiative", "how to size tasks", "when to decompose", "vertical slices", "task granularity", or needs guidance on breaking higher-level work into lower-level work items.
---

# Work Decomposition

This skill guides the process of breaking higher-level work into actionable lower-level items.

## The Decomposition Chain

```
Vision: "Make X a better experience"
    ↓
Strategy: "Focus on performance first"
    ↓
Initiative: "Reduce page load time by 50%"
    ↓
Tasks: "Profile slow queries", "Add caching layer", "Optimize images"
```

Each level breaks work above it into concrete, actionable pieces at appropriate scope.

## When to Decompose

Decompose **ahead of capacity**, not upfront:

- When team's current backlog is nearing its end
- During tail end of current work to prepare next batch
- When backlog is getting low (signal to look up and pull work down)

**Avoid**: Decomposing everything upfront (waterfall). Have work ready when capacity frees up, not entire project planned before starting.

## The Decompose Phase

Initiatives have an explicit "decompose" phase:

```
discovery → design → ready → decompose → active → completed
```

### Why Decompose is Explicit

The decompose phase creates a **visible buffer**:
- Solutions can pile up waiting to be broken into tasks
- Tracks how long things sit here
- Makes bottlenecks visible in multi-team environments

**Don't skip to decompose early.** Premature decomposition leads to tasks that solve wrong problems, rework when design changes, wasted effort.

## Sizing by Scope, Not Time

Size by scope and impact, not implementation time:

### Tasks: Atomic Units
- **Scope**: Discrete, completable piece with clear done criteria
- **Impact**: Moves the needle on parent initiative
- **Independence**: Can be worked without constant coordination
- **Examples**: "Add caching layer", "Write migration script", "Update API endpoint"

**If a task has meaningful subtasks**, it should probably be an initiative.

### Initiatives: Capability Increments
- **Scope**: Creates fundamental increment in capability
- **Impact**: Meaningfully changes what system can do
- **Coherence**: Tasks within work toward unified outcome
- **Examples**: "User authentication", "Search functionality", "Billing integration"

**If it doesn't change what system can do**, it might just be a task.

## Decomposition Patterns

### Vertical Slices (Preferred)
Break by user-visible functionality:
```
Initiative: "User authentication"
├── Task: "Login flow"
├── Task: "Registration flow"
├── Task: "Password reset"
└── Task: "Session management"
```
Each task delivers something user can see/use.

### Horizontal Layers (Use Sparingly)
Break by technical component:
```
Initiative: "User authentication"
├── Task: "Database schema"
├── Task: "API endpoints"
├── Task: "Frontend components"
└── Task: "Integration tests"
```
Creates dependencies between tasks. Prefer vertical slices.

### Risk-First
Break by unknowns:
```
Initiative: "ML recommendation engine"
├── Task: "Spike: Evaluate model options" (high uncertainty)
├── Task: "Build training pipeline" (after spike)
└── Task: "Integration with product" (low uncertainty)
```
Address risky/uncertain work first to fail fast.

### Milestone-Based
Break by deliverable checkpoints:
```
Initiative: "Platform migration"
├── Task: "Phase 1: Read path on new platform"
├── Task: "Phase 2: Write path on new platform"
├── Task: "Phase 3: Deprecate old platform"
└── Task: "Phase 4: Cleanup"
```
Each milestone independently valuable and deployable.

## Quality Checklist

Good decomposition - each child item:
- **Independently valuable**: Delivers something useful alone
- **Clearly scoped**: Know when it's done
- **Right-sized**: Matches scope expectations for level
- **Aligned to parent**: Clearly contributes to level above

Bad decomposition smells:
- **Too granular**: "write line 42" - steps, not tasks
- **Too vague**: "make it better" - no completion criteria
- **Wrong level**: Doesn't match document type scope
- **Orphaned**: Doesn't trace back to parent
- **Overlapping**: Multiple items covering same ground

## Common Mistakes

| Mistake | Problem | Fix |
|---------|---------|-----|
| Decomposing too early | Tasks solve wrong problem | Stay in discovery/design until approach clear |
| Decomposing too late | Initiative active with no tasks | Decompose before moving to active |
| Wrong granularity | Tasks that are initiatives or vice versa | Apply scope heuristics |
| Missing alignment | Tasks don't contribute to initiative | Each task needs obvious connection to parent |

## Judgment Calls

- **Uncertain scope?** Create spike/research task first, then decompose based on findings
- **Large initiative?** Consider if it's really multiple capability increments
- **Tiny initiative?** Consider if it's really just a task
- **Cross-cutting?** May need tasks under multiple initiatives, or dedicated "platform" initiative

## Additional Resources

For detailed decomposition patterns and examples:
- **`references/decomposition-patterns.md`** - Complete pattern catalog with examples
