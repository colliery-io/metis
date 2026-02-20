---
id: feature-sprint-ui-polish-and-agent
level: initiative
title: "Feature Sprint - UI Polish and Agent Guidance"
short_code: "METIS-I-0019"
created_at: 2026-01-28T14:45:53.046103+00:00
updated_at: 2026-01-28T17:06:48.702040+00:00
parent: METIS-V-0001
blocked_by: []
archived: true

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: feature-sprint-ui-polish-and-agent
---

# Feature Sprint - UI Polish and Agent Guidance Initiative

**Target Release**: v1.1

## Context

Feature sprint to address accumulated polish items and improve agent guidance. This is a mechanical tracking initiative for a batch of related improvements spanning GUI usability and MCP instruction quality.

## Goals & Non-Goals

**Goals:**

- Improve GUI usability with better interaction patterns and scrolling
- Strengthen MCP instructions to improve agent behavior
- Fix the reassign_parent MCP bug

**Non-Goals:**

- Major architectural changes
- New core features beyond the scoped items

## Tasks

| Code | Title | Category |
| --- | --- | --- |
| METIS-T-0059 | Strengthen MCP instruction language on document editing | Agent Guidance |
| METIS-T-0060 | Add human-in-the-loop guidance for initiatives/strategies | Agent Guidance |
| METIS-T-0061 | Add scrollable project list in GUI sidebar | GUI |
| METIS-T-0062 | Redesign ticket detail view with read-first mode | GUI |
| METIS-T-0063 | Improve spacing for document metadata display | GUI |
| METIS-T-0064 | Make clicking a ticket open view mode directly | GUI |
| METIS-T-0065 | reassign_parent tool not appearing in MCP server | Bug Fix |

## Implementation Approach

### Agent Guidance (T-0059, T-0060)

Update MCP server instruction text to be more authoritative about:

1. Requiring document content population after creation
2. Human check-ins during strategic work (initiatives/strategies)

### GUI Polish (T-0061, T-0062, T-0063, T-0064)

These are related UI improvements that should be tackled together:

1. Sidebar scrolling (T-0061) - quick CSS fix
2. Interaction model overhaul (T-0062, T-0064) - read-first mode, click to view
3. Metadata spacing (T-0063) - CSS spacing improvements

### Bug Fix (T-0065)

Investigate and fix missing reassign_parent tool in MCP server.

## Progress

*Updates will be added as work progresses*