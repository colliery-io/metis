---
id: 001-centralized-architecture-for-multi
level: adr
title: "Centralized Architecture for Multi-Team Work Management"
number: 1
short_code: "METIS-A-0007"
created_at: 2026-03-03T02:21:35.203422+00:00
updated_at: 2026-03-03T02:21:35.203422+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/draft"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-1: Centralized Architecture for Multi-Team Work Management

*This template includes sections for various types of architectural decisions. Delete sections that don't apply to your specific use case.*

## Context

Metis currently uses a file-based architecture where work management documents live in a `.metis` directory within a repository. This works well for single-team, single-repo workflows but creates fundamental problems for real-world organizations:

- **Flight Levels is inherently cross-cutting**: Visions, strategies, and initiatives rarely map 1:1 to a single repository. A stream-aligned team may work across 3-4 repos but have one coherent set of initiatives.
- **Work tracking is bound to code boundaries, not team/value-stream boundaries**: The `.metis` directory ties work management to where code lives, not how teams organize.
- **Multi-repo coordination is awkward**: The `metis-sync` approach (syncing file-based stores across repos) adds complexity without solving the fundamental mismatch — you either pick one repo as source of truth or duplicate work tracking.
- **No shared visibility**: Teams working across repos cannot easily see each other's work without accessing multiple repositories.

The `feature/multi-workspace-sync` branch attempted to solve this with a sync crate, but this approach layers complexity on top of the wrong abstraction.

## Decision

Deprecate the file-based multi-workspace sync approach (`metis-sync` crate, `feature/multi-workspace-sync` branch) and move toward a centralized architecture:

- **Postgres database** as the single source of truth for work management documents
- **API layer** for CRUD operations and team/value-stream organization
- **MCP server** points at the API instead of the filesystem
- **CLI/plugin** integrate with repos but are not bound to them
- The existing file-based `.metis` directory remains supported for single-repo/local use cases

## Rationale

- The file-based approach is fundamentally repo-scoped. Syncing files across repos is fighting the architecture rather than fixing it.
- A centralized store naturally supports organizing work by team/value stream rather than by repository.
- The MCP interface and CLI can remain largely unchanged — only the backend storage changes.
- Postgres is a proven choice for this kind of structured document storage with relational queries.
- This aligns with how Flight Levels is actually practiced: work flows through organizational structures (teams, streams), not through code repositories.

## Consequences

### Positive
- Work management organized around teams and value streams, not repos
- Shared visibility across an organization
- Single source of truth — no sync conflicts
- Natural support for cross-repo initiatives
- Existing MCP/CLI interfaces can be preserved

### Negative
- Requires infrastructure (Postgres, API hosting) — higher operational overhead than a directory of markdown files
- Migration path needed for existing `.metis` projects
- Loses the simplicity of "just a directory in your repo"
- Development effort to build and maintain API layer

### Neutral
- File-based mode can coexist as a local/offline fallback
- The `feature/multi-workspace-sync` branch work is deprecated but the domain modeling insights (strategy phases, etc.) may inform the centralized design