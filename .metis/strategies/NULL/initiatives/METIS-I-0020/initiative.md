---
id: evolution-of-metis-multi-layer
level: initiative
title: "Evolution of Metis: Multi-Layer Sync Architecture"
short_code: "METIS-I-0020"
created_at: 2026-01-29T20:27:41.395016+00:00
updated_at: 2026-01-29T20:27:41.395016+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


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

### Storage Layout

```
my-project/
  .git/                   # the project's own repo
  .gitignore              # includes .metis/
  src/
  .metis/
    config.toml           # workspace config (upstream URL, team identity, credentials ref)
    metis.db              # ephemeral cache, rebuilt on startup from files
    api/                  # this workspace's documents (owned, read-write)
      API-V-0001.md
      API-T-0001.md
      API-T-0002.md
    strat/                # upstream documents (hydrated from central, read-only)
      STRAT-V-0001.md
      STRAT-S-0001.md
    alpha/                # peer documents (hydrated from central, read-only)
      ALPHA-I-0001.md
```

No `.git/` directory inside `.metis/`. Git operations happen transiently during `metis sync`.

- **`*.md` files** - markdown with YAML frontmatter. One per document. The source of truth for both content and operational state. Exactly like current Metis.
- **`metis.db`** - ephemeral cache. Rebuilt from all local markdown files on startup (own workspace + hydrated remote workspaces). Used for fast queries and cross-team visibility.
- **Owned folder** (e.g. `api/`) - read-write. This workspace's documents. Pushed to central on sync.
- **Remote folders** (e.g. `strat/`, `alpha/`) - read-only locally. Hydrated from central on sync. Provide cross-team visibility.

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

### Conflict Resolution

**Cross-workspace conflicts don't exist.** Each workspace folder is single-writer. The transport layer never merges files from different workspaces.

**Within-workspace conflicts** (two people on the same team editing the same document) are handled the same way as current single-workspace Metis. In practice, within-team contention is low - planning is collaborative, not concurrent. When conflicts do occur (rare), they're meaningful - two humans wrote different things. Resolution is a conversation, not an algorithm.

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

### Onboarding & Credential Management

`metis init` (CLI) or the first-run wizard (GUI) handles:

1. **Configure central repo URL** - provide the URL (or scan/paste an invite link). Stored in `.metis/config.toml`.
2. **Authenticate** - OAuth device flow for GitHub/GitLab ("Sign in with GitHub"). Best UX, no tokens or SSH keys to manage. Falls back to personal access token entry for self-hosted or unsupported platforms.
3. **Identity** - name and email pulled from OAuth profile automatically. Stored in `.metis/config.toml`. User can override if needed.
4. **Team assignment** - select or confirm which team/layer this workspace belongs to. Determines write scope (enforced by Metis application).
5. **Initial sync** - first pull from central to get upstream context. If this is a new team, the first push creates the team folder in central (registration = first push).

**Registration model:** There is no admin step to "add a team" to central. A new team configures their `.metis/config.toml` with the central URL and their team identity, then runs `metis sync`. The first push creates their team folder. The central repo is passive - it accepts pushes from any authenticated user.

**Credential storage:**
- Delegate to OS-native credential stores (macOS Keychain, Windows Credential Manager, Linux secret service)
- Credentials scoped to this workspace's central repo URL, not global git config
- Refresh/rotation handled transparently by the OAuth flow where possible

**For CLI users:** `metis init --upstream <url>` walks through the same steps interactively. Tests credentials with `git ls-remote` before proceeding. If auth fails, provides clear instructions for the platform.

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

## Open Questions
- How does the GUI discover and connect to central on first launch? Likely "someone gives you a URL to paste in." Exact UX TBD.
- What does authentication look like in `config.toml`? Token reference? Credential store key? OAuth refresh token?
- What is the exact serialization format for transport? Git tree of markdown files is the current assumption, but could be any format that describes files + locations. Implementation detail.
- What YAML frontmatter fields need to be added/standardized for cross-workspace relationships (e.g. `supports`, `blocked_by` referencing remote short codes)?

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