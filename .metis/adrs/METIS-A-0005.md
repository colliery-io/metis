---
id: 005-generic-markdown-format
level: adr
title: "Generic Markdown Format"
number: 5
short_code: "METIS-A-0005"
created_at: 2025-10-16T01:52:57.213070+00:00
updated_at: 2025-10-16T01:52:57.213070+00:00
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
supersedes: "[[METIS-A-0002]]"
---

## Context 

With the development of TUI and GUI interfaces for Metis, we need to support multiple text editors beyond Obsidian. The Obsidian-specific markdown features (enhanced linking, callouts, folding) create compatibility issues when documents are edited through:

- Terminal UI (TUI) with various system editors
- Graphical UI (GUI) with embedded text editing components
- Command-line tools and scripts
- AI assistants working with plain text

While Obsidian remains an excellent authoring environment, the original intent of using Obsidian as a “display layer” have been superseded now they we’ve brought view/edit capabilities within the service set we provide. 

## Decision 

We will use **Generic Markdown** (CommonMark-compatible) with YAML frontmatter as our document format standard.

**Allowed Features:**

- Standard markdown syntax (headers, lists, bold, italic, code blocks, links)
- YAML frontmatter for structured metadata
- GitHub-style markdown checkboxes for exit criteria
- Standard markdown tables
- UTF-8 encoding

**Unsupported Features:**

- Obsidian wiki-links (`[[document-title]]`) → Standard markdown links
- Obsidian callouts (`> [!note]`) → Standard blockquotes or **bold** emphasis
- Obsidian folding → Standard document structure with headers
- Obsidian-specific attachment handling → Standard markdown image syntax

**Migration Strategy:**

- Convert existing wiki-links to standard markdown links with document paths
- Replace callouts with standard markdown equivalents
- Preserve all content and metadata during transition

## Rationale 

Generic markdown ensures documents remain fully functional across all Metis interfaces (CLI, TUI, GUI) and external tools. This aligns with our core principle of "Open Formats Over Vendor Lock-in" and supports the multi-interface architecture we've built.

The enhanced features of Obsidian markdown, while valuable, create friction when documents are edited through other interfaces, potentially corrupting formatting or breaking links.

This decision also doesn’t stop people from using Obsidian specific markdown syntax, we just won’t be supporting it within the metis ecosystem directly. 

## Consequences **\[REQUIRED\]**

### Positive

- Complete compatibility across all Metis interfaces (CLI, TUI, GUI)
- Documents work correctly in any markdown editor
- No vendor lock-in or tool dependency
- Simplified parsing and validation logic
- Better AI assistant compatibility
- Preserved portability and future-proofing

### Negative

- Loss of enhanced linking features (wiki-links)
- No callout formatting for highlighted information
- Reduced visual richness in Obsidian
- Manual link management required
- Some migration effort for existing documents

### Neutral

- Standard markdown provides sufficient functionality for documentation needs
- YAML frontmatter still provides structured metadata capabilities

## Review Schedule **\[CONDITIONAL: Temporary Decision\]**

### Review Triggers

- Significant usability issues arise from generic markdown limitations
- New universal markdown extensions gain widespread adoption
- Major changes to supported interface requirements

### Scheduled Review

- **Next Review Date**: 2026-10-16
- **Review Criteria**: Multi-interface compatibility, team productivity, document quality
- **Sunset Date**: N/A (format standardization decision)