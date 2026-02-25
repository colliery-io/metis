#!/bin/bash
# SessionStart hook for Metis projects
# Detects .metis directory and provides comprehensive project context

# Exit silently if not in a Metis project
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

# Check if metis is installed
if ! command -v metis &> /dev/null; then
    cat << 'ENDJSON'
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "WARNING: This is a Metis project (`.metis` directory found) but the `metis` command is not installed or not in PATH. Install it from: https://github.com/colliery-io/metis"
    }
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

# Check for code index
CODE_INDEX_PATH="$CLAUDE_PROJECT_DIR/.metis/code-index.md"
if [ -f "$CODE_INDEX_PATH" ]; then
    CODE_INDEX_MSG="Code index available at \`.metis/code-index.md\` — read it for codebase orientation (project structure, key symbols, module summaries)."
else
    CODE_INDEX_MSG="No code index found. Run \`metis index\` or use the \`index_code\` MCP tool to generate \`.metis/code-index.md\` for codebase navigation."
fi

# Build context message for active Metis project
read -r -d '' CONTEXT << EOF
This is a **Metis project** (detected \`.metis\` directory).

## CRITICAL: Work Tracking Rules
- **Do NOT use TodoWrite** for tracking work in this project. Metis documents ARE your work tracking system.
- **ALWAYS update active Metis tasks** with progress as you work - they serve as persistent memory across sessions.
- Before starting work, check for active tasks with \`mcp__metis__list_documents\`.

## Current Project State
${STATE_SUMMARY}

### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready tasks found}
\`\`\`

## Code Index
${CODE_INDEX_MSG}

## MCP Tools (Preferred)
Use these MCP tools for all Metis operations:
- \`mcp__metis__list_documents\` - List all documents with their short codes and phases
- \`mcp__metis__read_document\` - Read a document by short code (e.g., METIS-T-0001)
- \`mcp__metis__edit_document\` - Update document content (search and replace)
- \`mcp__metis__transition_phase\` - Move documents through phases (todo->active->completed)
- \`mcp__metis__create_document\` - Create new vision, initiative, task, or ADR documents
- \`mcp__metis__reassign_parent\` - Move tasks between initiatives or to/from backlog

## CRITICAL: Creating Documents
When you create a document, you MUST immediately populate it with content:
1. \`mcp__metis__create_document\` - Creates document with template
2. \`mcp__metis__read_document\` - Read the template structure
3. \`mcp__metis__edit_document\` - Replace ALL placeholders with real content

**A document with template placeholders is INCOMPLETE. Never leave {placeholder} text.**

## CRITICAL: Human-in-the-Loop for Initiatives/Strategies
For initiatives and strategies, you MUST check in with the human before:
- Transitioning to a new phase
- Making design/architectural decisions
- Decomposing into tasks
- Any significant directional choice

Present options, ask clarifying questions, and get explicit approval. Do NOT proceed autonomously on strategic work.

## Working on a Task
When you receive a task short code:
1. \`mcp__metis__read_document\` - Read the task to understand requirements
2. \`mcp__metis__transition_phase\` - Transition to "active" (from todo)
3. Work on the task, updating the document with progress regularly
4. \`mcp__metis__transition_phase\` - Transition to "completed" when done

## Available Skills
- \`/metis-ralph <short-code>\` - Execute a task with iterative Ralph loop
- \`/metis-decompose <short-code>\` - Break an initiative into tasks
- \`/cancel-metis-ralph\` - Cancel active Ralph loop
EOF

# Output JSON for Claude
cat << ENDJSON
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
    }
}
ENDJSON

exit 0
