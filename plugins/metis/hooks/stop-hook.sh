#!/bin/bash

# Metis Ralph Stop Hook
# Prevents session exit when a metis-ralph loop is active
# Feeds the task prompt back to continue the loop
# Progress is logged to the task document via MCP

set -euo pipefail

# Check for dirty index and run incremental re-index before exit
DIRTY_FILE="$CLAUDE_PROJECT_DIR/.metis/.index-dirty"
if [ -f "$DIRTY_FILE" ] && [ -s "$DIRTY_FILE" ]; then
    metis index --incremental 2>/dev/null || true
    rm -f "$DIRTY_FILE"
fi

# Read hook input from stdin
HOOK_INPUT=$(cat)

# Check if metis-ralph loop is active
STATE_FILE=".claude/metis-ralph-active.yaml"

if [[ ! -f "$STATE_FILE" ]]; then
  # No active loop - allow exit
  exit 0
fi

# Parse YAML values (simple grep-based parsing)
# Use || true to prevent set -e from killing the script on optional fields
ITERATION=$(grep '^iteration:' "$STATE_FILE" | sed 's/iteration: *//' || true)
MAX_ITERATIONS=$(grep '^max_iterations:' "$STATE_FILE" | sed 's/max_iterations: *//' || true)
MODE=$(grep '^mode:' "$STATE_FILE" | sed 's/mode: *//' || true)
SHORT_CODE=$(grep '^short_code:' "$STATE_FILE" | sed 's/short_code: *//' | tr -d '"' || true)
PROJECT_PATH=$(grep '^project_path:' "$STATE_FILE" | sed 's/project_path: *//' | tr -d '"' || true)
COMPLETION_PROMISE=$(grep '^completion_promise:' "$STATE_FILE" | sed 's/completion_promise: *//' | tr -d '"' || true)
CURRENT_TASK_INDEX=$(grep '^current_task_index:' "$STATE_FILE" | sed 's/current_task_index: *//' || true)

# For tasks mode, extract the task list
TASK_LIST=()
if [[ "$MODE" == "tasks" ]]; then
  while IFS= read -r line; do
    task=$(echo "$line" | sed 's/.*- *"\{0,1\}\([^"]*\)"\{0,1\}/\1/')
    TASK_LIST+=("$task")
  done < <(grep '^ *- ' "$STATE_FILE")
fi

# Validate numeric fields
if [[ ! "$ITERATION" =~ ^[0-9]+$ ]]; then
  echo "Metis Ralph: State file corrupted (invalid iteration: '$ITERATION')" >&2
  rm "$STATE_FILE"
  exit 0
fi

if [[ ! "$MAX_ITERATIONS" =~ ^[0-9]+$ ]]; then
  echo "Metis Ralph: State file corrupted (invalid max_iterations: '$MAX_ITERATIONS')" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Check if max iterations reached
if [[ $MAX_ITERATIONS -gt 0 ]] && [[ $ITERATION -ge $MAX_ITERATIONS ]]; then
  if [[ "$MODE" == "tasks" ]]; then
    echo "Metis Ralph: Max iterations ($MAX_ITERATIONS) reached for multi-task execution"
  else
    echo "Metis Ralph: Max iterations ($MAX_ITERATIONS) reached for $SHORT_CODE"
  fi
  echo ""
  echo "The loop has stopped. The Metis document may be in an intermediate state."
  echo "Use mcp__metis__read_document to check the current phase."
  rm "$STATE_FILE"
  exit 0
fi

# Get transcript path from hook input
TRANSCRIPT_PATH=$(echo "$HOOK_INPUT" | jq -r '.transcript_path')

if [[ ! -f "$TRANSCRIPT_PATH" ]]; then
  echo "Metis Ralph: Transcript file not found" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Check for assistant messages
if ! grep -q '"role":"assistant"' "$TRANSCRIPT_PATH"; then
  echo "Metis Ralph: No assistant messages in transcript" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Extract last assistant message
LAST_LINE=$(grep '"role":"assistant"' "$TRANSCRIPT_PATH" | tail -1)
if [[ -z "$LAST_LINE" ]]; then
  echo "Metis Ralph: Failed to extract last message" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Parse JSON to get text content
LAST_OUTPUT=$(echo "$LAST_LINE" | jq -r '
  .message.content |
  map(select(.type == "text")) |
  map(.text) |
  join("\n")
' 2>&1) || true

if [[ -z "$LAST_OUTPUT" ]]; then
  echo "Metis Ralph: Failed to parse assistant message" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Check for completion promise in <promise> tags
if [[ -n "$COMPLETION_PROMISE" ]]; then
  PROMISE_TEXT=$(echo "$LAST_OUTPUT" | perl -0777 -pe 's/.*?<promise>(.*?)<\/promise>.*/$1/s; s/^\s+|\s+$//g; s/\s+/ /g' 2>/dev/null || echo "")

  if [[ -n "$PROMISE_TEXT" ]] && [[ "$PROMISE_TEXT" = "$COMPLETION_PROMISE" ]]; then
    if [[ "$MODE" == "task" ]]; then
      echo "Metis Ralph: Task $SHORT_CODE completed successfully"
    elif [[ "$MODE" == "tasks" ]]; then
      echo "Metis Ralph: All tasks completed successfully"
    else
      echo "Metis Ralph: Initiative $SHORT_CODE decomposition completed"
    fi
    rm "$STATE_FILE"
    exit 0
  fi
fi

# Not complete - continue loop
NEXT_ITERATION=$((ITERATION + 1))

# Update iteration in state file
TEMP_FILE="${STATE_FILE}.tmp.$$"
sed "s/^iteration: .*/iteration: $NEXT_ITERATION/" "$STATE_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$STATE_FILE"

# Common code index hint for all modes
CODE_INDEX_HINT="If you need to locate code and the task doesn't already tell you which files to edit, read .metis/code-index.md first — do not explore the codebase from scratch."

# Build prompt based on mode
if [[ "$MODE" == "tasks" ]]; then
  # Multi-task serial execution mode
  TASK_NAMES=""
  for t in "${TASK_LIST[@]}"; do
    TASK_NAMES="${TASK_NAMES}  - ${t}\n"
  done
  CURRENT_IDX="${CURRENT_TASK_INDEX:-0}"
  PROMPT_TEXT="Continue executing Metis tasks serially.

$CODE_INDEX_HINT

Tasks to execute:
$(echo -e "$TASK_NAMES")
Current task index: $CURRENT_IDX (0-based)

1. Check which tasks are already completed (phase=\"completed\")
2. Find the next incomplete task in the list
3. For each incomplete task:
   - Read it using mcp__metis__read_document with project_path=\"$PROJECT_PATH\"
   - Transition to \"active\" if in \"todo\"
   - Implement what it describes
   - Log progress to the task's Status Updates section
   - Transition to \"completed\" when done
   - Move to the next task
4. When ALL tasks are complete:
   - Output: <promise>$COMPLETION_PROMISE</promise>"
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Multi-task execution | Complete remaining tasks, output <promise>$COMPLETION_PROMISE</promise> when all done"
elif [[ "$MODE" == "task" ]]; then
  PROMPT_TEXT="Continue working on Metis task $SHORT_CODE.

$CODE_INDEX_HINT

1. Read the task using mcp__metis__read_document with short_code=\"$SHORT_CODE\" and project_path=\"$PROJECT_PATH\"
2. Review what you've done so far (check the Status Updates section)
3. Continue implementing what the task describes
4. Log your progress to the task's Status Updates section using mcp__metis__edit_document
5. When FULLY complete:
   - Do NOT transition to \"completed\" (user will review and approve)
   - Output: <promise>$COMPLETION_PROMISE</promise>"
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Task: $SHORT_CODE | Log progress, complete work, output <promise>$COMPLETION_PROMISE</promise> when ready for review"
elif [[ "$MODE" == "initiative" ]]; then
  PROMPT_TEXT="Continue executing tasks under Metis initiative $SHORT_CODE.

$CODE_INDEX_HINT

1. Read the initiative using mcp__metis__read_document with short_code=\"$SHORT_CODE\" and project_path=\"$PROJECT_PATH\"
2. List all tasks under it using mcp__metis__list_documents
3. Find tasks in \"todo\" or \"active\" phase
4. For each incomplete task:
   - If in \"todo\", transition to \"active\"
   - Implement the task
   - Log progress to the task's Status Updates section
   - Transition the task to \"completed\" when done
5. When ALL tasks are complete (no todo/active remain):
   - Do NOT transition the initiative (user reviews)
   - Output: <promise>$COMPLETION_PROMISE</promise>"
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Initiative: $SHORT_CODE | Execute and complete tasks, output <promise>$COMPLETION_PROMISE</promise> when all done"
else
  # decompose mode
  PROMPT_TEXT="Continue decomposing Metis initiative $SHORT_CODE.

$CODE_INDEX_HINT

1. Read the initiative using mcp__metis__read_document with short_code=\"$SHORT_CODE\" and project_path=\"$PROJECT_PATH\"
2. Review existing tasks created so far
3. Continue creating tasks to fully decompose the initiative
4. Log your progress to the initiative's Status Updates section using mcp__metis__edit_document
5. When FULLY decomposed:
   - Do NOT transition to \"active\" (user will review and approve)
   - Output: <promise>$COMPLETION_PROMISE</promise>"
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Initiative: $SHORT_CODE | Complete decomposition, output <promise>$COMPLETION_PROMISE</promise> when ready for review"
fi

# Output JSON to block stop and feed prompt back
jq -n \
  --arg prompt "$PROMPT_TEXT" \
  --arg msg "$SYSTEM_MSG" \
  '{
    "decision": "block",
    "reason": $prompt,
    "systemMessage": $msg
  }'

exit 0
