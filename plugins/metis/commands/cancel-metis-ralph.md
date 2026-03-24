---
description: "Cancel active Metis Ralph loop"
allowed-tools: ["Bash(test -f .claude/metis-ralph-active*:*)", "Bash(rm .claude/metis-ralph-active*:*)", "Bash(ls .claude/metis-ralph-active*:*)", "Read(.claude/metis-ralph-active*)"]
hide-from-slash-command-tool: "true"
---

# Cancel Metis Ralph

To cancel the Metis Ralph loop:

1. Find the active state file. Check for session-scoped file first, then legacy:
   ```bash
   if [ -n "$CLAUDE_SESSION_ID" ] && [ -f ".claude/metis-ralph-active-${CLAUDE_SESSION_ID}.yaml" ]; then
     echo "FOUND:.claude/metis-ralph-active-${CLAUDE_SESSION_ID}.yaml"
   elif [ -f ".claude/metis-ralph-active.yaml" ]; then
     echo "FOUND:.claude/metis-ralph-active.yaml"
   else
     echo "NOT_FOUND"
   fi
   ```

2. **If NOT_FOUND**: Say "No active Metis Ralph loop found."

3. **If FOUND**:
   - Read the state file to get the current state:
     - `iteration:` field for iteration count
     - `mode:` field for loop type (task or decompose)
     - `short_code:` field for the document being worked on
   - Remove the file using Bash: `rm <state_file_path>`
   - Report: "Cancelled Metis Ralph loop for [SHORT_CODE] (was at iteration N, mode: MODE)"

Note: Cancelling does NOT revert any Metis document phase transitions. Progress logged to the task document is preserved.
