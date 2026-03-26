---
id: read-before-edit-guard
level: task
title: "Read-before-edit guard"
short_code: "METIS-T-0108"
created_at: 2026-03-26T14:59:09.789307+00:00
updated_at: 2026-03-26T16:49:11.009457+00:00
parent: METIS-I-0027
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0027
---

# Read-before-edit guard

## Parent Initiative

[[METIS-I-0027]] — External Document Viewer Integration

## Objective

Implement an mtime-based guard in the MCP server that prevents `edit_document` from overwriting changes made externally (e.g., by a user editing in VSCode or the GUI). The server tracks when each document was last read and rejects edits if the file has been modified since.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] MCP server maintains an in-memory `HashMap<ShortCode, SystemTime>` of last-read timestamps
- [ ] `read_document` updates the last-read timestamp for the requested document
- [ ] `edit_document` checks file `mtime` against last-read — rejects with a clear error if stale
- [ ] `edit_document` without a prior `read_document` is rejected with "must read before editing"
- [ ] On successful edit, last-read timestamp is updated to current time
- [ ] Error messages instruct the agent to re-read the document before retrying
- [ ] Existing tests pass, new tests cover: read-then-edit (pass), edit-without-read (fail), read-edit-external-modify-edit (fail)

## Implementation Notes

### Technical Approach
- Add a `last_read: Arc<Mutex<HashMap<ShortCode, SystemTime>>>` (or similar) to the MCP server state
- Hook into `read_document` handler to record `SystemTime::now()` after successful read
- Hook into `edit_document` handler to stat file mtime and compare before applying edit
- Use `std::fs::metadata().modified()` for mtime
- Keep it in-memory only — no persistence needed, resets on server restart

### Dependencies
- None — can be implemented independently of the viewer work

### Risk Considerations
- Filesystem mtime granularity varies by OS (typically 1s on macOS HFS+, sub-ms on APFS). May need a small tolerance window.
- If the MCP server itself writes the file via `edit_document`, mtime will update — make sure to refresh last-read after our own writes.

## Status Updates

- **2026-03-26**: Implemented. Created `DocumentReadTracker` with `Mutex<HashMap<PathBuf, SystemTime>>`. Wired into server handler as `Arc<DocumentReadTracker>`. `read_document` records timestamps via `call_tool_with_tracker`. `edit_document` checks mtime guard before editing, records after successful write. 1-second tolerance for filesystem granularity. 4 unit tests: read-then-edit passes, edit-without-read rejected, edit-after-external-modify rejected, record-edit-allows-subsequent. All 18 MCP tests pass.