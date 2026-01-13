---
description: "Cancel active Metis Ralph loop"
allowed-tools: ["Bash(test -f .claude/metis-ralph-active.yaml:*)", "Bash(rm .claude/metis-ralph-active.yaml)", "Read(.claude/metis-ralph-active.yaml)"]
hide-from-slash-command-tool: "true"
---

# Cancel Metis Ralph

To cancel the Metis Ralph loop:

1. Check if `.claude/metis-ralph-active.yaml` exists using Bash: `test -f .claude/metis-ralph-active.yaml && echo "EXISTS" || echo "NOT_FOUND"`

2. **If NOT_FOUND**: Say "No active Metis Ralph loop found."

3. **If EXISTS**:
   - Read `.claude/metis-ralph-active.yaml` to get the current state:
     - `iteration:` field for iteration count
     - `mode:` field for loop type (task or decompose)
     - `short_code:` field for the document being worked on
   - Remove the file using Bash: `rm .claude/metis-ralph-active.yaml`
   - Report: "Cancelled Metis Ralph loop for [SHORT_CODE] (was at iteration N, mode: MODE)"

Note: Cancelling does NOT revert any Metis document phase transitions. Progress logged to the task document is preserved.
