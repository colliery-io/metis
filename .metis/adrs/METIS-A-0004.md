---
id: 004-frontmatter-metadata-system
level: adr
title: "Frontmatter Metadata System"
number: 4
short_code: "METIS-A-0004"
created_at: 2025-07-03T13:10:00Z
updated_at: 2025-07-03T13:10:00Z
decision_date: 2025-07-03
decision_maker: Engineering Team
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"

exit_criteria_met: true
strategy_id: NULL
initiative_id: NULL
---

# ADR-004: Frontmatter Metadata System

**Status**: Decided  
**Date**: 2025-07-03  
**Driver**: Engineering Team

## Context **[REQUIRED]**

We need a consistent system for tracking document metadata across all Metis document types. This metadata drives:
- Document lifecycle management (phases, status)
- Relationships between documents (parent-child)
- Search and filtering capabilities
- Workflow automation
- Progress tracking

Previous ADRs (004-008) were overly prescriptive about document content structure. This ADR focuses solely on the metadata system that all documents use.

## Decision **[REQUIRED]**

All Metis documents use YAML frontmatter to track metadata. Fields are categorized as:
1. **Core fields** - Required for all documents
2. **Document-type fields** - Required for specific document types
3. **Optional fields** - Additional metadata as needed

### Core Fields (All Documents)

```yaml
id: {type}-{slug}                    # Unique identifier
level: {vision|strategy|initiative|task|adr}  # Document type
status: {status}                     # Current status
created_at: {ISO-8601}              # Creation timestamp
updated_at: {ISO-8601}              # Last update timestamp
parent: "[[{parent-title}]]"        # Parent document link (optional for vision)
blocked_by:                         # List of blocking dependencies
  - "[[{blocker-title}]]"
phase: {phase}                      # Current phase
tags:                               # Categorization tags
  - "#{level}"
  - "#phase/{phase}"
exit_criteria_met: {true|false}     # Whether exit criteria are complete
```

### Comment/Uncomment Pattern for Phases

Templates include all valid phases/statuses as comments. Users uncomment the current state:

```yaml
# Phase progression for initiatives
# phase: discovery
# phase: design
phase: ready
# phase: decompose
# phase: active
# phase: completed

tags:
  - "#initiative"
  # - "#phase/discovery"
  # - "#phase/design"
  - "#phase/ready"
  # - "#phase/decompose"
  # - "#phase/active"
  # - "#phase/completed"
```

### Document-Type Specific Fields

**Vision:**
- No additional required fields (minimal metadata)

**Strategy:**
- `success_metrics: []` - Measurable outcomes
- `risk_level: {low|medium|high|critical}` - Risk assessment
- `stakeholders: []` - Key stakeholders
- `review_date: {ISO-8601}` - Next review date

**Initiative:**
- `technical_lead: {name}` - Responsible person
- `estimated_complexity: {s|m|l|xl}` - Size estimate
- `related_adrs: []` - Design decisions

**Task:**
- `assignee: {name}` - Person responsible
- `estimated_hours: {number}` - Time estimate
- `pr_links: []` - Related pull requests

**ADR:**
- `decision_date: {ISO-8601}` - When decided
- `decision_maker: {name}` - Who made decision
- `superseded_by: "[[{adr-title}]]"` - If replaced

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

**YAML Frontmatter**
- Pros: Standard, readable, tooling support
- Cons: YAML syntax errors
- Risk Level: Low
- Implementation Cost: Low

**JSON Frontmatter**
- Pros: Simple parsing, no syntax issues
- Cons: Less human-readable
- Risk Level: Medium
- Implementation Cost: Low

**Custom Format**
- Pros: Perfect fit for needs
- Cons: No tooling support
- Risk Level: High
- Implementation Cost: High

## Rationale **[REQUIRED]**

YAML frontmatter provides the best balance of human readability, machine parseability, and tooling ecosystem support. The comment/uncomment pattern for phases provides self-documenting templates that prevent invalid states.

## Consequences **[REQUIRED]**

### Positive
- Consistent metadata across all documents
- Clear progression paths with self-documenting templates
- Machine-readable for automation
- Standard YAML format understood by many tools
- Enables powerful queries and filters

### Negative
- Frontmatter can become verbose
- YAML syntax errors break parsing
- Comment/uncomment requires manual editing
- Some redundancy (level appears in id and as field)

### Neutral
- Requires tooling to validate frontmatter consistency
- Templates must be kept in sync with this specification
- May need migration tools for schema changes

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- Significant YAML parsing issues arise
- Need for different metadata structures emerges
- Integration problems with tooling ecosystem

### Scheduled Review
- **Next Review Date**: 2026-07-03
- **Review Criteria**: Metadata system effectiveness, tooling compatibility, team adoption
- **Sunset Date**: N/A (foundational system decision)