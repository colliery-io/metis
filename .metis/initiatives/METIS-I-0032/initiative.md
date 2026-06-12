---
id: metis-3-0-web-and-desktop-ui-dioxus
level: initiative
title: "Metis 3.0 Web and Desktop UI (Dioxus)"
short_code: "METIS-I-0032"
created_at: 2026-06-12T14:09:55.523348+00:00
updated_at: 2026-06-12T20:00:06.031144+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
initiative_id: metis-3-0-web-and-desktop-ui-dioxus
---

# Metis 3.0 Web and Desktop UI (Dioxus) Initiative

## Context

Split out of METIS-I-0031 (the server backbone) on 2026-06-12: the server, REST `/api/v1`, hosted + stdio MCP, auth, repo registry/briefing, search, and saved views are built and pushed on `initiative/METIS-I-0031`. The human-facing UI is a large, distinct body of work (a new frontend stack, a wasm toolchain, two render targets) and gets its own initiative so I-0031 can close on its server-side remainder (markdown export, skills/commands port).

This initiative builds the UI on top of the existing REST API; it depends on that API surface (already shipped) but not on I-0031's remaining items.

## Goals & Non-Goals

**Goals:**
- An all-Rust **Dioxus** UI delivered as two targets from one component crate: `dioxus-web` (WASM) embedded in `metis-server` at `/`, and `dioxus-desktop` (wry/tao webview) as the desktop app that replaces the retired 2.x Tauri GUI
- Extract a dependency-light `metis-types` crate (plain serde DTOs) shared by `metis-core`, `metis-server`, and the UI — one type set across server and client (metis-core can't compile to WASM)
- MVP, read-first then edit: token login; project board (items grouped by phase); item detail (rendered markdown body, links, exit criteria); create/edit item; free-text search (`?q=`); saved views
- Server serves the embedded WASM bundle at `/` (a static-file fallback outside the auth-protected `/api` + `/mcp`)
- Shareable deep links: `/p/:project/i/:short_code`, board at `/p/:project/board`

**Non-Goals:**
- Reusing the 2.x Vue/TipTap frontend (decided against — see Alternatives); the Tauri GUI crate is retired
- A rich WYSIWYG editor — markdown is a textarea + rendered preview (a wasm markdown renderer); TipTap-class editing is out
- Real-time/multiplayer (presence, live cursors), notifications, charts/burnups beyond simple counts
- Auth beyond pasting a PAT (SSO/session niceties follow the server's auth roadmap)
- Mobile-specific layouts

## Detailed Design

Carried from METIS-I-0031 design D6 (the decision record):

- **Stack:** one `dioxus` component crate, likely `crates/metis-web`, rendering to `dioxus-web` (WASM, embedded via `rust-embed`) and `dioxus-desktop`. No npm/Vite; cargo + dioxus-cli/trunk for the wasm build.
- **Shared types:** new `crates/metis-types` (serde-only DTOs: `ItemSummary`, `ItemDetail`, `ProjectConfig`, `ItemFilter`, repo/briefing/view payloads). `metis-core` re-exports/depends on it; the UI depends on it directly. This is the first slice — a mechanical but cross-cutting refactor.
- **Data:** the UI is a REST client (bearer token in a header; token entered at login and held client-side). Rendering runs in the browser/webview, keeping server CPU to data work (aligns with the METIS-T-0137 noisy-neighbor posture).
- **Markdown:** render with a wasm-compatible renderer (`pulldown-cmark`/`comrak`); edit via textarea + live preview.
- **Server change:** add a static-file fallback at `/` serving the embedded bundle, kept outside the auth layer that guards `/api` + `/mcp`.

## Alternatives Considered

- **Reuse the 2.x Vue + Vite + Tailwind + TipTap SPA** (the original D6 plan): fastest to a rich UI and reuses the existing editor, but drags an npm/Vite toolchain into the repo and duplicates the Rust DTOs in TS. Rejected for the all-Rust, single-toolchain path.
- **Server-rendered HTML + htmx** (maud/askama): lightest to build/maintain and a great fit for a no-frills tracker, but **can't be shipped as a desktop app** without wrapping a browser, which was a hard requirement. Rejected on the desktop constraint.
- **Dioxus (chosen):** single language/toolchain, shared DTOs, and web + desktop from one codebase. Costs: less mature ecosystem, building board/forms from scratch, no TipTap, a wasm build step.

## Implementation Plan

To be decomposed (this initiative is in discovery pending human sign-off). Expected rough slices:
1. `metis-types` extraction — move DTOs out of `metis-core`; `metis-core`/`metis-server` consume it; suite stays green
2. `metis-web` Dioxus skeleton + REST client + token login; server static-file fallback serving the embedded bundle at `/`
3. Project board (items by phase) + item detail (rendered markdown, links, exit criteria)
4. Create/edit item; search box; saved views
5. `dioxus-desktop` target (same components) replacing the Tauri GUI; retire `crates/metis-docs-gui`
6. CI: wasm build + embed in the `3.0` workflow; packaging