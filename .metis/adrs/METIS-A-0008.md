---
id: 001-metis-as-a-deployed-service-with
level: adr
title: "Metis as a Deployed Service with Dual-Backend Storage"
number: 1
short_code: "METIS-A-0008"
created_at: 2026-06-11T11:05:00.095+00:00
updated_at: 2026-06-11T11:06:02.703679+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-008: Metis as a Deployed Service with Dual-Backend Storage

**Supersedes:** METIS-A-0007 (Multi-Team / Cross-Repo Work Management Is Out of Scope)

## Context

METIS-A-0007 declared multi-team and cross-repo work management out of scope, on the grounds that Metis's strength was its zero-infrastructure, repo-coupled modality: clone the repo, get the planning context. That decision was honest at the time, but the constraint it protected has become the limiting factor:

- **Real work spans repositories.** A team's initiatives rarely map 1:1 to a single repo. Per-repo `.metis` directories fragment one coherent stream of work into disconnected islands with no shared visibility and no cross-repo relationships (blocking, linking, unified boards).
- **The previous answer was "use JIRA."** METIS-I-0026 proposed making Metis a stateless pass-through over JIRA to get organizational visibility. That preserves the agent experience but surrenders the data model, workflow semantics, and operational simplicity to a tool whose complexity is exactly what teams adopt Metis to avoid.
- **File/git-based federation was rejected deliberately.** Hub planning repos, cross-repo sync (`metis-sync`), and read-only aggregation layers all hack around the storage model rather than fixing it. A-0007 was right that those approaches compromise the tool; it was wrong that the only alternative was staying repo-scoped.
- **The storage layer is already 90% of the way to portable.** `metis-docs-core` uses Diesel 2 with SQLite and `returning_clauses_for_sqlite_3_35`. The `diesel-dualdb` crate (write-once Diesel queries against both PostgreSQL and SQLite, including portable UUID/timestamp/JSON types and single-arm `RETURNING`) closes the remaining gap without forking the query code.

## Decision

**Metis becomes a deployable service backed by PostgreSQL, while retaining a local SQLite mode — one core, two backends, no file-based federation.**

Concretely:

1. **A Metis server** (single binary) exposes the document model over HTTP and MCP. A team deploys it self-hosted (e.g., docker-compose: `metis-server` + `postgres`). The server is the system of record for that team's work.
2. **PostgreSQL is the server backend; SQLite remains the local/solo backend.** Both run through one storage core via `diesel-dualdb` — queries are written once, the backend is selected by connection URL. No second query codebase, no sync between modes.
3. **Work items reference repositories instead of living inside them.** The `.metis`-directory-per-repo model stops being the unit of ownership for team deployments. Cross-repo links (blocks/relates-to) and fleet-wide views become first-class.
4. **The local mode survives** for solo and offline use — same crates, same MCP tools, SQLite backend. It is a deployment mode, not a separate product.
5. **Markdown stops being the storage format and becomes an interchange format** (export/snapshot for portability, review, and lock-in avoidance) as the server mode matures. The database is the source of truth.
6. **Day-one tenancy target: one team, self-hosted.** Users, assignees, and token-based auth from the start; org-level multi-tenancy and roles are explicitly later.
7. **Scope expands toward a general-purpose tracker**: configurable work item types and workflows (loosening the hard Vision→Initiative→Task hierarchy), and a real query layer (filter by phase/repo/assignee/tag, saved views). JIRA's daily-use core, without its administrative sprawl.

METIS-I-0026 (Back Metis with JIRA) is archived: Metis becomes the lightweight tracker itself rather than a frontend over someone else's.

## Alternatives Analysis

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| Stay repo-scoped (A-0007 status quo) | Zero infrastructure; simplest | Multi-repo work stays fragmented; pushes teams to JIRA for coordination | Low | None |
| JIRA as backing store (METIS-I-0026) | Org visibility for free; no storage to build | Inherits JIRA's data model and admin burden; Metis becomes a translation shim; Atlassian dependency | Medium | Medium |
| File/git federation (hub repo or aggregation over `.metis` dirs) | Stays git-native | Sync conflicts, eventual-consistency hacks, git as an unfit database; already rejected once (`metis-sync`) | High | High |
| Deployed service on Postgres + SQLite local mode (chosen) | Real multi-repo model; honest system of record; one query codebase via diesel-dualdb; local mode preserved | Metis now has a server to operate; auth/identity to build; migration for existing projects | Medium | High |

## Rationale

- The repo-coupling defended by A-0007 was a proxy for what actually matters: **low operational burden and no vendor lock-in**. A single self-hosted binary on Postgres preserves both better than git-federation hacks or an Atlassian dependency.
- `diesel-dualdb` removes the historical cost of dual backends — the reason "SQLite or Postgres, pick one" used to be forced. One code path serves the solo developer and the team deployment.
- AI agents work per-repo, but the *work* they execute does not. A server lets an agent in any repo see and update the same backbone, which is the multi-repo flow A-0007 couldn't serve.
- Becoming a lightweight JIRA alternative is a smaller total system than maintaining a faithful bidirectional JIRA translation layer forever.

## Consequences

### Positive
- One coherent work backbone across a fleet of repositories: cross-repo links, unified views, shared visibility
- Team features (users, assignees, query/reporting) become possible on a data model Metis owns
- Single storage codebase for both deployment modes via diesel-dualdb
- Drops the JIRA dependency path entirely

### Negative
- Metis acquires operational surface: a server to deploy, a database to back up, auth to get right
- Existing `.metis` projects need a migration story (local SQLite → server import)
- The "clone the repo, you have the context" property is lost for team deployments
- Vision and plugin/hook integrations assume repo-local storage and must be reworked

### Neutral
- Markdown shifts from storage format to export/interchange format
- Flight Levels remains the default methodology but becomes one configuration among configurable workflows
- METIS-A-0007 is superseded, not deleted — the reasoning that file-based federation is a dead end remains endorsed