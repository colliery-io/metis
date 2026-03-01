---
id: multi-workspace-sync-backend
level: initiative
title: "Multi-Workspace Sync Backend"
short_code: "METIS-I-0023"
created_at: 2026-03-01T14:14:24.203747+00:00
updated_at: 2026-03-01T14:14:24.203747+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: multi-workspace-sync-backend
---

# Multi-Workspace Sync Backend

## Context

Metis is a single-workspace tool. Each `.metis/` directory is an island — no awareness of work happening in other repos or teams. The strategy document type exists but is underutilized because real strategy coordination happens *across* projects.

This initiative adds a `metis-sync` crate and `metis sync` CLI command that let independent Metis workspaces share documents through a central git repository. The central repo is a passive beacon — teams push their documents up and pull everyone else's down. No server, no database, no custom protocol. Git is the transport.

The core property to preserve: **work travels with the team**. A team's documents live in their project repo and work offline. The central repo enables cross-team visibility without taking ownership of the data.

**Prior art:** The `feature/multi-workspace-sync` branch has a working implementation of this architecture. This initiative captures the design cleanly and scopes the work to the sync engine and CLI only. MCP integration, GUI, and projection cache are separate follow-on initiatives.

### Three-Layer Organizational Model

Sync exists to serve a specific organizational structure with three layers:

**1. Strategy Layer** — One workspace (e.g. `strat/`). Owned by a strategy team. Sets organizational direction with visions and strategies. Publishes downward.

**2. Initiative Layer** — Working unit boards (e.g. `wg-reliability/`, `wg-platform/`). This is the synchronization layer between strategy and delivery. Each board is owned by an individual (manager, coordinator, product lead) who is accountable for it, but delivery team leads collaborate to populate and prioritize the board. Cross-team initiatives live here. Individual people from delivery teams own specific tasks on these boards.

**3. Delivery Layer** — Team workspaces (e.g. `api/`, `sre/`, `mkt/`). Where teams deliver work. Tasks reference upstream items from working unit initiative boards. Delivery teams may also have their **own** initiatives for team-internal concerns (tech debt, refactors, tooling) that live in the team's workspace, not on the working unit board.

**Visibility principle:** All initiatives are visible to everyone regardless of where they live. A cross-team initiative on `wg-reliability/` and a team-internal initiative on `api/` are both synced to central and available to any workspace that pulls. The projection layer (future initiative) aggregates them for unified views.

**Relationship flow:**
```
Strategy (strat/)
  └─ publishes strategies downward

Working Unit Initiative Boards (wg-reliability/, wg-platform/)
  ├─ initiatives support upstream strategies
  ├─ owned by a coordinator, populated by delivery team leads
  └─ tasks owned by individuals from delivery teams

Delivery Teams (api/, sre/, mkt/)
  ├─ tasks reference upstream initiative board items
  └─ may have team-internal initiatives (visible but self-managed)
```

The sync engine doesn't enforce this hierarchy — it just moves files. The hierarchy is expressed through `parent` references in YAML frontmatter. The sync engine's job is to make all documents available everywhere so these references resolve.

## Goals & Non-Goals

**Goals:**
- `metis-sync` crate: git-based sync engine using libgit2
- `metis sync` CLI command for manual sync
- `metis init --upstream` for workspace configuration
- Config schema for workspace identity and upstream URL
- Bidirectional document sync (push owned, pull remote)
- Automatic push conflict retry

**Non-Goals:**
- MCP server integration (separate initiative)
- GUI sync features (separate initiative)
- Projection cache / cross-workspace queries (separate initiative)
- ACLs or authentication management (delegates to git)
- Team groupings or organizational hierarchy views
- Strategies-gate (coupling strategy availability to sync config)

## Architecture

### Central Repository

One bare git repository holds all workspace documents in a flat-by-prefix structure:

```
central/
  strat/                              # Strategy layer
    STRAT-V-0001.md                   #   org vision
    STRAT-S-0001.md                   #   "Improve reliability to 99.99%"
    STRAT-S-0002.md                   #   "Expand to European market"

  wg-reliability/                     # Initiative layer (working unit board)
    WGR-V-0001.md                     #   working group vision
    WGR-I-0001.md                     #   "API Error Handling Overhaul"
                                      #     parent: STRAT-S-0001
    WGR-T-0001.md                     #   outcome task: "Circuit breaker pattern"
                                      #     parent: WGR-I-0001

  api/                                # Delivery layer
    API-V-0001.md                     #   team vision
    API-I-0001.md                     #   team-internal initiative: "Async runtime migration"
                                      #     parent: API-V-0001 (team-owned)
    API-T-0001.md                     #   "Implement circuit breaker in gateway"
                                      #     parent: WGR-T-0001 (cross-team reference)
    API-T-0005.md                     #   "Refactor connection pool"
                                      #     parent: API-I-0001 (team-internal)

  sre/                                # Delivery layer
    SRE-V-0001.md                     #   team vision
    SRE-T-0001.md                     #   "Configure failover for US-East"
                                      #     parent: WGR-T-0001 (cross-team reference)
```

Each top-level folder is a workspace prefix. A workspace only writes to its own folder. Read access is open — every workspace fetches everything. The hierarchy (strategy → initiative → delivery) is encoded in `parent` references in frontmatter, not in the directory structure.

### Local Layout

After sync, a local workspace has its own documents (read-write) plus hydrated remote documents (read-only):

```
my-project/.metis/
  config.toml           # workspace prefix, upstream URL, last_synced_commit
  metis.db              # ephemeral cache (gitignored)
  visions/              # owned documents — standard Metis layout
    vision.md
  strategies/NULL/
    initiatives/
      PROJ-I-0001/
        initiative.md
        tasks/
          PROJ-T-0001.md
  api/                  # hydrated from central (read-only, gitignored)
    API-V-0001.md
    API-T-0001.md
  sre/                  # hydrated from central (read-only, gitignored)
    SRE-V-0001.md
```

Owned documents are tracked in the project repo. Hydrated remote folders are gitignored.

### Sync Transport

The transport layer is intentionally dumb. It doesn't merge, doesn't understand Metis schemas, doesn't apply domain rules. It moves files.

**Dehydration (push):** Flatten owned workspace documents into `<prefix>/<SHORT_CODE>.md` and commit to central under the owned folder.

**Hydration (pull):** Fetch all workspace folders from central. Write each remote folder's files into local `.metis/<prefix>/` directories.

**No persistent `.git/` inside `.metis/`.** Sync uses libgit2 to create a transient git context — initialize, fetch, commit, push, teardown. Credentials come from the OS/git config for the duration of the operation.

### Sync Flow

```
metis sync
  1. Read config.toml → upstream_url, workspace_prefix, last_synced_commit
  2. SyncContext::new() → in-memory libgit2 repo pointing at upstream
  3. Fetch from central (all branches/refs)
  4. Hydrate: for each remote prefix folder, write files to .metis/<prefix>/
  5. Dehydrate: flatten owned workspace → <prefix>/<SHORT_CODE>.md
  6. Commit & push to central (own folder only)
  7. On push rejection (non-fast-forward): re-fetch, re-graft, retry (up to N times)
  8. Update last_synced_commit in config.toml
  9. Teardown transient git context
```

Pull-only mode (`sync_pull_only`) skips steps 5-7 for read-only operations.

### Config Schema

Additions to `.metis/config.toml`:

```toml
[workspace]
prefix = "api"

[sync]
upstream_url = "git@github.com:org/metis-central.git"
last_synced_commit = "abc123..."
```

Both sections are optional. When absent, sync is unavailable and Metis operates in single-workspace mode (current behavior).

### Conflict Model

- **Within-team:** Standard git merge in the project repo. Two devs editing the same document get a normal merge conflict. Resolved by humans using standard git tools.
- **Cross-team:** Doesn't exist. Each workspace owns its folder exclusively. Push contention (two workspaces pushing simultaneously) is resolved by automatic retry — re-fetch, re-graft, re-push.
- **Stale local state:** Before pushing, sync should verify the local project repo is reasonably current. A warning (not a block) if unpulled commits touch `.metis/`.

### CLI Interface

```bash
metis sync                    # Full sync (pull + push)
metis sync --dry-run          # Preview without pushing
metis sync --pull-only        # Fetch and hydrate only, no push
metis init --upstream <url>   # Configure upstream during init
    --workspace-prefix <pfx>  # Set workspace prefix
```

## Detailed Design

### metis-sync Crate

**Public API:**

| Module | Purpose |
|--------|---------|
| `SyncContext` | libgit2 wrapper — clone/fetch, read blobs, commit trees, push with callbacks |
| `dehydration` | Flatten workspace docs → `FlatDoc` list, diff against previous commit |
| `hydration` | Read central tree → write remote workspace files to local `.metis/` |
| `orchestration` | Compose fetch → hydrate → dehydrate → push with retry loop |

**Key types:**

```rust
pub struct SyncConfig {
    pub upstream_url: String,
    pub workspace_prefix: String,
    pub last_synced_commit: Option<String>,
}

pub struct SyncOptions {
    pub force: bool,
    pub max_retries: u32,  // default: 3
}

pub enum SyncMode {
    Full,       // pull + push
    PullOnly,   // fetch + hydrate only
}

// Entry points
pub fn sync(config, metis_dir, local_documents, options) -> Result<SyncResult, SyncError>;
pub fn sync_pull_only(config, metis_dir) -> Result<SyncResult, SyncError>;
```

**SyncContext** manages the transient libgit2 repository in a tempdir. RAII cleanup on drop. Supports SSH and HTTPS via git credential callbacks. `file://` URLs for testing.

**Dehydration** takes the output of `metis-docs-core::layout::flatten_workspace()` (which walks the standard Metis directory structure and produces flat `SHORT_CODE.md` files) and diffs against the previous central state to produce a minimal changeset.

**Hydration** reads the central tree, identifies remote workspace folders (everything except the owned prefix), and writes their contents to local `.metis/<prefix>/` directories. Cleans up files that were removed upstream.

**Orchestration** ties it all together with a retry loop for push conflicts. On `PushRejected`, re-fetches, re-grafts the owned tree onto the new HEAD, and retries.

### Error Handling

Sync errors are categorized:
- **Fatal:** Invalid config, auth failure → surface to user
- **Retryable:** Push rejected (non-fast-forward) → automatic retry
- **Non-fatal warnings:** Hydration errors for individual remote workspaces → log, continue

### Testing Strategy

- **Unit tests:** SyncContext operations, dehydration/hydration logic, retry behavior
- **Integration tests:** Two workspaces syncing through a bare `file://` repo — create, edit, propagate, verify bidirectional visibility
- **Edge cases:** Empty remote, concurrent push contention, unreachable remote (non-fatal), first-time registration

## Alternatives Considered

Extensively explored in METIS-I-0020. Key rejections:
- **PostgreSQL:** Destroys "work travels with the team"
- **CRDT/SQLite sync extensions:** Unnecessary once centralization eliminated distributed conflict resolution
- **Event sourcing:** Too complex; git history provides audit trail
- **Dual source of truth (content.md + state.json):** Single markdown files with YAML frontmatter are sufficient

## Implementation Plan

The `feature/multi-workspace-sync` branch has a working implementation. The plan is to cherry-pick or rewrite the `metis-sync` crate and CLI integration from that branch onto main, task by task, with clean commits and tests at each step.