---
id: adr-005-strategy-document-template
level: adr
status: decided
created_at: 2025-07-02T15:50:00Z
updated_at: 2025-07-02T15:50:00Z
parent: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: true
decision_maker: team
supersedes: 
---

# ADR-005: Strategy Document Template

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need standardized templates for Strategy documents that define problems worth solving to advance the vision. Strategies must provide clear problem statements, success metrics, and high-level approaches without diving into implementation details.

## Decision

We will use **standardized Strategy document templates** stored in the `strategies/` directory.

**File Location**: `strategies/{strategy-slug}.md`

**Frontmatter Schema**:
```yaml
---
id: strategy-{slug}
level: strategy
status: shaping
created_at: 2025-07-02T15:50:00Z
updated_at: 2025-07-02T15:50:00Z
parent: "[[Vision]]"
blocked_by: 
  - "[[Document Name]]"
phase: shaping
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
success_metrics: []
risk_level: low
stakeholders: []
review_date: 2025-12-31
---
```

**Phase Flow**: `shaping → design → ready → active → complete`

**Phase Values & Tags**:
- `shaping` - Initial problem exploration and approach investigation
  - Tag: `#phase/shaping`
  - Status: Initial problem definition, exploring solution approaches
- `design` - Solution design and validation with stakeholders
  - Tag: `#phase/design` 
  - Status: Detailed solution approach documented and validated
- `ready` - All prerequisites complete, ready for implementation
  - Tag: `#phase/ready`
  - Status: Exit criteria met, resources allocated, dependencies resolved
- `active` - Implementation in progress across multiple initiatives
  - Tag: `#phase/active`
  - Status: Active execution with regular progress updates
- `complete` - Strategy fully implemented and success metrics achieved
  - Tag: `#phase/complete`
  - Status: All success metrics met, strategy goals achieved

**Required Content Sections**:
1. **Problem Statement** - What problem and why it matters
2. **Success Metrics** - Measurable outcomes
3. **Solution Approach** - High-level approach without implementation
4. **Scope** - In/Out boundaries
5. **Risks & Unknowns** - Major risks identified
6. **Exit Criteria** - Phase transition requirements
7. **Implementation Dependencies** - Critical path description
8. **Change Log** - Evolution tracking with rationale

**Template**:
```markdown
# {Title} Strategy

## Problem Statement
{1-2 paragraphs describing the problem and why it matters}

## Success Metrics
- {Measurable outcome 1}
- {Measurable outcome 2}

## Solution Approach
{High-level approach without implementation details}

## Scope
**In Scope:**
- {What we will address}

**Out of Scope:**
- {What we won't address}

## Risks & Unknowns
- {Major risk or unknown 1}
- {Major risk or unknown 2}

## Exit Criteria
- [ ] Problem statement is clear and agreed upon
- [ ] Success metrics are measurable and defined
- [ ] Solution approach is sketched at high level
- [ ] Scope boundaries are documented and validated
- [ ] Major risks are identified and assessed

## Implementation Dependencies
{Describe the critical path and initiative dependencies}

## Change Log

### 2025-07-02 - Initial Strategy
- **Change**: Created initial strategy document
- **Rationale**: Identified key problem requiring strategic approach
- **Impact**: Baseline established for strategic direction
- **Next Review**: 2025-12-31
```

**Process Considerations**:
- Strategies must reference Vision as parent to ensure alignment
- Risk level helps prioritize attention and resources
- Success metrics must be measurable to enable objective evaluation
- Scope boundaries prevent feature creep during execution

## Consequences

**Positive:**
- Clear problem-solution mapping ensures strategic focus
- Success metrics enable objective strategy evaluation
- Scope boundaries prevent scope creep
- Risk assessment enables proactive mitigation
- Standardized structure improves communication

**Negative:**
- High-level approach may be too abstract for some teams
- Success metrics definition can be challenging
- Scope boundary discussions can be contentious
- Risk assessment requires domain expertise

## Validation

We'll know this was right if:
- Strategies clearly trace back to vision advancement
- Success metrics are consistently defined and measurable
- Scope boundaries effectively prevent scope creep
- Risk assessments prove valuable for planning