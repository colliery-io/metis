# Configuration Reference

Metis configuration is stored in `.metis/config.toml` and the SQLite database. Both are kept in sync.

## config.toml

Located at `.metis/config.toml` in every Metis project.

```toml
[project]
prefix = "PROJ"

[flight_levels]
initiatives_enabled = true
```

### [project]

| Key | Type | Description |
|-----|------|-------------|
| `prefix` | String | Short code prefix for all documents. 2-8 uppercase ASCII letters. Set during `metis init`. |

### [flight_levels]

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `initiatives_enabled` | Boolean | `true` | Enable initiative documents. `true` = streamlined preset, `false` = direct preset. |

Note: Older workspaces (v1) may contain a `strategies_enabled` field. This is a legacy field that is parsed but ignored for backward compatibility.

## Flight Level Presets

| Preset | `initiatives_enabled` | Hierarchy |
|--------|----------------------|-----------|
| `streamlined` | `true` | Vision → Initiative → Task |
| `direct` | `false` | Vision → Task |

Both presets always include: Vision, ADR, Specification.

## Environment Variables

| Variable | Used By | Description |
|----------|---------|-------------|
| `METIS_LOG_LEVEL` | MCP server | Log level for the MCP server. Set by `metis mcp --log-level`. Values: `trace`, `debug`, `info`, `warn`, `error`. |
| `METIS_VERSION` | Install script | Pin a specific version for the install script instead of using the latest release. |

## Database Configuration

The SQLite database at `.metis/metis.db` stores:

| Table | Purpose |
|-------|---------|
| `documents` | All document metadata and content |
| `document_relationships` | Parent-child links |
| `document_tags` | Tags per document |
| `document_search` | FTS5 full-text search index |
| `configuration` | Key-value configuration store |

### Database Constants

| Constant | Value | Description |
|----------|-------|-------------|
| Connection timeout | 30 seconds | Maximum wait for database lock |
| Max retries | 3 | Retry count for transient errors |
| Max file size | 10 MB | Maximum document file size |
| Backup retention | 30 days | How long `.metis/metis.db.backup` is kept |

## File Conventions

| Path | Purpose |
|------|---------|
| `.metis/` | Root workspace directory |
| `.metis/metis.db` | SQLite database |
| `.metis/metis.db.backup` | Database backup |
| `.metis/metis-mcp-server.log` | MCP server logs |
| `.metis/config.toml` | Project configuration |
| `.metis/vision.md` | Vision document |
| `.metis/initiatives/` | Initiative documents |
| `.metis/tasks/` | Task documents (backlog) |
| `.metis/adrs/` | Architecture Decision Records |
| `.metis/specifications/` | Specification documents |
| `.metis/archived/` | Archived documents |
| `.metis/code-index.md` | Generated code index |
| `.metis/code-index-hashes.json` | File content hashes for incremental indexing |
| `.metis/code-index-symbols.json` | Cached extracted symbols |
| `.metis/.index-dirty` | Flag file indicating index needs refresh |
| `.metis/.gitignore` | Git ignore rules for database and cache files |

## .gitignore (Auto-Generated)

The `.metis/.gitignore` file excludes:

```
metis.db
metis.db-shm
metis.db-wal
metis-mcp-server.log
code-index.md
code-index-hashes.json
code-index-symbols.json
.index-dirty
```

This ensures the database, logs, generated indexes, and cache files are not committed to version control. The document markdown files (vision.md, initiative.md, task.md, etc.) and config.toml are version-controlled.

## MCP Server Configuration

The MCP server is registered in Claude Code as:

```bash
claude mcp add --scope user metis metis mcp
```

This creates an entry in Claude Code's MCP configuration:

```json
{
  "mcpServers": {
    "metis": {
      "command": "metis",
      "args": ["mcp"],
      "env": {}
    }
  }
}
```

## Claude Code Plugin Configuration

The Metis plugin for Claude Code is installed via:

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
```

### Ralph Loop State

Ralph loop state is stored at `.claude/metis-ralph.local.md`:

```yaml
---
active: true
mode: task          # or "decompose"
short_code: "PROJ-T-0001"
project_path: "/path/to/.metis"
iteration: 1
max_iterations: 20
completion_promise: "TASK COMPLETE"
started_at: "2026-03-25T14:30:00Z"
---
```

| Field | Description |
|-------|-------------|
| `active` | Whether the loop is currently running |
| `mode` | `task` for task execution, `decompose` for initiative decomposition |
| `short_code` | Document being worked on |
| `project_path` | Path to the `.metis` directory |
| `iteration` | Current iteration count |
| `max_iterations` | Safety limit (0 = unlimited) |
| `completion_promise` | Signal text that allows the loop to exit |
| `started_at` | ISO 8601 timestamp of loop start |

## GUI Configuration

The desktop app stores configuration in the OS-specific application data directory:

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/io.colliery.metis/` |
| Linux | `~/.config/io.colliery.metis/` |
| Windows | `%APPDATA%/io.colliery.metis/` |

Contents:
- `bin/metis` — Bundled CLI binary
- `cli-version.json` — CLI version tracking for auto-updates

### GUI Preferences (localStorage)

| Key | Values | Description |
|-----|--------|-------------|
| `metis-theme` | `light`, `dark`, `hyper` | Selected UI theme |
| `metis-recent-projects` | JSON array | List of recently opened project paths |
