# Metis - Flight Levels Project Management for AI Assistants

Metis is a hierarchical project management system built on the Flight Levels methodology, providing a structured approach to organizing work across Strategy, Initiative, and Task levels. It features an MCP (Model Context Protocol) interface for seamless integration with AI assistants like Claude and Cursor.

## Quick Start (< 5 minutes)

### 1. Installation

```bash
# For command-line use
cargo install metis-docs-mcp

# For GUI applications (Claude Desktop, etc.) that need system PATH access
sudo cargo install metis-docs-mcp --root /usr/local
```

**Note**: GUI applications like Claude Desktop may not have access to your shell's PATH. If you get "ENOENT" errors when using the MCP server with GUI applications, use the second installation command to install to `/usr/local/bin` where GUI apps can find it.

### 2. Start MCP Server

```bash
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

Flight Levels is a methodology for organizing work across four levels:

```
Vision (30,000 feet)       - Overall purpose and direction  
└── Strategy (10,000 feet) - How to achieve the vision
    └── Initiative (1,000 feet) - Concrete projects implementing strategies
        └── Task (100 feet) - Individual work items
```

Each level has defined workflows:
- **Vision**: draft → review → published
- **Strategy**: shaping → design → ready → active → completed
- **Initiative**: discovery → design → ready → decompose → active → completed  
- **Task**: todo → doing → completed

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

Metis provides 11 MCP tools for complete project management:

| Tool | Purpose |
|------|---------|
| `initialize_project` | Set up a new Metis project by creating a `metis/` subdirectory with project structure |
| `create_document` | Create new documents at any level (vision/strategy/initiative/task) |
| `validate_document` | Validate document structure and metadata |
| `update_document_content` | Update any section of a document |
| `update_exit_criterion` | Update specific exit criteria checkboxes |
| `update_blocked_by` | Manage document dependencies and blockers |
| `transition_phase` | Move documents through workflow phases |
| `check_phase_transition` | Validate if a document can transition to a new phase |
| `validate_exit_criteria` | Check completion status of exit criteria |
| `list_documents` | Find documents by type, phase, or other criteria |
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

## Directory Structure

After running `initialize_project`, your project will have this structure:

```
your-project/
└── metis/
    ├── vision.md        # Initial vision document
    ├── strategies/      # Strategy documents will be created here
    ├── decisions/       # ADR (Architectural Decision Record) documents
    └── .metis.db       # SQLite database with FTS index
```

The `initialize_project` tool creates a clean `metis/` subdirectory to keep all project files organized. Documents are created as individual markdown files and automatically indexed in the SQLite database for fast search.

## Technical Overview

**Architecture**: Metis consists of two main components:
- **metis-core**: Rust library handling document management, workflows, and database
- **metis-mcp-server**: MCP server providing AI assistant integration

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