# Metis - Flight Levels Project Management for AI Assistants

<div align="center">
  <img src="crates/metis-docs-gui/src-tauri/icons/icon.png" alt="Metis Owl Logo" width="128" height="128">
</div>

Metis is a hierarchical project management system built on the Flight Levels methodology, providing a structured approach to organizing work across Strategy, Initiative, and Task levels. It features an MCP (Model Context Protocol) interface for seamless integration with AI assistants like Claude and Cursor.

## Quick Start (< 5 minutes)

### 1. Installation

#### Quick Install (Recommended)
```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
```

This installs `metis` CLI and `metis-tui` to `~/.local/bin`. The MCP server is bundled with the CLI and accessible via `metis mcp`.

#### Desktop GUI Application
For a visual interface, download the Metis desktop application:

- **macOS (Apple Silicon)**: `Metis_x.x.x_aarch64.dmg`
- **macOS (Intel)**: `Metis_x.x.x_x64.dmg`
- **Windows**: `Metis_x.x.x_x64-setup.exe`
- **Linux**: `Metis_x.x.x_amd64.AppImage`

[Download from GitHub Releases](https://github.com/colliery-io/metis/releases/latest)

**What's Included:**
- Visual kanban interface with project management and document editing
- **Automatic CLI installation**: On first launch, Metis will prompt to install the `metis` CLI command system-wide
- MCP server for AI assistant integration (accessible via the installed CLI)
- Built-in project initialization wizard

**macOS Security Notes**:
- After installing, you may need to remove the quarantine attribute:
  ```bash
  sudo xattr -rd com.apple.quarantine "/Applications/Metis.app"
  ```
- On first launch, you'll be prompted to allow CLI installation to `/usr/local/bin`

#### Build from Source
```bash
# Install CLI (includes bundled MCP server)
cargo install metis-docs-cli

# Install TUI interface
cargo install metis-docs-tui
```

### 2. Choose Your Interface

#### Desktop GUI (Primary Interface)
Launch the installed Metis application for a full-featured visual interface:
- **Project Browser**: Manage multiple projects from a single interface
- **Kanban Boards**: Visual boards for Vision, Initiative, Task, ADR, and Backlog
- **Rich Text Editor**: Built-in markdown editor with table support
- **Project Initialization**: Guided setup with preset selection (Full/Streamlined/Direct)
- **Real-time Sync**: Automatic synchronization with file system

#### Command Line Interface
```bash
# Direct CLI commands (for scripting and automation)
metis init --name "My Vision"                           # Creates streamlined config
metis create initiative "Core Initiative" --vision "my-vision"  # No strategies in default
metis list --type task                                   # List all tasks

# MCP Server (for AI assistant integration)
metis mcp
```

#### Terminal User Interface (Deprecated)
> **Note**: The TUI (`metis tui`) is deprecated and will be removed in a future release. Please use the Desktop GUI application instead for visual project management.

### 3. Configure Your AI Assistant

#### For Claude Desktop
Add to your `claude_desktop_config.json`:
```json
{
  "mcpServers": {
    "metis": {
      "command": "metis",
      "args": ["mcp"]
    }
  }
}
```

#### For Claude Code
```bash
claude mcp add --scope user --transport stdio metis -- metis mcp
```

#### For Cursor
Add to your Cursor MCP configuration:
```json
{
  "metis": {
    "command": "metis",
    "args": ["mcp"]
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
- **Task**: todo → active → completed
- **ADR**: draft → discussion → decided → superseded

> **Pro Tip**: Start with the streamlined configuration (default). You can always upgrade to full flight levels as your project complexity grows, or simplify to direct for straightforward work.  

## The Metis Process: Vision to Execution

Metis follows a structured decomposition process:

1. **Start with Vision** - Create an overall purpose and direction document defining why the project exists
2. **Create Strategies** - Develop strategies that reference the Vision as parent, moving through shaping → design → ready → active
3. **Build Initiatives** - When strategies are active, create concrete initiatives that reference Strategy as parent
4. **Break into Tasks** - When initiatives reach decompose phase, create actionable tasks that reference Initiative as parent
5. **Execute Tasks** - Work through tasks: todo → active → completed
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

## Desktop GUI Application

The Metis desktop application is the primary interface for project management with full Flight Levels support:

### Key Features
- **Multi-Project Management**: Switch between projects with a sidebar browser
- **Integrated CLI Installation**: Automatic installation of `metis` CLI command on first launch for AI assistant integration
- **Dynamic Board Configuration**: Available boards adapt based on your project's Flight Level preset
- **Rich Text Editing**: Built-in markdown editor with:
  - Live preview and editing modes
  - Table creation and manipulation tools
  - Syntax highlighting and formatting
- **Project Initialization Wizard**: Guided setup with preset selection
- **Real-time Synchronization**: Refresh button syncs with file system and reloads data
- **Document Lifecycle Management**:
  - Drag-and-drop between phases
  - Archive completed work
  - View detailed document information

### Interface Overview
The GUI organizes work into kanban-style boards:
- **Vision Board**: Single document editor for project vision
- **Strategy Board**: Strategy documents through their lifecycle (Full preset only)
- **Initiative Board**: Initiative management with phase tracking
- **Task Board**: Task execution with Blocked → Todo → Active → Completed flow
- **ADR Board**: Architectural Decision Records
- **Backlog Board**: Unassigned work items organized by type (Bug/Feature/Tech Debt/General)
- **Refresh Button**: Syncs the filesystem to the database and refreshes the UI. Use when external systems (like the MCP server) are editing things.

### Getting Started with GUI
1. **Download and Install**: Get the appropriate installer for your platform from GitHub Releases
2. **Allow CLI Installation**: On first launch, approve the system prompt to install CLI tools (enables AI assistant integration)
3. **Create or Open Project**: Use the project browser to initialize a new project or open existing ones
4. **Select Configuration**: Choose Full/Streamlined/Direct preset during initialization
5. **Start Working**: Create documents, move them through phases, and track progress visually

## Terminal User Interface (TUI) - Deprecated

> **Deprecation Notice**: The Terminal User Interface has been deprecated in favor of the Desktop GUI application. The TUI will be removed in a future release.

If you still need to use the TUI:

```bash
metis tui
```

**Migration**: All TUI functionality is available in the Desktop GUI with a more user-friendly interface and better performance. Please migrate to the GUI application.

## Technical Overview

**Architecture**: Metis consists of five main components:
- **metis-docs-core**: Rust library handling document management, workflows, and database
- **metis-docs-gui**: Primary desktop application with visual kanban interface built with Tauri and Vue 3, includes automatic CLI installation
- **metis-docs-cli**: Command-line interface with full project management capabilities, bundled with GUI installer
- **metis-docs-mcp**: MCP server providing AI assistant integration with tools for document management
- **metis-docs-tui**: Interactive terminal user interface (deprecated, will be removed in future release)

**Installation Flow**: The GUI installer bundles the CLI binary and automatically installs it to the system PATH on first launch, providing seamless integration between visual and AI-assisted workflows.

**Direct Path Approach**: Documents are stored as markdown files with YAML frontmatter, indexed in SQLite with FTS5 for fast search. No complex abstractions - what you see is what you get.

**Background Sync**: File system changes are synced to the database index, ensuring consistency between markdown files and search capabilities.

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

## License & Contributing

Apache 2.0 License. Contributions welcome:
1. Fork repository
2. Create feature branch  
3. Add tests for new functionality
4. Ensure `angreal check` passes
5. Ensure `angreal test` passes
6. For GUI changes, provide a journey for testing the new feature in writing. 
7. Submit pull request

---

*Built with ❤️ for AI-assisted project management*