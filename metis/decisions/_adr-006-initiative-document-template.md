---
id: adr-006-initiative-document-template
level: adr
status: decided
created_at: 2025-07-02T15:55:00Z
updated_at: 2025-07-02T15:55:00Z
parent: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: true
decision_maker: team
supersedes: 
---

# ADR-006: Initiative Document Template

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need standardized templates for Initiative documents that transform strategies into implementable designs. Initiatives bridge the gap between strategic intent and executable tasks through detailed technical specifications.

## Decision

We will use **standardized Initiative document templates** stored in the `initiatives/` directory.

**File Location**: `initiatives/{initiative-slug}.md`

**Frontmatter Schema**:
```yaml
---
id: initiative-{slug}
level: initiative
status: discovery
created_at: 2025-07-02T15:55:00Z
updated_at: 2025-07-02T15:55:00Z
parent: "[[Strategy Name]]"
blocked_by: 
  - "[[Document Name]]"
phase: discovery
tags:
  - "#initiative"
  - "#phase/discovery"
exit_criteria_met: false
technical_lead: 
estimated_complexity: m
related_adrs: 
  - "[[ADR-001: Document Format]]"
---
```

**Phase Flow**: `discovery → design → ready → decompose → active → complete`

**Phase Values & Tags**:
- `discovery` - Requirements exploration and initial research
  - Tag: `#phase/discovery`
  - Status: Understanding problem space and constraints
- `design` - Technical specifications and architecture planning
  - Tag: `#phase/design`
  - Status: Detailed technical design and validation
- `ready` - Design complete and ready for task decomposition
  - Tag: `#phase/ready`
  - Status: Exit criteria met, awaiting task breakdown
- `decompose` - Breaking initiative into executable tasks
  - Tag: `#phase/decompose`
  - Status: Creating specific task documents for implementation
- `active` - Tasks in progress, coordinating implementation
  - Tag: `#phase/active`
  - Status: Managing task execution and progress tracking
- `complete` - All tasks done and initiative goals achieved
  - Tag: `#phase/complete`
  - Status: Implementation complete and validated

**Required Content Sections**:
1. **Context** - Parent strategy reference and role description
2. **Goals & Non-Goals** - Specific objectives and boundaries
3. **Detailed Design** - Technical specifications, APIs, data models
4. **Alternatives Considered** - Options evaluated and rejected
5. **Implementation Plan** - Major milestones and approach
6. **Testing Strategy** - Validation approach
7. **Exit Criteria** - Phase transition requirements
8. **Tasks** - Added during Decompose phase

**Process Considerations**:
- Discovery phase allows requirements exploration before design commitment
- Decompose phase explicitly separates design completion from task creation
- Related ADRs track technical decisions made during initiative work
- Estimated complexity helps with resource planning and prioritization
- Technical lead assignment ensures ownership and accountability

## Consequences

**Positive:**
- Clear bridge between strategy and implementation
- Detailed design prevents implementation ambiguity
- Alternatives section documents decision rationale
- Testing strategy ensures quality planning
- Phase separation allows proper design validation

**Negative:**
- Detailed design requires significant upfront investment
- Technical specifications may become outdated
- Alternatives documentation can be time-consuming
- Testing strategy may be premature during early phases

## Validation

We'll know this was right if:
- Initiatives clearly implement their parent strategies
- Technical designs provide sufficient implementation guidance
- Alternative analysis improves decision quality
- Testing strategies result in higher quality deliverables