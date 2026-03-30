# CLI Reference

The `metis` CLI provides commands for initializing projects, managing documents, and running the MCP server.

**Binary name:** `metis`

## Global Options

| Flag | Description |
|------|-------------|
| `-v, --verbose` | Increase verbosity. Repeat for more detail: `-v` (INFO), `-vv` (DEBUG), `-vvv` (TRACE). Default: WARN. |
| `--help` | Show help |
| `--version` | Show version |

---

## metis init

Initialize a new Metis workspace in the current directory.

```
metis init [OPTIONS]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `-n, --name <NAME>` | String | "Project Vision" | Project name for the vision document |
| `-P, --prefix <PREFIX>` | String | Auto-generated | Short code prefix (2-8 uppercase ASCII letters, e.g., "PROJ"). CLI truncates to 6 characters. |
| `-p, --preset <PRESET>` | String | "streamlined" | Configuration preset: `streamlined` or `direct` |
| `--initiatives <BOOL>` | Boolean | — | Enable/disable initiatives (overrides preset) |

**Creates:**
- `.metis/` directory
- `.metis/vision.md` — Default vision document
- `.metis/config.toml` — Project configuration
- `.metis/metis.db` — SQLite database
- `.metis/.gitignore` — Ignores database, logs, index cache, and overlay directory
- `.git/hooks/post-commit` — Git hook for automatic overlay flush (if in a git repo)

**Examples:**
```bash
metis init --name "My Project" --prefix "PROJ"
metis init --preset direct
metis init --initiatives false
```

---

## metis sync

Synchronize the workspace filesystem with the database.

```
metis sync
```

No arguments. Walks the `.metis/` directory and:

1. Checks if database recovery is needed
2. Syncs `config.toml` to database
3. Imports new documents, updates modified ones, removes deleted ones

**Sync result codes:**
| Code | Meaning |
|------|---------|
| `[+] Imported` | New document added to database |
| `[+] Updated` | Existing document modified |
| `[+] Deleted` | Document removed from database |
| `[.] Up to date` | No changes detected |
| `[?] Not found` | File disappeared |
| `[-] Error` | Sync error |
| `[>] Moved` | Document relocated |
| `[!] Renumbered` | Short code changed |

---

## metis create

Create new documents. Has four subcommands.

### metis create initiative

```
metis create initiative <TITLE> --vision <SHORT_CODE>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<TITLE>` | String | Yes | Initiative title |
| `-v, --vision <SHORT_CODE>` | String | Yes | Parent vision short code |

Prompts interactively for complexity level:
- `S` — Small (1-3 days)
- `M` — Medium (1-2 weeks) *(default)*
- `L` — Large (2-4 weeks)
- `XL` — Extra Large (1+ months)

**Example:**
```bash
metis create initiative "Q1 Product Launch" --vision PROJ-V-0001
```

### metis create task

```
metis create task <TITLE> --initiative <SHORT_CODE>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<TITLE>` | String | Yes | Task title |
| `-i, --initiative <SHORT_CODE>` | String | Yes | Parent initiative short code |

Tasks start in `todo` phase.

**Example:**
```bash
metis create task "Implement auth" --initiative PROJ-I-0001
```

### metis create adr

```
metis create adr <TITLE>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<TITLE>` | String | Yes | ADR title |

ADRs are auto-numbered (PROJ-A-0001, PROJ-A-0002, ...) and start in `draft` phase.

**Example:**
```bash
metis create adr "Use PostgreSQL for primary database"
```

### metis create specification

```
metis create specification <TITLE> --parent <SHORT_CODE>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<TITLE>` | String | Yes | Specification title |
| `-p, --parent <SHORT_CODE>` | String | Yes | Parent short code (Vision or Initiative) |

Starts in `discovery` phase.

**Example:**
```bash
metis create specification "API Contract v2" --parent PROJ-V-0001
```

---

## metis list

List documents with optional filtering.

```
metis list [OPTIONS]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `-t, --document-type <TYPE>` | String | — | Filter by type: `vision`, `initiative`, `task`, `adr`, `specification` |
| `-p, --phase <PHASE>` | String | — | Filter by phase (e.g., `active`, `draft`, `completed`) |
| `-a, --all` | Flag | false | Show all documents |
| `--include-archived` | Flag | false | Include archived documents |
| `-f, --format <FORMAT>` | String | `table` | Output format: `table`, `compact`, `json` |

**Examples:**
```bash
metis list                                    # All non-archived documents
metis list -t task -p active                  # Active tasks only
metis list --include-archived -f json         # All documents as JSON
metis list -t initiative --format compact     # Initiatives in compact format
```

**Output formats:**

`table` — Human-readable table:
```
Type           Code           Title                   Phase
vision         PROJ-V-0001    Company Vision 2025     published
task           PROJ-T-0001    Implement auth          active
```

`compact` — One line per document (for scripts), format: `CODE PHASE TITLE`:
```
PROJ-V-0001 published Company Vision 2025
PROJ-T-0001 active Implement auth
```

`json` — JSON array:
```json
[{"type":"vision","code":"PROJ-V-0001","title":"Company Vision 2025","phase":"published"}]
```

---

## metis search

Full-text search across document content and titles.

```
metis search <QUERY> [OPTIONS]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<QUERY>` | String | Yes | Search text |
| `-l, --limit <N>` | usize | 20 | Maximum results |
| `-f, --format <FORMAT>` | String | `table` | Output format: `table`, `compact`, `json` |

**Example:**
```bash
metis search "authentication" -l 5 -f json
```

---

## metis status

Show workspace status sorted by actionability.

```
metis status [OPTIONS]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--include-archived` | Flag | false | Include archived documents |
| `-f, --format <FORMAT>` | String | `table` | Output format: `table`, `compact`, `json` |

Documents are sorted by actionability priority: Blocked (highest) > Todo > Active > Other phases. Within each priority level, most recently updated documents appear first.

Output includes phase insights (counts of active, todo, blocked documents) and blocked-by information.

---

## metis transition

Transition a document to a new phase.

```
metis transition <SHORT_CODE> [PHASE]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<SHORT_CODE>` | String | Yes | Document short code |
| `[PHASE]` | String | No | Target phase. If omitted, auto-advances to next phase. |

Phase names are case-insensitive. Only adjacent transitions are valid — you cannot skip phases.

**Examples:**
```bash
metis transition PROJ-V-0001 review          # Explicit target
metis transition PROJ-T-0001                  # Auto-advance to next phase
metis transition PROJ-T-0001 blocked          # Move to blocked
```

See [Phase Lifecycle Reference](./phase-lifecycle.md) for valid transitions per document type.

---

## metis archive

Archive a document and all its children.

```
metis archive <SHORT_CODE> [OPTIONS]
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<SHORT_CODE>` | String | Yes | Document short code |
| `-t, --document-type <TYPE>` | String | Auto-detected | Document type hint |

Moves documents to `.metis/archived/` and marks them as archived in the database.

**Example:**
```bash
metis archive PROJ-I-0001                    # Archives initiative and all child tasks
```

---

## metis validate

Validate a document file's structure.

```
metis validate <FILE_PATH>
```

| Argument | Type | Required | Description |
|----------|------|----------|-------------|
| `<FILE_PATH>` | Path | Yes | Path to document file |

Checks: required frontmatter fields, recognized document type, valid phase tag, proper parent references.

**Example:**
```bash
metis validate .metis/vision.md
```

---

## metis config

Manage flight level configuration. Has three subcommands.

### metis config show

Display current configuration, hierarchy, and available document types.

```
metis config show
```

### metis config set

Update configuration.

```
metis config set [OPTIONS]
```

| Option | Type | Description |
|--------|------|-------------|
| `-p, --preset <PRESET>` | String | `streamlined` or `direct` |
| `--initiatives <BOOL>` | Boolean | Enable/disable initiatives |

### metis config get

Query a specific setting.

```
metis config get <KEY>
```

| Argument | Type | Description |
|----------|------|-------------|
| `<KEY>` | String | Configuration key (e.g., `preset`, `initiatives_enabled`) |

---

## metis mcp

Launch the MCP server for AI integration.

```
metis mcp [OPTIONS]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--log-level <LEVEL>` | String | `info` | Log level: `trace`, `debug`, `info`, `warn`, `error` |

Runs the MCP server on stdio (JSON-RPC). The CLI process becomes the server — it does not return until the server shuts down. Logs are written to `.metis/metis-mcp-server.log` if a workspace is detected.

**Example:**
```bash
metis mcp --log-level debug
```

Typically configured as an MCP server in Claude Code:
```bash
claude mcp add --scope user metis metis mcp
```

---

## metis index

Generate a code index for AI agent navigation.

```
metis index [OPTIONS]
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `--structure-only` | Flag | false | Skip symbol extraction, only generate directory tree |
| `--incremental` | Flag | false | Only re-index changed files (uses content hashes) |

Walks the project directory, parses source files with tree-sitter, extracts symbols, and writes `.metis/code-index.md`.

Supports: Rust, Python, TypeScript, JavaScript, Go.

Incremental mode uses:
- `.metis/code-index-hashes.json` — BLAKE3 file content hashes
- `.metis/code-index-symbols.json` — Cached extracted symbols

**Examples:**
```bash
metis index                              # Full index
metis index --incremental                # Only changed files
metis index --structure-only             # Directory tree only
metis index --incremental --structure-only
```

---

## metis flush

Flush pending `.metis/` overlay changes to the main branch.

```
metis flush
```

No arguments. This command is the core of Metis's branch-independent storage:

- When working on a **feature branch**, Metis document reads come from main's git tree and writes go to a `.metis/.pending/` overlay directory
- `metis flush` takes all pending overlay changes and commits them to main as a single commit using git plumbing — **without checking out main**
- Tombstone files (`.md.deleted`) in the overlay cause the corresponding file to be removed from main's tree
- After a successful flush, `.metis/.pending/` is cleaned up
- If there's nothing to flush, the command is a silent no-op
- When already on main, the command is a no-op (writes go directly to the filesystem)

**Commit message:** `metis: sync document changes`

**Automatic execution:** A `post-commit` git hook calls `metis flush` after every commit, so pending document changes are automatically flushed to main whenever you commit code. The hook is installed automatically by `metis init` or lazily on the first Metis CLI operation in a git repo.

**Example:**
```bash
metis flush                              # Manual flush (usually not needed)
```
