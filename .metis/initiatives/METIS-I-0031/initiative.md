---
id: metis-service-deployed-multi-repo
level: initiative
title: "Metis Service: Deployed Multi-Repo Work Backbone"
short_code: "METIS-I-0031"
created_at: 2026-06-11T11:07:21.982767+00:00
updated_at: 2026-06-12T19:59:52.822995+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: XL
initiative_id: metis-service-deployed-multi-repo
---

# Metis Service: Deployed Multi-Repo Work Backbone Initiative

## Context

Per METIS-A-0008 (supersedes METIS-A-0007), Metis evolves from a repo-scoped, file-based planning tool into a deployable service: a lightweight, self-hosted work tracker backed by PostgreSQL that serves as the honest backbone for a team's work across multiple repositories. The JIRA-backing path (METIS-I-0026) is archived — Metis becomes the tracker itself.

Strategic decisions already made (see ADR-0008):
- **Architecture**: client/server. A single-binary Metis server exposes the document model over HTTP and MCP; clients are AI agents (MCP), the GUI, and the CLI.
- **Storage**: one core, two backends via `diesel-dualdb` (`../diesel-dualdb`) — PostgreSQL for team deployments, SQLite for solo/local mode. Write-once Diesel queries; backend selected by connection URL. No file/git federation, no sync between modes.
- **Tenancy (day one)**: one team, self-hosted (docker-compose: metis-server + postgres). Users, assignees, token-based auth from the start. Org-level multi-tenancy is later.
- **Markdown** becomes an export/interchange format; the database is the source of truth.

## Goals & Non-Goals

**Goals:**
- A deployable `metis serve` binary: HTTP + MCP API over the work item model, backed by Postgres
- One storage codebase on `diesel-dualdb`: a fresh logical schema in a new `metis-core` crate, with the same queries running on Postgres and SQLite
- Work items reference repositories (rather than living inside one); cross-repo links (blocks/relates-to) and fleet-wide views
- Users and assignment: humans and agents identified, work assignable, token auth
- Flexible work types and workflows: Flight Levels (Vision→Initiative→Task) becomes the default configuration, not a hard constraint; general types (bug, feature, chore) supported
- A real query layer: filter by phase/repo/assignee/tag, saved views
- Local SQLite mode keeps working (same core, same MCP tools) for solo use
- ~~Markdown export for portability and lock-in avoidance~~ — **moved to METIS-I-0033** (3.0 round-out), scheduled post-UI

**Non-Goals:**
- Importing existing 2.x `.metis` projects — clean break per discovery decision 7; 2.x stays available (bugfix-only), 3.0 starts fresh
- JIRA integration or sync (archived with METIS-I-0026)
- Org-level multi-tenancy, roles/permissions hierarchies, SSO — later phase
- File/git-based federation or cross-repo `.metis` sync — rejected per A-0007/A-0008
- Real-time collaboration features (presence, live editing)
- Recreating JIRA's administrative surface (custom field schemes, screen schemes, workflow editors, etc.)

## Discovery Decisions (resolved 2026-06-11)

1. **API/MCP topology — REST + hosted MCP.** One binary, one core service layer, two transports: REST `/api/v1` for GUI/CLI/integrations, and an MCP endpoint over streamable HTTP that agents connect to directly. Solo mode runs the identical MCP as stdio against SQLite. No local proxy binaries.

2. **Work item model — generic item + typed config.** A single `work_item` entity: short code, type, workflow/phase, title, markdown body, assignee, tags, repo refs, links (blocks/relates/parent), structured exit criteria. Types and their workflows are per-project configuration; Flight Levels (Vision→Initiative→Task) ships as the default config, bug/feature/chore are just types, and ADR supersede semantics survive as a workflow. No JIRA-style dynamic custom-field administration.

3. **GUI — server-embedded web UI.** The server ships a built-in web frontend: deploy the binary, the team has boards, item pages, queries, and shareable deep links in the browser. Solo mode gets the same UI at localhost (`metis serve --local`). The Tauri desktop app is retired (at most a thin shell over the localhost UI).

4. **Repo layout — evolve this workspace in place.** Port `metis-docs-core`'s DAL to diesel-dualdb, add a new `metis-server` crate (axum: REST + MCP + embedded UI), let `metis-docs-mcp` become thin or be absorbed, retire the GUI crate per decision 3. History, CI, and plugin stay; 2.x can keep shipping from main while 3.0 lands incrementally.

5. **Repo references — registry + auto-detect from git remote.** The server keeps a per-project repo registry (slug + git URL); work items carry repo slugs. Clients discover their context by matching `git remote` URLs against the registry — no committed per-repo link file. The server URL and credentials come from user-level config (`~/.config/metis/`), which auth requires anyway. Design must handle: remote URL normalization (ssh/https forms of the same repo), multiple remotes, and the unmatched-remote fallback (prompt? local mode?).

6. **Auth — PATs, agents act as their humans.** Human users with personal access tokens (bearer, hashed at rest); agent tokens are tied to the human they act for plus an agent-name marker, so attribution reads "dylan via claude-code". Admin bootstrap via CLI on first run. SSO/OIDC is a later phase.

7. **Migration/compat — clean break.** No 2.x importer. 3.0 deployments start empty; existing `.metis` projects stay on 2.x, which gets bugfixes only. 3.0 is a semver-major new product mode, not an upgrade path. (Markdown *export* from 3.0 remains in scope for lock-in avoidance — the clean break is about not importing 2.x data.)

8. **Plugin/hooks — same UX, server-backed.** The session hook resolves context via git-remote auto-detect (per decision 5), fetches a repo-scoped briefing from the server (active items, ready work touching this repo), and injects the same-shaped context as today. MCP tool verbs/shapes are preserved where sensible so existing agent muscle memory and the Ralph loop carry over. Falls back to local SQLite mode where user config says so.

## Architecture

### Overview (as decided in ADR-0008; detail in design phase)

```
                 ┌──────────────────────────────┐
                 │  metis serve (single binary) │
  Agent (MCP) ──▶│  ├── MCP endpoint            │──▶ PostgreSQL (team mode)
  GUI / Web   ──▶│  ├── HTTP API                │    SQLite (solo/local mode)
  CLI         ──▶│  └── core service layer      │       one query codebase
                 └──────────────┬───────────────┘       via diesel-dualdb
                                └──▶ ObjectStore (content k=>v):
                                     db-blob (default) | filesystem | s3
```

- Repos hold no Metis files: clients match `git remote` against the server's repo registry to find their project/repo context; server URL + credentials live in user-level config (`~/.config/metis/`)
- Solo mode runs the same core against SQLite with no server process required

## Detailed Design

### D1. Storage: a new logical schema, not a port

The 2.x database is an *index over markdown files* — `documents` is keyed by `filepath` and carries `file_hash` and `frontmatter_json`. In 3.0 the server owns the system of record: **structure lives in the relational schema, content lives in a pluggable object store** (see below), and `filepath` becomes nothing more than a key/URI into that store. The schema is written fresh in diesel-dualdb **logical DDL** (`schema/migrations/*.sql`), and `diesel-dualdb-schema` generates the Postgres migrations, SQLite migrations, and unified `schema.rs`. The clean break (discovery decision 7) is what makes this free: nothing in the old schema needs to survive.

Core tables (logical types; names final at implementation):

```sql
projects        (id UUID PK, slug TEXT UNIQUE, name TEXT, config JSON,
                 created_at, updated_at)
users           (id UUID PK, username TEXT UNIQUE, display_name TEXT,
                 email TEXT, is_admin BOOL, active BOOL, created_at)
tokens          (id UUID PK, user_id FK, name TEXT, agent_name TEXT NULL,
                 token_hash TEXT, created_at, last_used_at, revoked_at NULL)
repos           (id UUID PK, project_id FK, slug TEXT, git_urls JSON,
                 created_at)                  -- multiple URL forms per repo
work_items      (id UUID PK, project_id FK, short_code TEXT UNIQUE,
                 seq INT, type TEXT, phase TEXT, title TEXT,
                 content_key TEXT NULL,        -- body lives in the object store
                 parent_id UUID NULL FK, assignee UUID NULL FK,
                 created_by UUID FK, archived BOOL, created_at, updated_at)
objects         (key TEXT PK, value BYTES, size INT, created_at)
                                              -- the db-blob ObjectStore backend;
                                              -- absent when fs/s3 backend is used
work_item_links (from_id FK, to_id FK, kind TEXT,        -- blocks|relates|supersedes
                 PK (from_id, to_id, kind))
work_item_repos (item_id FK, repo_id FK, PK (item_id, repo_id))
work_item_tags  (item_id FK, tag TEXT, PK (item_id, tag))
exit_criteria   (id UUID PK, item_id FK, position INT, text TEXT,
                 met BOOL, met_by UUID NULL, met_at NULL)
events          (id UUID PK, item_id FK, actor_user UUID, actor_agent TEXT NULL,
                 kind TEXT, payload JSON, created_at)     -- audit + history
views           (id UUID PK, project_id FK, owner UUID, name TEXT,
                 query JSON, shared BOOL)                 -- saved queries
```

**Content storage — the ObjectStore abstraction.** The relational schema holds *structure* (identity, phase, links, assignment, criteria); item *content* (the markdown body) lives in an object store — conceptually a k=>v store of `key → file contents`, behind a `trait ObjectStore { get, put, delete, list }` with three backends:

- **db-blob** (default, team and solo): the `objects` table in the same database. Zero extra infrastructure, and content writes share the row transaction — the simplest deployment stays one docker-compose. **This is the day-one object store (decision 2026-06-12): "database as object store" via Postgres/SQLite.**
- **filesystem**: objects as files under a configured root — for deployments that want content on disk. Implemented (T-0128).
- **s3**: any S3-compatible store — for teams that want DB backups small and content durable/cheap. **Deferred (2026-06-12):** `ObjectStoreConfig::S3` parses but returns `Unsupported`; no s3 task is scheduled. Revisit when DB-blob backup size or content durability becomes a real constraint.

Design rules that keep this clean:
- **Keys are content-addressed** (`sha256(content)`), so objects are immutable: an edit writes a new object and repoints `work_items.content_key`. This buys dedup, makes the store append-only (cache-friendly), and gives **body history for free** — `events` rows record the prior `content_key`, so any past version of an item is retrievable. A periodic GC sweeps objects unreferenced by any item or event.
- **Atomicity**: object is written *before* the row transaction commits; a crash in between leaves only an orphaned immutable object (harmless, GC'd later). The db-blob backend gets true atomicity for free.
- **Search indexes are fed at write time** — the FTS index (see below) is populated from content as it passes through the service layer, so search never needs to read the object store and works identically across backends.
- **Browsability comes from export, not the store.** Hash-named objects aren't human-readable on disk; readable markdown trees are the export feature's job.

Other key choices:
- **Hierarchy is a `parent_id` column** (single parent, enforced in core); `work_item_links` carries the non-hierarchical relations (blocks/relates/supersedes — supersedes gives ADRs their semantics).
- **Types and workflows live in `projects.config` (JSON)**, validated by core — not normalized config tables. Each type defines its phase sequence, short-code letter, and template. Flight Levels ships as the default config; adding a `bug` type is config, not schema. This is the deliberate stop short of JIRA's custom-field machinery.
- **Short codes** are `{PROJECT}-{TYPE_LETTER}-{NNNN}` as today, allocated from a per-project+type sequence inside the insert transaction.
- **Full-text search is the one sanctioned per-backend divergence**: FTS5 on SQLite, `tsvector` on Postgres, behind one `search_items()` DAL function using diesel-dualdb's `dispatch` escape hatch. Everything else is single-arm.

### D2. Crate layout (refines discovery decision 4)

Same workspace, but the "port the DAL" framing sharpens: 2.x crates are *frozen* (bugfix-only), not churned — the 3.0 store is a new crate built clean, importing domain logic from `metis-docs-core` selectively (phase-gate validation, short-code rules, template rendering) rather than rewiring the file-index DAL in place.

```
crates/
├── metis-core         NEW — domain (work items, workflow engine, config
│                      validation), DAL on diesel-dualdb, ObjectStore trait
│                      + backends (db-blob, fs, s3), service layer
├── metis-server       NEW — axum: REST /api/v1, MCP /mcp (streamable HTTP),
│                      embedded web UI (rust-embed), auth middleware,
│                      `serve` / `serve --local` / `admin` / `mcp --stdio`
├── metis-docs-core    frozen (2.x)
├── metis-docs-cli     frozen (2.x)
├── metis-docs-mcp     frozen (2.x)
└── metis-docs-gui     Vue frontend migrates to metis-server's embedded UI;
                       Tauri shell retired at 3.0 release
```

**Async note:** diesel-dualdb v1 is sync. The service layer holds a connection pool and runs DAL calls via `spawn_blocking`; when dualdb's async lands, the swap is internal to `metis-core`.

### D3. Service layer and API surface

One service layer in `metis-core` (`ItemService`, `ProjectService`, `UserService`, `QueryService`); REST handlers, MCP tools, and CLI are thin adapters over it — the discovery decision's "two transports, one impl."

REST sketch (`/api/v1`):

```
POST   /projects                          create (slug, name, config?)
GET    /projects/:slug                    read (incl. type/workflow config)
PATCH  /projects/:slug                    update config
GET    /items?project=&type=&phase=&assignee=&tag=&repo=&q=&view=
POST   /items                             create (type, title, body, parent?, repos?)
GET    /items/:short_code
PATCH  /items/:short_code                 title/body/assignee/tags/repos/criteria
POST   /items/:short_code/transition      { phase, force? }   (phase gates enforced)
POST   /items/:short_code/links           { kind, target }
DELETE /items/:short_code/links/...
POST   /items/:short_code/archive
GET    /repos/resolve?remote=<git-url>    → { project, repo } | 404
POST   /projects/:slug/repos              register repo (slug, git_urls)
GET    /briefing?repo=<slug>              repo-scoped session-hook payload
GET    /projects/:slug/export             markdown bundle (tar.gz)
GET/POST /views                           saved queries
admin: /users, /tokens (create/revoke)
```

MCP tools are the same verbs agents know, renamed document→item at the 3.0 boundary (`list_items`, `read_item`, `create_item`, `edit_item`, `transition_phase`, `search_items`, `link_items`, `briefing`). The plugin ships in lockstep with the server, so no alias layer — but tool *shapes* (short-code addressing, search-and-replace edits, phase-gate errors) carry over so skills like Ralph port with renames only.

### D4. Auth (implements discovery decision 6)

- Token format `mtk_<32 bytes base62>`; only a SHA-256 hash is stored. Bearer header on REST and MCP alike.
- Middleware resolves a token to `Actor { user, agent: Option<String> }`; every mutation writes an `events` row with both — "dylan via claude-code".
- Bootstrap: `metis admin create-user --admin` connects to the database directly (same binary, `DATABASE_URL`); all other administration goes through the API.
- Web UI: paste a token at login; server mints an HttpOnly session cookie. SSO/OIDC explicitly later.
- Solo mode (`--local`): auth off, implicit single user.

### D5. Repo resolution (implements discovery decision 5)

- Registration: `repos.git_urls` accepts multiple URL forms; the server also stores a *normalized* form (lowercase host, path without `.git`, ssh/https collapsed) and matches on that.
- Client flow: read `git remote get-url` for each remote (origin preferred) → normalize → `GET /repos/resolve`. Server URL + token come from `~/.config/metis/config.toml` (list of servers, default marked; no per-repo files).
- Unmatched remote → explicit `unregistered` response; the session hook degrades to a one-line notice with the `metis repo add` command; MCP tools return the same actionable error. Multiple matches → disambiguate by the user config's default project, else error listing candidates.

### D6. Web UI — split into its own initiative (METIS-I-0032, 2026-06-12)

The UI was moved out of this initiative into **[[metis-3-0-web-and-desktop-ui-dioxus]]** (METIS-I-0032) — it's a large, distinct body of work (new frontend stack, wasm toolchain, two render targets) that *builds on* this server's REST API rather than belonging to it. Decision summary (full design + alternatives live in I-0032): an all-Rust **Dioxus** app, web (WASM, embedded at `/`) + desktop (wry/tao) from one codebase, replacing the retired Tauri GUI; prerequisite is a dependency-light `metis-types` DTO crate (metis-core can't compile to WASM). Discovery decision 3 (server-embedded web UI) still holds — it's just tracked in I-0032.

### D7. Testing strategy

- DAL tests run on **both backends** via `#[diesel_dualdb::test]`; CI adds a Postgres service container next to the existing suite.
- The `ObjectStore` trait gets one conformance suite run against the implemented backends (db-blob, fs; s3 deferred), plus crash-window tests for the write-object-then-commit ordering.
- Service-layer tests against in-memory SQLite (fast path); API tests drive the axum router with `tower::ServiceExt::oneshot`; the FTS divergence gets explicit parity tests (same corpus, same queries, both backends).
- The frozen 2.x crates keep their existing test suite untouched.

### Open items deferred to decompose
- Exact JSON schema for `projects.config` (type/workflow definitions) — first design artifact of implementation
- Briefing payload shape (mirror today's session-hook context)
- Export bundle layout (close to 2.x file conventions, but explicitly an interchange format)

## Alternatives Considered

Captured in METIS-A-0008's alternatives analysis: stay repo-scoped (status quo), JIRA as backing store (METIS-I-0026, archived), file/git federation (rejected twice). The deployed-service-with-dual-backend option was chosen.

## Implementation Plan

To be developed at decompose. Expected rough sequencing (per Detailed Design):
1. Logical schema + `metis-core` crate: domain model, DAL on diesel-dualdb, service layer — tests green on both backends
2. `metis-server` skeleton: axum, auth middleware, REST `/api/v1` core routes, `admin` bootstrap
3. MCP endpoint (streamable HTTP + stdio solo mode) with the renamed tool surface
4. Repo registry + resolution (`/repos/resolve`, normalization) and the `briefing` endpoint
5. ~~Web UI migration~~ — **moved to METIS-I-0032** (Dioxus web + desktop UI); not part of this initiative
6. Query layer + saved views; markdown export; docker-compose packaging; plugin/hook update

**Status (2026-06-12):** the server backbone is **complete** — schema/core, server, REST, hosted+stdio MCP, auth, repo registry/briefing, session hook + 3.0 plugin, FTS + saved views, and resource guardrails are merged on `initiative/METIS-I-0031` (all tasks T-0126…T-0137 done). The web/desktop UI was split to **METIS-I-0032**, and the tail items (markdown export, 2.x skills/commands port, small carried deferrals) to **METIS-I-0033** (post-UI). With those moved out, this initiative's scope is delivered and ready to close.