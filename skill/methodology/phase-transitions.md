# Phase Transitions

Documents in Metis advance through phases. Each phase represents a stage in the work's lifecycle, and transitions happen when exit criteria are met.

## Phase Sequences by Document Type

### Vision
```
draft -> review -> published
```
- **draft**: Initial capture of objectives and values
- **review**: Stakeholder feedback and refinement
- **published**: Stable north star, ready to drive work

### Strategy
```
shaping -> design -> ready -> active -> completed
```
- **shaping**: Exploring approaches, gathering constraints
- **design**: Defining the strategic approach
- **ready**: Strategy validated, waiting for capacity
- **active**: Initiatives being executed under this strategy
- **completed**: Strategic objectives achieved

### Initiative
```
discovery -> design -> ready -> decompose -> active -> completed
```
- **discovery**: Understanding the problem space
- **design**: Defining the solution approach
- **ready**: Solution validated, ready to break down
- **decompose**: Creating tasks from the design
- **active**: Tasks being executed
- **completed**: Initiative outcomes delivered

### Task
```
todo -> doing -> completed
```
Or for backlog items:
```
backlog -> todo -> doing -> completed
```
- **backlog**: Captured but not committed to
- **todo**: Ready to be pulled when capacity exists
- **doing**: Actively being worked
- **completed**: Done, acceptance criteria met

### ADR
```
draft -> discussion -> decided -> superseded
```
- **draft**: Initial proposal
- **discussion**: Gathering input, debating options
- **decided**: Decision made and binding
- **superseded**: Replaced by a newer decision

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

**For doing -> completed (tasks):**
- Acceptance criteria met
- Work verified/tested as appropriate
- No known defects

## When to Transition

### Pull-Based Transitions

Most transitions happen when capacity exists at the next phase:

- Move an initiative to **active** when you have capacity to work tasks
- Move a task to **doing** when you're ready to start it
- Move work to **completed** when it's actually done

### Don't Rush Transitions

**Common mistake**: Advancing phases to feel productive without meeting criteria.

- An initiative in "active" with no tasks isn't really active
- A task marked "completed" that doesn't meet acceptance criteria isn't done
- A design marked "ready" without review isn't actually ready

**The phases protect you.** They force the discipline that prevents rework.

### Blocked Work

Sometimes work gets stuck. Rather than forcing it through phases:

1. Identify the blocker explicitly (update `blocked_by` field)
2. Address the blocker (may require work at a different level)
3. Resume the phase once unblocked

Don't skip phases to work around blockers. That just moves the problem downstream.

## Using transition_phase

The `transition_phase` tool advances documents through their phase sequence.

**Auto-advance (recommended):**
```
transition_phase(short_code="PROJ-I-0001")
```
Moves to the next valid phase. Validates exit criteria.

**Explicit phase (when needed):**
```
transition_phase(short_code="PROJ-T-0042", phase="blocked")
```
Use explicit phases only for non-linear transitions like marking work blocked.

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
