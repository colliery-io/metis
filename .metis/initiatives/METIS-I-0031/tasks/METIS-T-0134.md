---
id: session-hook-and-3-0-plugin-server
level: task
title: "Session hook and 3.0 plugin (server-backed)"
short_code: "METIS-T-0134"
created_at: 2026-06-12T01:58:22.976827+00:00
updated_at: 2026-06-12T02:18:11.358724+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Session hook and 3.0 plugin (server-backed)

## Parent Initiative

[[METIS-I-0031]]

## Objective

The client half of initiative design D5/D8: when an agent opens a session in a repo checkout, detect the git remote, resolve it to a Metis project/repo via the server, fetch the repo-scoped briefing, and inject it as session context — the same-shaped briefing the 2.x hook produced, now server-backed. Ship a 3.0 Claude Code plugin wiring the MCP server + this hook.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] User config: `~/.config/metis/config.toml` (XDG-aware) with `database_url`/`token`/`default_project`; `metis mcp` falls back to it for `--database-url`/`--token`; unknown fields ignored
- [x] `metis hook session-start` subcommand: detects git remote (origin, else first), resolves, briefs, prints SessionStart JSON with active/ready items + item-vocabulary MCP tool list
- [x] Graceful degradation: silent on no-config / no-git / no-remote; one-line "register this repo" notice on unmatched remote; errors → silent exit 0
- [x] Hook logic is `hook::session_start(cwd, &UserConfig) -> Option<String>` (lib fn), tested without a stdio session
- [x] 3.0 plugin `plugins/metis-3.0/`: `plugin.json`, `.mcp.json` (`metis mcp`), `hooks/hooks.json` (SessionStart → `metis hook session-start`), README with config/setup
- [x] Tests: user-config parse; lib hook on temp git repo + temp DB (registered → briefing, unregistered → notice, no-config + non-git → silent); CLI verified live against a running server

## Non-Goals (this task)

- Porting the 2.x skills/commands (Ralph, decomposition, etc.) to 3.0 — separate task; this delivers the MCP wiring + session hook + core instructions.
- The hosted streamable-HTTP MCP transport (separate); the plugin's `.mcp.json` uses `metis mcp` stdio against the configured database/token (the T-0132 model).
- `PostToolUse`/`PreCompact`/`Stop` hooks (Ralph-loop machinery) — carried in the skills/commands port.

## Implementation Notes

### Technical Approach
- `metis-server`: a `UserConfig` loader (`~/.config/metis/config.toml`, honoring `XDG_CONFIG_HOME`), a `hook` module with `session_start(cwd, &UserConfig) -> Option<String>` (git remote via `git -C <cwd> remote get-url`, then `repo::resolve` + `repo::briefing` through a pool), and a `metis hook session-start` CLI subcommand that prints the JSON. `metis mcp` gains the same config fallback for `--database-url`/`--token`.
- Plugin: thin — `.mcp.json` → `metis mcp`; `hooks.json` SessionStart → `metis hook session-start`; instructions adapted from the 2.x hook text but using item/short-code vocabulary and the server model.

### Dependencies
- METIS-T-0133 (resolve + briefing), T-0132 (`metis mcp`, McpState/config), T-0130 (config/auth).

### Risk Considerations
- DB-direct model: the hook/`metis mcp` connect straight to Postgres with a token resolved for attribution (no HTTP server needed yet). When the hosted-HTTP MCP lands, the config grows a `server_url` and the hook can prefer HTTP. Keep `UserConfig` forward-compatible (extra fields ignored).
- Hook must never break a session: any failure (no git, no config, server down) exits 0 with at most a notice.

## Status Updates

### 2026-06-12 — Complete (branch `feat/T-0134-session-hook` off `initiative/METIS-I-0031`)

`metis-server::{user_config, hook}` + `metis hook session-start` subcommand + `plugins/metis-3.0/`. 5 hook tests + live CLI verification; full suite green, clippy clean.

**Decisions / deviations:**
- **DB-direct, like `metis mcp`** (T-0132 model): the hook connects straight to the configured database and resolves the token to an actor for attribution; no running HTTP server is needed. `UserConfig` is forward-compatible (extra fields ignored) so a `server_url` can be added when the hosted-HTTP MCP transport lands.
- **Hook command points directly at `metis hook session-start`** in `hooks.json` (no shell wrapper) — `metis` reads stdin-free, uses CWD for git detection, and self-silences on any failure.
- **Plugin lives at `plugins/metis-3.0/`** (name `metis-3.0`, v3.0.0-dev) rather than overwriting the shipped 2.x `plugins/metis`. They're install-one-or-the-other; the 3.0 plugin will supersede the 2.x one at release. README documents the required `~/.config/metis/config.toml`.
- **Skills/commands (Ralph, decomposition, …) NOT ported** — explicit non-goal; this is MCP wiring + session hook + core instructions only. `PostToolUse`/`PreCompact`/`Stop` (Ralph machinery) come with that port.
- Verified end-to-end with the real binary: `serve --local` → register repo + activate item via REST → `metis hook session-start` in a checkout emits the briefing; unregistered → notice; non-git → silent.