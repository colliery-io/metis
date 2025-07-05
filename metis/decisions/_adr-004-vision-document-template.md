---
id: adr-004-vision-document-template
level: adr
status: decided
created_at: 2025-07-02T15:45:00Z
updated_at: 2025-07-02T15:45:00Z
parent: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: true
decision_maker: team
supersedes: 
---

# ADR-004: Vision Document Template

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need a standardized template for Vision documents that serves as the singular north star for each project. The Vision must be stable, inspiring, and provide clear guidance for all strategic decisions.

## Decision

We will use a **singular Vision document** with fixed naming and specific frontmatter schema.

**File Location**: `vision.md` (fixed filename in project root)

**Frontmatter Schema**:
```yaml
---
id: vision
level: vision
status: draft
created_at: 2025-07-02T15:45:00Z
updated_at: 2025-07-02T15:45:00Z
phase: draft
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
core_values: []
stakeholders: []
---
```

**Phase Flow**: `draft → review → published`

**Phase Values & Tags**:
- `draft` - Initial vision creation and development
  - Tag: `#phase/draft`
  - Status: Vision being written and refined internally
- `review` - Stakeholder review and feedback incorporation
  - Tag: `#phase/review`
  - Status: Under stakeholder review, gathering feedback and consensus
- `published` - Final approved vision guiding all project decisions
  - Tag: `#phase/published`
  - Status: Approved and active as project north star

**Required Content Sections**:
1. **Purpose** - Why this project exists
2. **Core Values** - Principles that guide decisions
3. **Long-term Vision** - 3-5 year aspirational state
4. **Success Definition** - How we measure achievement
5. **Principles** - Operational guidelines
6. **Exit Criteria** - Checkboxes for phase transitions

**Template**:
```markdown
# {Project Name} Vision

## Purpose
{Why this project exists - the fundamental problem it solves}

## Core Values
- {Value 1}: {Description of what this means for decisions}
- {Value 2}: {Description of what this means for decisions}

## Long-term Vision
{Where we want to be in 3-5 years}

## Success Definition
{How we'll know when we've achieved our vision}

## Principles
- {Principle 1}: {How this guides our work}
- {Principle 2}: {How this guides our work}

## Exit Criteria
- [ ] Purpose is clear and resonates with all stakeholders
- [ ] Core values are defined and actionable
- [ ] Long-term vision is inspiring and achievable
- [ ] Success definition is measurable
- [ ] Principles provide clear guidance for decisions
```

## Consequences

**Positive:**
- Single source of truth for project direction
- Fixed filename eliminates confusion about which document is "the vision"
- Stable structure provides consistency across projects
- Core values and principles enable decision-making guidance
- Phase flow ensures stakeholder alignment before publication

**Negative:**
- Fixed structure may not fit all project types
- Single document may become unwieldy for very large projects
- Consensus-building on vision can be time-consuming
- Changes to published vision create significant organizational impact

## Validation

We'll know this was right if:
- Teams consistently reference the vision in strategic decisions
- Vision provides clear guidance for strategy prioritization
- Vision remains stable and doesn't require frequent changes
- Stakeholders can easily understand and communicate project purpose