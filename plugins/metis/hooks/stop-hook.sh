#!/bin/bash

# Metis Ralph Stop Hook
# Prevents session exit when a metis-ralph loop is active
# Feeds the task/initiative prompt back to continue the loop

set -euo pipefail

# Read hook input from stdin
HOOK_INPUT=$(cat)

# Check if metis-ralph loop is active
STATE_FILE=".claude/metis-ralph.local.md"

if [[ ! -f "$STATE_FILE" ]]; then
  # No active loop - allow exit
  exit 0
fi

# Parse markdown frontmatter (YAML between ---) and extract values
FRONTMATTER=$(sed -n '/^---$/,/^---$/{ /^---$/d; p; }' "$STATE_FILE")

# Extract values from frontmatter
ITERATION=$(echo "$FRONTMATTER" | grep '^iteration:' | sed 's/iteration: *//')
MAX_ITERATIONS=$(echo "$FRONTMATTER" | grep '^max_iterations:' | sed 's/max_iterations: *//')
MODE=$(echo "$FRONTMATTER" | grep '^mode:' | sed 's/mode: *//')
SHORT_CODE=$(echo "$FRONTMATTER" | grep '^short_code:' | sed 's/short_code: *//' | tr -d '"')
PROJECT_PATH=$(echo "$FRONTMATTER" | grep '^project_path:' | sed 's/project_path: *//' | tr -d '"')
COMPLETION_PROMISE=$(echo "$FRONTMATTER" | grep '^completion_promise:' | sed 's/completion_promise: *//' | tr -d '"')

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
      echo "Metis Ralph: Task $SHORT_CODE completed"
    else
      echo "Metis Ralph: Initiative $SHORT_CODE decomposition completed"
    fi
    rm "$STATE_FILE"
    exit 0
  fi
fi

# Not complete - continue loop
NEXT_ITERATION=$((ITERATION + 1))

# Extract prompt (everything after closing ---)
PROMPT_TEXT=$(awk '/^---$/{i++; next} i>=2' "$STATE_FILE")

if [[ -z "$PROMPT_TEXT" ]]; then
  echo "Metis Ralph: State file missing prompt" >&2
  rm "$STATE_FILE"
  exit 0
fi

# Update iteration in state file
TEMP_FILE="${STATE_FILE}.tmp.$$"
sed "s/^iteration: .*/iteration: $NEXT_ITERATION/" "$STATE_FILE" > "$TEMP_FILE"
mv "$TEMP_FILE" "$STATE_FILE"

# Build context message based on mode
if [[ "$MODE" == "task" ]]; then
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Task: $SHORT_CODE | Complete the task, transition to 'completed', then output <promise>$COMPLETION_PROMISE</promise>"
else
  SYSTEM_MSG="Metis Ralph iteration $NEXT_ITERATION | Initiative: $SHORT_CODE | Complete decomposition, transition to 'active', then output <promise>$COMPLETION_PROMISE</promise>"
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
