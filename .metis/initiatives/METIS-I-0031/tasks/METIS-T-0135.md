---
id: hosted-mcp-over-http-with-per
level: task
title: "Hosted MCP over HTTP with per-connection auth"
short_code: "METIS-T-0135"
created_at: 2026-06-12T02:30:20.626880+00:00
updated_at: 2026-06-12T02:48:52.158999+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Hosted MCP over HTTP with per-connection auth

## Parent Initiative

[[METIS-I-0031]]

## Objective

Let agents reach the work-item tools over HTTP against the hosted server — each connection authenticating with its own PAT and acting as its own user — instead of only the direct-DB stdio mode. This is the team-deployment MCP story: an agent in any repo points at `https://metis.team/mcp` with its token.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `POST /mcp` speaks MCP over Streamable HTTP (JSON-RPC): `initialize`, `tools/list`, `tools/call`, `ping`, `notifications/*` (202)
- [x] Per-connection auth: behind the bearer-token middleware → each request its own `Actor` (item attributed to the token's user, verified in test); 401 without token; local mode = implicit user
- [x] Tool list + dispatch shared with stdio (`mcp::call_tool` + `mcp::tool_list`); same 11 tools
- [x] Errors: unknown method → -32601; tool errors → `CallToolResult` `isError` (message preserved); bad JSON → -32700; non-object → -32600
- [x] `GET /mcp` → 405 (verified in local mode; auth-first means an unauthenticated GET is 401); `application/json`, stateless
- [x] Tests: oneshot initialize/tools-list/tools-call + gated-transition isError + 401 + 405 + -32601; live curl smoke against `metis serve` (401, 11 tools, create → METIS-T-0001)

## Non-Goals (this task)

- SSE streaming / server-initiated messages (`GET /mcp` is 405) — the tools are request/response; add SSE only if a future tool needs to stream.
- OAuth/JWT (the SDK's `auth` feature) — out of scope; we authenticate Metis PATs with the existing middleware.
- Strict MCP session management (`Mcp-Session-Id` lifecycle) — operate statelessly; revisit if a client requires it.

## Implementation Notes

### Technical Approach
- **Implement Streamable HTTP natively in axum**, not via the SDK's hyper-server. Rationale: the SDK's hosted transport is OAuth/JWT-oriented (`OauthTokenVerifier`, `AuthClaims`, introspection) and runs a standalone server — it neither matches the `mtk_` PAT model nor mounts into the existing axum app. We keep the SDK for tool **schemas** (`MetisItemTools::tools()`) and the `#[mcp_tool]`/`CallToolResult` types; we own the transport.
- Refactor `mcp::tools` dispatch into a shared `call_tool(&McpState, name, args) -> Result<CallToolResult, CallToolError>` + `tool_list()`; the stdio `ServerHandler` and the new HTTP handler both call them.
- `POST /mcp` handler: read `Extension<Actor>` (from auth middleware) → build `McpState` (pool/store/actor) → parse JSON-RPC → dispatch → JSON-RPC response (`application/json`). Mounted under the protected router so auth is enforced exactly like `/api/v1`.

### Dependencies
- METIS-T-0132 (stdio MCP, tools, McpState), T-0130 (auth middleware, AppState). No new SDK features.

### Risk Considerations
- Hand-rolling the JSON-RPC envelope is a small, well-defined surface (4 methods); keep it minimal and spec-aligned (single messages, not batches — batching was dropped from the spec).
- Keep stdio and HTTP behavior identical by routing both through the shared dispatch; avoid divergence.

## Status Updates

### 2026-06-12 — Complete (branch `feat/T-0135-hosted-mcp` off `initiative/METIS-I-0031`)

`POST /mcp` native streamable-HTTP transport in `metis-server::mcp::http`; shared `mcp::call_tool`/`mcp::tool_list`/`mcp::server_info` extracted from the stdio handler. 11 server tests + live curl smoke; full suite (72) green, clippy clean.

**Decisions / deviations:**
- **Native axum, not the SDK hyper-server.** Confirmed by reading the SDK source: its hosted auth is OAuth/JWT (`OauthTokenVerifier`, `AuthClaims`, introspection, JWT) and the hyper-server runs standalone — neither fits the `mtk_` PAT model nor mounts into our axum app. We reuse only the SDK's tool *schemas* + `CallToolResult`/`CallToolError` types and own the JSON-RPC transport. **No new SDK features enabled.**
- **Per-connection auth comes for free** by mounting `/mcp` under the same `route_layer(require_auth)` as `/api/v1` — each POST resolves its own `Actor`; the test asserts the created item's `created_by` is the token's user.
- **Tool errors are in-band** (`CallToolResult.isError`), not JSON-RPC errors — per MCP semantics; only protocol problems (unknown method, parse) are JSON-RPC errors.
- **Stateless**: no `Mcp-Session-Id` lifecycle, no SSE; `GET /mcp` → 405 (auth-first means an *unauthenticated* GET is 401, which the 405 test sidesteps via local mode). Fine for request/response tools; SSE is a later add if a tool needs to stream.
- The 3.0 plugin still ships `metis mcp` (stdio) by default; pointing it at the hosted endpoint is a config/doc change for a later polish (both transports share dispatch, so behavior is identical).