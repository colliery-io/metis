---
id: metis
level: vision
title: "Metis"
short_code: "METIS-V-0001"
created_at: 2025-10-07T20:44:44.799698+00:00
updated_at: 2025-10-11T01:15:45.511843+00:00
archived: false

tags:
  - "#vision"
  - "#vision"
  - "#phase/published"


exit_criteria_met: false
strategy_id: 
initiative_id:
---

# Metis Vision

## Purpose **[REQUIRED]**

Metis is a hierarchical project management system that bridges the gap between high-level strategic thinking and tactical execution. Built on the Flight Levels methodology, Metis provides AI assistants and development teams with a structured approach to organizing work across Vision, Strategy, Initiative, and Task levels, ensuring clear line-of-sight from purpose to implementation.

## Product/Solution Overview **[CONDITIONAL: Product/Solution Vision]**

Metis is a Rust-based project management toolkit consisting of four integrated components:
- **metis-docs-core**: Core library handling document management, workflows, and database operations
- **metis-docs-cli**: Command-line interface for direct project management and automation
- **metis-docs-tui**: Interactive terminal user interface for visual project management
- **metis-docs-mcp**: MCP (Model Context Protocol) server enabling seamless AI assistant integration

Target audience includes AI assistants (Claude, Cursor), software development teams, and project managers who need structured work decomposition with clear traceability from vision to execution.

## Current State **[REQUIRED]**

Metis has reached a mature state with:
- Complete Flight Levels implementation with configurable presets (full, streamlined, direct)
- Functional CLI, TUI, and MCP interfaces
- SQLite database with FTS5 search capabilities
- Comprehensive test coverage and documentation
- Multi-crate Rust workspace architecture
- Active development with regular releases (current v0.5.0)

## Future State **[REQUIRED]**

Metis will be the standard tool for AI-assisted project management, providing:
- Seamless integration across all major AI assistants and development environments
- Intuitive workflow management that scales from simple projects to complex organizational initiatives
- Robust ecosystem integration with existing development tools
- Community-driven adoption with extensive documentation and examples
- Enterprise-ready features for large-scale project coordination

## Major Features **[CONDITIONAL: Product Vision]**

- **Hierarchical Work Management**: Vision → Strategy → Initiative → Task decomposition with configurable levels
- **Phase-based Workflows**: Structured progression through defined phases (draft → review → published, etc.)
- **AI Assistant Integration**: 11 MCP tools for complete project management within AI conversations
- **Multiple Interfaces**: CLI for automation, TUI for visual management, MCP for AI integration
- **Full-text Search**: SQLite FTS5 indexing for fast document discovery
- **Document Lifecycle Management**: Archive/restore functionality with cascading operations
- **Flexible Configuration**: Three presets (full, streamlined, direct) adaptable to project complexity

## Success Criteria **[REQUIRED]**

- AI assistants can successfully manage complex multi-level projects using Metis MCP tools
- Development teams adopt Metis for project planning and execution tracking
- Documentation remains comprehensive and up-to-date across all interfaces
- Performance scales effectively for projects with hundreds of documents
- Integration ecosystem grows with additional tool integrations
- Community contributes improvements and extensions

## Principles **[REQUIRED]**

- **Direct Path Approach**: Markdown files with YAML frontmatter - what you see is what you get
- **AI-First Design**: Optimized for AI assistant workflows while remaining human-usable
- **Flexible Hierarchy**: Configurable levels to match project complexity needs
- **Consistency Across Interfaces**: Unified behavior whether using CLI, TUI, or MCP
- **Performance First**: Fast search and operations even with large document sets
- **Clear Traceability**: Every task traces back to strategic vision through parent relationships

## Constraints **[REQUIRED]**

- Rust ecosystem dependency for core development
- SQLite storage limits (suitable for team-scale, not enterprise-wide databases)
- MCP protocol dependency for AI assistant integration
- File system-based storage (not distributed by design)
- Markdown format constraints for document structure