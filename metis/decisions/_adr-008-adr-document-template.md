---
id: adr-008-adr-document-template
level: adr
status: decided
created_at: 2025-07-02T16:05:00Z
updated_at: 2025-07-02T16:05:00Z
parent: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: true
decision_maker: team
supersedes: 
---

# ADR-008: ADR Document Template

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need standardized templates for Architecture Decision Records (ADRs) that capture important decisions with their context and rationale. ADRs must handle superseding relationships and status updates while maintaining historical integrity.

## Decision

We will use **standardized ADR document templates** stored in the `decisions/` directory with support for decision lifecycle management.

**File Location**: `decisions/adr-{number}-{decision-slug}.md`

**Frontmatter Schema**:
```yaml
---
id: adr-{number}-{slug}
level: adr
status: draft
created_at: 2025-07-02T16:05:00Z
updated_at: 2025-07-02T16:05:00Z
parent: 
phase: draft
tags:
  - "#adr"
  - "#phase/draft"
exit_criteria_met: false
decision_maker: 
supersedes: 
  - "[[ADR-000: Previous Decision]]"
superseded_by: 
  - "[[ADR-010: Newer Decision]]"
---
```

**Phase Flow**: `draft → discussion → decided → superseded`

**Phase Values & Tags**:
- `draft` - Initial decision documentation and context gathering
  - Tag: `#phase/draft`
  - Status: Problem and proposed solution being documented
- `discussion` - Stakeholder review and alternative evaluation
  - Tag: `#phase/discussion`
  - Status: Under review, gathering input and considering alternatives
- `decided` - Final decision made and documented
  - Tag: `#phase/decided`
  - Status: Decision approved and ready for implementation
- `superseded` - Decision replaced by a newer ADR
  - Tag: `#phase/superseded`
  - Status: No longer active, superseded by referenced ADR

**Required Content Sections**:
1. **Context** - The problem motivating this decision
2. **Decision** - What we're choosing to do
3. **Consequences** - Positive and negative impacts
4. **Validation** - How we'll know if this was right

**Template**:
```markdown
# ADR-{number}: {Title}

**Status**: Draft  
**Date**: YYYY-MM-DD  
**Driver**: {Who is driving this decision}

## Context
{The issue motivating this decision}

## Decision
{The change we're making}

## Consequences
**Positive:**
- {Good thing 1}

**Negative:**
- {Trade-off 1}

## Validation
{How we'll know if this was right}
```

**Superseding Process**:
1. New ADR created with `supersedes: ["[[ADR-XXX]]"]` reference
2. When new ADR reaches "decided" status, update superseded ADR:
   - Change status from "decided" to "superseded"
   - Add `superseded_by: ["[[ADR-YYY]]"]` field
   - Add callout at top of content: `> [!info] Superseded by [[ADR-YYY: New Decision]]`
3. Original ADR content remains unchanged for historical reference

**Process Considerations**:
- ADRs are not blocked by other documents (created when ready to decide)
- Sequential numbering ensures clear ordering and reference
- Bidirectional superseding links maintain decision history
- Status updates preserve decision timeline
- Cross-cutting nature means no parent document required

## Consequences

**Positive:**
- Clear decision documentation with context and rationale
- Superseding process maintains historical integrity
- Bidirectional linking enables decision traceability
- Status updates track decision lifecycle
- Standardized format improves decision communication

**Negative:**
- Superseding process requires manual updates to multiple documents
- Decision numbering requires coordination
- Lightweight template may miss domain-specific considerations
- No built-in approval workflow for complex decisions

## Validation

We'll know this was right if:
- Teams consistently document significant decisions as ADRs
- Superseding relationships clearly track decision evolution
- Historical decisions remain accessible and understandable
- Decision rationale proves valuable for future reference