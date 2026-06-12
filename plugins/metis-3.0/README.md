# Metis 3.0 plugin (server-backed)

Connects Claude Code to a self-hosted **Metis 3.0** server. Unlike the 2.x
plugin (which read a per-repo `.metis/` directory), 3.0 keeps work in a central
server and discovers context from the repo's **git remote**.

## What it does

- **MCP** (`.mcp.json`): runs `metis mcp` (stdio), exposing the work-item tools
  — `list_items`, `read_item`, `create_item`, `edit_item`, `transition_phase`,
  `link_item`, `archive_item`, `resolve_repo`, `briefing`, `create_project`,
  `read_project`.
- **SessionStart hook** (`hooks/hooks.json` → `metis hook session-start`):
  detects the checkout's git remote, resolves it to a Metis project/repo, and
  injects the repo's active + ready work as session context. Silent when the
  repo isn't a git checkout or isn't registered (it prints a short "register
  this repo" notice in the latter case).

## Prerequisites

1. The `metis` 3.0 binary on `PATH` (`cargo install` / your distribution).
2. A client config at `~/.config/metis/config.toml`:

   ```toml
   database_url = "postgres://user:pass@metis-db.internal/metis"
   token        = "mtk_…"          # your personal access token (attribution)
   # default_project = "metis"     # optional
   ```

   Both `metis mcp` and `metis hook session-start` read this. For solo/offline
   use, run against a local SQLite file instead (`metis mcp --local`).

> **Note:** This plugin supersedes the file-based 2.x `metis` plugin; install
> one or the other. The Ralph loops and methodology skills from 2.x are ported
> separately.
