#!/bin/bash

# Metis Ralph Stop Hook
# Prevents session exit when a metis-ralph loop is active
# Feeds the task prompt back to continue the loop
# Progress is logged to the task document via MCP

set -euo pipefail

# Read hook input from stdin
HOOK_INPUT=$(cat)

# Check if metis-ralph loop is active
STATE_FILE=".claude/metis-ralph-active.yaml"

if [[ ! -f "$STATE_FILE" ]]; then
  # No active loop - allow exit
  exit 0
fi

# Parse YAML values (simple grep-based parsing)
ITERATION=$(grep '^iteration:' "$STATE_FILE" | sed 's/iteration: *//')
MAX_ITERATIONS=$(grep '^max_iterations:' "$STATE_FILE" | sed 's/max_iterations: *//')
MODE=$(grep '^mode:' "$STATE_FILE" | sed 's/mode: *//')
SHORT_CODE=$(grep '^short_code:' "$STATE_FILE" | sed 's/short_code: *//' | tr -d '"')
PROJECT_PATH=$(grep '^project_path:' "$STATE_FILE" | sed 's/project_path: *//' | tr -d '"')
COMPLETION_PROMISE=$(grep '^completion_promise:' "$STATE_FILE" | sed 's/completion_promise: *//' | tr -d '"')

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
  echo "Metis Ralph: Max iterations ($MAX_ITERATIONS) reached for $SHORT_CODE"
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
' 2>&1)

if [[ $? -ne 0 ]] || [[ -z "$LAST_OUTPUT" ]]; then
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

# Build prompt based on mode
if [[ "$MODE" == "task" ]]; then
  PROMPT_TEXT="Continue working on Metis task $SHORT_CODE.

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
