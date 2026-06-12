---
id: metis-3-0-round-out-export-plugin
level: initiative
title: "Metis 3.0 Round-out (export, plugin skills, polish)"
short_code: "METIS-I-0033"
created_at: 2026-06-12T14:19:55.816181+00:00
updated_at: 2026-06-12T14:19:55.816181+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: metis-3-0-round-out-export-plugin
---

# Metis 3.0 Round-out (export, plugin skills, polish) Initiative

## Context

The server backbone (METIS-I-0031) shipped its core and the web/desktop UI was split into METIS-I-0032. This initiative collects the **tail items** that round out 3.0 but don't belong in either: data export, the agent-experience port, and the small carried deferrals. Scheduled **after METIS-I-0032** (the UI is the higher-value next step); parked in discovery until then. None of these block the server or the UI.

## Goals & Non-Goals

**Goals:**
- **Markdown export** — `GET /api/v1/projects/:slug/export` produces a readable markdown bundle (tar.gz) of a project's items, the lock-in-avoidance escape hatch promised in the vision (DB is the source of truth; export makes the data portable). Browsable markdown on disk is this feature's job (the object store is hash-named).
- **2.x skills/commands port** — bring the Flight Levels methodology skills + Ralph loop commands into the `metis-3.0` plugin, against the server-backed MCP tools (renamed `item` verbs). Includes the `PostToolUse`/`PreCompact`/`Stop` hooks the Ralph loop relies on.
- **Carried deferrals (small):** Postgres-threaded short-code-allocation concurrency test; per-request `tracing` span layer on the server; a real migration-version table (replacing the per-migration probe in `ensure_migrated`); `statement_timeout` wiring if a clean hook appears.

**Non-Goals:**
- s3 object backend (separately deferred on I-0031; revisit only on real need).
- Anything in the UI (METIS-I-0032) or the server core (METIS-I-0031, shipped).
- Org-level multi-tenancy / SSO (later phase, per ADR-0008).

## Detailed Design

To be developed in design (deferred until post-I-0032). Sketches:
- **Export:** stream a tar.gz; one markdown file per item (`{short_code}.md`) with frontmatter (type/phase/assignee/tags/links/exit-criteria) + body, foldered by type or flat; project `config` included. Reuses the service/object-store read path. Bundle layout close to 2.x file conventions but explicitly an interchange format, not a re-importable store (clean break stands).
- **Skills/commands port:** copy `plugins/metis/{skills,commands}` into `plugins/metis-3.0`, adapt MCP tool names (`document`→`item`) and the Ralph setup scripts; wire the Ralph hooks into `plugins/metis-3.0/hooks/hooks.json`.

## Implementation Plan

To be decomposed when this initiative is picked up (post-UI). Rough order: markdown export (service + REST + test) → skills/commands port → carried deferrals as small independent tasks.