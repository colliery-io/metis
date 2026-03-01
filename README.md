# Metis

<div align="center">
  <img src="crates/metis-docs-gui/src-tauri/icons/icon.png" alt="Metis Owl Logo" width="128" height="128">
  <br><br>
  <strong>Persistent, structured project management for AI coding agents.</strong>
  <br>
  Give your AI assistant a memory that survives sessions, context windows, and compaction.
</div>

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
```

This installs the Metis desktop app. **Launch it once** -- on first launch it installs the `metis` CLI (which includes the MCP server) to your system PATH. After that, you're ready to connect Claude Code.

<details>
<summary>Other install methods</summary>

**Manual download** from [GitHub Releases](https://github.com/colliery-io/metis/releases/latest):
- macOS (Apple Silicon): `Metis_x.x.x_aarch64.dmg`
- macOS (Intel): `Metis_x.x.x_x64.dmg`
- Windows: `Metis_x.x.x_x64-setup.exe`
- Linux: `Metis_x.x.x_amd64.AppImage`

**macOS note**: You may need to run `sudo xattr -rd com.apple.quarantine "/Applications/Metis.app"` after installing.
</details>

## Why Metis?

AI coding agents lose context constantly -- not just between sessions, but during them. Context compaction, token limits, and session boundaries all erase the plan, the progress, and the reasoning behind decisions already made.

Metis is a **memory system** for AI agents. It gives them a persistent, file-backed place to generate work, track progress, and record decisions -- so nothing is lost when context shifts. It uses the [Flight Levels](https://www.flightlevels.io/) methodology to add opinionated structure around *how* work is created and tracked, so your AI doesn't just remember what to do, it works through it intentionally.

- **Persistent across everything.** Progress, decisions, and next steps live in markdown files that survive compaction, session restarts, and context window limits. Your AI reads the task and picks up where it left off.
- **Intentional work generation.** Instead of jumping straight to code, Metis structures work top-down: Vision > Initiative > Task. Each level has phases that enforce thinking before doing -- discovery before design, design before decomposition.
- **Human-in-the-loop where it matters.** Strategic decisions (initiative direction, architectural choices, task breakdown) require your approval. Tactical execution can be autonomous. You control the "what," the AI handles the "how."
- **Plain markdown, git-friendly.** Documents are markdown with YAML frontmatter in a `.metis/` directory. No lock-in, no proprietary formats. Read them in any editor, commit them with your code, diff them in PRs.
## How It Works

### 1. Set up

After launching the Metis app once (so the CLI is installed), add the MCP server and plugin:

```bash
claude mcp add --scope user --transport stdio metis -- metis mcp
```

Then inside Claude Code:

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
```

### 2. Start working

With the plugin installed, Metis detects your project automatically. Just talk to Claude:

> **You:** "Initialize Metis for this project."

Claude creates the `.metis/` directory and works with you to write a vision statement -- the purpose and direction of the project. Even for an existing codebase, the vision captures *where the project is headed* so all future work traces back to it.

> **You:** "I want to add real-time collaboration to this app."

Claude creates initiatives under the vision. You discuss scope, approve the direction, and shape what gets built:

```
"Build a real-time collaboration platform"     # Vision - the why
├── "Implement WebSocket infrastructure"       # Initiative - a project
├── "Build presence system"                    # Initiative
└── "Design conflict resolution"               # Initiative
```

> **You:** "Break down the WebSocket initiative into tasks."

Claude proposes a task breakdown, you review and adjust, and Claude creates the tasks:

```
"Implement WebSocket infrastructure"
├── "Set up WebSocket server"                  # Task
├── "Add connection pooling"                   # Task
└── "Write integration tests"                  # Task
```

> **You:** "/metis-ralph PROJ-T-0001"

Claude picks up the task, transitions it to active, and works on it autonomously -- writing code, running tests, updating the task document with progress. When it's done, it signals for your review.

### 3. Tasks as working memory

This is the key idea. While a task is active, the AI continuously updates it with:
- What it completed, what files it changed
- What it discovered (unexpected patterns, blockers)
- Decisions made and why
- What's left to do

If the session ends or context compacts, the next session reads the task and picks up seamlessly. No work is lost.

## Flight Level Configurations

Choose the right level of structure for your project:

**Streamlined** (default) -- good for most projects:
```
Vision → Initiative → Task
```

**Full** -- for complex, multi-team efforts:
```
Vision → Strategy → Initiative → Task
```

**Direct** -- for simple, solo work:
```
Vision → Task
```

Set during init or change later:
```bash
metis init --name "My Project" --preset full
metis config set --preset direct
```

### Document Phases

Each document type progresses through defined phases:

| Type | Phases |
|------|--------|
| **Vision** | draft > review > published |
| **Strategy** | shaping > design > ready > active > completed |
| **Initiative** | discovery > design > ready > decompose > active > completed |
| **Task** | todo > active > completed (+ blocked) |
| **ADR** | draft > discussion > decided > superseded |

Phases are forward-only. You can't skip from `todo` to `completed` -- the task must go through `active` first. This prevents cutting corners.

## MCP Tools

Claude interacts with Metis through MCP tools:

| Tool | Purpose |
|------|---------|
| `initialize_project` | Create a `.metis/` workspace |
| `create_document` | Create vision, initiative, task, ADR, or backlog item |
| `read_document` | Read document content by short code |
| `edit_document` | Update document content |
| `transition_phase` | Advance through workflow phases |
| `list_documents` | List all documents with filtering |
| `search_documents` | Full-text search across documents |
| `archive_document` | Archive completed documents and children |
| `reassign_parent` | Move tasks between initiatives or to/from backlog |
| `index_code` | Generate a code index for AI codebase navigation |

Every document gets a unique short code (e.g., `PROJ-T-0042`) used to reference it in all operations.

## CLI

The `metis` CLI works directly from your terminal too:

```bash
metis init --name "Project Vision"                              # Initialize
metis create initiative "Auth System" --vision "project-vision"  # Create documents
metis list --type task                                           # List documents
metis transition "PROJ-T-0001"                                   # Advance phase
metis search "authentication"                                    # Full-text search
metis status                                                     # Project overview
metis index                                                      # Generate code index
metis index --incremental                                        # Re-index changed files only
```

## Desktop GUI

A visual kanban interface for managing projects. Boards for each document type, a rich markdown editor, drag-and-drop phase transitions, and multi-project support. Installed automatically via the install script.

## Plugin Details

**`/metis-ralph <SHORT_CODE>`** -- Autonomous task execution loop. Claude picks up a task, works on it iteratively (writing code, running tests, fixing issues), updates the task document with progress, and signals when done for your review. Use `/cancel-metis-ralph` to stop a running loop.

The plugin also includes **skills** (contextual methodology guidance for decomposition, phase transitions, and project setup) and **hooks** (automatic project detection on session start, state re-injection after context compaction).

See the [full plugin documentation](docs/claude-code-plugin.md) for details.

## Code Indexing

Metis can generate a structured codebase map at `.metis/code-index.md` for AI agent navigation. It uses tree-sitter to extract symbols (functions, structs, traits, classes, interfaces) across five languages: **Rust, Python, TypeScript, JavaScript, and Go**.

The index has two layers:

1. **Structural (automated)** -- deterministic symbol extraction with file paths, signatures, visibility, and line numbers. Generated by `metis index` or the `index_code` MCP tool.
2. **Semantic (AI-generated)** -- module-level summaries explaining what code *means*, not just what symbols exist. Generated by the plugin's code-index skill using a background agent.

```bash
metis index                # Full index
metis index --incremental  # Re-index only changed files (uses BLAKE3 hashing)
```

**Incremental indexing** tracks file content hashes so only modified files are re-parsed. On large codebases this takes seconds instead of minutes. The plugin automatically tracks which files you edit and triggers incremental re-indexing before context compaction, so the index stays fresh without manual intervention.

AI-generated summaries are preserved across re-indexing -- only new or changed modules get placeholder text.

## Custom Templates

Documents are generated from [Tera](https://tera.netlify.app/) templates with a fallback chain:

1. **Project**: `.metis/templates/{type}/content.md` (highest priority)
2. **Global**: `~/.config/metis/templates/{type}/content.md`
3. **Built-in defaults**

Available variables include `{{ title }}`, `{{ slug }}`, `{{ short_code }}`, `{{ created_at }}`, and type-specific variables like `{{ parent_title }}` for tasks.

## Architecture

Metis is built in Rust with six crates:

| Crate | Role |
|-------|------|
| `metis-docs-core` | Domain logic, document management, SQLite + FTS5 search |
| `metis-docs-cli` | CLI binary (`metis`), bundled with GUI installer |
| `metis-docs-mcp` | MCP server for AI assistant integration |
| `metis-docs-gui` | Desktop app (Tauri + Vue 3), auto-installs CLI on first launch |
| `metis-code-index` | Tree-sitter code extraction and indexing engine |
| `metis-docs-tui` | Terminal UI (deprecated) |

Documents are markdown files with YAML frontmatter, indexed in SQLite with FTS5 for fast full-text search. The filesystem is the source of truth; the database is a disposable cache.

## Development

```bash
angreal test      # Run all tests
angreal build     # Build all crates
angreal check     # Clippy + format + check
angreal coverage  # Generate coverage report
angreal gui       # Launch GUI in dev mode
```

## Contributing

Apache 2.0 License. Contributions welcome:

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure `angreal check` and `angreal test` pass
5. For GUI changes, include a written test journey
6. Submit a pull request
