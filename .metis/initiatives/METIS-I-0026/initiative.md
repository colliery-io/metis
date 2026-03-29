---
id: back-metis-with-jira
level: initiative
title: "Back Metis with JIRA"
short_code: "METIS-I-0026"
created_at: 2026-03-10T13:54:31.301933+00:00
updated_at: 2026-03-10T20:05:26.339518+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/ready"


exit_criteria_met: false
estimated_complexity: L
initiative_id: back-metis-with-jira
---

# Back Metis with JIRA Initiative

## Context

Metis is gaining traction at a company where JIRA is the established work management and tracking system. Rather than replacing JIRA, Metis should integrate with it — using JIRA as the backing store and source of truth for organizational work tracking while Metis provides the AI-native planning and execution layer on top.

The initial model maps Metis concepts to JIRA constructs on a single JIRA board:
- **Metis Initiatives** → **JIRA Epics**
- **Metis Tasks** → **JIRA Tickets** (linked to their parent Epic)

This allows teams to continue using JIRA dashboards, reports, and workflows while developers and AI agents interact with work through Metis.

## Goals & Non-Goals

**Goals:**
- Sync Metis initiatives and tasks bidirectionally with a JIRA board (epics and tickets)
- Allow Metis to create, read, update, and transition JIRA issues
- Maintain JIRA as the system of record — Metis reads from and writes to JIRA
- Preserve the Metis developer/agent experience (MCP tools, flight levels, Ralph loop) while backed by JIRA
- Support a single-board model as the starting point

**Non-Goals:**
- Replacing JIRA or duplicating all JIRA functionality
- Supporting multiple boards or complex JIRA project configurations initially
- Syncing Metis visions, ADRs, or specifications to JIRA (these remain Metis-local for now)
- Real-time webhook-driven sync (polling or on-demand sync is fine initially)
- Migrating existing Metis markdown documents into JIRA

## Detailed Design

### Architecture: Stateless JIRA Pass-Through

The MCP server becomes a **thin translation layer** over JIRA's REST API. No local database, no local files, no cache, no sync. Every MCP tool call is one or more JIRA API calls.

- The MCP tool interface stays identical — agents see no difference
- No SQLite database — removed entirely
- No filesystem storage — removed entirely
- No sync logic — JIRA is always fresh, always authoritative
- Separate repository: `colliery-io/metis-jira` (private), forked from `colliery-io/metis` with shared git history

### MCP Tool → JIRA API Mapping

| MCP Tool | JIRA API Call |
|---|---|
| `list_documents` | `GET /search` (JQL query) |
| `read_document` | `GET /issue/{key}` |
| `create_document` | `POST /issue` (Epic for initiative, Task for task) |
| `edit_document` | `PUT /issue/{key}` (update description/fields) |
| `transition_phase` | `POST /issue/{key}/transitions` + label swap |
| `archive_document` | `POST /issue/{key}/transitions` → Done + `metis:archived` label |

### Hierarchy

- Initiatives = JIRA Epics
- Tasks = JIRA Tasks, linked to parent Epic
- Querying an epic and its children (`parent = PROJ-123`) gives the full hierarchy

### Key Design Decisions

- **JIRA Cloud REST API v3** for all operations
- **No local state** — no SQLite, no files, no cache. Every read hits JIRA.
- **Metis short codes as JIRA labels** for cross-referencing (e.g., label `METIS-I-0026`)
- **Metis phases as JIRA labels** (e.g., `metis:discovery`, `metis:design`) — JIRA board columns are the lossy view
- **Default Kanban board**: Backlog → Selected for Development → In Progress → Done (no customization)
- **Phase → Status mapping**:

| JIRA Status | Initiative Phases | Task Phases |
|---|---|---|
| Backlog | discovery, design | backlog, todo |
| Selected for Development | ready, decompose | todo (ready to pick up) |
| In Progress | active | active |
| Done | completed | completed |

- Metis enforces phase gates — can't transition JIRA status forward until prerequisite Metis phases are satisfied
- **No import** of pre-existing JIRA tickets — only tracks what Metis creates
- **Minimal JIRA footprint** — labels only, no custom fields, no workflow modifications
- **Authentication**: API tokens initially (Atlassian Cloud)
- **Error handling**: JIRA unreachable = operation fails, no silent fallback

### Configuration

Project needs to know:
- JIRA instance URL (e.g., `https://mycompany.atlassian.net`)
- JIRA project key (e.g., `PROJ`)
- API token + email for auth
- Board ID (for validation)

## Alternatives Considered

- **JIRA-only (no Metis)**: Loses the AI-native planning experience, flight levels methodology, and MCP tool integration that makes Metis valuable for agent-driven development.
- **Metis-only (no JIRA)**: Not viable in an organization where JIRA is the mandated tracking system. Teams need JIRA for cross-org visibility, reporting, and compliance.
- **One-way sync (Metis → JIRA)**: Simpler but doesn't capture updates made directly in JIRA by other team members.
- **Webhook-based real-time sync**: More complex infrastructure requirement; polling/on-demand is sufficient for initial adoption.

## Task Breakdown

1. **JIRA API client crate** — Build a Rust HTTP client for JIRA Cloud REST v3 (auth, issue CRUD, transitions, labels, JQL queries)
2. **Strip local storage** — Remove SQLite, filesystem service, sync service, database service, and all related code from the fork
3. **Rewrite MCP tools** — Rewire the 6 MCP tools (list, read, create, edit, transition, archive) to call the JIRA client instead of the old storage layer
4. **Phase/label management** — Implement phase gate logic using JIRA labels (`metis:phase` labels, short code labels, status ↔ phase mapping)
5. **Configuration** — Add JIRA connection config (instance URL, project key, API token/email, board ID) and board validation on startup
6. **Plugin/hook updates** — Update Claude Code plugin instructions and session hooks to work with the JIRA-backed variant