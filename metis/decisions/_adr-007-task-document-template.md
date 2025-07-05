---
id: adr-007-task-document-template
level: adr
status: decided
created_at: 2025-07-02T16:00:00Z
updated_at: 2025-07-02T16:00:00Z
parent: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: true
decision_maker: team
supersedes: 
---

# ADR-007: Task Document Template

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need standardized templates for Task documents that represent specific, actionable work items. Tasks are the execution layer where actual implementation happens, requiring clear objectives and acceptance criteria.

## Decision

We will use **standardized Task document templates** stored in the `tasks/` directory.

**File Location**: `tasks/{task-slug}.md`

**Frontmatter Schema**:
```yaml
---
id: task-{slug}
level: task
status: todo
created_at: 2025-07-02T16:00:00Z
updated_at: 2025-07-02T16:00:00Z
parent: "[[Initiative Name]]"
blocked_by: 
  - "[[Document Name]]"
phase: todo
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
assignee: 
estimated_hours: 
pr_links: []
---
```

**Phase Flow**: `todo → doing → complete`

**Phase Values & Tags**:
- `todo` - Task defined and ready for assignment
  - Tag: `#phase/todo`
  - Status: Clear objective and acceptance criteria defined
- `doing` - Task actively being worked on
  - Tag: `#phase/doing`
  - Status: Assigned and in progress with regular updates
- `complete` - Task finished and acceptance criteria met
  - Tag: `#phase/complete`
  - Status: All acceptance criteria satisfied and verified

**Required Content Sections**:
1. **Parent Initiative** - Reference to parent initiative
2. **Objective** - What needs to be accomplished
3. **Acceptance Criteria** - Definition of done
4. **Implementation Notes** - Technical details and findings (added during work)
5. **Status Updates** - Progress tracking (added during work)

**Template**:
```markdown
# {Task Description}

## Parent Initiative
[[{initiative-name}]]

## Objective
{What needs to be done}

## Acceptance Criteria
- [ ] Specific outcome 1
- [ ] Specific outcome 2

## Implementation Notes
{Technical details, approach, findings during implementation}

## Status Updates
{Date} - {What was done}
{Date} - {Blockers or changes}
```

**Process Considerations**:
- Tasks are created during parent initiative's Decompose phase
- Simple kanban-style phase flow (todo → doing → complete)
- Acceptance criteria serve as exit criteria for completion
- Implementation Notes capture learning and decisions made during execution
- Status Updates provide progress visibility without heavyweight tracking

## Consequences

**Positive:**
- Simple, familiar workflow (kanban-style)
- Clear objectives and acceptance criteria prevent scope creep
- Implementation notes capture valuable learning
- Direct parent relationship maintains traceability
- Lightweight structure reduces overhead for execution work

**Negative:**
- Simple structure may be insufficient for complex tasks
- Manual status tracking requires discipline
- No built-in time tracking or estimation validation
- Limited workflow compared to full project management tools

## Validation

We'll know this was right if:
- Tasks provide clear, actionable work items
- Acceptance criteria effectively define completion
- Implementation notes capture valuable insights
- Task completion directly advances parent initiatives