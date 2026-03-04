# Core Principles of Flight Levels

Flight Levels is a thinking model for organizing work at different altitudes. Metis implements this as a Kanban system to help you manage work from north star objectives down to individual tasks.

## The Hierarchy

```
Vision (Flight Level 3)         - THE BIG WHY: North star, objectives, values
    └── Initiative (FL 1)       - CONCRETE PROJECTS: Deliverable work packages
            └── Task (FL 0)     - EXECUTION: Individual work items
```

Each level answers a different question:
- **Vision**: "Why is this worth doing? What's our north star?"
- **Initiative**: "What projects will move us toward that vision?"
- **Task**: "What specific work needs to be done?"

## Why Hierarchy Matters: Value Alignment

The hierarchy exists to enforce alignment and prevent waste.

**Every piece of work should trace back to the vision.** Tasks support initiatives, initiatives support the vision. This chain of alignment ensures:

- No "shadow work" - hidden efforts that don't contribute to objectives
- No "shadow backlogs" - secret lists of work that bypass prioritization
- Clear value justification - if work doesn't align to anything above it, question its value

**If you can't trace work back to the vision, that's a signal.** Either:
1. The work shouldn't be done
2. There's a missing initiative that should exist
3. It belongs in the backlog (see below)

### Vision: The North Star

A Vision is a statement document. It codifies:
- **Objectives**: What we're trying to achieve
- **Values**: Why this matters, why it's worth doing well
- **Success criteria**: How we'll know we've arrived

Example: "Make X a better experience" or "Fix the problems of Y" - these are vision-level statements. They don't change often because they represent fundamental purpose.

## The Backlog: Ad-Hoc Work

The backlog is a special area for work that doesn't fit neatly into the initiative hierarchy but still has value:

**Maintenance and operations:**
- Keeping systems running
- Support tasks
- Routine upkeep

**Entry points for future initiatives:**
- **Tech debt** - May feed into a future "improve code quality" initiative
- **Bugs** - May feed into a "stability" initiative or get attached to existing work
- **Feature requests** - May seed a new initiative or enhance an existing one

**Critical standalone work:**
- A P0 bug might need immediate attention independent of any initiative
- Emergency fixes that can't wait for proper alignment

**Judgment call**: Backlog items aren't unaligned - they're either operational necessities or entry points waiting to be pulled into initiatives. If your backlog keeps growing with items that never get pulled up, that's a smell.

## Pull-Based Flow (Kanban)

**Work is PULLED, never pushed.** This is fundamental.

Each flight level has backlogs. When a backlog runs low, that's the signal to look UP to the next level for what to pull down:

- **Task backlog low?** Look to the Initiative for what to decompose next
- **Initiative backlog low?** Look to the Vision for what to pursue next

This pull-based system:
- Prevents overloading any level
- Ensures work only enters when there's capacity
- Makes bottlenecks visible
- Lets teams self-organize around actual capacity

**Feedback flows UP through completion:**
- Completed tasks inform initiative progress
- Completed initiatives inform whether the vision is achievable as stated

## Time Horizons

Each document type operates at a different timescale:

| Type | Horizon | Stability |
|------|---------|-----------|
| Vision | 6mo - 2yr | Rarely changes - it's the north star |
| Initiative | 1-6mo | Changes when blocked or scope shifts |
| Task | 1-14 days | Changes frequently |
| ADR | Permanent | Only superseded, never deleted |

**Judgment call**: If work doesn't fit these horizons, you're probably at the wrong level. A "task" taking months is really an initiative. A "vision" changing weekly isn't a north star.

## Phase-Gated Progress

Documents advance through phases with exit criteria:

- **Visions**: draft -> review -> published
- **Initiatives**: discovery -> design -> ready -> decompose -> active -> completed
- **Tasks**: todo -> active -> completed (or backlog -> todo -> ...), with blocked as a side state

**Key principle**: Don't skip phases. Each phase exists to prevent common failures:
- Skipping discovery leads to solving the wrong problem
- Skipping design leads to rework
- Skipping decompose leads to vague, untrackable work

Use `transition_phase` without specifying a target to auto-advance. Only specify a phase for non-linear transitions (e.g., marking something blocked).

## Using Active Tasks as Working Memory

**CRITICAL**: Active tasks and initiatives serve as persistent working memory. While a task is in the `active` phase, you MUST regularly update it with progress, findings, and plan changes as you work.

### Why This Matters
- Long-running tasks may experience context compaction (memory loss)
- Documents persist across sessions and context windows
- Future work can reference past decisions and discoveries
- Other agents/humans can pick up where you left off

### What to Record in Active Tasks
Update frequently during active work:
- **Progress**: What you've completed, files modified, tests run
- **Findings**: Unexpected discoveries, code patterns found, blockers encountered
- **Decisions**: Why you chose approach A over B, trade-offs considered
- **Plan changes**: If original approach didn't work, document what changed and why
- **Next steps**: What remains to be done if work is interrupted

### How Often to Update
- After completing each significant step
- When you discover something unexpected
- When your approach changes from the original plan
- Every few tool calls during long operations
- Before ending a session with incomplete work

This ensures no work is lost even if context is compacted or the session ends unexpectedly.

## Source of Truth

The filesystem (markdown files) is the source of truth. The database is a cache that's rebuilt on every operation.

This means:
- You can edit markdown files directly
- Git operations (pull, merge) work naturally
- Database corruption self-heals
- Multi-developer collaboration just works

**Judgment call**: When in doubt about state, trust the markdown files.

## Document Identity

Every document has a short code: `PREFIX-TYPE-NNNN` (e.g., `PROJ-V-0001`, `ACME-I-0042`)

- **V** = Vision
- **I** = Initiative
- **T** = Task
- **A** = ADR

Always reference documents by short code, not title. Titles can change; short codes are permanent.

## Hierarchy Enforcement

Parent requirements guide proper document relationships based on type and preset configuration.

### Parent Requirements Table

| Document Type | Parent Type | Parent Should Be | Notes |
|---------------|-------------|------------------|-------|
| Vision | None | - | Top-level document |
| Initiative | Vision | `published` | Top of execution hierarchy |
| Task | Initiative | `decompose` or `active` | Regular tasks |
| Task (backlog) | None | - | Uses `backlog_category` parameter |
| ADR | None | - | Standalone decisions |

### What This Means

**Recommended workflow for creating child documents:**
- Initiative typically under a published Vision
- Task typically under an Initiative in `decompose` or `active` phase

Note: The `reassign_parent` tool enforces initiative phase (must be decompose/active). Task creation does not enforce this.

**Backlog items are the exception** - they have no parent requirement because they represent ad-hoc work not yet tied to an initiative. However, backlog items can be assigned to a parent initiative later when:
- The work gets prioritized into an existing initiative
- You create a periodic "bug bash" or "debt repayment" initiative to group and tackle accumulated backlog items

Use `reassign_parent` to move a backlog item into an initiative's task list, if desired.

**Judgment call**: If you're creating orphan tasks, you're probably missing an initiative. Step back and ask what project this work belongs to.

## Configuration Presets

Metis supports different configurations:

| Preset | Hierarchy | Best For |
|--------|-----------|----------|
| Streamlined | Vision -> Initiative -> Task | Small teams, clear vision-to-execution path |
| Direct | Vision -> Task | Solo work, simple projects |

The current project's preset is shown in MCP tool responses. Don't fight the configuration - if you need more structure, that's a signal to reconsider the preset.

## Reference

For tool parameters, workflows, and operational details, see the Metis MCP server description. This document teaches *when* and *why*; the MCP server description teaches *how*.
