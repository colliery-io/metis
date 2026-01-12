# Anti-Patterns

Common mistakes when using Metis, how to recognize them, and how to recover.

## Shadow Work

**What it is**: Work happening outside the tracked system - side projects, favors, "quick fixes" that bypass the hierarchy.

**Why it's bad**:
- Breaks value alignment (work doesn't trace to vision)
- Hidden capacity drain
- Creates technical debt and coordination surprises
- Undermines the system's usefulness

**How to recognize**:
- "Oh, I also did this other thing..."
- Work completed that wasn't in any task
- Surprises in code reviews or deployments
- Team members "busy" but tracked work isn't moving

**How to fix**:
- If the work has value, create a backlog item for it retroactively
- If it's recurring, create an initiative or operational category
- If it has no value alignment, question whether it should be done
- Make it safe to surface shadow work without punishment

## Shadow Backlogs

**What it is**: Secret lists of work maintained outside Metis - personal notes, spreadsheets, Slack threads, "I'll remember this".

**Why it's bad**:
- Priorities aren't visible
- Work gets lost or duplicated
- Can't coordinate across the team
- Defeats the purpose of having a system

**How to recognize**:
- "I have a list of things I want to do..."
- Work appearing from nowhere
- Different people tracking the same items differently
- Metis backlog doesn't reflect actual planned work

**How to fix**:
- Capture everything in Metis, even if it's rough
- Use backlog items for ad-hoc work
- Review and prune regularly - it's okay to archive things
- Trust the system or fix the system, don't route around it

## Too Many Active Items

**What it is**: Multiple initiatives or tasks in active state simultaneously.

**Why it's bad**:
- Context switching kills productivity
- Nothing actually finishes
- WIP (work in progress) hides problems
- Creates false sense of progress

**How to recognize**:
- Many items "in progress" for extended periods
- Difficulty answering "what are you working on?"
- Things taking much longer than expected
- Constant context switching

**How to fix**:
- Limit WIP explicitly (e.g., max 2 active tasks per person)
- Finish before starting
- If something is blocked, mark it blocked rather than starting something new
- Focus on throughput (things completed) not busyness (things started)

## Orphaned Work

**What it is**: Tasks or initiatives that don't connect to anything above them.

**Why it's bad**:
- No value alignment - why is this being done?
- Can't prioritize against other work
- Often represents scope creep or pet projects

**How to recognize**:
- Tasks without parent initiatives
- Initiatives without parent strategies/visions (unless streamlined preset)
- Work that nobody can explain the purpose of

**How to fix**:
- Connect it to an existing initiative if it belongs
- Create a new initiative if it represents real value
- Move to backlog if it's operational/maintenance
- Archive or delete if it shouldn't exist

## Skipping Phases

**What it is**: Advancing work through phases without meeting exit criteria.

**Why it's bad**:
- Problems discovered late (more expensive to fix)
- Rework and thrashing
- Quality issues
- Undermines the phase system

**How to recognize**:
- Initiatives in "active" that were never in "design"
- Tasks "completed" that don't meet acceptance criteria
- Phases advanced just to show progress

**How to fix**:
- Respect exit criteria
- Use `force` only when you consciously accept the risk
- If a phase feels unnecessary, discuss removing it rather than skipping
- Track phase skips and their outcomes

## Premature Decomposition

**What it is**: Breaking work into tasks before understanding the problem or designing the solution.

**Why it's bad**:
- Tasks that solve the wrong problem
- Massive rework when design changes
- Wasted effort on abandoned approaches
- False sense of progress

**How to recognize**:
- Tasks created in discovery phase
- Tasks that keep changing scope
- "We'll figure it out as we go"
- Design happening during execution

**How to fix**:
- Stay in discovery until the problem is clear
- Stay in design until the approach is validated
- Use the decompose phase for its intended purpose
- Create spike/research tasks for unknowns rather than guessing

## Stale Work

**What it is**: Items sitting untouched for extended periods.

**Why it's bad**:
- Clutter obscures actual work
- Context is lost over time
- Creates false inventory
- Demoralizing to see undone work pile up

**How to recognize**:
- Items with old `updated_at` timestamps
- Backlog items that never get pulled
- Initiatives stuck in early phases indefinitely

**How to fix**:
- Regular grooming - use `archive_document` to remove stale items from active listings
- If something's been in backlog for months, question its value
- Set expectations: "We'll revisit this in X, or archive it"
- Don't let the backlog become a graveyard
- Archived items are preserved (can use `include_archived=true` to see them) but don't clutter daily work

## Wrong Granularity

**What it is**: Work items at the wrong level of abstraction.

**Why it's bad**:
- Tasks that are really initiatives (too big to track)
- Initiatives that are really tasks (overhead without benefit)
- Confused reporting and coordination

**How to recognize**:
- "Tasks" that have subtasks
- "Initiatives" completed in a day
- Inconsistent sizing across similar work

**How to fix**:
- Apply scope heuristics: Does it create a capability increment? Initiative. Atomic unit of work? Task.
- Refactor: split large tasks into initiatives, merge tiny initiatives into tasks
- Establish team norms for what belongs at each level

## Metric Gaming

**What it is**: Optimizing for visible metrics rather than actual value delivery.

**Why it's bad**:
- Completing many small tasks instead of important large ones
- Padding item counts
- Avoiding hard work that's harder to "complete"
- Corrupts the data

**How to recognize**:
- Lots of completed items but outcomes not improving
- Gaming acceptance criteria
- Splitting work unnecessarily to increase completion count
- Avoiding important work because it's hard to measure

**How to fix**:
- Focus on outcomes, not outputs
- Track initiative completion, not just task count
- Ask "did we achieve the vision?" not "did we close tickets?"
- Make gaming obvious and culturally unacceptable

## Recovery Mindset

When you spot an anti-pattern:

1. **Don't panic** - Anti-patterns are normal, especially early on
2. **Name it** - Acknowledge what's happening
3. **Understand why** - What caused this? Process gap? Cultural issue? Tooling problem?
4. **Fix forward** - Correct the immediate issue
5. **Prevent recurrence** - Address the root cause
6. **Share learning** - Help others avoid the same trap
