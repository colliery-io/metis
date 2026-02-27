---
id: update-sessionstart-hook-to-detect
level: task
title: "Update SessionStart hook to detect code index presence"
short_code: "METIS-T-0073"
created_at: 2026-02-20T14:47:12.574140+00:00
updated_at: 2026-02-25T05:19:47.350330+00:00
parent: METIS-I-0021
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

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

### Session 1 (2026-02-24)
- Updated `plugins/metis/hooks/session-start-hook.sh` (marketplace source)
- Added `CODE_INDEX_PATH` check for `.metis/code-index.md`
- If present: outputs "Code index available at `.metis/code-index.md` — read it for codebase orientation"
- If missing: outputs "No code index found. Run `metis index` or use the `index_code` MCP tool to generate..."
- Added `## Code Index` section to the CONTEXT heredoc between work items and MCP tools
- Does not block session start in either case
- Tested both scenarios (with and without code-index.md) — both produce correct output
- Copied updated hook to plugin cache
- All acceptance criteria met