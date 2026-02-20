---
id: add-human-in-the-loop-guidance-for
level: task
title: "Add human-in-the-loop guidance for initiative and strategy reviews"
short_code: "METIS-T-0060"
created_at: 2026-01-28T14:46:15.907244+00:00
updated_at: 2026-01-28T15:25:09.830049+00:00
parent: METIS-I-0019
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0019
---

# Add human-in-the-loop guidance for initiative and strategy reviews

## Parent Initiative

[[METIS-I-0019]]

## Objective

Add explicit guidance in MCP instructions requiring agents to check in with humans for review during initiatives and strategies, especially during key phase transitions. Agents can guide and even lead, but humans must be kept VERY in the loop.

## Problem Statement

Agents may be proceeding through initiative and strategy work without sufficient human oversight. The Flight Levels methodology emphasizes human decision-making at higher levels, and the tooling should reinforce this by prompting agents to pause, provide overviews, and ask clarifying questions.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MCP instructions include explicit guidance on human check-ins for initiatives/strategies
- [ ] Guidance specifies when to pause for human review (phase transitions, key decisions)
- [ ] Instructions encourage agents to provide overviews and ask clarifying questions
- [ ] Language makes clear agents should guide/lead but not proceed autonomously on strategic work
- [ ] Consider adding checkpoints at specific phases (e.g., before transitioning out of discovery/design)

## Key Behaviors to Encourage

1. **Before phase transitions**: Present current state and ask for approval to proceed
2. **During discovery**: Ask clarifying questions about scope and priorities
3. **During design**: Present options and trade-offs for human decision
4. **At decomposition**: Review task breakdown with human before creating tasks

## Implementation Notes

### Files to Update
- MCP server instruction text
- Possibly phase transition tool descriptions
- Initiative/strategy workflow documentation

### Suggested Guidance
- "For initiatives and strategies, ALWAYS check in with the human before major phase transitions"
- "Present an overview of current state and proposed next steps"
- "Ask clarifying questions rather than making assumptions about strategic direction"

## Status Updates

### 2026-01-28: Completed

**Files modified:**

1. `crates/metis-docs-mcp/instructions.md`
   - Added new "Human-in-the-Loop for Strategic Work" section with:
     - When to check in with humans (phase transitions, design decisions, decomposition)
     - Required behaviors for each phase (discovery, design, decomposition)
     - Example check-in template
     - "What NOT to Do" list
   - Added note to `transition_phase` tool: check in with human before transitioning initiatives/strategies

2. `plugins/metis/hooks/session-start-hook.sh`
   - Added "CRITICAL: Human-in-the-Loop for Initiatives/Strategies" section
   - Lists when to check in and emphasizes getting explicit approval

3. `plugins/metis/hooks/pre-compact-hook.sh`
   - Added "CRITICAL: Human-in-the-Loop" reminder for context restoration

**Key guidance added:**
- ALWAYS check in before phase transitions on initiatives/strategies
- Present options, don't decide unilaterally
- Ask clarifying questions during discovery
- Get explicit approval before decomposition
- "When in doubt, ask"