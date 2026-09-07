#!/bin/bash
# SessionStart hook for Metis projects
# Detects .metis directory and provides comprehensive project context

# Read hook input from stdin to extract session_id
HOOK_INPUT=$(cat)

# Export session ID so Bash tool commands can use it for session-scoped state files
SESSION_ID=$(echo "$HOOK_INPUT" | jq -r '.session_id // empty')
if [ -n "$CLAUDE_ENV_FILE" ] && [ -n "$SESSION_ID" ]; then
    echo "export CLAUDE_SESSION_ID='$SESSION_ID'" >> "$CLAUDE_ENV_FILE"
fi

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

# Always run code index on session start — it's fast (<1s)
metis index --incremental 2>/dev/null

# Check if semantic summaries need generating
CODE_INDEX_PATH="$CLAUDE_PROJECT_DIR/.metis/code-index.md"
PLACEHOLDER_COUNT=0
if [ -f "$CODE_INDEX_PATH" ]; then
    PLACEHOLDER_COUNT=$(grep -c "Semantic summary to be generated" "$CODE_INDEX_PATH" 2>/dev/null || true)
fi

SUMMARIZER_MSG=""
if [ "$PLACEHOLDER_COUNT" -gt 0 ]; then
    SUMMARIZER_MSG="**${PLACEHOLDER_COUNT} modules are missing semantic summaries.** You SHOULD launch the code-index-summarizer agent in the background immediately (unless the user's request is urgent). Requires Edit(.metis/code-index.md) in allowed tools via /permissions. If not configured, inform the user and offer to run in the foreground instead."
fi

# Build context message for active Metis project
read -r -d '' CONTEXT << EOF
This is a **Metis project** (detected \`.metis\` directory).

## CRITICAL: Metis IS Your System of Record AND Working Memory
- **Metis is the system of record for all work planning and execution.** Every plan, decision, finding, and progress note that needs to outlive this moment belongs in a Metis document — not in plan mode, not in a todo list, not only in the conversation.
- **Metis documents are persistent working memory.** Unlike Claude's plan mode and todo list (ephemeral — wiped by context compaction), Metis documents survive across compactions, sessions, and even different agents. Long-running sessions WILL compact; the only work that survives is what you wrote to Metis.
- **Update active tasks CONSTANTLY.** Treat the active task/initiative as your working memory: record progress, findings, decisions, plan changes, and next steps as you go — every few tool calls, not just at the end. Assume you could be compacted at any moment. If it isn't in Metis, it's lost.
- **For planning work**: Create Metis initiatives and decompose them into tasks. Plan mode is fine for thinking through an approach, but the durable output MUST become a Metis initiative/task — never a standalone markdown plan left outside Metis. Don't use TaskCreate as a planning substitute.
- **TodoWrite is allowed ONLY as a tactical scratchpad** — e.g., sequencing the in-session steps of an implementation that spans multiple Metis tickets out of order. It is ephemeral and disposable. Anything durable (real plans, progress, decisions, next steps) MUST live in Metis. Never let the todo list become a shadow plan or a substitute for updating tasks.
- Before starting work, check for active tasks with \`mcp__metis__list_documents\`.

## CRITICAL: Use Metis's Names, Never Invent Your Own
- **A work item's name is its Metis short code** (\`PREFIX-T-0042\`), and nothing else. Never guess, predict, reserve, renumber, or abbreviate one. Only \`create_document\` mints a short code — read it back from the tool result and use that exact string. If you don't know an item's code, look it up with \`list_documents\`/\`search_documents\`.
- **Never invent an interim ID** for work that doesn't exist yet ("Task 1", "T-3", "AUTH-01", "task A"). Refer to proposed work by its full quoted title, marked as not yet created, and switch to the short code the moment the document exists. Numbers in a list are question numbers for that message only — never carry them forward.
- **Metis's enumerations are closed sets. Translate outside terms; don't adopt them.** Types are \`vision\`, \`initiative\`, \`task\`, \`adr\`, \`specification\` — there is no epic, story, sub-task, chore, or milestone. Backlog categories are \`bug\`, \`feature\`, \`tech-debt\`. Task phases are \`backlog\`, \`todo\`, \`active\`, \`blocked\`, \`completed\` — a task is \`active\`, not "in progress"; \`completed\`, not "done".
- **Never invent grouping entities or numbering schemes** ("workstream A", "batch 2", "phase 1 tasks", "Slice 2", \`D3\` for the third decision). Nothing in Metis holds them. Work that belongs together is children of the same initiative; a slice becomes a task; a decision is an ADR short code (\`PROJ-A-0003\`) or, while still open, a named decision area.
- **Plans are written in short codes.** An implementation plan, a decision log, a status summary — every item names the task, initiative, specification, or ADR it refers to by short code, or by quoted title if the document does not exist yet. **A short code you write into a document must be one you just created or just read back** — never one recalled from memory, and never a forward reference to work that was never created.
- **One carve-out**: a specification's requirements tables keep their IDs (\`REQ-x.y.z\` functional, \`NFR-x.y.z\` non-functional, each with a rationale) — that is where PRD-style requirements live and they must stay citable. Cite one from outside its specification qualified: \`METIS-S-0002 REQ-2.1.1\`. No other document type gets a numbering scheme.
- **Short codes travel outside the conversation too**: commits, PRs, branch names, code TODOs, TodoWrite entries, and status summaries all carry the real short code or no ID at all — never a made-up one.
- Write into the headings the template already gives you (a task's \`Status Updates\` is the progress log). Don't add your own sections or a parallel progress file.
- Load the \`metis-vocabulary\` skill for the full tables and for how to repair an invented name once you spot one.

## Current Project State
${STATE_SUMMARY}

### Actionable Work Items
\`\`\`
${ACTIVE_WORK:-No active or ready tasks found}
\`\`\`

## Code Index — Use Before Exploring
\`.metis/code-index.md\` contains the project structure, key symbols, and semantic summaries for every module. **If you need to find where code lives, read the code index BEFORE using Glob, Grep, or the Explore agent.** It tells you which modules and files are relevant so you can go straight there instead of discovering the codebase from scratch.

- If a task or context already tells you which files to edit — skip the index and go straight to implementation
- If you need to discover where something lives — read the index first, then target your search
- When spawning subagents for implementation — pass the relevant file paths downstream so they don't need to rediscover; if that's not possible, instruct them to read \`.metis/code-index.md\` before exploring
${SUMMARIZER_MSG}

## MCP Tools (Preferred)
Use these MCP tools for all Metis operations:
- \`mcp__metis__list_documents\` - List all documents with their short codes and phases
- \`mcp__metis__read_document\` - Read a document by short code (e.g., METIS-T-0001)
- \`mcp__metis__edit_document\` - Update document content (search and replace)
- \`mcp__metis__transition_phase\` - Move documents through phases (todo->active->completed)
- \`mcp__metis__create_document\` - Create new vision, initiative, task, ADR, or specification documents
- \`mcp__metis__reassign_parent\` - Move tasks between initiatives or to/from backlog
- \`mcp__metis__open_document\` - Open a document in an external viewer (VSCode/system editor) for review. Use \`include_children: true\` to open an initiative with all its tasks.

## CRITICAL: Creating Documents
When you create a document, you MUST immediately populate it with content:
1. \`mcp__metis__create_document\` - Creates document with template
2. \`mcp__metis__read_document\` - Read the template structure
3. \`mcp__metis__edit_document\` - Replace ALL placeholders with real content

**A document with template placeholders is INCOMPLETE. Never leave {placeholder} text.**

## CRITICAL: Human-in-the-Loop for Initiatives
For initiatives, you MUST check in with the human before:
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
