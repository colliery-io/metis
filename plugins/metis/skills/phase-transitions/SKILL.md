---
name: phase-transitions
description: This skill should be used when the user asks "when to transition phases", "move to active", "exit criteria", "what phase comes next", "how to complete a task", "transition to completed", "mark as blocked", "phase flow", or needs guidance on advancing documents through their lifecycle phases.
---

# Phase Transitions

This skill guides moving Metis documents through their lifecycle phases.

## Phase Sequences

Phases move forward only. You cannot go backward to a previous phase. Terminal phases cannot transition further.

### Vision
```
draft → review → published
```
- **draft** → review
- **review** → published
- **published**: terminal

### Strategy (Full preset only)
```
draft → review → published → active → completed
```
- **draft** → review
- **review** → published
- **published** → active
- **active** → completed
- **completed**: terminal

### Initiative
```
discovery → design → ready → decompose → active → completed
```
- **discovery** → design
- **design** → ready
- **ready** → decompose
- **decompose** → active
- **active** → completed
- **completed**: terminal

### Task
```
backlog → todo → active → completed
            ↓       ↓
         blocked ←──┘
```
- **backlog** → todo
- **todo** → active OR blocked
- **active** → completed OR blocked
- **blocked** → todo OR active (return from blocked state)
- **completed**: terminal

### ADR
```
draft → discussion → decided → superseded
```
- **draft** → discussion
- **discussion** → decided
- **decided** → superseded
- **superseded**: terminal

**WARNING**: Auto-advancing from `decided` moves to `superseded`. Most ADRs should stay in `decided` indefinitely. Only manually transition to `superseded` when explicitly replacing with a new ADR.

## Default Phases

When documents are created, they start in these phases:
- **Vision**: `draft`
- **Strategy**: `draft`
- **Initiative**: `discovery`
- **Task**: `todo` (or `backlog` for backlog items)
- **ADR**: `draft`

**Backlog note**: Tasks created with `backlog_category` start in `backlog` phase and do NOT auto-transition. You must explicitly transition from `backlog` → `todo` before the task can be worked.

## Critical Rule: No Phase Skipping

**Transitions are constrained to adjacent phases only.**

Invalid transitions (will error):
- `todo → completed` (must go todo → active → completed)
- `discovery → active` (must progress through all intermediate phases)
- `draft → published` (must go draft → review → published)

**To complete a task**, call `transition_phase` twice:
1. `transition_phase(short_code)` → todo to active
2. `transition_phase(short_code)` → active to completed

**To publish a vision**, call `transition_phase` twice:
1. `transition_phase(short_code)` → draft to review
2. `transition_phase(short_code)` → review to published

## Using transition_phase

**Auto-advance (recommended):**
```
transition_phase(short_code="PROJ-I-0001")
```
Moves to next valid phase. Validates exit criteria.

**Explicit phase (for blocked state):**
```
transition_phase(short_code="PROJ-T-0042", phase="blocked")
```
Use explicit phase only for moving to/from blocked state (tasks only).

**Force (use sparingly):**
```
transition_phase(short_code="PROJ-I-0001", force=true)
```
Skips exit criteria validation. Use only when accepting the risk.

## Exit Criteria

Exit criteria are conditions that must be true before transitioning.

### Good Exit Criteria
- **Observable**: Can be verified
- **Specific**: Clear what "done" means
- **Relevant**: Matters for next phase
- **Achievable**: Realistic for scope

### Common Exit Criteria Patterns

**discovery → design:**
- Problem statement clear and validated
- Key constraints identified
- Stakeholders aligned on scope

**design → ready:**
- Solution approach documented
- Technical risks identified
- Dependencies mapped

**ready → decompose:**
- Design reviewed and approved
- Team capacity available
- No blocking dependencies

**decompose → active:**
- Tasks created with acceptance criteria
- Task backlog sufficient to start
- Team understands the work

**active → completed (tasks):**
- Acceptance criteria met
- Work verified/tested
- No known defects

### Tracking Exit Criteria

Documents have `exit_criteria_met` frontmatter field:
```yaml
exit_criteria_met: false  # or true
```
Set to `true` when criteria are met. Used by `transition_phase` with `force: false`.

## Blocked Work

Handle blocked work explicitly:

1. Transition to blocked: `transition_phase(short_code, phase="blocked")`
2. Update `blocked_by` field in document to record what's blocking
3. Address the blocker
4. Return from blocked: `transition_phase(short_code, phase="active")` or `phase="todo"`

**Note**: Only tasks can be blocked - visions, strategies, initiatives, and ADRs cannot use the blocked phase.

Blocked is a special state that allows returning to todo or active. This is the only case where you can move "backward" - but it's really returning from a paused state, not reversing progress.

## Working in Active Phase

**CRITICAL**: Active tasks and initiatives serve as persistent working memory. While in `active` phase, regularly update the document with:

- **Progress**: What's been completed
- **Findings**: Unexpected discoveries, blockers
- **Decisions**: Why you chose approach A over B
- **Next steps**: What remains if work is interrupted

This ensures no work is lost if context is compacted or the session ends.

## When to Transition

### Pull-Based Transitions
- Move initiative to **active** when capacity exists
- Move task to **active** when ready to start
- Move to **completed** when actually done

### Don't Rush Transitions
Common mistakes:
- Initiative in "active" with no tasks isn't really active
- Task marked "completed" that doesn't meet criteria isn't done
- Design marked "ready" without review isn't ready

**The phases protect you.** They force discipline that prevents rework.

## Monitoring Phase Health

### Healthy Signs
- Work moves steadily through phases
- Exit criteria met before transitions
- Blocked items rare and resolved quickly
- Completed work stays completed

### Unhealthy Signs
- Work stuck in early phases → unclear requirements or analysis paralysis
- Work stuck in decompose → team struggling to understand work
- Work jumping to active → skipping preparation
- "Completed" work reopening → exit criteria not met

## Additional Resources

For detailed phase flow:
- **`references/phase-flow.md`** - Complete phase documentation with all transition rules
