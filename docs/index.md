# Metis Documentation

Metis is a work management system for software projects, built around the Flight Levels methodology — a hierarchical approach to organizing work at strategic, project, and task levels. It provides a CLI, an MCP (Model Context Protocol) server for AI assistant integration, and a desktop GUI with Kanban boards.

## Tutorials

Step-by-step guides for learning Metis.

- [Getting Started with Metis](tutorials/getting-started.md) — Create your first project, from initialization to task completion
- [Using Metis with Claude Code](tutorials/using-with-claude-code.md) — Set up AI integration with the MCP server and plugin
- [Running Autonomous Tasks with Ralph Loops](tutorials/ralph-loops.md) — Autonomous AI execution from task creation to completion

## How-To Guides

Task-oriented recipes for specific problems.

- [How to Install Metis](how-to/install.md) — Desktop app, CLI, and Claude Code plugin installation
- [How to Manage Documents](how-to/manage-documents.md) — Create, edit, search, and archive documents
- [How to Transition Document Phases](how-to/transition-phases.md) — Move documents through their lifecycle
- [How to Configure Flight Levels](how-to/configure-flight-levels.md) — Switch between streamlined and direct presets
- [How to Use the Code Index](how-to/code-index.md) — Generate and maintain code indexes for AI navigation
- [How to Use the Desktop GUI](how-to/desktop-gui.md) — Board navigation, drag-and-drop, themes
- [How to Run Metis in a Docker Sandbox](how-to/docker-sandbox.md) — Isolated autonomous execution

## Reference

Technical descriptions for lookup.

- [CLI Reference](reference/cli.md) — All commands, flags, arguments, and output formats
- [MCP Tools Reference](reference/mcp-tools.md) — All MCP tools, parameters, and return values
- [Document Types Reference](reference/document-types.md) — Vision, Initiative, Task, ADR, Specification schemas
- [Phase Lifecycle Reference](reference/phase-lifecycle.md) — All phase sequences and valid transitions
- [Configuration Reference](reference/configuration.md) — config.toml, environment variables, database, plugin settings
- [Project Structure Reference](reference/project-structure.md) — Directory layout, database schema, file conventions

## Explanation

Understanding-oriented discussion.

- [Flight Levels Methodology](explanation/flight-levels.md) — Why Metis uses hierarchical document types
- [Architecture Overview](explanation/architecture.md) — Crate structure, data flow, dual storage model
- [The Ralph Loop Pattern](explanation/ralph-loops.md) — How autonomous AI execution works and when to use it
