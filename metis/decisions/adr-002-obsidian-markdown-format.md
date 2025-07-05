---
# Document identity
id: adr-002-obsidian-markdown-format
level: adr
status: decided
created_at: 2025-07-02T15:15:00Z
updated_at: 2025-07-02T15:15:00Z

# Relationships
parent: null  # ADRs are cross-cutting

# Dependencies
blocked_by: []

# Stage-specific metadata
phase: decided
exit_criteria_met: true

# ADR-specific fields
decision_maker: team
supersedes: []
---

# ADR-002: Obsidian Markdown Format

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need to decide which markdown variant to use for documents. Standard CommonMark provides basic formatting, but Obsidian offers extended features like properties (YAML frontmatter), callouts, folding, and enhanced linking that could benefit our process documentation.

## Decision

We will use **Obsidian Flavored Markdown** as our document format standard.

**Obsidian Features We'll Utilize:**
- **Properties** (YAML frontmatter) for structured metadata
- **Enhanced linking** with `[[document-title]]` syntax and automatic link resolution
- **Callouts** for highlighting important information (`> [!note]`, `> [!warning]`, etc.)
- **Folding** for collapsible sections and better document organization
- **Attachments** handling for embedded images and files

**Rationale:**
All Obsidian markdown remains valid standard markdown - it's a superset, not a replacement. Other markdown processors will render the core content correctly, just without the enhanced features.

**References:**
- [Obsidian Flavored Markdown](https://help.obsidian.md/obsidian-flavored-markdown)
- [Properties](https://help.obsidian.md/properties)
- [Attachments](https://help.obsidian.md/attachments)
- [Callouts](https://help.obsidian.md/callouts)
- [Folding](https://help.obsidian.md/folding)

## Consequences

**Positive:**
- Enhanced linking makes document relationships more discoverable
- Callouts improve document readability and highlight important information
- Properties (YAML frontmatter) enable structured metadata while remaining human-readable
- Folding improves navigation of long documents
- Obsidian provides excellent authoring experience
- All content remains portable as standard markdown

**Negative:**
- Enhanced features may not render correctly in other markdown processors
- Creates soft dependency on Obsidian for optimal viewing experience
- Team must learn Obsidian-specific syntax extensions
- Tooling must handle Obsidian-flavored parsing

## Validation

We'll know this was right if:
- Document authoring and navigation experience is significantly improved
- Enhanced features are consistently used and provide value
- Standard markdown compatibility doesn't create issues
- Team adoption of Obsidian features is smooth