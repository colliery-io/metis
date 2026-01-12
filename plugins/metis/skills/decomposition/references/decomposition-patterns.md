# Work Decomposition

Decomposition is the process of breaking higher-level work into lower-level work items. Done well, it creates a clear path from vision to execution. Done poorly, it creates confusion, orphaned work, and lost alignment.

## The Decomposition Chain

```
Vision: "Make X a better experience"
    ↓ decomposes into
Strategy: "Focus on performance first"
    ↓ decomposes into
Initiative: "Reduce page load time by 50%"
    ↓ decomposes into
Tasks: "Profile slow queries", "Add caching layer", "Optimize images"
```

Each level breaks the work above it into concrete, actionable pieces at the appropriate scope.

## When to Decompose

Decomposition takes time and investment. Plan for it:

- **Ahead of capacity** - Decompose when the team's current backlog is nearing its end
- **As work wraps up** - Use the tail end of current work to prepare the next batch
- **Well in advance if needed** - You can decompose early, just don't start execution until it's time

**What to avoid**: Decomposing everything upfront (waterfall). The goal is to have work ready when capacity frees up, not to plan the entire project before starting.

**The signal to decompose**: Your backlog is getting low. That's when you look up to the next level and pull work down through decomposition.

## Decomposition Quality

### Good Decomposition

Each child item:
- **Independently valuable** - Delivers something useful on its own
- **Clearly scoped** - You know when it's done
- **Right-sized for its level** - Matches the scope expectations (see below)
- **Aligned to parent** - Clearly contributes to the level above

### Bad Decomposition

Watch for these smells:

- **Too granular**: Tasks like "write line 42" or "add semicolon" - these aren't tasks, they're steps
- **Too vague**: Tasks like "make it better" or "fix things" - no clear completion criteria
- **Wrong level**: Work that doesn't match the scope of its document type
- **Orphaned**: Work that doesn't trace back to something above it
- **Overlapping**: Multiple items covering the same ground

## Sizing by Scope, Not Time

Implementation time varies dramatically based on tooling, team, and automation. Instead of time-based rules, size by scope and impact:

### Tasks: Atomic Units of Work
- **Scope**: A discrete, completable piece of work with clear done criteria
- **Impact**: Moves the needle on its parent initiative
- **Independence**: Can be worked without constant coordination
- **Examples**: "Add caching layer", "Write migration script", "Update API endpoint"

**If a task has subtasks that are themselves meaningful**, it should probably be an initiative.

### Initiatives: Fundamental Capability Increments
- **Scope**: Creates a fundamental increment in capability or feature in the system
- **Impact**: Meaningfully changes what the system can do
- **Coherence**: Tasks within it work toward a unified outcome
- **Examples**: "User authentication", "Search functionality", "Billing integration"

The "system" here can be software (technical projects) or people/processes (operational projects). An initiative delivers a capability increment to that system.

**If an initiative doesn't change what the system can do**, it might just be a task. **If it changes multiple unrelated capabilities**, it might be multiple initiatives.

### Strategies: Coherent Approaches
- **Scope**: A direction that coordinates multiple capability increments
- **Impact**: Shapes which initiatives get pursued and how
- **Coherence**: Initiatives within it share a common approach or trade-off
- **Examples**: "Performance-first approach", "Mobile expansion", "Enterprise readiness"

## The Decompose Phase

Initiatives have an explicit "decompose" phase between ready and active:

1. **Discovery** - Understand the problem space
2. **Design** - Define the solution approach
3. **Ready** - Solution validated, ready to break down
4. **Decompose** - Create tasks that implement the solution
5. **Active** - Execute the tasks

### Why Decompose is Explicit

The decompose phase exists as a **visible buffer**:

- **Solutions can pile up** - Designed initiatives waiting to be broken into tasks
- **Tracks decomposition time** - How long are things sitting here?
- **Makes bottlenecks visible** - Especially in multi-team environments

**Coordination signal**: If initiatives are stuck in decompose, it may indicate:
- A team is struggling to understand the work
- The design wasn't clear enough
- Support or a check-in is needed from the initiative working group

This visibility is valuable. Without an explicit decompose phase, these problems stay hidden until work doesn't get done.

### Don't Skip Phases

**Don't skip to decompose early.** Premature decomposition leads to:
- Tasks that solve the wrong problem
- Rework when design changes
- Wasted effort on abandoned approaches

## Decomposition Patterns

### Vertical Slices
Break by user-visible functionality:
```
Initiative: "User authentication"
├── Task: "Login flow"
├── Task: "Registration flow"
├── Task: "Password reset"
└── Task: "Session management"
```

Each task delivers something a user can see/use.

### Horizontal Layers
Break by technical component (use sparingly):
```
Initiative: "User authentication"
├── Task: "Database schema"
├── Task: "API endpoints"
├── Task: "Frontend components"
└── Task: "Integration tests"
```

Creates dependencies between tasks. Prefer vertical slices when possible.

### Risk-First
Break by unknowns:
```
Initiative: "ML recommendation engine"
├── Task: "Spike: Evaluate model options" (high uncertainty)
├── Task: "Build training pipeline" (after spike)
├── Task: "Integration with product" (low uncertainty)
└── Task: "A/B test framework" (low uncertainty)
```

Address risky/uncertain work first to fail fast.

### Milestone-Based
Break by deliverable checkpoints:
```
Initiative: "Platform migration"
├── Task: "Phase 1: Read path on new platform"
├── Task: "Phase 2: Write path on new platform"
├── Task: "Phase 3: Deprecate old platform"
└── Task: "Phase 4: Cleanup and optimization"
```

Each milestone is independently valuable and deployable.

## Common Mistakes

### Decomposing Too Early
Creating all tasks before understanding the problem. Leads to rework.

**Fix**: Stay in discovery/design until approach is clear.

### Decomposing Too Late
Having an initiative in active phase with no tasks. Nothing to pull.

**Fix**: Decompose before moving to active. Use the decompose phase.

### Wrong Granularity
Tasks that are actually subtasks, or initiatives that are actually epics.

**Fix**: Apply scope heuristics. Does this create a capability increment? Then it's an initiative. Is it an atomic unit of work? Then it's a task.

### Missing Alignment
Tasks that don't clearly contribute to initiative goals.

**Fix**: Each task should have an obvious connection to its parent. If you can't explain the connection, question the task.

## Judgment Calls

- **Uncertain scope?** Create a spike/research task first, then decompose based on findings
- **Large initiative?** Consider whether it's really multiple capability increments
- **Tiny initiative?** Consider whether it's really just a task
- **Cross-cutting work?** May need tasks under multiple initiatives, or a dedicated "platform" initiative
