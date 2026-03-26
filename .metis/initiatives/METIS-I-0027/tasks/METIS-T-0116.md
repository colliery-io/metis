---
id: plugin-skill-and-prompt-integration
level: task
title: "Plugin skill and prompt integration"
short_code: "METIS-T-0116"
created_at: 2026-03-26T14:59:09.999021+00:00
updated_at: 2026-03-26T14:59:09.999021+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# Plugin skill and prompt integration

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Update Claude Code plugin skills (Ralph loop, decomposition, etc.) and agent prompts to leverage `open_document` and the viewer workflow — open for review, prompt user to confirm, re-read before continuing.

## Acceptance Criteria

- [ ] Decomposition skill opens initiative + child tasks after decomposition as a final step for user review and sign-off before ralphing begins
- [ ] Decomposition prompts user to review and confirm before any tasks are executed
- [ ] MCP tool descriptions for `open_document` are clear enough for agents to use without additional guidance
- [ ] Agent system prompts / plugin instructions mention the viewer workflow and when to use it
- [ ] Human-in-the-loop checkpoints in skills leverage the viewer rather than dumping raw content

## Implementation Notes

### Technical Approach
- Update decomposition skill (metis-decompose) to call `open_document` with `include_children: true` as the final step after creating tasks — this is the review/sign-off gate before ralphing
- Ralph loop does NOT open documents — by the time we're ralphing, the ticket has already been reviewed and approved
- Update MCP server tool descriptions to include usage guidance for `open_document`
- Update plugin system prompts / SessionStart hook to make agents aware of the open → review → re-read workflow
- Add guidance: "After decomposition, open documents for user review and wait for confirmation before proceeding to execution"

### Dependencies
- METIS-T-0109 (open_document MCP tool)
- METIS-T-0115 (proactive open) — should be working so the plugin can rely on auto-open behavior
- At least one viewer backend functional

## Status Updates

*To be added during implementation*