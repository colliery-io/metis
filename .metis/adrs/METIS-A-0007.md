---
id: 001-centralized-architecture-for-multi
level: adr
title: "Centralized Architecture for Multi-Team Work Management"
number: 1
short_code: "METIS-A-0007"
created_at: 2026-03-03T02:21:35.203422+00:00
updated_at: 2026-03-03T17:44:14.763653+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-007: Multi-Team / Cross-Repo Work Management Is Out of Scope

## Context

Metis uses a file-based architecture where work management documents live in a `.metis` directory within a repository. This works well for single-team, single-repo workflows but creates real problems for organizations where:

- **Flight Levels is inherently cross-cutting**: Visions and initiatives rarely map 1:1 to a single repository. A stream-aligned team may work across 3-4 repos but have one coherent set of initiatives.
- **Work tracking is bound to code boundaries, not team/value-stream boundaries**: The `.metis` directory ties work management to where code lives, not how teams organize.
- **Multi-repo coordination is awkward**: Syncing file-based stores across repos adds complexity without solving the fundamental mismatch.
- **No shared visibility**: Teams working across repos cannot easily see each other's work without accessing multiple repositories.

These are real problems. The `feature/multi-workspace-sync` branch and `metis-sync` crate attempted to address them.

## Decision

**Multi-team and cross-repo work management is out of scope for Metis.**

Metis is operationally coupled to the repository. It lives in `.metis/` inside the repo, it's invoked by AI agents working in that repo's context, and its value is in bridging the gap between "what are we building" and "what code do I write next." This repo-scoped operational modality is a strength, not a limitation to be engineered around.

The problems listed in the context section are real, but they belong to a different tool — one designed from the ground up for organizational-level coordination, not bolted onto a repo-scoped planning system. Attempting to solve them in Metis would compromise the simplicity and zero-infrastructure nature that makes Metis useful.

Concretely:
- The `metis-sync` crate and `feature/multi-workspace-sync` branch are abandoned
- The Strategy document type (an organizational coordination concept) is being removed from Metis entirely (see METIS-I-0024)
- No centralized database, API layer, or cross-repo sync will be built
- Metis remains a directory of markdown files and a SQLite database inside your repo

## Rationale

- Metis's strength is that it's zero-infrastructure: clone the repo, you have the planning context. Adding Postgres, an API layer, and cross-repo sync destroys this.
- The repo is the natural scope for AI-agent-assisted development. The agent works in one repo at a time. The planning tool should match that scope.
- Even the Strategy document type — the most "organizational" concept in Metis — has proven to not map to anything real at the repo level. If strategy doesn't work at this scope, multi-team coordination certainly won't.
- Better to do one thing well (repo-scoped planning for AI-assisted development) than to do two things poorly.

## Consequences

### Positive

- Metis stays simple: markdown files, SQLite, no infrastructure
- Clear scope boundary prevents feature creep into organizational tooling
- Development effort stays focused on making repo-scoped planning excellent
- The Strategy removal (METIS-I-0024) cleans up dead abstraction weight

### Negative

- Organizations needing cross-repo work coordination must use a separate tool
- Metis cannot provide a unified view of work across repositories
- Teams working across multiple repos will have separate `.metis` contexts per repo

### Neutral

- Nothing prevents a future tool from consuming `.metis` directories across repos as a read-only aggregation layer — Metis just won't be that tool