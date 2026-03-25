# Architecture Overview

Metis is a Rust workspace with five crates, a Tauri desktop app, and a Claude Code plugin. This document explains how the pieces fit together.

## Crate Structure

```
metis/
├── crates/
│   ├── metis-docs-core     # Domain logic, services, database (library)
│   ├── metis-docs-cli      # CLI binary ("metis")
│   ├── metis-docs-mcp      # MCP server binary ("metis-mcp")
│   ├── metis-docs-gui      # Tauri desktop app + Vue frontend
│   └── metis-code-index    # Tree-sitter code indexing (library)
└── plugins/
    └── metis/              # Claude Code plugin
```

### Dependency Graph

```
metis-docs-cli ──→ metis-docs-core
                ──→ metis-docs-mcp
                ──→ metis-code-index

metis-docs-mcp ──→ metis-docs-core
               ──→ metis-code-index

metis-docs-gui ──→ metis-docs-core
(src-tauri)

metis-code-index (standalone, no Metis dependencies)
```

**metis-docs-core** is the heart of the system. Both the CLI, MCP server, and GUI depend on it for domain models, services, and database access.

**metis-code-index** is independently usable — it knows nothing about Metis documents. It's a general-purpose code indexer that happens to be used by both the CLI and MCP server.

## metis-docs-core

The core crate follows a layered architecture:

```
┌─────────────────────────────────┐
│         Application Layer       │
│   (Services: creation, sync,    │
│    transition, archive, etc.)   │
├─────────────────────────────────┤
│          Domain Layer           │
│   (Documents, Phases, Config,   │
│    Templates, Traits)           │
├─────────────────────────────────┤
│       Data Access Layer         │
│   (SQLite/Diesel, Filesystem)   │
└─────────────────────────────────┘
```

### Domain Layer

Defines the core types:

- **Document types:** Vision, Initiative, Task, ADR, Specification — each with type-specific fields and phase sequences
- **Document trait:** Common interface all document types implement, including `transition_phase()`, `validate()`, and content access
- **Phase/Tag system:** Phases stored as tags in frontmatter; the `Phase` enum covers all phases across all document types
- **FlightLevelConfig:** Controls which document types are enabled and their hierarchy relationships

### Application Layer

Services coordinate domain objects and infrastructure:

- **DocumentCreationService** — Creates documents with proper defaults, templates, and short codes
- **PhaseTransitionService** — Validates and executes phase transitions
- **SyncService** — Reconciles filesystem with database
- **WorkspaceInitializationService** — Creates new workspaces
- **WorkspaceArchiveService** — Archives documents and children
- **WorkspaceReassignmentService** — Moves tasks between parents
- **ConfigurationRecoveryService** — Detects and repairs database/config issues
- **TemplateLoader** — Loads templates with project → global → embedded fallback chain

### Data Access Layer

Dual storage with filesystem as source of truth:

- **Database (SQLite via Diesel ORM):** Provides search (FTS5), relationship queries, and fast lookups. Tables: documents, document_relationships, document_tags, document_search, configuration.
- **Filesystem:** Documents are Markdown files with YAML frontmatter. The filesystem is always authoritative — the database is a derived index.

## Dual Storage Model

This is the most important architectural decision in Metis. Documents live in two places:

1. **Markdown files** — Human-readable, version-controllable, editable with any text editor
2. **SQLite database** — Enables search, filtering, relationship queries, and fast lookups

The filesystem is the source of truth. If the database is deleted, `metis sync` rebuilds it from the files. If a file and database record disagree, the file wins.

**Why not just files?** Full-text search across dozens of Markdown files is slow. Relationship queries (find all tasks under this initiative) require walking the directory tree. The database makes these operations instant.

**Why not just a database?** Markdown files are portable, diffable, and version-controllable with Git. They work with every text editor. They don't require Metis to read.

**The sync mechanism** (`metis sync` / auto-sync) walks the `.metis/` directory, compares file hashes against database records, and imports/updates/deletes as needed. Most operations auto-sync after completing.

## metis-docs-cli

The CLI is a thin layer over metis-docs-core. Each command:

1. Checks for a Metis workspace (walks up directory tree looking for `.metis/`)
2. Opens the database
3. Calls the appropriate core service
4. Formats output (table, compact, or JSON)

The CLI uses Clap for argument parsing, dialoguer for interactive prompts (complexity selection), tabled for table formatting, and colored for terminal colors.

The `metis mcp` command is special — instead of running a one-shot operation, it starts the MCP server process (from metis-docs-mcp) which runs until the client disconnects.

## metis-docs-mcp

The MCP server bridges AI agents and Metis:

```
AI Agent ←→ JSON-RPC (stdio) ←→ MCP Server ←→ metis-docs-core ←→ Filesystem/DB
```

Each MCP tool:
1. Deserializes JSON parameters
2. Finds the workspace
3. Calls the appropriate core service
4. Formats the result using `ToolOutput` (consistent markdown with status icons)
5. Returns a `CallToolResult`

The server uses `rust-mcp-sdk` for protocol handling and the `#[mcp_tool]` macro for tool schema generation. The `tool_box!` macro generates the dispatch enum.

**Dynamic instructions:** On startup, the server loads the current flight level configuration and generates MCP instructions that include the active preset, enabled types, and transition rules. This means the AI agent always gets up-to-date configuration information.

## metis-docs-gui

A Tauri 2 desktop application with a Rust backend and Vue 3 frontend:

```
Vue 3 Frontend ←→ Tauri IPC (invoke) ←→ Rust Backend ←→ metis-docs-core
```

### Backend (src-tauri)

Exposes Tauri commands that wrap metis-docs-core services. Key patterns:

- **AppState:** Session-scoped state holding the current project path
- **Auto-sync:** Most commands sync the workspace before and/or after the operation
- **CLI installer:** Bundles the `metis` binary and can install it to the user's PATH

### Frontend (src/)

- **Board system:** Seven board types (Vision, Initiative, Task, Backlog, ADR, Specification, Strategy), each with phase-based columns
- **Drag-and-drop:** Moving cards between columns triggers phase transitions
- **Composables:** `useProject` (state), `useTheme` (three themes), `useToolbar` (Tiptap editor)
- **Tauri API bridge:** TypeScript wrappers around `invoke()` calls

## metis-code-index

A standalone library for multi-language code indexing:

```
walk_directory() → parse_file() → extract_symbols() → format_index()
```

- **Walker:** Gitignore-aware file discovery, skips build directories
- **Parser:** Tree-sitter-based, lazy language initialization
- **Extractors:** Language-specific symbol extraction (Rust, Python, TypeScript, JavaScript, Go)
- **Formatter:** Generates markdown with project tree, grouped symbols, and preserved summaries
- **Hasher:** BLAKE3 content hashing for incremental re-indexing

The incremental flow: load previous hashes → compute current hashes → diff → parse only changed files → merge with cached symbols → format.

## Claude Code Plugin

The plugin (`plugins/metis/`) teaches Claude Code how to work with Metis:

- **Skills:** Methodology guidance (document selection, decomposition, phase transitions, project patterns)
- **Commands:** `/metis-ralph` (task execution loop), `/metis-decompose` (initiative breakdown), `/cancel-metis-ralph`
- **Agents:** Flight Levels methodology expert, code-index-summarizer
- **Hooks:** SessionStart (project detection, code indexing), PreCompact (re-index), PostToolUse (track changes), Stop (Ralph loop control)
- **MCP config:** Registers the `metis mcp` server

The Ralph loop is implemented through the Stop hook: when Claude tries to exit, the hook checks for a completion promise (`<promise>TASK COMPLETE</promise>`). If not found, it feeds the prompt back and increments the iteration counter.

## Data Flow Example: Creating a Task

```
1. User: "metis create task 'Fix login' --initiative PROJ-I-0001"

2. CLI (cli/src/commands/create/task.rs):
   - Validates workspace exists
   - Finds initiative in database

3. Core (DocumentCreationService):
   - Generates short code (PROJ-T-0003)
   - Loads task template (project → global → embedded fallback)
   - Renders template with title and parent
   - Writes .metis/initiatives/PROJ-I-0001/tasks/PROJ-T-0003/task.md

4. Core (SyncService):
   - Detects new file
   - Parses frontmatter
   - Inserts database record
   - Creates parent-child relationship
   - Updates FTS5 search index

5. CLI:
   - Prints short code and file path
```

The same flow happens through MCP (AI agent → JSON-RPC → MCP server → core) and through the GUI (Vue → Tauri IPC → Rust backend → core). The core services are the single point of truth for all business logic.
