---
id: on-demand-architecture-and
level: initiative
title: "On-Demand Architecture and Documentation Review Agents"
short_code: "METIS-I-0029"
created_at: 2026-03-28T14:11:40.228865+00:00
updated_at: 2026-03-28T14:11:40.228865+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: on-demand-architecture-and
---

# On-Demand Architecture and Documentation Review Agents Initiative

## Context

Metis manages architecture decisions (ADRs) and project documentation, but there's no tooling to verify that the codebase still reflects those decisions or that documentation follows consistent quality standards. Over time, code drifts from documented decisions, and docs accumulate without structure.

These are audit problems — not continuous enforcement, but periodic health checks run on a cadence (or on demand). Two agents that read existing artifacts, cross-reference against the codebase, and produce actionable reports.

## Goals & Non-Goals

**Goals:**
- Build an on-demand architecture review agent that cross-references ADRs against the code index and source code to flag drift
- Build an on-demand documentation review agent that classifies docs against the Diataxis framework and identifies gaps/misclassifications
- Both agents produce structured reports and can optionally create backlog items for findings
- Both are invokable via slash commands (`/review-architecture`, `/review-docs`) — not continuous or gating
- Reports should be useful without being noisy — flag genuine drift, not cosmetic issues

**Non-Goals:**
- Continuous enforcement or CI integration (these are periodic audits)
- Blocking transitions or deployments based on review results
- Auto-fixing findings — the agents report, humans decide

## Use Cases

### UC-1: Periodic Architecture Health Check
- **Actor**: Developer or tech lead on a cadence (e.g., monthly, before a release)
- **Scenario**: Runs `/review-architecture`. Agent reads all decided ADRs, scans the code index and relevant source files, produces a report listing each ADR with compliance status (compliant / drift detected / unable to verify). Findings with drift include specific file references and a description of the divergence.
- **Expected Outcome**: A structured report the team can review. Optionally, backlog items created for significant drift.

### UC-2: Documentation Gap Analysis
- **Actor**: Developer or docs maintainer
- **Scenario**: Runs `/review-docs`. Agent scans all markdown documentation, classifies each doc into Diataxis quadrants (tutorial, how-to, reference, explanation), flags misclassifications (e.g., a how-to that's actually a tutorial), and identifies coverage gaps (e.g., "no tutorials exist for the MCP server").
- **Expected Outcome**: A Diataxis coverage matrix and list of actionable improvements. Optionally, backlog items for documentation work.

## Detailed Design

### Architecture Review Agent

**Inputs:**
- All ADRs in `decided` status (read via Metis MCP tools)
- Code index (`.metis/code-index.md`) for project structure and symbols
- Source files as needed for verification

**Process:**
1. Read all decided ADRs, extract the decision and its implications
2. For each ADR, determine what code patterns/files/structures should exist (or not exist) based on the decision
3. Cross-reference against code index and source files
4. Classify each ADR as: compliant, drift detected, unable to verify
5. Produce a structured report

**Output:** Markdown report with per-ADR findings. Optional backlog item creation for drift items.

### Documentation Review Agent

**Inputs:**
- All markdown files in the docs/ directory (or configured doc paths)
- Diataxis framework reference (bundled as skill reference material)

**Process:**
1. Scan all documentation files
2. For each doc, classify into Diataxis quadrant based on content analysis
3. Check if the doc's location/naming matches its classification
4. Build a coverage matrix: which quadrants are covered, for which areas
5. Identify gaps (missing quadrants for key features) and misclassifications

**Output:** Diataxis coverage matrix, per-doc classification with confidence, list of gaps and misclassifications. Optional backlog item creation.

### Implementation as Claude Code Agents

Both are implemented as Claude Code plugin agents (not skills) — they need to do exploratory work across multiple files and produce a deliverable. Each gets:
- An agent definition (`.md` file with frontmatter)
- A slash command to invoke it
- Reference material bundled in the plugin

## Alternatives Considered

### Skills instead of agents
Rejected — skills inject guidance at decision points, but these need to read many files, cross-reference, and produce a report. That's agent territory.

### Continuous enforcement via hooks
Rejected — too noisy for the value. Architecture drift and doc quality degrade slowly; periodic audits match the cadence of the problem.

### Single combined agent
Considered — but architecture review and doc review are distinct enough in inputs, process, and output that separate agents are cleaner. They share a pattern but not implementation.

## Implementation Plan

### Phase 1: Architecture Review Agent
- Define agent with prompt, reference material, and tool access
- Create `/review-architecture` command
- Build ADR reading and cross-referencing logic into the prompt
- Test against existing Metis ADRs

### Phase 2: Documentation Review Agent
- Define agent with Diataxis reference material
- Create `/review-docs` command
- Build doc scanning and classification logic into the prompt
- Test against existing Metis docs

### Phase 3: Backlog Integration
- Add optional backlog item creation for findings from both agents
- Configurable: prompt user before creating, or auto-create with a tag