---
id: extract-metis-types-shared-dto
level: task
title: "Extract metis-types shared DTO crate"
short_code: "METIS-T-0138"
created_at: 2026-06-12T20:00:11.885142+00:00
updated_at: 2026-06-12T20:00:11.885142+00:00
parent: METIS-I-0032
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# Extract metis-types shared DTO crate

## Parent Initiative

[[METIS-I-0032]]

## Objective

Extract the plain wire DTOs into a new dependency-light `metis-types` crate so the server and the (WASM) UI share one type set. `metis-core` can't compile to WASM (diesel + bundled libsqlite3), so the UI can't depend on it; the DTOs must live in a crate that compiles to both native and `wasm32`. First slice of METIS-I-0032 — prerequisite for everything else.

## Acceptance Criteria

- [ ] New `crates/metis-types`: `serde` only (no diesel/tokio/db deps); compiles for both the host and `wasm32-unknown-unknown`
- [ ] Pure DTOs moved/defined there: `ItemSummary`, `ItemDetail`, `ExitCriterion`, `Link`, `ProjectConfig` + `ItemTypeConfig`, `ItemFilter` (with the `type` rename), `RepoContext`, `Briefing`, `View` — the shapes the REST API returns/accepts
- [ ] `metis-core` depends on `metis-types` and re-uses those types (no duplicate definitions); the workflow engine/validation logic stays in `metis-core`
- [ ] `metis-server` request/response structs reference `metis-types` where they mirror DTOs
- [ ] `cargo build -p metis-types --target wasm32-unknown-unknown` succeeds (wasm-safe)
- [ ] Full workspace builds; existing suite stays green

## Implementation Notes

### Technical Approach
- Move the *plain* types out of `metis-core::{models, workflow::config, service::{query,repo,view}}`; keep the Diesel `Queryable` row structs (`WorkItemRow`, etc.) in `metis-core` (they need diesel) — only the serde DTOs migrate.
- `ItemFilter`'s `#[serde(rename = "type")]` and defaults travel with the type.
- Re-export from `metis-core` (e.g. `pub use metis_types::ItemDetail`) to minimize churn at call sites.

### Dependencies
None — first task of the initiative. Blocks METIS-T-0139.

### Risk Considerations
- Keep `metis-types` genuinely deps-light — a stray diesel/tokio dep would break the wasm build and defeat the purpose. Add a CI check that it builds for `wasm32`.

## Status Updates

*To be added during implementation*