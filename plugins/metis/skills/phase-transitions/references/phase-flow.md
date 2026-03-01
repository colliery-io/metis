# Phase Transitions

Documents in Metis advance through phases. Each phase represents a stage in the work's lifecycle, and transitions happen when exit criteria are met.

## Phase Sequences by Document Type

Phases move forward only. You cannot go backward to a previous phase (except returning from blocked state for tasks).

### Vision
```
draft → review → published
```
- **draft**: Initial capture of objectives and values
- **review**: Stakeholder feedback and refinement
- **published**: Stable north star, ready to drive work (terminal)

### Strategy
```
draft → review → published → active → completed
```
- **draft**: Strategy being shaped by the strategy team
- **review**: Shared with stakeholders for feedback and debate
- **published**: Approved direction — initiative boards should act on it
- **active**: Initiatives are executing against this strategy
- **completed**: Terminal state

### Initiative
```
discovery → design → ready → decompose → active → completed
```
- **discovery**: Understanding the problem space
- **design**: Defining the solution approach
- **ready**: Solution validated, ready to break down
- **decompose**: Creating tasks from the design
- **active**: Tasks being executed
- **completed**: Terminal state

### Task
```
backlog → todo → active → completed
            ↓       ↓
         blocked ←──┘
```
- **backlog**: Captured but not committed to
- **todo**: Ready to be pulled when capacity exists
- **active**: Actively being worked
- **blocked**: Waiting on external dependency (can return to todo or active)
- **completed**: Terminal state

Valid transitions:
- backlog → todo
- todo → active
- todo → blocked
- active → completed
- active → blocked
- blocked → todo (return from blocked)
- blocked → active (return from blocked)

### ADR
```
draft → discussion → decided → superseded
```
- **draft**: Initial proposal
- **discussion**: Gathering input, debating options
- **decided**: Decision made and binding
- **superseded**: Replaced by a newer decision (terminal)

**WARNING**: Auto-advancing from `decided` moves to `superseded`. Most ADRs should stay in `decided` indefinitely. Only manually transition to `superseded` when explicitly replacing with a new ADR.

## Exit Criteria

Each phase has exit criteria - conditions that must be true before transitioning. Exit criteria prevent premature advancement and ensure quality gates are met.

### What Makes Good Exit Criteria

- **Observable**: Can be verified, not just assumed
- **Specific**: Clear what "done" means
- **Relevant**: Actually matters for the next phase
- **Achievable**: Realistic given the work scope

### Common Exit Criteria Patterns

**For discovery -> design:**
- Problem statement is clear and validated
- Key constraints and requirements identified
- Stakeholders aligned on scope

**For design -> ready:**
- Solution approach documented
- Technical risks identified and mitigated
- Dependencies mapped

**For ready -> decompose:**
- Design reviewed and approved
- Team capacity available
- No blocking dependencies

**For decompose -> active:**
- Tasks created with clear acceptance criteria
- Task backlog is sufficient to start
- Team understands the work

**For active -> completed (tasks):**
- Acceptance criteria met
- Work verified/tested as appropriate
- No known defects

### Tracking Exit Criteria Status

Documents have an `exit_criteria_met` frontmatter field that tracks whether exit criteria have been satisfied:

```yaml
exit_criteria_met: false  # or true
```

This field is:
- Stored in the document frontmatter and database
- Currently requires manual update (set to `true` when criteria are met)
- Used by `transition_phase` with `force: false` to prevent premature transitions

When transitioning without `force: true`, the system may warn if exit criteria haven't been marked as met. To force a transition anyway (acknowledging the risk), use `force: true`.

## Phase Transition Constraints

**IMPORTANT**: Phases move forward only. You cannot skip phases or go backward.

### What This Means

- **Cannot skip phases**: A task in "todo" cannot go directly to "completed" - it must go through "active" first
- **Cannot skip phases**: An initiative in "discovery" cannot jump to "active" - it must progress through design, ready, decompose
- **Forward-only**: Once you advance a phase, you cannot go back (except returning from blocked for tasks)

### Common Mistakes

**Phase skipping will fail**: These transitions are INVALID and will error:
- `todo → completed` (must go todo → active → completed)
- `discovery → active` (must progress through all intermediate phases)
- `draft → published` (must go draft → review → published)

**Backward transitions are not supported**: You cannot move from review back to draft, or from design back to discovery.

**To complete a task**, call `transition_phase` twice:
1. First call: todo → active (start working)
2. Second call: active → completed (finish work)

**To publish a vision**, call `transition_phase` twice:
1. First call: draft → review
2. Second call: review → published

## When to Transition

### Pull-Based Transitions

Most transitions happen when capacity exists at the next phase:

- Move an initiative to **active** when you have capacity to work tasks
- Move a task to **active** when you're ready to start it
- Move work to **completed** when it's actually done

### Don't Rush Transitions

**Common mistake**: Advancing phases to feel productive without meeting criteria.

- An initiative in "active" with no tasks isn't really active
- A task marked "completed" that doesn't meet acceptance criteria isn't done
- A design marked "ready" without review isn't actually ready

**The phases protect you.** They force the discipline that prevents rework.

### Blocked Work

Sometimes work gets stuck. Use the blocked state for tasks:

1. Transition to blocked: `transition_phase(short_code, phase="blocked")`
2. Identify the blocker explicitly (update `blocked_by` field)
3. Address the blocker (may require work at a different level)
4. Return from blocked: `transition_phase(short_code, phase="active")` or `phase="todo"`

Blocked is the only state that allows "backward" movement - but it's returning from a paused state, not reversing progress. Only tasks support the blocked state.

## Using transition_phase

The `transition_phase` tool advances documents through their phase sequence.

**Auto-advance (recommended):**
```
transition_phase(short_code="PROJ-I-0001")
```
Moves to the next valid phase. Validates exit criteria.

**Explicit phase (for blocked state only):**
```
transition_phase(short_code="PROJ-T-0042", phase="blocked")
```
Use explicit phases only for moving to/from the blocked state (tasks only).

**Force (use sparingly):**
```
transition_phase(short_code="PROJ-I-0001", force=true)
```
Skips exit criteria validation. Use only when you understand why criteria aren't met and it's acceptable.

## Monitoring Phase Health

### Signs of Healthy Flow
- Work moves steadily through phases
- Exit criteria are met before transitions
- Blocked items are rare and resolved quickly
- Completed work stays completed

### Signs of Unhealthy Flow
- Work stuck in early phases (discovery/design) - may indicate unclear requirements or analysis paralysis
- Work stuck in decompose - may indicate team struggling to understand the work
- Work jumping straight to active - skipping necessary preparation
- "Completed" work reopening - exit criteria weren't actually met

### Phase as Communication

Phases communicate status without requiring updates:

- "Where is Project X?" - Check what phase the initiative is in
- "Why isn't this moving?" - Check what's blocking the transition
- "Is this ready to start?" - Is it in the appropriate phase?

The phase is the single source of truth for work status.

## Judgment Calls

- **Exit criteria feel bureaucratic?** They might be. Adjust them to what actually matters.
- **Want to skip a phase?** Ask why. Usually there's a reason that phase exists.
- **Work keeps bouncing back?** Exit criteria might be too weak, or upstream work isn't solid.
- **Everything stuck in one phase?** Bottleneck. Either add capacity or reduce WIP.
