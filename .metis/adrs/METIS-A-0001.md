---
id: 001-document-format-and-storage
level: adr
title: "Document Format and Storage"
number: 1
short_code: "METIS-A-0001"
created_at: 2025-07-02T15:10:00+00:00
updated_at: 2025-10-16T02:01:15.803156+00:00
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

# ADR-001: Document Format and Storage

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context **[REQUIRED]**

We need to decide how to store and format documents in the Metis system. The choice affects tooling, human readability, version control integration, and cross-platform compatibility.

## Decision **[REQUIRED]**

We will use **markdown files with YAML frontmatter** stored in a **hierarchical directory structure**.

**File Format:**
- Markdown (.md) for human-readable content
- YAML frontmatter for machine-readable metadata
- UTF-8 encoding

**Naming Conventions:**
- Slugs: lowercase, hyphenated, max 50 characters
- ADRs: sequential numbering starting from 001
- No spaces, special characters, or unicode in filenames
- Vision document: fixed name "vision.md"

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

**Markdown + YAML**
- Pros: Human-readable, Git-friendly, tool-compatible
- Cons: Complex parsing, YAML errors
- Risk Level: Low
- Implementation Cost: Low

**Pure JSON**
- Pros: Machine-readable, simple parsing
- Cons: Not human-friendly, poor diffs
- Risk Level: Medium
- Implementation Cost: Low

**XML**
- Pros: Structured, validatable
- Cons: Verbose, developer-unfriendly
- Risk Level: Medium
- Implementation Cost: Medium

## Rationale **[REQUIRED]**

Markdown with YAML frontmatter provides the best balance of human readability and machine parseability. It integrates well with existing developer tools and version control systems while maintaining the structured metadata needed for automation.

## Consequences **[REQUIRED]**

### Positive
- Human-readable and editable in any text editor
- Git-friendly with meaningful diffs
- Works with existing tools (Obsidian, VS Code, etc.)
- Cross-platform compatible
- YAML frontmatter separates metadata from content cleanly
- Directory structure makes document types immediately obvious

### Negative
- More complex parsing than pure JSON or XML
- YAML syntax can be error-prone for non-technical users
- File naming conventions must be enforced programmatically
- Directory structure needs to be created and maintained

### Neutral
- Requires tooling to validate YAML frontmatter consistency

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- Significant parsing errors or format issues arise
- Major changes to Git workflow or version control needs
- Introduction of non-technical team members requiring different editing approaches

### Scheduled Review
- **Next Review Date**: 2026-07-02
- **Review Criteria**: Format adoption success, tooling compatibility, team feedback
- **Sunset Date**: N/A (foundational decision)