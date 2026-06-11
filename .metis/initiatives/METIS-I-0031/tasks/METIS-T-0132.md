---
id: mcp-endpoint-over-the-service-layer
level: task
title: "MCP endpoint over the service layer"
short_code: "METIS-T-0132"
created_at: 2026-06-11T18:14:15.973423+00:00
updated_at: 2026-06-11T19:08:03.038784+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# MCP endpoint over the service layer

## Parent Initiative

[[METIS-I-0031]]

## Objective

Expose the `metis-core` service layer to AI agents over MCP — the first piece of the METIS-I-0031 second tranche. Agents get the same work-item verbs they know from 2.x (renamed `document`→`item`), backed by the 3.0 server's database + object store and carrying proper actor attribution. Uses `rust-mcp-sdk` (same SDK as the frozen 2.x `metis-docs-mcp`).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `metis mcp` subcommand runs an MCP **stdio** server on the same `AppState` as `serve` (`--local` implicit user, or `--database-url` + `--token` → `Actor`)
- [x] Tool surface backed by the service layer: list_items, read_item, create_item, edit_item, transition_phase, link_item, archive_item, create_project, read_project (9 tools)
- [x] Tool shapes mirror REST/service DTOs and 2.x muscle memory; JSON results are the metis-core DTOs
- [x] Errors map through `ServiceError` into tool errors preserving message (not-found / validation / phase-gate / conflict)
- [x] Mutating tools record the `Actor` (inherited from the service layer)
- [x] Tests: tool-layer integration test (create→read→gated transition→edit→list) + live stdio JSON-RPC smoke verified

## Non-Goals (this task)

- **Hosted streamable-HTTP transport with per-connection PAT auth** — follow-up task; this task delivers stdio + a single resolved actor per session. (The tool/dispatch layer is written so the HTTP transport can wrap it later.)
- `search_items` free-text (needs FTS — separate task) and `briefing` (needs the repo registry — separate task). `list_items` structured filters cover querying for now.
- Plugin/session-hook rewrite to talk to the server (separate task).

## Implementation Notes

### Technical Approach
- New `mcp` module in `metis-server`: `#[mcp_tool]` structs + `tool_box!` aggregation + a `ServerHandler` dispatching by name, exactly the 2.x pattern (`crates/metis-docs-mcp/src/{server,tools}`), but each tool calls `metis-core::service::*` through a shared handler state holding `{ pool, store, actor }` and running blocking DAL on `spawn_blocking`/a blocking section.
- Add `rust-mcp-sdk` (stdio + macros + server), `schemars`, `async-trait` to `metis-server`.
- Reuse the REST request/DTO shapes where possible so REST and MCP stay in lockstep.

### Dependencies
- METIS-T-0129 (service layer), METIS-T-0130 (AppState/auth/config). Builds on the published diesel-dualdb 0.1.0.

### Risk Considerations
- rust-mcp-sdk async + sync diesel: keep DAL on a blocking boundary; the handler owns a `Pool` and resolves a connection per call.
- Tool naming is a compatibility surface for agent skills (Ralph) — pick the `item` names deliberately now; document the 2.x→3.0 rename.

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0132-mcp-endpoint` off `initiative/METIS-I-0031`)

`metis-server::mcp` module + `metis mcp` subcommand. 9 tools over stdio backed by the service layer; tool-layer test + live stdio JSON-RPC smoke both green; full suite (56 tests) + clippy clean.

**Decisions / deviations:**
- Tools are `#[mcp_tool]` structs whose `run(&McpState)` calls `metis-core::service::*`; the `ServerHandler` dispatches by name via a small macro. Matches the 2.x SDK pattern but state-backed instead of per-call workspace detection.
- **`ExitCriterionArg`** is an MCP-local schemars type mapped to `metis_core::models::ExitCriterion`, so metis-core stays free of a schemars dependency.
- **SQLite pools are now single-connection** (`db::connect` sets `max_size(1)` for SQLite) — single-writer DB; avoids lock contention. Discovered while smoke-testing: firing pipelined (un-awaited) tool calls raced create-then-read across pooled connections. That's expected JSON-RPC behavior (servers may process pipelined requests in any order); real clients await each response, and the sequential smoke + tool-layer test confirm correct behavior. Pool sizing doesn't impose ordering (nor should it).
- **Hosted streamable-HTTP transport + per-connection PAT auth is the explicit follow-up.** The tool/dispatch layer is transport-agnostic, so the HTTP transport wraps the same handler later. `search_items` (FTS) and `briefing` (repo registry) remain separate tranche tasks.