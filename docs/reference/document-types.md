# Document Types Reference

Metis supports five document types, each with a defined structure, phase lifecycle, and role in the Flight Levels hierarchy.

## Short Code Format

Every document gets a unique short code: `PREFIX-TYPE-NNNN`

| Type Code | Document Type |
|-----------|---------------|
| `V` | Vision |
| `I` | Initiative |
| `T` | Task |
| `A` | ADR |
| `S` | Specification |

Examples: `PROJ-V-0001`, `ACME-I-0003`, `MFP-T-0042`

The prefix is set during `metis init` (2-8 uppercase ASCII letters). The counter auto-increments per type.

---

## Vision

**Purpose:** Define strategic direction and high-level goals (6-month to 2-year horizon).

**Hierarchy:** Top-level document. Every project has exactly one vision.

**Phases:** `draft → review → published`

**Parent:** None (visions cannot have parents)

**Can be blocked:** No

**File location:** `.metis/vision.md`

### Frontmatter Fields

```yaml
---
id: my-project-vision
level: vision
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: null
blocked_by: []
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
short_code: "PROJ-V-0001"
archived: false
initiative_id: null
---
```

### Content Template

```markdown
# {Title} Vision

## Purpose
{Why this vision exists}

## Current State
{Baseline — where things are now}

## Future State
{Target — where things should be}

## Success Criteria
{How success is measured}

## Principles
{Core decision-making principles}

## Constraints
{Known limitations and boundaries}
```

### Acceptance Criteria

Vision exit criteria validate that the strategic direction is clear, communicated, and actionable.

---

## Initiative

**Purpose:** Concrete implementation project aligned with the vision (1-6 month horizon).

**Hierarchy:** Child of a Vision.

**Phases:** `discovery → design → ready → decompose → active → completed`

**Parent:** Required (must reference a Vision)

**Can be blocked:** Yes (by other initiatives or tasks)

**File location:** `.metis/initiatives/PROJ-I-NNNN/initiative.md`

### Frontmatter Fields

```yaml
---
id: build-core-api
level: initiative
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: "[[my-project-vision]]"
blocked_by: []
tags:
  - "#initiative"
  - "#phase/discovery"
exit_criteria_met: false
short_code: "PROJ-I-0001"
archived: false
initiative_id: "build-core-api"
estimated_complexity: "M"
technical_lead: ""
related_adrs: []
---
```

### Complexity Levels

| Level | Label | Typical Duration |
|-------|-------|-----------------|
| `xs` | Extra Small | < 1 day |
| `s` | Small | 1-3 days |
| `m` | Medium | 1-2 weeks |
| `l` | Large | 2-4 weeks |
| `xl` | Extra Large | 1+ months |

### Content Template

```markdown
# {Title} Initiative

## Context
{Background and rationale for this initiative}

## Goals & Non-Goals
{Explicit objectives and exclusions}

## Detailed Design
{Technical approach and implementation details}

## Alternatives Considered
{Rejected approaches and reasoning}

## Implementation Plan
{Phases and timeline}

## Testing Strategy
{Validation approach}
```

---

## Task

**Purpose:** Actionable, concrete piece of work (1-14 day horizon).

**Hierarchy:** Child of an Initiative (streamlined preset) or Vision (direct preset). Can also be a backlog item with no parent.

**Phases:** `todo → active → completed` (with `blocked` as alternate state; backlog items start at `backlog`)

**Parent:** Required in streamlined preset (references an Initiative). Optional in direct preset.

**Can be blocked:** Yes (by other tasks)

**File location:**
- Under initiative: `.metis/initiatives/PROJ-I-NNNN/tasks/PROJ-T-NNNN.md`
- Backlog: `.metis/backlog/PROJ-T-NNNN.md` (or `.metis/backlog/bugs/`, `.metis/backlog/features/`, `.metis/backlog/tech-debt/` for categorized items)

### Frontmatter Fields

```yaml
---
id: implement-authentication
level: task
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: "[[build-core-api]]"
blocked_by: []
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
short_code: "PROJ-T-0001"
archived: false
initiative_id: "build-core-api"
---
```

Backlog items include a category tag:

```yaml
tags:
  - "#task"
  - "#phase/backlog"
  - "#bug"          # or "#feature" or "#tech-debt"
```

### Content Template

```markdown
# {Title}

## Parent Initiative
[[{parent_title}]]

## Objective
{Clear statement of what this task accomplishes}

## Acceptance Criteria
- [ ] Requirement 1
- [ ] Requirement 2

## Implementation Notes
{Technical details and approach}

## Status Updates
{Filled in during implementation — progress, findings, decisions}
```

### Bidirectional Transitions

Tasks support moving back from `blocked`:
- `blocked → todo`
- `blocked → active`

---

## ADR (Architecture Decision Record)

**Purpose:** Document architectural decisions and their rationale.

**Hierarchy:** Standalone (no parent required). Can optionally reference the work that prompted the decision.

**Phases:** `draft → discussion → decided → superseded`

**Parent:** Optional

**Can be blocked:** No

**File location:** `.metis/adrs/PROJ-A-NNNN.md`

### Frontmatter Fields

```yaml
---
id: "001-use-postgresql"
level: adr
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: null
blocked_by: []
tags:
  - "#adr"
  - "#phase/draft"
exit_criteria_met: false
short_code: "PROJ-A-0001"
archived: false
initiative_id: null
decision_maker: ""
decision_date: null
---
```

### Content Template

```markdown
# ADR-{number}: {Title}

## Context
{Problem or issue motivating the decision}

## Decision
{What change is being proposed}

## Rationale
{Why this option was chosen}

## Consequences

### Positive
{Benefits}

### Negative
{Costs and drawbacks}

### Neutral
{Other consequences}
```

---

## Specification

**Purpose:** Capture system-level or project-level design — PRDs, requirements, architecture framing. Specifications are "living documents" that remain editable even after publishing.

**Hierarchy:** Attached to a Vision or Initiative (not a hierarchy node — specifications don't have children).

**Phases:** `discovery → drafting → review → published`

**Parent:** Required (Vision or Initiative)

**Can be blocked:** No

**File location:** `.metis/specifications/PROJ-S-NNNN/specification.md`

### Frontmatter Fields

```yaml
---
id: api-contract-v2
level: specification
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: "[[my-project-vision]]"
blocked_by: []
tags:
  - "#specification"
  - "#phase/discovery"
exit_criteria_met: false
short_code: "PROJ-S-0001"
archived: false
initiative_id: null
---
```

### Content Template

```markdown
# {Title} Specification

## System Overview
{High-level description}

## Requirements
{Functional and non-functional requirements}

## Architecture
{System design and components}

## Data Model
{Entities and relationships}

## APIs
{Interfaces and contracts}

## Security Considerations
{Authentication, authorization, encryption}
```

---

## Document Availability by Preset

| Document Type | Streamlined | Direct |
|---------------|-------------|--------|
| Vision | Yes | Yes |
| Initiative | Yes | No |
| Task | Yes (parent: Initiative) | Yes (parent: Vision or none) |
| ADR | Yes | Yes |
| Specification | Yes | Yes |

ADRs and Specifications are always available regardless of preset.

---

## Template Customization

Templates follow a fallback chain:

1. **Project-level:** `.metis/templates/{type}/content.md`
2. **Global-level:** `~/.config/metis/templates/{type}/content.md`
3. **Embedded defaults:** Compiled into the binary

To customize a template, create the file at level 1 or 2. Templates use simple `{{ variable }}` substitution syntax.

Available template variables:

| Variable | Available In | Description |
|----------|-------------|-------------|
| `title` | All types | Document title |
| `slug` | All types | Title-derived slug |
| `short_code` | All types | Generated short code |
| `created_at` | All types | Creation timestamp |
| `updated_at` | All types | Last update timestamp |
| `parent_id` | All types | Parent document ID |
| `parent_title` | Task | Parent initiative title |
| `estimated_complexity` | Initiative | Complexity level |
| `initiative_id` | Initiative, Task | Lineage tracking ID |
| `number` | ADR | ADR number |
| `decision_maker` | ADR | Decision maker name |
| `decision_date` | ADR | Decision date |

You can also customize exit criteria templates using `exit_criteria.md` in the same fallback chain locations.

Custom templates are validated by rendering with sample data before use. If a template has syntax errors or references undefined variables, it is rejected with an error.
