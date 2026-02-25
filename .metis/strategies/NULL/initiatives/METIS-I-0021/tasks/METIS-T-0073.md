---
id: update-sessionstart-hook-to-detect
level: task
title: "Update SessionStart hook to detect code index presence"
short_code: "METIS-T-0073"
created_at: 2026-02-20T14:47:12.574140+00:00
updated_at: 2026-02-20T14:47:12.574140+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0021
---

# Update SessionStart hook to detect code index presence

## Parent Initiative
[[METIS-I-0021]]

## Objective

Update the Metis plugin's SessionStart hook to check for `.metis/code-index.md`. If missing, inform the agent it doesn't exist and suggest creating one. If present, inform the agent where to find it for codebase orientation.

## Acceptance Criteria

- [ ] SessionStart hook checks for `.metis/code-index.md`
- [ ] If missing: outputs message suggesting index creation
- [ ] If present: outputs message with file path for agent reference
- [ ] Does not block session start regardless of index presence
- [ ] Hook tested in a project with and without an index file

## Implementation Notes

Update `plugins/metis/hooks/session-start-hook.sh`. Add a file existence check after the existing Metis project detection. Keep the message concise -- the agent should know where to look, not get a wall of text.

No blockers -- can be done in parallel with the Rust crate work.

## Progress

*Updated during implementation*