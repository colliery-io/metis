---
id: ci-wasm-build-embed-ui-packaging
level: task
title: "CI: WASM build + embed; UI packaging"
short_code: "METIS-T-0143"
created_at: 2026-06-12T20:00:18.009767+00:00
updated_at: 2026-06-12T20:00:18.009767+00:00
parent: METIS-I-0032
blocked_by: ["METIS-T-0139"]
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0032
---

# CI: WASM build + embed; UI packaging

## Parent Initiative

[[METIS-I-0032]]

## Objective

Make the UI build reproducibly: build the WASM bundle in CI, verify the server embeds and serves it at `/`, and wire the build into the release/Docker path. Closes the loop so the embedded UI ships with the binary.

## Acceptance Criteria

- [ ] `ci-3.0.yml` installs the pinned dioxus-cli/trunk and builds `metis-web` to WASM
- [ ] A smoke check confirms `metis-server` serves the bundle: `GET /` returns the app `index.html`/assets (200)
- [ ] The Docker image / release build includes the built UI bundle (server serves `/` in the running container)
- [ ] The desktop target builds in CI (at minimum compiles; full artifact packaging optional)
- [ ] The wasm-target check for `metis-types` (from METIS-T-0138) runs in CI
- [ ] Build pipeline documented (wasm → embed → server; desktop)

## Implementation Notes

### Technical Approach
- Order in CI: build wasm bundle → build/embed server → run the `/` smoke. Cache the dioxus toolchain.
- Update `crates/metis-server/Dockerfile` to build the UI bundle in the builder stage before compiling the server (multi-stage), or copy a prebuilt bundle.

### Dependencies
- Blocked by METIS-T-0139 (a buildable `metis-web`). Pairs with METIS-T-0142 for the desktop artifact.

### Risk Considerations
- WASM builds are slow; cache aggressively so the 3.0 CI stays usable.

## Status Updates

*To be added during implementation*