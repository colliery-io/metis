#!/bin/bash
# PostToolUse hook: track modified source files for deferred re-indexing

# Exit if not a Metis project
[ ! -d "$CLAUDE_PROJECT_DIR/.metis" ] && exit 0

# Read hook input from stdin
HOOK_INPUT=$(cat)

# Extract file_path from tool_input
FILE_PATH=$(echo "$HOOK_INPUT" | jq -r '.tool_input.file_path // empty')
[ -z "$FILE_PATH" ] && exit 0

# Skip non-source files
# Only track files that tree-sitter can parse: .rs .py .ts .tsx .js .jsx .go
case "$FILE_PATH" in
  *.rs|*.py|*.ts|*.tsx|*.js|*.jsx|*.go) ;;
  *) exit 0 ;;
esac

# Skip files inside .metis/ directory
case "$FILE_PATH" in
  */.metis/*) exit 0 ;;
esac

# Append to dirty file (deduplicated)
DIRTY_FILE="$CLAUDE_PROJECT_DIR/.metis/.index-dirty"
if [ -f "$DIRTY_FILE" ]; then
  grep -qxF "$FILE_PATH" "$DIRTY_FILE" || echo "$FILE_PATH" >> "$DIRTY_FILE"
else
  echo "$FILE_PATH" > "$DIRTY_FILE"
fi

exit 0
