# Feature Development Pattern

The standard pattern for feature work from vision alignment to completion.

## When to Use

- New feature requests
- Enhancements to existing features
- User-facing changes
- Most "normal" development work

## The Pattern

### 1. Capture the Request

Feature requests come from many sources:
- User feedback
- Stakeholder requests
- Market requirements
- Technical opportunities

Initial capture as backlog item:
```
create_document(
  type="task",
  title="Add export to CSV functionality",
  backlog_category="feature"
)
```

This gets it out of someone's head and into the system.

### 2. Evaluate Alignment

Before promoting to initiative, check:
- Does this align with the vision?
- Does this support current strategic direction?
- Is this the right time?

If yes, create an initiative. If no, it stays in backlog (or gets archived).

### 3. Create the Initiative

```
create_document(
  type="initiative",
  title="Data Export Feature",
  parent="VISION-ID"  # or STRATEGY-ID if using full preset
)
```

**Important**: After creation, immediately read and fill in the document:
```
read_document(short_code)    # See the template
edit_document(short_code, ...) # Fill in Context, Goals, etc.
edit_document(short_code, ...) # Remove unused optional sections
```

Never leave template placeholders. The initiative should have real content describing what's being built and why before moving on.

The feature request backlog item can be archived or kept as reference.

### 4. Discovery Phase

Understand the problem before solving it:
- Who needs this? Why?
- What do they do today?
- What's the actual job to be done?
- What are the constraints?

**Exit criteria:**
- Problem statement is clear
- User needs are understood
- Scope is bounded

### 5. Design Phase

Define the solution:
- How will it work?
- What's the user experience?
- What's the technical approach?
- What are the dependencies?

**Exit criteria:**
- Solution approach documented
- Technical risks identified
- Dependencies mapped
- Stakeholders aligned

### 6. Decompose Phase

Break into tasks:
```
Initiative: "Data Export Feature"
├── Task: "Design export UI component"
├── Task: "Implement CSV export backend"
├── Task: "Add Excel export format"
├── Task: "Add export progress indicator"
├── Task: "Write export documentation"
└── Task: "Add export analytics tracking"
```

Each task should be:
- Independently valuable (or clearly ordered)
- Clearly scoped
- Has acceptance criteria

### 7. Execute

Pull tasks as capacity allows:
```
transition_phase(PROJ-T-0042) # todo -> active
# ... do the work ...
transition_phase(PROJ-T-0042) # active -> completed
```

As tasks complete, the initiative progresses.

### 8. Complete the Initiative

When all tasks are done and the capability is delivered:
```
transition_phase(PROJ-I-0005) # active -> completed
```

## Feature Sizing

Not all features are initiatives:

| Size | Right Level | Characteristics |
|------|-------------|-----------------|
| Small | Task (backlog) | Single change, no design needed |
| Medium | Initiative | Multiple tasks, needs discovery/design |
| Large | Multiple initiatives | Multiple capability increments |

**Small example**: "Add tooltip to button" - just a task
**Medium example**: "Add data export" - an initiative
**Large example**: "Add reporting module" - probably multiple initiatives

## Handling Feature Creep

During development, new requirements emerge. Handle them:

### If small and related:
Add a task to the current initiative.

### If significant:
Create a new backlog item. Evaluate later.
```
create_document(
  type="task",
  title="Add scheduled exports",
  backlog_category="feature"
)
```

### If it changes the initiative scope:
Stop. Re-evaluate. Either:
- Adjust the initiative scope (and update design)
- Split into multiple initiatives
- Defer the new scope

Don't silently expand scope. Make it visible.

## Cross-Cutting Features

Some features touch multiple areas:

**Option 1: Single initiative with cross-cutting tasks**
```
Initiative: "Search functionality"
├── Task: "Search UI component"
├── Task: "Search API endpoint"
├── Task: "Search indexing backend"
└── Task: "Search analytics"
```

**Option 2: Multiple coordinated initiatives**
```
Initiative: "Search frontend"
Initiative: "Search backend"
Initiative: "Search infrastructure"
```

Use multiple initiatives when:
- Different teams own different parts
- Parts can ship independently
- Significant coordination overhead

## Common Mistakes

- **Skipping discovery**: Building the wrong thing well is still wrong.
- **Scope creep acceptance**: "While we're in there..." kills timelines.
- **No acceptance criteria**: "We'll know it when we see it" leads to churn.
- **Gold-plating**: Build what's needed, not what's cool.
- **Forgetting documentation**: User-facing features need user-facing docs.
