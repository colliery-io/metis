# Metis - Flight Levels Project Management for AI Assistants

Metis is a hierarchical project management system built on the Flight Levels methodology, providing a structured approach to organizing work across Strategy, Initiative, and Task levels. It features an MCP (Model Context Protocol) interface for seamless integration with AI assistants like Claude and Cursor.

## Quick Start (< 5 minutes)

### 1. Installation

```bash
# Install CLI and TUI (includes metis command and metis tui)
cargo install metis-docs-cli

# Install MCP server for AI assistants
cargo install metis-docs-mcp

# For GUI applications (Claude Desktop, etc.) that need system PATH access
sudo cargo install metis-docs-mcp --root /usr/local
```

**Note**: GUI applications like Claude Desktop may not have access to your shell's PATH. If you get "ENOENT" errors when using the MCP server with GUI applications, use the second installation command to install to `/usr/local/bin` where GUI apps can find it.

### 2. Choose Your Interface

```bash
# Terminal User Interface (recommended for interactive use)
metis tui

# Command Line Interface (for scripting and automation)
metis init --name "My Vision"                           # Creates streamlined config
metis create initiative "Core Initiative" --vision "my-vision"  # No strategies in default

# MCP Server (for AI assistant integration)
metis-mcp
```

### 3. Configure Your AI Assistant

#### For Claude Desktop
Add to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "metis": {
      "command": "metis-mcp"
    }
  }
}
```

#### For Claude Code
```bash
claude mcp add --scope user --transport stdio metis -- metis-mcp
```

#### For Cursor
Add to your Cursor MCP configuration:
```json
{
  "metis": {
    "command": "metis-mcp"
  }
}
```

## What is Flight Levels?

Flight Levels is a methodology for organizing work across different levels of abstraction. Metis supports flexible configurations to match your project's complexity:

### Flight Level Configurations

**Full** - Complete hierarchy for complex projects:
```
Vision                      - Overall purpose and direction  
└── Strategy                - How to achieve the vision
    └── Initiative          - Concrete projects implementing strategies
        └── Task            - Individual work items
```

**Streamlined** (Default) - Balanced approach for most projects:
```
Vision                      - Overall purpose and direction  
└── Initiative              - Concrete projects implementing the vision
    └── Task                - Individual work items
```

**Direct** - Minimal hierarchy for simple projects:
```
Vision                      - Overall purpose and direction  
└── Task                    - Individual work items
```

### Configuration Management

You can set your project's configuration during initialization:

```bash
# Use default streamlined configuration
metis init --name "My Project"

# Use full flight levels for complex projects  
metis init --name "My Project" --preset full

# Use direct approach for simple projects
metis init --name "My Project" --preset direct

# Custom configuration
metis init --name "My Project" --strategies false --initiatives true
```

Or change it later:

```bash
metis config set --preset streamlined
metis config show  # View current configuration
```

### Workflow Phases

Each document type has defined workflows:
- **Vision**: draft → review → published
- **Strategy**: shaping → design → ready → active → completed
- **Initiative**: discovery → design → ready → decompose → active → completed  
- **Task**: todo → doing → completed

> **Pro Tip**: Start with the streamlined configuration (default). You can always upgrade to full flight levels as your project complexity grows, or simplify to direct for straightforward work.  

## The Metis Process: Vision to Execution

Metis follows a structured decomposition process:

1. **Start with Vision** - Create an overall purpose and direction document defining why the project exists
2. **Create Strategies** - Develop strategies that reference the Vision as parent, moving through shaping → design → ready → active
3. **Build Initiatives** - When strategies are active, create concrete initiatives that reference Strategy as parent
4. **Break into Tasks** - When initiatives reach decompose phase, create actionable tasks that reference Initiative as parent
5. **Execute Tasks** - Work through tasks: todo → doing → completed
6. **Complete upward** - As tasks complete, initiatives progress; as initiatives complete, strategies are delivered

This creates a clear line of sight from high-level vision down to day-to-day work, with each level informing the next.

## MCP Tools Reference

Metis provides 11 MCP tools for complete project management. Available document types depend on your project's flight level configuration:

| Tool | Purpose |
|------|---------|
| `initialize_project` | Set up a new Metis project by creating a `.metis/` subdirectory with project structure |
| `create_document` | Create new documents at any level (vision/strategy/initiative/task/adr) |
| `validate_document` | Validate document structure and metadata |
| `update_document_content` | Update any section of a document |
| `update_exit_criterion` | Update specific exit criteria checkboxes |
| `update_blocked_by` | Manage document dependencies and blockers |
| `transition_phase` | Move documents through workflow phases |
| `validate_exit_criteria` | Check completion status of exit criteria |
| `archive_document` | Archive a document and all its children |
| `list_documents` | Find all documents in a project |
| `search_documents` | Full-text search across all documents |

## Common Workflows

### Starting a New Project
1. `initialize_project` - Set up project structure  
2. `create_document` - Create vision document first
3. `create_document` - Create strategy document referencing vision
4. `update_document_content` - Fill in strategy details
5. `transition_phase` - Move through shaping → design → ready → active

### Breaking Down Work
1. `list_documents` - Find active strategies
2. `create_document` - Create initiative under strategy
3. `create_document` - Create tasks under initiative
4. `update_blocked_by` - Set up dependencies

### Managing Progress
1. `validate_exit_criteria` - Check what's needed for next phase
2. `update_exit_criterion` - Mark criteria as complete
3. `transition_phase` - Move to next workflow stage
4. `search_documents` - Find related work or blockers

### Archiving Completed Work
1. `archive_document` - Archive completed documents and their children
2. Auto-sync updates the database after archiving
3. `list_documents` - Verify archived documents no longer appear in active lists

## Directory Structure

After running `initialize_project`, your project will have this structure:

```
your-project/
└── .metis/
    ├── vision.md        # Initial vision document
    ├── strategies/      # Strategy documents will be created here
    ├── adrs/           # ADR (Architectural Decision Record) documents
    └── metis.db        # SQLite database with FTS index
```

The `initialize_project` tool creates a clean `.metis/` subdirectory to keep all project files organized. Documents are created as individual markdown files and automatically indexed in the SQLite database for fast search.

## Command Line Interface (CLI)

Metis provides a comprehensive CLI for direct project management:

```bash
# Initialize a new project with configuration
metis init --name "Project Vision"                    # Default streamlined config
metis init --name "Project Vision" --preset full      # Full flight levels
metis init --name "Project Vision" --preset direct    # Minimal hierarchy  
metis init --name "Project Vision" --strategies false --initiatives true  # Custom

# Configuration management
metis config show                    # View current configuration
metis config set --preset full      # Change to full flight levels
metis config set --strategies true --initiatives false  # Custom settings

# Create documents (available types depend on your configuration)
metis create strategy "Core Strategy" --vision "project-vision"    # Only in full config
metis create initiative "Implementation" --strategy "core-strategy" # Full/streamlined
metis create task "Build Feature" --initiative "implementation"    # All configs
metis create adr "Database Choice"                                  # All configs

# Manage document lifecycle
metis transition "Project Vision" --phase review
metis validate "Project Vision"
metis status  # Show project overview with current configuration

# Archive and sync operations
metis archive "document-id"  # Archive completed documents
metis sync  # Synchronize workspace with file system

# Search and list documents
metis list --type initiative
metis search "database"

# Launch interactive interfaces
metis tui  # Terminal user interface
metis-mcp  # MCP server for AI assistants
```

## Terminal User Interface (TUI)

Interactive kanban-style interface for visual project management:

```bash
metis tui
```

**Key Features:**
- **Visual Board Layout**: See all documents organized by type (Vision, Strategy, Initiative, Task, ADR)
- **Phase-based Columns**: Documents organized by their current phase
- **Keyboard Navigation**: Arrow keys, Tab/Shift+Tab to navigate boards
- **Quick Actions**: 
  - `n` - Create new document
  - `Ctrl+n` - Create child document
  - `Ctrl+A` - Create ADR document
  - `Enter` - View/edit document
  - `t` - Transition phase
  - `d` - Delete document
  - `r` - Archive selected document
  - `y` - Sync database and reload
  - `v` - View vision document
  - `1-4` - Jump to specific boards

**Workflow Integration**: The TUI automatically syncs with the file system, so changes made externally are reflected immediately.

## Technical Overview

**Architecture**: Metis consists of four main components:
- **metis-docs-core**: Rust library handling document management, workflows, and database
- **metis-docs-cli**: Command-line interface with full project management capabilities  
- **metis-docs-tui**: Interactive terminal user interface for visual project management
- **metis-docs-mcp**: MCP server providing AI assistant integration with 11 tools

**Direct Path Approach**: Documents are stored as markdown files with YAML frontmatter, indexed in SQLite with FTS5 for fast search. No complex abstractions - what you see is what you get.

**Background Sync**: File system changes are automatically synced to the database index, ensuring consistency between markdown files and search capabilities.

## Development Commands

```bash
# Using angreal (recommended)
angreal test      # Run all tests
angreal build     # Build all crates  
angreal clean     # Clean build artifacts
angreal coverage  # Generate coverage reports
angreal check     # Run clippy + format + check

# Using cargo directly
cargo test
cargo build --release
```

## Configuration

Metis uses direct project paths - no global environment variables required. Each MCP tool call specifies the project path directly:

```json
{
  "name": "initialize_project",
  "arguments": {
    "project_path": "/path/to/your/project",
    "name": "my-project",
    "description": "Project description"
  }
}
```

## Troubleshooting

### MCP Server Won't Start
- **"ENOENT" errors in GUI apps**: GUI applications like Claude Desktop don't inherit your shell's PATH. Install with `sudo cargo install metis-docs-mcp --root /usr/local` to place the binary where GUI apps can find it
- Ensure `metis-mcp` is in your PATH after `cargo install`
- Check AI assistant MCP configuration matches examples above
- Restart AI assistant after configuration changes

### Documents Not Appearing
- Verify you've run `initialize_project` in the target directory
- Check document has valid YAML frontmatter
- Ensure using correct project path in tool calls

### AI Assistant Can't Find Tools  
- Verify MCP server is configured correctly in AI assistant
- Check command path is correct (`metis-mcp`)
- Restart AI assistant to reload MCP configuration

### Compilation Errors During Install
- Ensure you have a recent Rust version: `rustup update`
- Try: `cargo install metis-docs-mcp --force` to reinstall

## License & Contributing

Apache 2.0 License. Contributions welcome:
1. Fork repository
2. Create feature branch  
3. Add tests for new functionality
4. Ensure `angreal check` passes
5. Submit pull request

---

*Built with ❤️ for AI-assisted project management*