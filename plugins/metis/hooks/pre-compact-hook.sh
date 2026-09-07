#!/bin/bash
# PreCompact hook for Metis projects
# Re-injects Metis context after context compaction with current project state

# Exit silently if not in a Metis project
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

# Check if metis is installed
if ! command -v metis &> /dev/null; then
    cat << 'ENDJSON'
{
    "systemContext": "WARNING: This is a Metis project (`.metis` directory found) but the `metis` command is not installed or not in PATH. Install it from: https://github.com/colliery-io/metis"
}
ENDJSON
    exit 0
fi

# Get current project state
# Try compact format first (newer versions), fall back to default output
cd "$CLAUDE_PROJECT_DIR" || exit 0

# Check for dirty index and run incremental re-index
DIRTY_FILE="$CLAUDE_PROJECT_DIR/.metis/.index-dirty"
INDEX_UPDATE_MSG=""
if [ -f "$DIRTY_FILE" ] && [ -s "$DIRTY_FILE" ]; then
    DIRTY_COUNT=$(wc -l < "$DIRTY_FILE" | tr -d ' ')
    metis index --incremental 2>/dev/null
    INDEX_UPDATE_MSG="Code index updated (${DIRTY_COUNT} files re-indexed). Module summaries may need updating for changed directories."
    rm "$DIRTY_FILE"
fi

STATUS_OUTPUT=$(metis status --format compact 2>/dev/null)
if [ -z "$STATUS_OUTPUT" ]; then
    # Fall back to default output and extract what we can
    STATUS_OUTPUT=$(metis status 2>/dev/null | grep -E "^[A-Z]+-[A-Z]-[0-9]+")
fi
ACTIVE_WORK=$(echo "$STATUS_OUTPUT" | grep -E "(active|todo|blocked)" | head -10)
BLOCKED_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "blocked" 2>/dev/null || true)
ACTIVE_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "active" 2>/dev/null || true)
TODO_COUNT=$(echo "$STATUS_OUTPUT" | grep -c "todo" 2>/dev/null || true)
# Ensure counts are numbers (handle empty strings)
[ -z "$BLOCKED_COUNT" ] && BLOCKED_COUNT=0
[ -z "$ACTIVE_COUNT" ] && ACTIVE_COUNT=0
[ -z "$TODO_COUNT" ] && TODO_COUNT=0

# Build state summary
STATE_SUMMARY=""
if [ "$BLOCKED_COUNT" != "0" ]; then
    STATE_SUMMARY="**${BLOCKED_COUNT} BLOCKED**, "
fi
if [ "$ACTIVE_COUNT" != "0" ]; then
    STATE_SUMMARY="${STATE_SUMMARY}${ACTIVE_COUNT} active, "
fi
if [ "$TODO_COUNT" != "0" ]; then
    STATE_SUMMARY="${STATE_SUMMARY}${TODO_COUNT} ready to start"
fi
STATE_SUMMARY="${STATE_SUMMARY:-No actionable items}"

# Build context message for active Metis project
read -r -d '' CONTEXT << EOF
## CONTEXT RESTORED: Metis Project

### CRITICAL: Metis IS Your System of Record AND Working Memory
- **You were just compacted — this is exactly why Metis matters.** Anything not written to a Metis document before now is gone. Re-read your active task to recover state.
- **Metis is the system of record for all work planning and execution.** Plans, decisions, findings, and progress live in Metis documents, which survive compaction; plan mode and the todo list do not.
- **Update active tasks CONSTANTLY** as you work — progress, findings, decisions, next steps — every few tool calls. Assume you could be compacted again at any moment.
- **For planning**: Use Metis initiatives and tasks. Plan mode is fine for thinking, but its output MUST become a Metis ticket — not a stray markdown plan. Don't use TaskCreate as a planning substitute.
- **TodoWrite is allowed ONLY as an ephemeral tactical scratchpad** (e.g., sequencing steps across multiple tickets). Durable plans and progress MUST go in Metis — never let the todo list become a shadow plan.
- Check for active tasks with \`mcp__metis__list_documents\`, then \`read_document\` to restore context.

### Current Project State
${STATE_SUMMARY}

$([ -n "$INDEX_UPDATE_MSG" ] && printf '%s\n\n' "### Code Index" "${INDEX_UPDATE_MSG}")
### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready tasks found}
\`\`\`

### MCP Tools
- \`mcp__metis__list_documents\` - List all documents
- \`mcp__metis__read_document\` - Read by short code (e.g., METIS-T-0001)
- \`mcp__metis__edit_document\` - Update document content
- \`mcp__metis__transition_phase\` - Move through phases (todo->active->completed)
- \`mcp__metis__create_document\` - Create new documents (MUST populate content after!)
- \`mcp__metis__reassign_parent\` - Move tasks between initiatives

### CRITICAL: Creating Documents
After \`create_document\`, you MUST: \`read_document\` then \`edit_document\` to populate ALL content. Never leave placeholder text.

### CRITICAL: Human-in-the-Loop
For initiatives: ALWAYS check in with the human before phase transitions, design decisions, or decomposition. Present options and get approval.

### Task Workflow
1. \`read_document\` - Understand the task
2. \`transition_phase\` - Move to "active"
3. Work and update task with progress
4. \`transition_phase\` - Move to "completed"

### Skills
- \`/metis-ralph <short-code>\` - Execute task with Ralph loop
- \`/grill-vision <short-code|title>\` - Interview the human to sharpen a vision, writing answers into the document
- \`/grill-initiative <short-code|title>\` - Interview the human to sharpen an initiative before decomposition
- \`/grill-decomposition <short-code>\` - Review an initiative's drafted tasks as a set before execution
EOF

# Output JSON for Claude - PreCompact uses systemContext field
cat << ENDJSON
{
    "systemContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
}
ENDJSON

exit 0
