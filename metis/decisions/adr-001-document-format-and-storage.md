---
# Document identity
id: adr-001-document-format-and-storage
level: adr
status: decided
created_at: 2025-07-02T15:10:00Z
updated_at: 2025-07-02T15:10:00Z

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

# ADR-001: Document Format and Storage

**Status**: Decided  
**Date**: 2025-07-02  
**Driver**: Process Documentation Initiative

## Context

We need to decide how to store and format documents in the Metis system. The choice affects tooling, human readability, version control integration, and cross-platform compatibility.

## Decision

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

## Consequences

**Positive:**
- Human-readable and editable in any text editor
- Git-friendly with meaningful diffs
- Works with existing tools (Obsidian, VS Code, etc.)
- Cross-platform compatible
- YAML frontmatter separates metadata from content cleanly
- Directory structure makes document types immediately obvious

**Negative:**
- More complex parsing than pure JSON or XML
- YAML syntax can be error-prone for non-technical users
- File naming conventions must be enforced programmatically
- Directory structure needs to be created and maintained

## Validation

We'll know this was right if:
- Documents remain human-readable and editable
- Tooling can reliably parse metadata and content
- Version control provides meaningful diffs
- Team adoption is smooth without format-related friction