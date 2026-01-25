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

### CRITICAL: Work Tracking Rules
- **Do NOT use TodoWrite** for tracking work. Metis documents ARE your work tracking system.
- **ALWAYS update active Metis tasks** with progress as you work.
- Check for active tasks with \`mcp__metis__list_documents\`.

### Current Project State
${STATE_SUMMARY}

### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready tasks found}
\`\`\`

### MCP Tools
- \`mcp__metis__list_documents\` - List all documents
- \`mcp__metis__read_document\` - Read by short code (e.g., METIS-T-0001)
- \`mcp__metis__edit_document\` - Update document content
- \`mcp__metis__transition_phase\` - Move through phases (todo->active->completed)
- \`mcp__metis__create_document\` - Create new documents
- \`mcp__metis__reassign_parent\` - Move tasks between initiatives

### Task Workflow
1. \`read_document\` - Understand the task
2. \`transition_phase\` - Move to "active"
3. Work and update task with progress
4. \`transition_phase\` - Move to "completed"

### Skills
- \`/metis-ralph <short-code>\` - Execute task with Ralph loop
- \`/metis-decompose <short-code>\` - Break initiative into tasks
EOF

# Output JSON for Claude - PreCompact uses systemContext field
cat << ENDJSON
{
    "systemContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
}
ENDJSON

exit 0
