# Core Principles of Flight Levels

Flight Levels is a thinking model for organizing work at different altitudes. Metis implements this as a Kanban system to help you manage work from north star objectives down to individual tasks.

## The Hierarchy

```
Vision (Flight Level 3)         - THE BIG WHY: North star, objectives, values
    └── Strategy (FL 2)         - STRATEGIC DIRECTION: Coordinated approaches to achieve the vision
            └── Initiative (FL 1) - CONCRETE PROJECTS: Deliverable work packages
                    └── Task (FL 0) - EXECUTION: Individual work items
```

Each level answers a different question:
- **Vision**: "Why is this worth doing? What's our north star?"
- **Strategy**: "What approaches will move us toward that vision?"
- **Initiative**: "What projects will execute those approaches?"
- **Task**: "What specific work needs to be done?"

## Why Hierarchy Matters: Value Alignment

The hierarchy exists to enforce alignment and prevent waste.

**Every piece of work should trace back to the vision.** Tasks support initiatives, initiatives support strategies, strategies support the vision. This chain of alignment ensures:

- No "shadow work" - hidden efforts that don't contribute to objectives
- No "shadow backlogs" - secret lists of work that bypass prioritization
- Clear value justification - if work doesn't align to anything above it, question its value

**If you can't trace work back to the vision, that's a signal.** Either:
1. The work shouldn't be done
2. There's a missing initiative/strategy that should exist
3. It belongs in the backlog (see below)

### Vision: The North Star

A Vision is a statement document. It codifies:
- **Objectives**: What we're trying to achieve
- **Values**: Why this matters, why it's worth doing well
- **Success criteria**: How we'll know we've arrived

Example: "Make X a better experience" or "Fix the problems of Y" - these are vision-level statements. They don't change often because they represent fundamental purpose.

### Strategy: The Direction

Strategy defines HOW you'll pursue the vision. It's where you decide:
- Which approaches to take (and which to defer)
- How to allocate resources across initiatives
- What trade-offs you're willing to make

A vision might have multiple strategies. "Make X a better experience" could be pursued via "Improve performance" AND "Simplify the UI" - two different strategic directions.

### When Strategy is Skipped

Small teams and technically-focused projects often use the **streamlined** preset which omits Strategy. This works when:
- A single team owns the entire vision
- Initiatives don't require cross-team coordination
- The path from vision to execution is clear

Skipping Strategy is a valid choice, not a shortcut. But recognize what you're giving up: explicit coordination logic between initiatives.

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
- **Initiative backlog low?** Look to the Strategy for what direction to pursue next
- **Strategy backlog low?** Look to the Vision for what approach to develop next

This pull-based system:
- Prevents overloading any level
- Ensures work only enters when there's capacity
- Makes bottlenecks visible
- Lets teams self-organize around actual capacity

**Feedback flows UP through completion:**
- Completed tasks inform initiative progress
- Completed initiatives validate strategy assumptions
- Strategy outcomes inform whether the vision is achievable as stated

## Time Horizons

Each document type operates at a different timescale:

| Type | Horizon | Stability |
|------|---------|-----------|
| Vision | 6mo - 2yr | Rarely changes - it's the north star |
| Strategy | 3-12mo | Adjusts based on initiative outcomes |
| Initiative | 1-6mo | Changes when blocked or scope shifts |
| Task | 1-14 days | Changes frequently |
| ADR | Permanent | Only superseded, never deleted |

**Judgment call**: If work doesn't fit these horizons, you're probably at the wrong level. A "task" taking months is really an initiative. A "vision" changing weekly isn't a north star.

## Phase-Gated Progress

Documents advance through phases with exit criteria:

- **Visions**: draft -> review -> published
- **Strategies**: shaping -> design -> ready -> active -> completed
- **Initiatives**: discovery -> design -> ready -> decompose -> active -> completed
- **Tasks**: todo -> doing -> completed (or backlog -> todo -> doing -> completed)

**Key principle**: Don't skip phases. Each phase exists to prevent common failures:
- Skipping discovery leads to solving the wrong problem
- Skipping design leads to rework
- Skipping decompose leads to vague, untrackable work

Use `transition_phase` without specifying a target to auto-advance. Only specify a phase for non-linear transitions (e.g., marking something blocked).

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
- **S** = Strategy
- **I** = Initiative
- **T** = Task
- **A** = ADR

Always reference documents by short code, not title. Titles can change; short codes are permanent.

## Hierarchy Enforcement

Parent requirements are enforced:
- Strategies require a published Vision as parent
- Initiatives require an active Strategy as parent (or published Vision in streamlined mode)
- Tasks require an Initiative in decompose or active phase as parent
- Backlog items (standalone bugs/features/tech-debt) have no parent requirement

**Judgment call**: If you're creating orphan tasks, you're probably missing an initiative. Step back and ask what project this work belongs to.

## Configuration Presets

Metis supports different configurations:

| Preset | Hierarchy | Best For |
|--------|-----------|----------|
| Full | Vision -> Strategy -> Initiative -> Task | Large orgs, multiple coordinated approaches |
| Streamlined | Vision -> Initiative -> Task | Small teams, clear vision-to-execution path |
| Direct | Vision -> Task | Solo work, simple projects |

The current project's preset is shown in MCP tool responses. Don't fight the configuration - if you need strategies but they're disabled, that's a signal to reconsider the preset.

## Reference

For tool parameters, workflows, and operational details, see the Metis MCP server description. This document teaches *when* and *why*; the MCP server description teaches *how*.
