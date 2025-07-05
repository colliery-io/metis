---
id: adr-004-frontmatter-metadata-system
level: adr
status: decided
created_at: 2025-07-03T13:10:00Z
updated_at: 2025-07-03T13:10:00Z
decision_date: 2025-07-03
decision_maker: Engineering Team
parent: "[[Vision]]"
blocked_by: 
phase: decided
tags:
  - "#adr"
  - "#phase/decided"
exit_criteria_met: false
---

# ADR-010: Frontmatter Metadata System

## Status

Decided

## Context

We need a consistent system for tracking document metadata across all Metis document types. This metadata drives:
- Document lifecycle management (phases, status)
- Relationships between documents (parent-child)
- Search and filtering capabilities
- Workflow automation
- Progress tracking

Previous ADRs (004-008) were overly prescriptive about document content structure. This ADR focuses solely on the metadata system that all documents use.

## Decision

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

### Phase Definitions by Document Type

Each document type has specific valid phases:

- **Vision:** draft → review → published
- **Strategy:** shaping → design → ready → active → completed
- **Initiative:** discovery → design → ready → decompose → active → completed  
- **Task:** todo → doing → completed
- **ADR:** draft → discussion → decided → superseded

## Rationale

### Why frontmatter?
- Standard YAML format understood by many tools
- Separates metadata from content
- Machine-readable for automation
- Human-readable for manual editing

### Why comment/uncomment pattern?
- Self-documenting - shows all valid options
- Prevents invalid states
- Clear progression path
- No need to remember valid phases

### Why these specific fields?
- **id**: Enables unique references and linking
- **level**: Determines validation rules and workflows
- **phase/status**: Tracks progress and enables workflow
- **parent**: Maintains hierarchy
- **tags**: Enables filtering and organization
- **exit_criteria_met**: Gates phase transitions
- **timestamps**: Audit trail and freshness

## Consequences

### Positive
- Consistent metadata across all documents
- Clear progression paths
- Machine-readable for tooling
- Self-documenting templates
- Enables powerful queries and filters

### Negative
- Frontmatter can become verbose
- YAML syntax errors break parsing
- Comment/uncomment requires manual editing
- Some redundancy (level appears in id and as field)

### Neutral
- Requires tooling to validate frontmatter
- Templates must be kept in sync with this spec
- May need migration tools for schema changes

## Implementation

1. Templates include all fields with comments for options
2. Validation tools check required fields by document type
3. Phase transitions update phase field and corresponding tag
4. Parent links use Obsidian wiki-link format
5. Arrays use YAML list syntax
6. Dates use ISO-8601 format

## References

- Supersedes detailed content structure in ADRs 004-008
- [[Template Definition System]] - Task to implement this system
- [[Hierarchical Folder Structure]] - How documents are organized