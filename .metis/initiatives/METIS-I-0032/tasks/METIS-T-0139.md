---
id: metis-web-dioxus-skeleton-rest
level: task
title: "metis-web Dioxus skeleton: REST client, token login, served at /"
short_code: "METIS-T-0139"
created_at: 2026-06-12T20:00:13.312362+00:00
updated_at: 2026-06-12T20:00:13.312362+00:00
parent: METIS-I-0032
blocked_by: ["METIS-T-0138"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# metis-web Dioxus skeleton: REST client, token login, served at /

## Parent Initiative

[[METIS-I-0032]]

## Objective

Stand up the Dioxus app crate (`crates/metis-web`): a WASM SPA with a REST client, token login, app shell + routing, and have `metis-server` serve the built bundle at `/`. The "you can log in and the app loads" slice — the foundation every view builds on.

## Acceptance Criteria

- [ ] `crates/metis-web` (Dioxus) builds to WASM via dioxus-cli/trunk; depends on `metis-types`
- [ ] REST client holding a bearer token (reqwest/gloo-net); typed against `metis-types` DTOs
- [ ] Login screen: paste a `mtk_` token → validate via `GET /api/v1/whoami` → store (localStorage/in-memory); invalid token shows an error
- [ ] App shell + `dioxus-router` routes for login, board (`/p/:project/board`), item (`/p/:project/i/:short_code`) — deep-link-ready (views fill in later tasks)
- [ ] `metis-server` serves the embedded WASM bundle at `/` (rust-embed), as a fallback **outside** the auth-protected `/api` + `/mcp`; index + assets load
- [ ] Manual verify: `metis serve --local` → open `/` → log in → `whoami` round-trips and the shell renders

## Implementation Notes

### Technical Approach
- Same origin as the API, so no CORS; token sent as `Authorization: Bearer` on each request.
- rust-embed embeds the dioxus build output into `metis-server`; document the build ordering (wasm bundle built before/with the server — a committed bundle or a build step for now; CI wiring is METIS-T-0143).
- Keep components target-agnostic from the start so the desktop target (METIS-T-0142) reuses them.

### Dependencies
- Blocked by METIS-T-0138 (`metis-types`). Blocks METIS-T-0140 (views) and METIS-T-0143 (CI/packaging).

### Risk Considerations
- dioxus-cli/trunk is a new build tool; pin its version and document setup.
- Token in localStorage is fine for a self-hosted tool; note it (XSS exposure) and revisit if needed.

## Status Updates

*To be added during implementation*