---
id: explore-release-automation-to-keep
level: task
title: "Explore release automation to keep MCP server and skill docs in sync"
short_code: "METIS-T-0052"
created_at: 2025-12-29T16:52:24.081829+00:00
updated_at: 2025-12-29T16:52:24.081829+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Explore release automation to keep MCP server and skill docs in sync

## Objective

Explore and implement automation to prevent MCP server instructions and skill documentation from drifting out of sync.

## Problem Statement

The MCP server (`instructions.md`) and skill files (`skill/skills/metis/`) both contain phase definitions and methodology guidance. When one is updated, the other can drift:

- MCP server said `todo -> active -> completed`
- Skill files said `todo -> doing -> completed`
- Plugin cache compounded the issue by serving stale skill content

This caused agents to receive conflicting instructions.

## Technical Debt Impact

- **Current Problems**: Manual sync required between MCP instructions and skill docs; easy to forget; plugin cache adds another layer of staleness
- **Benefits of Fixing**: Single source of truth; automatic propagation; no more terminology drift
- **Risk Assessment**: Low risk if not addressed immediately, but recurring maintenance burden and potential for agent confusion

## Acceptance Criteria

## Acceptance Criteria

- [ ] Single source of truth for phase definitions identified
- [ ] Automation approach selected and documented
- [ ] CI/release step implemented to sync or validate consistency
- [ ] Manual drift like today's `doing` vs `active` would be caught before release

## Possible Approaches

### Option 1: Generate skill docs from MCP instructions
- Parse `instructions.md` at build time
- Template-generate skill methodology files
- Pros: True single source; MCP instructions are authoritative
- Cons: Skill docs become less readable/editable

### Option 2: Validate consistency in CI
- Add a check that greps for phase terminology across both
- Fail build if discrepancies found
- Pros: Simple; preserves manual editing of both
- Cons: Reactive, not preventive

### Option 3: Shared constants file
- Define phases in a single location (e.g., YAML or Rust constants)
- Both MCP instructions and skill docs reference or embed from it
- Pros: Clear single source; both can still be authored independently
- Cons: More complex build/embed process

## Implementation Notes

### Recommended Approach
Option 2 (CI validation) as a quick win, potentially evolving to Option 3 for robustness.

### Dependencies
- CI pipeline (GitHub Actions)
- Release process documentation

## Status Updates **[REQUIRED]**

*To be added during implementation*