---
id: 006-short-code-document-identification
level: adr
title: "Short Code Document Identification System"
number: 6
short_code: "METIS-A-0006"
created_at: 2025-10-16T01:58:55.024953+00:00
updated_at: 2025-10-16T01:58:55.024953+00:00
decision_date: 2025-10-16
decision_maker: team
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"

exit_criteria_met: true
strategy_id: NULL
initiative_id: NULL
supersedes: "[[METIS-A-0001]]"
---

# ADR-006: Short Code Document Identification System

**Status**: Decided\
**Date**: 2025-10-16\
**Driver**: Database Integration and MCP Server Development\
**Supersedes**: ADR-001: Document Format and Storage

## Context

The original slug-based naming system from ADR-001 created several issues as Metis evolved:

- **Inconsistent Identification**: Slugs were filesystem-only, requiring separate database keys
- **Manual Name Management**: Users had to create unique slugs manually, leading to conflicts
- **Poor Cross-Reference**: No standardized way to reference documents across interfaces
- **Database/Filesystem Mismatch**: Different identifiers in database vs. filesystem created complexity
- **MCP Integration Challenges**: AI assistants needed reliable, predictable document references
- **Parent/Path Construction:** It was difficult to parse and manage the variety of slugs that were showing up. 

As we increased our development and usage of the metis system we found consistent issues with a root cause in how we were generating, storing, and identifying documents via their slugified id. This system of assignment strikes a good balance of automated unique id generation and readability. 

## Decision

We will use **Short Codes** in the format `PREFIX-TYPE-NNNN` as the primary identification system for all Metis documents.

**Short Code Format:**

- **PREFIX**: Project identifier (e.g., "METIS", "PROJ", customizable per project)
- **TYPE**: Single letter for document type (V=Vision, S=Strategy, I=Initiative, T=Task, A=ADR)
- **NNNN**: Sequential four-digit number starting from 0001

**Examples:**

- `METIS-V-0001` (Metis project, Vision, document #1)
- `PROJ-I-0042` (Project, Initiative, document #42)
- `ACME-A-0003` (ACME project, ADR, document #3)

**Implementation:**

- Short codes serve as both filename and database primary key
- Automatic generation by system (no manual slug creation)
- Immutable once assigned (never reused even after document deletion)
- Sequential numbering within each document type
- Used in all cross-references and MCP tool operations

**File Organization:**

- Filename: `{SHORT_CODE}.md` (e.g., `METIS-A-0006.md`)
- Organized in type-based subdirectories (adrs/, visions/, initiatives/, tasks/)
- Short code also stored in YAML frontmatter for redundancy

## Alternatives Analysis

**Short Codes (PREFIX-TYPE-NNNN)**
- Pros: Unique, sequential, unified ID
- Cons: Less descriptive than slugs
- Risk Level: Low
- Implementation Cost: Medium

**Keep Slug System**
- Pros: Human-readable names
- Cons: Manual management, conflicts
- Risk Level: High
- Implementation Cost: Low

**UUID System**
- Pros: Globally unique, automatic
- Cons: Not human-readable
- Risk Level: Medium
- Implementation Cost: Medium

**Hybrid (Short Code + Slug)**
- Pros: Best of both worlds
- Cons: Complex, dual maintenance
- Risk Level: Medium
- Implementation Cost: High

## Rationale

Short codes provide the optimal balance of human readability and system reliability:

- **Predictable**: Sequential numbering makes references stable and discoverable
- **Unified**: Same identifier works across filesystem, database, and API layers
- **Automatic**: No manual naming decisions or conflict resolution needed
- **Compact**: Easy to reference in discussions and documentation
- **Sortable**: Natural ordering by creation sequence within each type
- **Future-proof**: Format supports growth and doesn't depend on content

This aligns with our principle of "Developer Experience First" by making document references simple and reliable across all interfaces.

## Consequences

### Positive

- Unified identification across all system layers (filesystem, database, MCP)
- Automatic conflict-free document creation
- Reliable cross-references that never break
- Clear sequential organization within document types
- Simplified MCP tool operations for AI assistants
- Easy to reference in conversations and documentation
- Database and filesystem stay synchronized automatically

### Negative

- Less descriptive than human-readable slugs
- Requires looking up short code to identify document content
- Migration effort for existing slug-based documents
- Short codes become meaningless if project context is lost
- Sequential numbering reveals document creation order

### Neutral

- Document titles remain in frontmatter for human readability
- File organization by type provides some discoverability
- System handles all ID generation automatically

## Review Schedule

### Review Triggers

- Significant issues with short code usability arise
- Need for more human-readable identification emerges
- Changes to database or MCP integration requirements

### Scheduled Review

- **Next Review Date**: 2026-10-16
- **Review Criteria**: System reliability, developer experience, cross-interface consistency
- **Sunset Date**: N/A (foundational identification system)