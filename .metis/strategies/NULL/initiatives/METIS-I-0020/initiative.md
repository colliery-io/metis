---
id: evolution-of-metis-multi-layer
level: initiative
title: "Evolution of Metis: Multi-Layer Sync Architecture"
short_code: "METIS-I-0020"
created_at: 2026-01-29T20:27:41.395016+00:00
updated_at: 2026-02-26T01:56:32.691744+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: XL
strategy_id: NULL
initiative_id: evolution-of-metis-multi-layer
---

# Evolution of Metis: Multi-Layer Sync Architecture Initiative

## Context

Metis is currently excellent for single-project work management. However, the "strategy" document type is underutilized because real strategy coordination happens *across* projects and teams, not within a single workspace. The fundamental problem is synchronization between independent Metis instances.

This initiative explores evolving Metis from a single-workspace tool into a multi-layer work coordination system. The core value to preserve: **work travels with the team**. In single-workspace Metis, "my work is in my repo" has been powerful - but it also means work can't go across repos. The goal is to keep "our work travels with us" while enabling synchronization and visibility across teams.

The key insight is that all Metis data is already files (markdown with YAML frontmatter). The filesystem is the source of truth - exactly like today. Sync is just serializing those files into a transport format, pushing to a central beacon, and hydrating them back into the filesystem on pull. The transport layer is dumb - it doesn't merge, doesn't understand schemas, doesn't apply domain rules. It moves files. All intelligence stays local.

**Critical design distinction**: Metis at the coordination layers is about *generating and tracking work*, not about software implementation specifically. Teams using this system may be writing code, producing documentation, running marketing campaigns, coordinating customer activities, staffing, or any other kind of work. The sync and coordination layers must be agnostic to the nature of the work being done. Git repos are one possible workspace - but a Metis instance could equally live in a shared drive, a documentation project, or any directory structure a team uses.

### Problem Statement

- Strategies are rarely used because they require coordination across workstreams
- Individual repos have no awareness of upstream strategic direction or downstream delivery status
- There is no mechanism for bidirectional state awareness between Metis instances
- Teams need to know when upstream priorities change AND upstream needs to know downstream progress

## Goals & Non-Goals

**Goals:**
- Enable multi-layer Metis deployments across organizational hierarchies
- Provide bidirectional sync of document state between Metis instances
- Maintain local-first, fully-offline-capable operation at each layer
- Keep sync explicit and deliberate (not ambient/continuous)
- Make strategies useful by enabling cross-repo coordination

**Non-Goals:**
- Building a SaaS platform or centralized service
- Real-time collaboration (sync is event-driven, not live)
- Replacing git as infrastructure (git is the universal transport and version control layer)
- Supporting arbitrary peer-to-peer topologies (hierarchy only, with possible cross-team blockers)
- Assuming all teams do software work - the coordination layers are work-type agnostic

## Architecture

### The Multi-Layer Model

Three organizational layers, each with their own workspaces and Metis instances. **Every layer has its own vision** that sets the scope and boundaries for that layer's work.

1. **Strategy Layer** - One workspace. Owns the top-level vision (org mission/purpose) and strategies. The vision **guides & constrains** downstream initiative groups. May have internal initiatives/tasks for operational work. Publishes strategies downward.

2. **Initiative Layer** - Multiple workspaces (probably 1-5 per strategy group). Each has **its own vision** that sets the scope and tone for that group's work. Initiatives **support** the upstream strategy vision (upward relationship). Initiatives **require implementation** from delivery teams (downward relationship). May have internal tasks for operational work.

3. **Delivery Layer** - Teams that do the work. Each team has **its own vision** defining team boundaries and purpose. All work **supports** upstream initiatives (upward relationship). May have internal initiatives/tasks for team-level concerns.

### Team Groupings

A team is not a workspace - it's a *grouping* that may span multiple workspaces. A software delivery team might have several git repos. A marketing team might have a content workspace and a campaign workspace. A documentation team might have per-product workspaces.

Each workspace folder in central declares a `team` key (e.g. `team = "platform"`) in its config as a simple join key. A delivery team that owns both `sre/` and `infra/` folders sets `team = "platform"` in both. The GUI aggregates all folders sharing the same team label into a unified view: "all in-flight work for Platform team across SRE and INFRA."

This is a **read-only concern, not a sync architecture concern.** Every workspace does a full fetch, so all data is already local. The team grouping is just a query filter against the local cache - no special coordination point or sync anchor needed.

### Named Relationship Types

The relationships between layers are directional and typed:

| Direction | Relationship | Meaning |
|-----------|-------------|---------|
| Down | **Guides & Constrains** | Strategy vision sets boundaries for initiative groups |
| Up | **Supports** | Initiative groups deliver against strategy; delivery supports initiatives |
| Down | **Requires Implementation** | Initiatives define work that delivery teams execute |
| Lateral | **Link Support** | Strategy connects related initiative groups |

### Centralized Model

**One central git repository** holds all Metis documents across all layers as the single source of truth. This is a dedicated coordination repo - its purpose is organizing work, not holding deliverables.

**The central repo is a passive beacon** - a known coordination point on the network. It never pulls from anyone. It doesn't run any logic. It's just a place teams push to and pull from.

- **Children push up** - teams push their changes to the central repo
- **Teams pull down** - teams do a full fetch from central (all documents, all teams - read is open)
- **Registration = first push** - a new team registers with central by pushing their team folder for the first time. No admin setup required.

Write access is enforced by the **Metis application itself**, not by git hosting platform features. See ACLs section.

**Why not a database?** A central Postgres instance would solve multi-user coordination but destroys the "work travels with the team" property. Work becomes "our work lives in a database somewhere." The git beacon preserves local ownership while enabling cross-team visibility.

### Directory Structure: Flat by Team

Each team owns a top-level folder. No flight-level grouping in the filesystem.

```
central/.metis/
  config.toml                       # central config (reserved, may be empty)
  strat/
    STRAT-V-0001.md                # org vision
    STRAT-S-0001.md                # strategy
    STRAT-S-0002.md
  alpha/
    ALPHA-V-0001.md                # team vision
    ALPHA-I-0001.md                # initiative (supports STRAT-S-0001 - in frontmatter)
    ALPHA-T-0001.md
  api/
    API-V-0001.md                  # team vision
    API-T-0001.md                  # task (supports ALPHA-I-0001 - in frontmatter)
  mkt/
    MKT-V-0001.md
```

**Why flat by workspace prefix, not nested by flight level:**
- **Full fetch, sparse push**: read everything, write only your folder
- **ACLs are trivial**: application enforces write = your prefix folder
- **No deep nesting**: every workspace is a filesystem peer regardless of flight level
- **Workspaces organize internally however they want**: Metis already handles within-project structure
- **Adding a workspace = adding a folder**: no restructuring needed
- **No short code collisions**: each workspace has a unique prefix, so short codes are globally unique by convention

The unit of sync is the **workspace prefix** (e.g. `api/`, `sre/`, `infra/`). Each prefix maps to one folder in central and one write scope. Teams are a read-only grouping layered on top via `team = "platform"` labels - multiple workspace prefixes can share a team label for unified views.

The filesystem tells you "these are the workspaces." Everything else - hierarchy, flight level, relationships, team groupings, status - lives in the YAML frontmatter of each document's markdown file. Cross-team relationships (ALPHA-I-0001 supports STRAT-S-0002, API-T-0001 blocked by ALPHA-I-0003) are encoded in each document's frontmatter. The local `metis.db` cache aggregates these for fast queries but is not a source of truth.

### Source of Truth: Files on Disk

**The filesystem is the source of truth.** Exactly like current single-workspace Metis. Each document is a markdown file with YAML frontmatter carrying operational state (phase, tags, assignments, relationships, timestamps). Content is the markdown body. This is how Metis works today - nothing changes for the local experience.

**`metis.db` is an ephemeral cache** - rebuilt from files on startup. In multi-workspace mode, it's expanded in scope: it also caches data from other workspaces (hydrated from the transport layer) for cross-team queries. But it is never the source of truth.

The transport layer doesn't need to understand the file format. It serializes files for transit and hydrates them back into the correct filesystem locations on pull. YAML frontmatter with unknown fields is handled naturally by every YAML parser - old Metis versions ignore fields they don't know, new versions write fields old versions preserve. **No schema migration problem.**

### Storage Layout & Git Tracking

**Critical design point**: in multi-workspace mode, the project repo and the central coordination repo coexist. The owned workspace's documents are tracked in BOTH — the project repo (for within-team collaboration) and central (for cross-team visibility). Remote workspace documents are hydrated from central and gitignored from the project repo.

```
my-project/
  .git/                   # the project's own repo
  .gitignore              # see tracking rules below
  src/
  .metis/
    config.toml           # TRACKED — workspace identity, upstream URL
    metis.db              # GITIGNORED — ephemeral cache
    api/                  # TRACKED — owned workspace documents (read-write)
      API-V-0001.md
      API-T-0001.md
      API-T-0002.md
    strat/                # GITIGNORED — hydrated from central (read-only)
      STRAT-V-0001.md
      STRAT-S-0001.md
    alpha/                # GITIGNORED — hydrated from central (read-only)
      ALPHA-I-0001.md
```

**What's tracked in the project repo (travels with the team):**
- `.metis/config.toml` — workspace identity, sync configuration
- `.metis/<owned-prefix>/*.md` — the team's own documents

**What's gitignored from the project repo:**
- `.metis/metis.db` — ephemeral cache, rebuilt on startup
- `.metis/<remote-prefix>/` — all hydrated remote workspace folders (come from central, not from the project repo)
- `.metis/code-index*.json` — code index caches

**.gitignore pattern:**
```gitignore
# Metis — ignore ephemeral state and hydrated remote workspaces
.metis/metis.db
.metis/code-index-hashes.json
.metis/code-index-symbols.json
.metis/.index-dirty

# Ignore all workspace folders EXCEPT the owned one
# (managed by `metis init` — adds ignore rules for remote prefixes as they're discovered)
```

The gitignore for remote workspace folders is managed dynamically — when `metis sync` hydrates a new remote workspace, it adds that prefix to `.metis/.gitignore`. The owned workspace prefix is never added.

**Why this split matters:**
- **Within-team**: teammates see document changes via normal `git pull`. Merge conflicts on documents are resolved the same way as code conflicts — in the project repo, using standard git tools. This is the primary collaboration path for a team.
- **Cross-team**: changes propagate via `metis sync` to/from central. The sync pushes whatever state the team has in their project repo (post-merge, post-resolution).
- **Central is eventually consistent** with each team's project repo. The project repo is the immediate source of truth for the team. Central aggregates across teams.

No `.git/` directory inside `.metis/`. Git operations for central sync happen transiently during `metis sync`.

- **`*.md` files** - markdown with YAML frontmatter. One per document. The source of truth for both content and operational state. Exactly like current Metis.
- **`metis.db`** - ephemeral cache. Rebuilt from all local markdown files on startup (own workspace + hydrated remote workspaces). Used for fast queries and cross-team visibility.
- **Owned folder** (e.g. `api/`) - read-write, tracked in project repo. This workspace's documents. Pushed to central on sync.
- **Remote folders** (e.g. `strat/`, `alpha/`) - read-only locally, gitignored from project repo. Hydrated from central on sync. Provide cross-team visibility.

### Sync Transport: Dumb Serialization

The transport layer is intentionally dumb. It doesn't merge, doesn't understand Metis schemas, doesn't apply domain rules. It serializes files for transit and hydrates them into filesystem locations on the other end.

#### How It Works

**Push (dehydrate):**
1. Serialize all markdown files in the owned workspace folder into the transport format
2. Push to central (the workspace's folder only)

**Pull (hydrate):**
1. Fetch all workspace folders from central
2. Hydrate each folder's files into the local `.metis/` filesystem in the correct location
3. Rebuild `metis.db` cache from all local files (owned + remote)

**The transport format can be anything** that describes files and their locations - a git tree of markdown files, a tarball, a zip, a JSON manifest. The format is an implementation detail, not an architectural decision. Git is the current choice because it's ubiquitous and provides history/CDC for free.

#### Why This Works

- **No merge logic in transport.** Each workspace folder is single-writer. There are no cross-workspace merge conflicts. Within-team contention is low (planning is collaborative) and handled by existing Metis sync.
- **No schema migration problem.** YAML frontmatter with unknown fields is handled naturally by every YAML parser. Old versions ignore fields they don't know. New versions write fields old versions preserve. Forward-compatible by default.
- **No special state format.** Operational state lives in YAML frontmatter, same as current Metis. No separate state.json, no JSON domain merge, no programmatic merge rules.
- **All intelligence is local.** The local Metis instance reads files, builds its cache, applies its rules. The transport layer just moves files around.

#### CDC via Git History

Git history serves as the change data capture (CDC) log. Each workspace tracks `last_synced_commit` in `config.toml`. On sync, diff from that commit to current HEAD gives the changeset: additions, modifications, and deletions. Deletions matter for archive cascading - if an upstream initiative is archived, the local Metis instance detects the change during hydration and cascades locally (archiving child tasks in its own files).

#### Consequences

- **No separate changelog, no snapshots, no compaction.** Files are the source of truth. Git history is the CDC log.
- **No SQLite sync extension.** Not needed. Transport is dumb file movement.
- **No custom event sourcing.** Current state in files is truth. Git history provides change tracking.
- **No domain merge rules in transport.** All Metis logic stays local.
- **`metis.db` is purely an ephemeral cache** - rebuilt from files on startup. Expanded in scope to include cross-team data, but still not a source of truth.
- **Git history is dual-purpose.** CDC log for sync + audit trail for compliance/debugging.
- **Version skew is a non-problem.** YAML frontmatter is naturally forward-compatible. No coordinated schema migrations needed across workspaces.

### Conflict Resolution: Scenario Walkthrough

There are two distinct conflict domains: the **project repo** (within-team, normal git) and the **central repo** (cross-team, sync). They never interact at the git level.

#### Scenario 1: Two devs edit the same document (WITHIN-TEAM)

Dev A and Dev B both edit `API-T-0001.md` — A marks it `phase: completed`, B adds a `blocked_by` entry.

**What happens:** Standard git merge conflict in the project repo. Both devs committed changes to the same file. Git reports a conflict on `.metis/api/API-T-0001.md`.

**Resolution:** Normal git merge. The conflict is in a markdown file with YAML frontmatter — humans read it and pick the right state. This is the same as any code conflict. Planning is collaborative, so this is rare and meaningful when it occurs — two humans wrote different things.

**Central impact:** None until resolved. The sync to central only happens after the project repo merge is complete (sync is triggered post-push). Central sees the resolved state.

#### Scenario 2: Two devs edit different documents (WITHIN-TEAM)

Dev A edits `API-T-0001.md`, Dev B edits `API-T-0002.md`.

**What happens:** Git merges cleanly. Different files, no conflict. Next `metis sync` (post-push hook) pushes both changes to central.

**Resolution:** None needed.

#### Scenario 3: Feature branch document edits (WITHIN-TEAM)

Dev A on branch `feature-x` transitions `API-T-0001.md` from `todo` → `active`. Dev B on branch `feature-y` also transitions `API-T-0001.md` from `todo` → `completed` (skipping active — maybe they completed it quickly).

**What happens:** When both branches merge to main, git reports a conflict on `API-T-0001.md`. The YAML frontmatter has divergent `phase` values.

**Resolution:** Human picks the correct phase. This IS a meaningful conflict — the team needs to agree on the task's actual status. Standard three-way merge in any git tool.

**Note:** This scenario is rare in practice. Task status changes are typically sequential (one person moves it), not concurrent. When it happens, it's a genuine coordination issue, not a tooling problem.

#### Scenario 4: Short code collision (WITHIN-TEAM)

Dev A and Dev B both create new tasks while working offline. Both get assigned `API-T-0050` (the short code counter was at 49 for both).

**What happens:** Git merge conflict — two different files with the same path (`.metis/api/API-T-0050.md`). Git sees this as a modify/modify conflict.

**Resolution:** One dev renames their document to the next available short code. The short code counter in config/state needs to be updated.

**Mitigation:** Short code generation should use a strategy that minimizes collision likelihood. Options:
- Counter in `config.toml` (tracked in git, conflicts surface immediately)
- Timestamp-based codes (collision only if created in the same second)
- Random suffix (collision probability negligible)

In practice, this is very rare — two devs creating documents simultaneously while offline. The counter approach surfaces it as a git conflict, which is the right behavior.

#### Scenario 5: Cross-workspace push contention (CROSS-TEAM)

API team and SRE team both sync to central at the same moment. API team's push succeeds (central moves from commit X to X+1). SRE team's push is rejected (their commit parents X, not X+1).

**What happens:** Push retry logic (METIS-T-0082) kicks in. SRE's sync re-fetches X+1, rebuilds their owned tree grafted onto the new central HEAD, and re-pushes. Succeeds because the two workspaces modify different folders — no content conflict.

**Resolution:** Fully automatic. The retry is mechanical (re-fetch, re-graft, re-push). Since workspaces only modify their own folder, there are never actual content conflicts in central. The only issue is the stale parent commit, which the retry resolves.

**Performance:** Retries are rare (syncs are infrequent, background operations) and cheap (small markdown files, fast git operations).

#### Scenario 6: Stale local state synced to central (WITHIN-TEAM → CROSS-TEAM)

Dev A edits `API-T-0001.md`, pushes to the project repo, and syncs to central. Dev B hasn't pulled from the project repo yet. Dev B then runs `metis sync` manually.

**What happens:** Dev B's sync dehydrates their local owned workspace state — which doesn't include Dev A's changes (because Dev B hasn't pulled). Dev B's push to central would overwrite Dev A's version of `API-T-0001.md`.

**Mitigation:** Sync should check that the project repo is up to date with its remote before pushing to central. Specifically:
1. Before dehydration, check if the project repo's main branch has unpulled commits that touch `.metis/<owned-prefix>/`
2. If yes, warn: "Your local documents are behind the project repo. Run `git pull` first."
3. Optionally: `metis sync --force` to override (for cases where you intentionally want to push local state)

This is the same principle as "don't push stale code" — you pull before you push. The sync engine should enforce this.

**Why this is primarily a within-team concern:** If the team uses git hooks (post-push triggers sync), this scenario doesn't arise — sync only runs after the project repo is pushed, which means the local state includes all merged changes. Manual sync is the risk path.

#### Scenario 7: Coordination-only workspace (no code project)

A strategy team or working group doesn't produce code — they only coordinate work. Their workspace is a git repo that contains `.metis/` and nothing else.

**What happens:** Identical to any other workspace. The repo just doesn't have a `src/` directory. Clone it, edit documents, commit, push. Within-team conflicts are handled by git merge. Sync to central works the same way.

**No special handling needed.** The architecture treats coordination-only workspaces identically to code project workspaces. A workspace is a git repo with a `.metis/` folder — what else is in the repo is irrelevant to Metis.

#### Summary: Where conflicts live

| Scenario | Domain | Mechanism | Resolution |
|---|---|---|---|
| Same document, two devs | Project repo | Git merge conflict | Human resolves (standard git) |
| Different documents, two devs | Project repo | Clean merge | None needed |
| Feature branch divergence | Project repo | Git merge conflict | Human resolves |
| Short code collision | Project repo | Git add/add conflict | Rename one, update counter |
| Cross-workspace push race | Central repo | Non-fast-forward | Automatic retry (re-fetch, re-graft) |
| Stale local state pushed | Central → project repo | Stale overwrite | Pre-sync check: project repo up to date? |
| Coordination-only workspace | Project repo | Same as any workspace | No special handling — it's just a git repo with only `.metis/` |

**Design principle:** Within-team conflicts are handled by the project repo's git — the same way code conflicts are handled. Cross-team conflicts don't exist (single-writer per workspace folder in central). The sync engine adds one safety check: verify the project repo is up to date before dehydrating to central.

### Two-Repo Model

There are conceptually **two git repos** in any workspace:

1. **The project repo** - the workspace's own `.git/` (software, docs, whatever the team produces)
2. **The coordination repo** - the central Metis repo that holds all team documents

These two repos never interact at the git level. The key design decision: **the coordination repo is ephemeral**. There is no persistent `.git/` directory inside `.metis/`. Instead, when `metis sync` runs, Metis:

1. Initializes an in-memory git context via libgit2
2. Full fetch from central (all workspaces, all documents)
3. Hydrates remote workspace files into local `.metis/` folders (read-only)
4. Pushes owned workspace files to central (own folder only)
5. Tears down the temporary git context
6. Rebuilds `metis.db` cache from all local files

No `.git/` inside `.metis/`. The files on disk are the working state. Git is used as a dumb transport mechanism during sync, not as persistent local version control for the coordination data.

**Why ephemeral, not a persistent clone?** Credentials are stored in the OS keychain and managed by Metis. A persistent `.git/` directory would be a pre-authenticated gateway to central - anyone who opens a terminal in `.metis/` could `git push` directly, bypassing all application-enforced ACLs. With the ephemeral model, there is no `.git/` to exploit. Credentials come out of the keychain for the sync operation and the authenticated context disappears. This isn't about preventing a determined attacker - it's about not leaving a loaded gun on the desk.

For non-software teams (coordination-only workspaces), the same model applies. `.metis/` holds the files, and sync uses transient git operations against the central repo. There's no outer project repo - the workspace is purely for work management.

### Sync UX

#### Trigger Model

Sync is **not a manual ceremony**. It piggybacks on the development workflow:

- **Primary trigger**: git hooks (post-commit to main, post-push to remote) fire `metis sync` automatically
- **Manual trigger**: `metis sync` (CLI) or "Sync" button (GUI) for on-demand use
- **GUI trigger**: background sync on app launch

Syncs are **infrequent** (tied to development cadence - a few times a day at most), carry **small deltas** (a few state changes or content edits per sync), and run in the **background**. Nobody watches a spinner. Work management sync is invisible, riding alongside the normal git workflow.

#### Sync Mechanics

Under the hood, `metis sync` uses libgit2 for in-memory git operations - no persistent `.git/` directory exists in `.metis/`:

1. **Initialize** - create an in-memory git context via libgit2, referencing the central repo URL from `config.toml`
2. **Fetch** - full fetch from central (all workspaces, all documents - read is open)
3. **Diff** - compare current HEAD against `last_synced_commit` (stored in `config.toml`). This diff is the CDC changeset: additions, modifications, and archives since last sync.
4. **Hydrate** - write remote workspace files into local `.metis/` folders (read-only). Local Metis processes changes (e.g. upstream initiative archived → cascade archive to local child tasks).
5. **Dehydrate & push** - serialize owned workspace files and push to central (own folder only - enforced by Metis application)
6. **Record** - store current HEAD as `last_synced_commit` in `config.toml`
7. **Teardown** - discard the in-memory git context
8. **Rebuild** - regenerate `metis.db` cache from all local files (owned + remote)

**Performance is a non-concern.** The central repo holds small markdown files. Even at 50 workspaces, total repo size is well under 100MB. A full fetch takes seconds, and syncs are infrequent background operations.

**Interruption safety:** If sync is interrupted before the push phase, central is untouched. If interrupted during local file writes, re-running sync recovers (pull everything, diff, apply updates).

**Push conflict retry:** If the push fails because remote HEAD moved (another workspace pushed first), Metis re-runs the entire sync from scratch: re-fetch, re-diff, re-apply local changes, re-push. Retries up to 5 times, then raises an error. With infrequent background syncs and small deltas, retries are rare and cheap.

Users never see git. The transient git operations are invisible.

### Git Conventions

- **Main is truth** - only the main branch syncs. Working branches are invisible to other teams.
- **Direct push to main** - no PRs for routine document updates. Status changes, task updates, etc. push directly.
- **Application-enforced write scope** - Metis controls what gets pushed (own workspace folder only). No dependency on git hosting platform features.
- **Many writers, one origin** - all workspaces push to the same central repo. The central repo is passive (receives pushes, never pulls).
- **Full fetch, sparse push** - every workspace fetches everything from central (full read access). Metis only pushes changes within the owned workspace folder.

### ACLs

**Read: open by default.** Every workspace fetches everything from the central repo (full fetch). Cross-team visibility is free - strategy leads browse any workspace, delivery teams see peer work, blockers are always visible.

**Write: your folder only.** Enforced by the **Metis application**, not by git hosting platform features. This is a deliberate choice - Metis must work on any git service (GitHub, GitLab, Gitea, self-hosted, etc.) without relying on provider-specific ACL mechanisms like push rulesets or protected paths.

**How it works:**
- Metis knows which workspace folder it owns (configured in `config.toml`)
- During `metis sync`, the application only stages and pushes files within the owned workspace folder
- The application refuses to push changes to other workspaces' folders, even if a user manually edits them on disk
- Repo access itself determines identity: if you can clone the central repo, you're a participant. Your team assignment determines what you can write.

**Registration:** A new workspace registers by performing its first push to central. This creates the workspace folder. No admin provisioning step - the central repo is passive and accepts pushes from any authenticated user. Workspace identity is self-declared via `config.toml` and enforced locally by the application.

**Trust model:** This is application-level enforcement, not server-level. A user with raw git access to central could theoretically bypass Metis and push to another workspace's folder. For most organizations this is acceptable - the same trust model as any shared repo. For high-security environments, server-side path restrictions can be layered on as defense-in-depth, but Metis does not depend on them.

**Private work:** If a team needs true privacy (sensitive initiatives, HR, legal, etc.), they run a separate Metis instance that is not in the shared central repo. That private instance can still subscribe to central for upstream context - it just doesn't publish back. Privacy is opt-in by separation, not opt-in by ACL complexity.

### Identity

Short codes remain the primary identifier. Uniqueness is by convention via per-project prefixes (STRAT-S-0003, MOBILE-I-0012, API-T-0044). No UUIDs needed at expected scale. The central repo's directory structure provides the canonical namespace.

### GUI as Primary Interface

For multi-team deployments, the GUI (Tauri app) is the primary interface for most users. Engineers may use the CLI, but strategy leads, project coordinators, marketing managers, and other non-technical roles interact with Metis entirely through the GUI. Every operation must have a GUI path - there is no "CLI-only" functionality in the multi-layer model.

**Implications:**
- Git is completely invisible. No user ever sees a git command, a branch, or a merge conflict in raw form.
- Sync happens on app launch and via a "Sync" button. The underlying libgit2 operations are abstracted away entirely.
- Status transitions, document editing, and relationship management are visual operations.
- Conflict resolution is a side-by-side visual diff with highlighted changes, not terminal output.

### Onboarding

`metis init` (CLI) or the first-run wizard (GUI) handles:

1. **Configure central repo URL** - paste the URL. Stored in `.metis/config.toml`.
2. **Team assignment** - select or confirm which workspace prefix this instance owns. Determines write scope (enforced by Metis application).
3. **Initial sync** - first pull from central to get upstream context. If this is a new team, the first push creates the team folder in central (registration = first push).

**Authentication** is entirely delegated to git. SSH keys, credential helpers, `.gitconfig` — whatever the user already has configured for the central repo's host. Metis does not manage credentials. If git can reach the remote, Metis can sync.

**Registration model:** There is no admin step to "add a team" to central. A new team configures their `.metis/config.toml` with the central URL and their workspace prefix, then runs `metis sync`. The first push creates their workspace folder. The central repo is passive.

**For CLI users:** `metis init --upstream <url>` walks through the same steps interactively. Tests connectivity with `git ls-remote` before proceeding.

### GUI-Specific Considerations

**Reading/browsing** (most common operation):
- Dashboard view: "my work" (owned documents), "driving my work" (upstream context), cross-team visibility
- Hierarchical navigation: drill from vision → strategy → initiative → task
- Status board / kanban view per team or initiative
- Search and filter across all visible documents

**Editing:**
- Rich markdown editor for content (not raw text for non-technical users)
- Operational state changes via dropdowns, buttons, drag-and-drop (phase transitions, tag management, assignments)
- Both map to the same file: editor writes markdown body, UI controls write YAML frontmatter

**Sync & conflicts:**
- Background sync on launch + manual "Sync now" button
- Sync status indicator (last synced, pending changes, conflicts)
- Conflict resolution for within-team conflicts: visual side-by-side diff with author identification, "Keep mine" / "Keep theirs" / "Edit manually" buttons
- Cross-workspace conflicts don't exist (single-writer per workspace folder)

## Alternatives Considered

### PostgreSQL Service
"Switch to Postgres and it becomes a service." Postgres solves multi-user coordination elegantly - transactions, row-level security, real-time queries, schema enforcement, audit trails. It eliminates merge conflicts entirely. However, it destroys the core property: **work travels with the team**. Work becomes "our work lives in a database somewhere" rather than "our work is in our repo." The entire value of single-workspace Metis comes from locality of work data. Postgres trades that for coordination convenience. The git beacon approach preserves local ownership while enabling cross-team visibility.

### sqlite-sync (SQLite Cloud)
Source-available CRDT sync extension for SQLite. Rejected due to Elastic License 2.0 (requires commercial license for production) and coupling to SQLite Cloud's hosted backend. However, this project inspired the initial exploration into SQLite-based synchronization.

### cr-sqlite (vlcn.io)
MIT-licensed CRDT extension for SQLite. Technically sound but provides generic LWW conflict resolution. Rejected as unnecessary once the centralized model eliminated the distributed conflict resolution problem.

### SQLite as Ephemeral Cache Only (first attempt)
Initially simplified to "SQLite is just a cache, markdown is the only source of truth." Temporarily rejected in favor of a dual source of truth split (content.md + state.json) to avoid merge conflicts when operational state and content are mixed in one file. However, the dual split was itself later rejected (see below) once we recognized within-team contention is low enough that single files with YAML frontmatter work fine. **This is the final design** - SQLite as ephemeral cache, files as source of truth.

### Distributed Changeset Exchange
Each workspace maintains its own DB and exchanges changesets via outbox/inbox files. Rejected as unnecessary complexity once we recognized that a central repo eliminates the need for custom sync protocols.

### Event Sourcing / Changelog
Append-only JSON changeset files in a changelog directory, with the DB as a materialized view rebuilt by replaying the log. Explored in depth including compaction via git-hash-tagged snapshots with lagged deletes. Rejected because it introduced significant complexity (log growth, compaction timing, concurrent snapshot edge cases) when files-as-current-state is simpler. Git history provides the audit trail and CDC log without a custom replay mechanism.

### SQLite Sync Extension
Custom SQLite extension for field-level LWW merge of operational state. Scope narrowed progressively through the design session (full doc sync → operational state only → not needed at all). Rejected because the transport layer doesn't need to understand Metis semantics at all.

### SQLite as Source of Truth
Explored pivoting from files to SQLite as the source of truth, with git transporting .db files between workspaces. SQLite solves schema enforcement, migrations, and queries elegantly. However: (1) binary .db files can't be merged by git, requiring either single-writer semantics or a changeset protocol; (2) distributed clients must agree on schema version, reintroducing the version skew problem; (3) forward-only additive schema changes (ALTER TABLE ADD COLUMN with defaults) mitigate version skew but impose the same constraint as YAML's forward-compatible fields. The pivot led to a key realization: the db can hydrate files on pull and rebuild from files on push, making files the source of truth and the db just a cache. If files are truth and transport is dumb serialization, the db isn't needed in the transport layer at all. Rejected in favor of files-as-truth with dumb transport.

### Dual Source of Truth (content.md + state.json)
Split documents into separate files for human content (markdown) and operational state (JSON per team). Designed elaborate programmatic domain merge rules for state.json via libgit2 (phase forward-only, sets union, scalars last-write-wins). Rejected because single markdown files with YAML frontmatter handle both concerns without any special merge logic. The split was solving a problem (concurrent edits to different concerns in the same file) that doesn't occur in practice - within-team contention is low and planning is collaborative.

## Design: Full Sync Cycle Walkthrough

This section traces one complete work cycle through all three layers — from strategy publication through delivery and status propagation back up. The goal is to validate that the architecture handles every step concretely, and to surface gaps.

### Cast

- **Sarah** — Strategy Lead. Owns the `strat/` workspace. Uses the GUI exclusively.
- **Alex** — Initiative Coordinator for the "Alpha" product group. Owns the `alpha/` workspace. Uses the GUI.
- **Dev (API team)** — Delivery team. Owns the `api/` workspace. Uses CLI + IDE. Sync piggybacks on git hooks.

### Step 1: Strategy Publication

Sarah creates the organizational vision and a strategy in her workspace:

```
strat/
  STRAT-V-0001.md   # "Become the most reliable API platform in the industry"
  STRAT-S-0001.md   # "Improve API reliability to 99.99% uptime"
                     #   parent: STRAT-V-0001
                     #   phase: active
```

Sarah writes the strategy content, transitions it through phases (shaping → design → ready → active), and hits **Sync** in the GUI.

**What happens under the hood:**
1. GUI calls `metis sync`
2. libgit2 initializes an in-memory context against the central repo URL (from `config.toml`)
3. Full fetch from central (first time — nothing to pull)
4. Dehydrate: serialize `strat/*.md` files, commit to central's `strat/` folder
5. Push to central. This is strat's first push — **registration happens implicitly**
6. Record `last_synced_commit` in `config.toml`
7. Teardown git context

**Central repo state after:**
```
central/.metis/
  strat/
    STRAT-V-0001.md
    STRAT-S-0001.md
```

### Step 2: Initiative Coordinator Discovers Strategy

Alex opens the GUI. On launch, background sync fires.

**What happens:**
1. Full fetch from central → gets `strat/STRAT-V-0001.md` and `strat/STRAT-S-0001.md`
2. Hydrate: write these files into Alex's local `.metis/strat/` folder (read-only)
3. No owned changes to push yet (alpha/ folder is empty or doesn't exist in central)
4. Rebuild `metis.db` — now contains both Alex's own documents and the upstream strategy docs

**Alex's local state:**
```
my-product-docs/.metis/
  config.toml          # upstream: central URL, workspace: alpha, team: alpha-product
  metis.db             # rebuilt — includes strat/* documents
  alpha/               # owned, read-write
    ALPHA-V-0001.md    # "Alpha product group delivers reliable APIs" (Alex's own vision)
  strat/               # hydrated from central, read-only
    STRAT-V-0001.md
    STRAT-S-0001.md
```

Alex sees STRAT-S-0001 ("Improve API reliability to 99.99%") in the GUI's "Upstream Context" view. Alex creates an initiative that supports this strategy:

```
alpha/
  ALPHA-V-0001.md
  ALPHA-I-0001.md      # "API Error Handling Overhaul"
                        #   parent: STRAT-S-0001       ← cross-workspace reference
                        #   phase: discovery
```

The `parent: STRAT-S-0001` field is a standard Metis frontmatter field — it just happens to reference a short code from another workspace. Since prefixes are globally unique and all documents are hydrated locally, the projection cache resolves this.

Alex works through the initiative phases (discovery → design → ready → decompose). During decompose, Alex **does not create tasks for the delivery team** — Alex creates tasks within the initiative describing what *outcomes* are needed, scoped to the initiative layer:

```
alpha/
  ALPHA-I-0001.md      # phase: decompose
  ALPHA-T-0001.md      # "Circuit breaker pattern for external API calls"
                        #   parent: ALPHA-I-0001
                        #   phase: todo
                        #   note: "API team to implement — discussed in planning"
```

Alex hits **Sync**.

**Central repo state after:**
```
central/.metis/
  strat/
    STRAT-V-0001.md
    STRAT-S-0001.md
  alpha/
    ALPHA-V-0001.md
    ALPHA-I-0001.md
    ALPHA-T-0001.md
```

### Step 3: Delivery Team Picks Up Work

The API team's sync fires automatically (post-push git hook, or background sync in their IDE).

**What happens:**
1. Full fetch from central → gets all of `strat/` and `alpha/`
2. Hydrate: write remote files into local `.metis/strat/` and `.metis/alpha/` (read-only)
3. Rebuild `metis.db` — now has full cross-team visibility

**API team's local state:**
```
api-service/.metis/
  config.toml          # workspace: api, team: api-team
  metis.db
  api/                 # owned, read-write
    API-V-0001.md      # "API team builds and maintains core service reliability"
  alpha/               # hydrated, read-only
    ALPHA-V-0001.md
    ALPHA-I-0001.md
    ALPHA-T-0001.md
  strat/               # hydrated, read-only
    STRAT-V-0001.md
    STRAT-S-0001.md
```

The team sees ALPHA-T-0001 ("Circuit breaker pattern for external API calls") in the "Upstream Work" view. In their planning session, they create their own tasks to implement it:

```
api/
  API-V-0001.md
  API-T-0001.md        # "Implement circuit breaker in gateway service"
                        #   parent: ALPHA-T-0001    ← cross-workspace reference
                        #   phase: active
  API-T-0002.md        # "Add circuit breaker integration tests"
                        #   parent: ALPHA-T-0001
                        #   phase: todo
```

The dev starts working on API-T-0001. Normal development workflow — write code, commit, push. The post-push git hook fires `metis sync` in the background.

**Central repo state after:**
```
central/.metis/
  strat/
    STRAT-V-0001.md
    STRAT-S-0001.md
  alpha/
    ALPHA-V-0001.md
    ALPHA-I-0001.md
    ALPHA-T-0001.md
  api/
    API-V-0001.md
    API-T-0001.md
    API-T-0002.md
```

### Step 4: Work Completes, Status Flows Up

The dev finishes the circuit breaker implementation. They transition API-T-0001 to completed, then API-T-0002 after tests pass. Next `metis sync` pushes the updated files.

**What Alex sees on next sync:**

Alex's GUI syncs. The hydrated `api/` folder updates with the completed tasks. Alex's projection cache (`metis.db`) computes:

- ALPHA-T-0001 has 2 child tasks across workspaces (API-T-0001, API-T-0002)
- Both are completed
- Aggregate: 2/2 tasks done → ALPHA-T-0001 is fully implemented

Alex reviews and transitions ALPHA-T-0001 to completed. When all tasks under ALPHA-I-0001 are done, Alex transitions the initiative to completed. Sync pushes the update.

**What Sarah sees on next sync:**

Sarah's GUI syncs. The hydrated `alpha/` folder shows ALPHA-I-0001 is completed. Sarah's projection cache computes:

- STRAT-S-0001 has 1 child initiative (ALPHA-I-0001) — completed
- Strategy progress: 1/1 initiatives done

Sarah can see the full chain: Strategy → Initiative → Tasks → Completed. The status flowed up through the system via synced files and projection cache aggregation.

### Step 5: Disruption — Priority Change

Sarah reprioritizes. STRAT-S-0001 gets deprioritized and a new STRAT-S-0002 ("Expand to European market") takes priority. Sarah archives STRAT-S-0001.

**On next sync downstream:**

Alex syncs. The hydrated `strat/STRAT-S-0001.md` now has `archived: true` in its frontmatter. Alex's local Metis detects this during hydration:

- ALPHA-I-0001 (which has `parent: STRAT-S-0001`) is flagged: "upstream parent archived"
- **Metis does NOT auto-archive downstream** — it surfaces the change to Alex
- Alex decides what to do: archive the initiative too, or reassign it to a different strategy, or keep it as standalone work

This is a **notification, not an automatic cascade**. Strategic decisions require human judgment.

### Step 6: Disruption — Cross-Team Blocker

The API team discovers their circuit breaker work is blocked by a networking issue that the SRE team needs to fix.

```
api/
  API-T-0001.md        # phase: blocked
                        #   blocked_by: SRE-T-0015   ← cross-workspace reference
```

On next sync, this is visible everywhere:
- SRE team sees they have a blocker affecting the API team
- Alex sees ALPHA-T-0001's downstream work is blocked
- Sarah sees strategy progress is stalled

The `blocked_by` field works exactly like `parent` — it's a short code reference resolved by the projection cache across all hydrated workspaces.

### Gaps Identified by Walkthrough

**1. Pull-not-push task model** (RESOLVED — explicit design choice): Delivery teams pull work by creating their own tasks referencing upstream outcomes. Initiative coordinators describe *what* is needed; delivery teams decide *how* and create their own work items. This is an explicit requirement of good team flow — autonomous teams own their execution. Coordinators define outcomes, not assignments.

**2. Upstream visibility via regular sync** (RESOLVED): Teams discover upstream work through regular sync, not explicit notification. Sync runs automatically (git hooks for dev teams, background sync on GUI launch for coordinators). New upstream documents appear in the "Upstream Work" view after each sync. No separate notification mechanism needed — the sync *is* the notification channel.

**3. Upstream archive handling** (RESOLVED — no special logic needed): Hydrated files are just files on disk. When sync pulls down a document with `archived: true`, it writes the file. The projection cache rebuilds from all files (owned + hydrated) and computes that any local document with `parent: ARCHIVED-DOC` has an archived parent. This is functionally identical to how single-workspace Metis already handles archived parents — no new capability needed. The GUI surfaces the relationship state from the cache.

**4. Cross-workspace projection cache** (RESOLVED — implementation concern): The cache indexes ALL `.metis/**/*.md` files (owned + hydrated) and computes relationships across workspace boundaries. This is a one-time rebuild on sync — fast given the small document count. Not an architectural concern, just an expansion of the existing cache rebuild scope.

**5. Sync frequency asymmetry** (ACCEPTED): Status propagation has inherent latency tied to each layer's sync cadence. Delivery teams sync frequently (every git push). Coordinators sync on GUI launch. Strategy leads sync less often. A task completed Monday may not be visible to the strategy lead until their next sync. This is acceptable — it matches the planning cadence at each layer.

**6. Default scope for new teams** (RESOLVED — design choice): New teams are scoped to their own work by default. The GUI default view shows the team's owned documents plus their direct upstream context (parent chain). Teams don't see "everything and filter down" — they see "their stuff with the ability to browse out." Full fetch still happens (all data is local), but the default display is focused. Expanding scope is opt-in via the GUI.

### Cross-Team Initiatives

**Problem**: How does an initiative that requires work from multiple delivery teams function?

**Example**: ALPHA-I-0001 ("API Error Handling Overhaul") needs work from both the API team and the SRE team.

**How it works in the multi-workspace model**:

The initiative coordinator (Alex) decomposes the initiative into outcome tasks, some of which are relevant to different delivery teams:

```
alpha/
  ALPHA-I-0001.md          # "API Error Handling Overhaul" — phase: decompose
  ALPHA-T-0001.md          # "Circuit breaker pattern for external API calls"
  ALPHA-T-0002.md          # "Network resilience and failover at infrastructure layer"
```

Each delivery team syncs, sees the outcome tasks, and creates their own implementation work:

```
api/
  API-T-0001.md            # "Implement circuit breaker in gateway"
                            #   parent: ALPHA-T-0001
  API-T-0002.md            # "Circuit breaker integration tests"
                            #   parent: ALPHA-T-0001

sre/
  SRE-T-0001.md            # "Configure failover for US-East region"
                            #   parent: ALPHA-T-0002
  SRE-T-0002.md            # "Add health check endpoints for circuit breaker"
                            #   parent: ALPHA-T-0001    ← SRE also contributes to this outcome
```

**Key observations**:

1. **Multiple teams can reference the same outcome task.** ALPHA-T-0001 has children from both API and SRE. The projection cache aggregates all of them when computing progress.

2. **The initiative coordinator sees everything.** Alex's projection cache shows ALPHA-T-0001 has 3 child tasks across 2 workspaces (API-T-0001, API-T-0002, SRE-T-0002), with progress computed across all of them.

3. **Cross-team dependencies use `blocked_by`.** If API-T-0001 depends on SRE-T-0001 being done first, the API team marks `blocked_by: SRE-T-0001`. Both teams see this on next sync.

4. **No shared ownership of documents.** Every document lives in exactly one workspace. Cross-team coordination happens through references (parent, blocked_by), not through shared editing. The initiative coordinator owns the initiative and outcome tasks; delivery teams own their implementation tasks.

5. **The initiative coordinator is the integration point.** Alex monitors progress across teams, identifies when outcome tasks are fully implemented, transitions them to completed, and manages the initiative lifecycle. This is deliberate — cross-team work needs a human coordination point.

**What about two initiative groups contributing to the same strategy?**

This is already natural. Multiple initiatives can have `parent: STRAT-S-0001`:

```
alpha/
  ALPHA-I-0001.md          # parent: STRAT-S-0001 — reliability from the API side
beta/
  BETA-I-0003.md           # parent: STRAT-S-0001 — reliability from the mobile side
```

Sarah (strategy lead) sees both initiatives under her strategy. Progress is aggregated. No special mechanism needed — this is just the parent reference working across workspaces.

### Ownership Model: Any Team Can Create Initiatives

**Design principle**: Document types are not locked to organizational layers. Any team can create any document type. Workspaces enforce *ownership*, not *document type restrictions*.

**The realistic organizational model**:

- **Strategy team** (`strat/`) — Owns strategies. The only team creating strategies in practice, but by convention, not enforcement. Strategies are the one document type that is effectively single-owner because strategic direction must come from one place.
- **Working group(s)** (`wg-reliability/`, `wg-platform/`) — Own cross-team initiative boards. A coordination team that decomposes cross-team work into outcome tasks for delivery teams. These are the "initiative layer" from the Flight Levels model, but they're just another team with a workspace.
- **Delivery teams** (`api/`, `sre/`, `mkt/`) — Own their implementation tasks AND their own initiatives for team-level concerns (internal refactors, tech debt campaigns, tooling improvements, process changes). A delivery team's workspace may contain a vision, multiple initiatives, and tasks — all team-internal.

**Full read transparency**: Because every workspace does a full fetch, there are no shadow initiatives. A delivery team's internal initiative is visible to the strategy lead. A working group's cross-team initiative is visible to all delivery teams. This is intentional — organizational visibility is the whole point.

**What this means in practice**:

```
central/.metis/
  strat/
    STRAT-V-0001.md                  # org vision
    STRAT-S-0001.md                  # "Improve reliability"
    STRAT-S-0002.md                  # "Expand to EU"

  wg-reliability/                     # working group for cross-team reliability
    WGR-V-0001.md                    # working group vision
    WGR-I-0001.md                    # "API Error Handling Overhaul"
                                      #   parent: STRAT-S-0001
    WGR-T-0001.md                    # outcome task: "Circuit breaker pattern"
                                      #   parent: WGR-I-0001

  api/
    API-V-0001.md                    # team vision
    API-I-0001.md                    # team initiative: "Migrate to async runtime"
                                      #   parent: API-V-0001 (team-internal)
    API-T-0001.md                    # "Implement circuit breaker"
                                      #   parent: WGR-T-0001 (cross-team)
    API-T-0005.md                    # "Refactor connection pool"
                                      #   parent: API-I-0001 (team-internal)

  sre/
    SRE-V-0001.md                    # team vision
    SRE-I-0001.md                    # team initiative: "Observability overhaul"
                                      #   parent: SRE-V-0001 (team-internal)
    SRE-T-0001.md                    # "Configure failover"
                                      #   parent: WGR-T-0001 (cross-team)
```

The hierarchy is determined by **parent references**, not by workspace location. A task in `api/` can have a parent in `wg-reliability/` (cross-team work) or in `api/` itself (team-internal work). The projection cache resolves all of this from the files on disk.

## Open Questions (Resolved)

All discovery-phase open questions have been resolved:

1. **GUI discovery UX** — Resolved: URL paste field. `metis init --upstream <url>` for CLI, URL input in GUI first-run wizard. No magic discovery, no invite links. Teams share the central repo URL the same way they share any repo URL.

2. **Authentication** — Resolved: Use git's existing auth. SSH keys, credential helpers, whatever the user already has configured. Metis delegates entirely to git for transport authentication — no custom credential management, no OAuth layer, no tokens in `config.toml`. Application-enforced write scope (only push your own folder) is a separate concern from transport auth.

3. **Transport format** — Resolved: Git is the storage layer, HTTPS/SSH is the transport. Not an abstraction — git trees of markdown files, committed and pushed. No need to keep the door open to tarballs or JSON manifests.

4. **Cross-workspace relationship fields** — Resolved: No new frontmatter fields needed. Existing `parent` and `blocked_by` fields already accept short codes. Since prefixes are globally unique and all data is hydrated locally after sync, the projection cache (`metis.db`) computes all cross-workspace relationships by joining on short codes across all local files. Inverse relationships (`drives`, `supports`) are derived — the cache computes them from `parent` references. No schema changes required.

## Exploration Log

The design evolved through multiple iterations across sessions, each shedding complexity:

> sqlite-sync → cr-sqlite → custom SQLite extension → centralized git (no extension) → event sourcing changelog → per-document state files → state.json per team with programmatic domain merge → SQLite as source of truth → **files as source of truth with dumb transport**

### Key Decision Points

1. **sqlite-sync / cr-sqlite rejected** - CRDT-based SQLite sync extensions. sqlite-sync has Elastic License 2.0; cr-sqlite (MIT) provides generic LWW without domain semantics. Neither needed once centralization eliminated the distributed sync problem.

2. **Centralization over distribution** - One central git repo as single source of truth. Dramatically simpler than peer-to-peer sync. Scale is not a concern (small documents, not enterprise-scale data).

3. **Dual source of truth explored and abandoned** - Splitting content (markdown) from operational state (JSON) was explored to solve within-team merge conflicts. Later abandoned when we recognized contention is low enough that single files with YAML frontmatter work fine.

4. **Event sourcing rejected** - Changelog with compaction via git-hash snapshots was explored in depth. Too complex (log growth, concurrent snapshots, ordering ambiguity). Current-state files are simpler; git history provides the audit trail.

5. **No SQLite extension needed** - Scope narrowed from "full sync extension" → "operational state only" → "not needed at all." No native extension, no CRDT, no custom sync protocol.

6. **Ephemeral git context** - No persistent `.git/` in `.metis/`. libgit2 in-memory operations during sync, torn down afterward. Git is transport, not local version control for coordination data.

7. **Application-enforced ACLs** - Rejected platform-specific mechanisms (GitHub push rulesets, GitLab protected paths). Metis controls what gets pushed. Works on any git service. Trust model same as any shared repo.

8. **Full fetch, sparse push** - Every workspace reads everything. Write scope limited to own workspace folder by the application.

9. **Passive central repo** - Central never pulls. Children push up. Registration = first push. No admin provisioning.

10. **GUI is primary** - Non-technical users are the main audience for multi-team. Git is completely invisible. Every operation has a GUI path.

11. **Sync piggybacks on dev workflow** - Triggered by git hooks (post-commit, post-push), not manual ceremony. Infrequent, small deltas, background operation. Performance is a non-concern. Non-dev teams use the GUI "Sync" button.

12. **Ephemeral context for credential safety** - No persistent `.git/` prevents casual bypass of application-enforced ACLs. Not about determined attackers; about not leaving pre-authenticated push access lying around.

13. **Cross-workspace conflicts don't exist** - Single-writer per workspace folder. Within-workspace contention is low (planning is collaborative) and handled by existing Metis.

14. **"Work travels with the team" is the core value** - Not "no server" or "local-first" as abstract principles. The concrete property to preserve: work data lives where the team lives, works offline, no dependency on a service for daily operations. Central is a beacon for cross-team sync, not the home of the data.

15. **Dual source of truth abandoned** - state.json + content.md split explored in depth but ultimately unnecessary. Single markdown files with YAML frontmatter (exactly like current Metis) carry both content and operational state. No special merge rules, no JSON domain logic.

16. **SQLite pivot explored and reversed** - SQLite as source of truth solves schema/query elegantly but introduces binary merge problems and client version skew. Realized the db can hydrate files on pull and rebuild from files on push - making files the source of truth and the db a transport/cache concern. Then realized: if files are truth and transport is dumb, the db isn't needed in transport at all. Just serialize files.

17. **Dumb transport layer** - Transport doesn't merge, doesn't understand schemas, doesn't apply domain rules. It serializes files for transit and hydrates them into filesystem locations. All intelligence stays local. Format is an implementation detail (git tree of markdown files currently).

18. **Version skew is a non-problem** - YAML frontmatter is naturally forward-compatible. Old versions ignore unknown fields, new versions write fields old versions preserve. No coordinated schema migrations needed.

19. **No custom auth layer** - Metis delegates transport authentication entirely to git's existing mechanisms (SSH keys, credential helpers). No OAuth, no keychain management, no tokens in config files. Application-enforced write scope is separate from transport auth.

20. **No new frontmatter schema for cross-workspace relationships** - Existing `parent` and `blocked_by` fields work with remote short codes out of the box. The projection cache computes inverse relationships (`drives`, `supports`) from `parent` references. Globally unique prefixes make short codes unambiguous across workspaces.