---
name: project-patterns
description: This skill should be used when the user asks to "start a new project", "greenfield project", "tech debt campaign", "incident response", "feature development", "which preset", "initialize metis", "set up project", or needs guidance on project setup, choosing presets, and applying patterns for different work types.
---

# Project Patterns

This skill guides project setup and provides patterns for different types of work.

## Choosing a Preset

| Team Size | Coordination | Recommended Preset |
|-----------|--------------|-------------------|
| Solo | None | Direct |
| 2-5 | Informal | Streamlined |
| 5+ | Formal | Full (or Streamlined) |

**Preset differences:**
- **Direct**: Vision → Task (simplest, solo work)
- **Streamlined**: Vision → Initiative → Task (most common, **default**)
- **Full**: Vision → Strategy → Initiative → Task (multi-team coordination)

When in doubt, use the default Streamlined preset. Add Strategy layer later if needed.

## Greenfield Projects

Starting a new project from scratch.

### When to Use
- New product or system
- Major rewrite (treat as new)
- Proof of concept → production
- New team forming

### The Pattern

**1. Initialize:**
```
initialize_project(project_path="/path/to/project", prefix="PROJ")
```
Choose short (2-8 char, uppercase), memorable, unique prefix.

**2. Define Vision:**
Create vision answering: "Why does this project exist?"
- Objectives: What outcomes?
- Values: Why does it matter?
- Success criteria: How to know we succeeded?

Transition: draft → review → published

**3. Create Initial Initiatives:**
Common greenfield initiatives:
- **Foundation/Setup**: Dev environment, CI/CD, architecture
- **Core Feature**: Main thing this project does
- **Integration**: Connecting to other systems
- **Release/Launch**: Getting to production

Don't create all initiatives upfront. Create enough to start, pull more as backlogs empty.

**4. Decompose and Execute:**
For each initiative: discovery → design → decompose → execute

### Greenfield Tips
- **Start small**: Maximum uncertainty, learn as you go
- **Foundation first**: But don't gold-plate it
- **Expect pivots**: Early initiatives may invalidate later plans
- **ADRs from day one**: Capture decisions while context fresh

## Tech Debt Campaigns

Systematic debt reduction.

### When to Use
- Accumulated technical debt affecting velocity
- Scheduled "debt sprints"
- Pre-migration cleanup
- Quality improvement initiatives

### The Pattern

**1. Inventory:** Create backlog items for each debt item:
```
create_document(type="task", title="Refactor X module", backlog_category="tech-debt")
```

**2. Prioritize:** Group related debt into initiative when ready to tackle:
```
create_document(type="initiative", title="API Cleanup Campaign", parent_id="PROJ-V-0001")
```

**3. Execute:** Pull debt tasks into initiative, work systematically

### Tech Debt Tips
- Don't boil ocean: Focused campaigns > everything at once
- Tie to value: Why does fixing this matter?
- Measure impact: Before/after metrics validate effort
- Prevent recurrence: ADRs for decisions that caused debt

## Incident Response

Handling urgent, unplanned work.

### When to Use
- Production incidents
- Critical bugs
- Security vulnerabilities
- Customer escalations

### The Pattern

**1. Immediate:** Create backlog item for tracking:
```
create_document(type="task", title="INCIDENT: Service X down", backlog_category="bug")
```

**2. Triage:** Work the incident, update task with findings

**3. Resolution:** Complete immediate fix, transition task to completed

**4. Follow-up:** For systemic fixes, create initiative:
```
create_document(type="initiative", title="Prevent X recurrence", parent_id="PROJ-V-0001")
```

**5. Postmortem:** Create ADR if architectural decisions made

### Incident Tips
- Track immediately: Even if "just a quick fix"
- Separate fix from prevention: Quick fix now, proper fix later
- Don't skip follow-up: Initiatives for systemic improvements
- Document decisions: ADRs capture why you chose approach

## Feature Development

Standard feature flow.

### When to Use
- New features planned in roadmap
- Enhancements to existing features
- Customer-requested functionality

### The Pattern

**1. Initiative:** Create initiative for the feature:
```
create_document(type="initiative", title="User Dashboard", parent_id="PROJ-V-0001", complexity="m")
```

**2. Discovery:** Understand requirements, constraints, users

**3. Design:** Define solution approach, create ADRs for decisions

**4. Decompose:** Break into tasks (prefer vertical slices)

**5. Execute:** Pull tasks, implement, test

**6. Complete:** Verify acceptance criteria, transition to completed

### Feature Tips
- Discovery matters: Understand before designing
- Vertical slices: User-visible increments over technical layers
- Exit criteria: Define "done" clearly
- Iterate: Ship incrementally, get feedback

## Anti-Patterns to Avoid

| Anti-Pattern | Problem | Fix |
|--------------|---------|-----|
| **Shadow work** | Work outside system | Track everything; create backlog items retroactively |
| **Shadow backlogs** | Secret lists elsewhere | Capture in Metis; trust or fix the system |
| **Too many active** | Context switching | Limit WIP; finish before starting |
| **Orphaned work** | No value alignment | Connect to parent or move to backlog |
| **Skipping phases** | Problems found late | Respect exit criteria |
| **Premature decomposition** | Wrong tasks | Stay in discovery/design first |
| **Stale work** | Clutter | Archive unused items regularly |
| **Wrong granularity** | Confused tracking | Apply scope heuristics |

## Core Principles

- **Work is pulled, never pushed**: Low backlog signals to look up
- **All work traces to vision**: If it doesn't align, question value
- **Phases exist for a reason**: Don't skip them
- **Filesystem is truth**: Database is cache
- **Scope over time**: Size by capability, not duration

## Additional Resources

For detailed patterns and anti-patterns:
- **`references/greenfield.md`** - Complete greenfield guide
- **`references/tech-debt.md`** - Debt campaign patterns
- **`references/incident-response.md`** - Incident handling
- **`references/feature-development.md`** - Feature flow details
- **`references/anti-patterns.md`** - Full anti-pattern catalog
- **`references/preset-selection.md`** - Preset decision guide
