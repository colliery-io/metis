# Tech Debt Pattern

Running technical debt reduction campaigns with Metis.

## When to Use

- Accumulated code quality issues
- Outdated dependencies needing upgrades
- Architecture improvements
- Performance optimization campaigns
- Security hardening efforts

## Types of Tech Debt

### Backlog Items (Ad-Hoc)
Individual debt items captured as they're discovered:
```
create_document(type="task", title="Upgrade React to v18", backlog_category="tech-debt")
```

Use for:
- Small, isolated fixes
- Items discovered during other work
- Things to do "when we have time"

### Initiative (Campaigns)
Coordinated debt reduction efforts:
```
create_document(type="initiative", title="Q1 Performance Optimization", parent="VISION-ID")
```

Use for:
- Related debt items that should be tackled together
- Debt requiring discovery/design phases
- Debt with significant scope

## The Pattern

### 1. Capture Debt as Discovered

When you encounter debt during other work:
```
create_document(type="task", title="[DEBT] Refactor auth module", backlog_category="tech-debt")
```

Don't fix it immediately (unless trivial). Capture and move on.

### 2. Periodic Debt Review

Regularly review accumulated debt:
- What's the impact of each item?
- What's related to what?
- What's getting worse over time?

Group related items. Identify candidates for campaigns.

### 3. Create Debt Initiatives

For significant debt reduction:

```
create_document(
  type="initiative",
  title="Auth System Modernization",
  parent="VISION-ID"
)
```

The initiative goes through normal phases:
- **Discovery**: What's actually wrong? What's the impact?
- **Design**: What's the target state? Migration path?
- **Decompose**: What are the specific tasks?
- **Execute**: Do the work

### 4. Balance Debt with Features

Tech debt competes with feature work for capacity. Strategies:

**Dedicated allocation**: "20% of capacity goes to debt"
- Predictable
- May feel slow
- Good for steady-state

**Debt sprints**: "Every 4th sprint is debt-focused"
- Concentrated effort
- Can feel disruptive
- Good for big cleanup

**Embedded in features**: "Pay down debt in areas we're touching"
- Natural integration
- Can scope-creep features
- Good for targeted improvement

## Tech Debt Initiatives vs Feature Initiatives

| Aspect | Feature Initiative | Debt Initiative |
|--------|-------------------|-----------------|
| Value | New capability | Preserved/improved capability |
| Visibility | Users notice | Users don't notice (if done right) |
| Risk | Unknown unknowns | Usually known scope |
| Urgency | Market-driven | Risk-driven |

Debt initiatives need explicit justification because the value is less visible.

## Tracking Debt Health

### Indicators of Growing Debt
- Backlog items tagged `tech-debt` increasing over time
- Feature work taking longer due to code quality
- Developers complaining about specific areas
- Increasing bug rates in certain modules

### Indicators of Healthy Debt Management
- Debt items get pulled and completed
- No areas are "too scary to touch"
- New features don't create excessive new debt
- Debt initiatives complete successfully

## Example: Dependency Upgrade Campaign

```
Initiative: "Dependency Modernization Q1"
├── Task: "Audit current dependency versions"
├── Task: "Identify breaking changes in target versions"
├── Task: "Upgrade React ecosystem"
├── Task: "Upgrade testing libraries"
├── Task: "Upgrade build tooling"
└── Task: "Update CI/CD for new versions"
```

Each task is deployable. Order matters (audit first).

## Example: Performance Campaign

```
Initiative: "API Performance Optimization"
├── Task: "Profile slow endpoints"
├── Task: "Add caching layer"
├── Task: "Optimize database queries"
├── Task: "Add performance monitoring"
└── Task: "Document performance baseline"
```

Discovery (profiling) comes first. Other tasks depend on findings.

## Common Mistakes

- **Never prioritizing debt**: It compounds. Small regular investment beats emergency cleanup.
- **Boiling the ocean**: "Rewrite everything" initiatives fail. Scope tightly.
- **No acceptance criteria**: "Make it better" isn't testable. Define done.
- **Ignoring root causes**: If debt keeps appearing in the same areas, something systemic is wrong.
- **Gold-plating**: Not all debt needs to be paid. Some is acceptable. Prioritize impact.
