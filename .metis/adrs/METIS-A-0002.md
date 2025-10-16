---
id: 002-obsidian-markdown-format
level: adr
title: "Obsidian Markdown Format"
number: 2
short_code: "METIS-A-0002"
created_at: 2025-07-02T15:15:00+00:00
updated_at: 2025-10-16T01:51:49.147746+00:00
decision_date: 
decision_maker: team
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/superseded"


exit_criteria_met: true
strategy_id: NULL
initiative_id: NULL
---

# ADR-002: Obsidian Markdown Format

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context **[REQUIRED]**

We need to decide which markdown variant to use for documents. Standard CommonMark provides basic formatting, but Obsidian offers extended features like properties (YAML frontmatter), callouts, folding, and enhanced linking that could benefit our process documentation.

## Decision **[REQUIRED]**

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

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

**Obsidian Markdown**
- Pros: Enhanced features, excellent tooling
- Cons: Soft vendor dependency
- Risk Level: Low
- Implementation Cost: Low

**CommonMark**
- Pros: Universal compatibility
- Cons: Limited features
- Risk Level: Low
- Implementation Cost: Low

**GitHub Flavored**
- Pros: Good compatibility, tables
- Cons: Limited metadata support
- Risk Level: Medium
- Implementation Cost: Low

## Rationale **[REQUIRED]**

Obsidian Flavored Markdown provides enhanced authoring experience while maintaining compatibility with standard markdown. The benefits of enhanced linking, callouts, and properties outweigh the soft dependency on Obsidian tooling.

## Consequences **[REQUIRED]**

### Positive
- Enhanced linking makes document relationships more discoverable
- Callouts improve document readability and highlight important information
- Properties (YAML frontmatter) enable structured metadata while remaining human-readable
- Folding improves navigation of long documents
- Obsidian provides excellent authoring experience
- All content remains portable as standard markdown

### Negative
- Enhanced features may not render correctly in other markdown processors
- Creates soft dependency on Obsidian for optimal viewing experience
- Team must learn Obsidian-specific syntax extensions
- Tooling must handle Obsidian-flavored parsing

### Neutral
- Standard markdown processors will still render core content correctly

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- Team reports difficulties with Obsidian-specific features
- Need to support tools that can't handle enhanced features
- Migration to different documentation platform

### Scheduled Review
- **Next Review Date**: 2026-07-02
- **Review Criteria**: Team adoption, tooling compatibility, feature utilization
- **Sunset Date**: N/A (format decision)