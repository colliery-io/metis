---
id: repo-registry-git-remote
level: task
title: "Repo registry, git-remote resolution, and briefing"
short_code: "METIS-T-0133"
created_at: 2026-06-11T23:12:39.977266+00:00
updated_at: 2026-06-12T01:48:16.514654+00:00
parent: METIS-I-0031
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0031
---

# Repo registry, git-remote resolution, and briefing

## Parent Initiative

[[METIS-I-0031]]

## Objective

Make work items discoverable from a repo checkout (initiative design D5) and give the session hook something to fetch. The server keeps a per-project repo registry; clients match their `git remote` URLs against it to find their project/repo context; and a `briefing` endpoint returns the repo-scoped work an agent should see on session start.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `metis_core::repo::normalize_git_url` collapses ssh/https/scp/git URL forms to a canonical match key (lowercased host, no scheme/userinfo/port, `.git` + trailing slash stripped); unit-tested across forms
- [x] Repo registry service: register (computes + stores normalized keys), list — single-arm, both backends
- [x] `resolve(remote)` matches a normalized remote against the registry → `RepoContext` or `None`; unmatched handled explicitly
- [x] `briefing(repo_slug)` returns repo-scoped work bucketed active (phase=active) / ready (todo|ready) as `ItemSummary`s + project slug
- [x] REST: `POST`/`GET /projects/:slug/repos`, `GET /repos/resolve` (404 `unregistered_repo` when unmatched), `GET /briefing`
- [x] MCP: `resolve_repo` (returns `{matched, context}`) + `briefing` tools
- [x] Tests: normalization unit; register→resolve (match + unmatched) + briefing bucketing on both backends; REST + MCP smokes

## Non-Goals (this task)

- Client-side `git remote` reading / the session-hook rewrite itself (separate task) — this delivers the server endpoints the hook will call. Multi-remote iteration and the "default project" disambiguation are client concerns; the server resolves one remote at a time.
- Cross-project repo sharing semantics beyond "first match wins" (documented).

## Implementation Notes

### Technical Approach
- `metis-core`: new `repo` module (`normalize_git_url`) + `service::repo` (register/list/resolve/briefing). Refactor the existing `project::register_repo`/`list_repos` (added in T-0129, unused) to compute normalized keys internally and move them here.
- `metis-server`: REST handlers + two MCP tools, reusing `AppState::blocking` and the MCP `run_blocking` pattern. `briefing`'s active/ready phase heuristic matches the Flight Levels defaults; documented.

### Dependencies
- METIS-T-0129 (services, repos table), T-0131 (REST), T-0132 (MCP dispatch).

### Risk Considerations
- URL normalization is fiddly (scp `git@host:path` vs `ssh://`, ports, `.git`, case). Cover the common forms with tests; over-normalizing risks false matches across distinct repos.
- "active/ready" phases are config-dependent; the heuristic is fine for the default config and documented as such (a config-driven notion of "in flight" can come later).

## Status Updates

### 2026-06-11 — Complete (branch `feat/T-0133-repo-registry` off `initiative/METIS-I-0031`)

`metis-core::repo` + `service::repo`; REST endpoints + two MCP tools. Full suite green (5 new core repo tests + REST/MCP repo smokes), clippy clean.

**Decisions / deviations:**
- **Normalization** parses scheme://, scp `user@host:path`, and bare `host/path`; drops scheme/userinfo/port, lowercases host, strips `.git` + trailing slash. Path case preserved (only host is lowercased). scp detection guards against `host:1234` ports.
- **resolve scans + matches in Rust** rather than a JSON-containment SQL predicate — avoids a per-backend divergence for a small registry; ordered by creation so "first match wins" is deterministic for the rare cross-project duplicate.
- **`resolve_repo` MCP tool returns `{matched, context}`** (not a tool error on miss) so an agent can branch without parsing a string. REST `/repos/resolve` uses 404 + `unregistered_repo`.
- Moved `register_repo`/`list_repos` from `service::project` into `service::repo` (`register`/`list`); register computes normalized keys internally (dropped the caller-supplied param).
- **active/ready heuristic** = phase `active` / phase in {`todo`,`ready`}; matches Flight Levels defaults, documented as config-dependent.
- Repo **registration is REST/CLI only** (not an MCP tool) — fleet setup is an admin act; agents get `resolve_repo` + `briefing`.