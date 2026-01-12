#!/bin/bash

# Metis Ralph Setup Script - Task Execution Mode
# Creates state file for Ralph loop driven by a Metis task

set -euo pipefail

# Parse arguments
SHORT_CODE=""
MAX_ITERATIONS=0
PROJECT_PATH=""

show_help() {
  cat << 'HELP_EOF'
Metis Ralph - Execute a Metis task with Ralph loop

USAGE:
  /metis-ralph <SHORT_CODE> [OPTIONS]

ARGUMENTS:
  SHORT_CODE    Metis task short code (e.g., PROJ-T-0001)

OPTIONS:
  --project-path <path>    Path to .metis folder (default: auto-detect)
  --max-iterations <n>     Maximum iterations before auto-stop (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Starts a Ralph loop to execute a Metis task. Claude will:
  1. Read the task content from Metis
  2. Transition the task to "active" phase
  3. Work on implementing the task
  4. Iterate until complete
  5. Transition to "completed" and exit

EXAMPLES:
  /metis-ralph PROJ-T-0001
  /metis-ralph PROJ-T-0001 --max-iterations 20
  /metis-ralph PROJ-T-0001 --project-path /path/to/.metis

STOPPING:
  The loop stops when Claude outputs <promise>TASK COMPLETE</promise>
  after transitioning the task to completed phase.

  Use /cancel-metis-ralph to manually cancel.
HELP_EOF
  exit 0
}

# Parse options
while [[ $# -gt 0 ]]; do
  case $1 in
    -h|--help)
      show_help
      ;;
    --max-iterations)
      if [[ -z "${2:-}" ]] || ! [[ "$2" =~ ^[0-9]+$ ]]; then
        echo "Error: --max-iterations requires a positive integer" >&2
        exit 1
      fi
      MAX_ITERATIONS="$2"
      shift 2
      ;;
    --project-path)
      if [[ -z "${2:-}" ]]; then
        echo "Error: --project-path requires a path argument" >&2
        exit 1
      fi
      PROJECT_PATH="$2"
      shift 2
      ;;
    -*)
      echo "Error: Unknown option: $1" >&2
      echo "Run with --help for usage information" >&2
      exit 1
      ;;
    *)
      if [[ -z "$SHORT_CODE" ]]; then
        SHORT_CODE="$1"
      else
        echo "Error: Unexpected argument: $1" >&2
        exit 1
      fi
      shift
      ;;
  esac
done

# Validate short code
if [[ -z "$SHORT_CODE" ]]; then
  echo "Error: No task short code provided" >&2
  echo "" >&2
  echo "Usage: /metis-ralph <SHORT_CODE> [OPTIONS]" >&2
  echo "" >&2
  echo "Examples:" >&2
  echo "  /metis-ralph PROJ-T-0001" >&2
  echo "  /metis-ralph PROJ-T-0001 --max-iterations 20" >&2
  exit 1
fi

# Validate short code format (should be a task: PREFIX-T-NNNN)
if ! [[ "$SHORT_CODE" =~ ^[A-Z]+-T-[0-9]+$ ]]; then
  echo "Error: Invalid task short code format: $SHORT_CODE" >&2
  echo "" >&2
  echo "Expected format: PREFIX-T-NNNN (e.g., PROJ-T-0001)" >&2
  echo "Note: /metis-ralph is for tasks. Use /metis-decompose for initiatives." >&2
  exit 1
fi

# Auto-detect project path if not provided
if [[ -z "$PROJECT_PATH" ]]; then
  # Look for .metis in current directory or parents
  SEARCH_DIR="$(pwd)"
  while [[ "$SEARCH_DIR" != "/" ]]; do
    if [[ -d "$SEARCH_DIR/.metis" ]]; then
      PROJECT_PATH="$SEARCH_DIR/.metis"
      break
    fi
    SEARCH_DIR="$(dirname "$SEARCH_DIR")"
  done

  if [[ -z "$PROJECT_PATH" ]]; then
    echo "Error: Could not find .metis directory" >&2
    echo "" >&2
    echo "Either:" >&2
    echo "  - Run from within a Metis project directory" >&2
    echo "  - Use --project-path to specify the .metis folder location" >&2
    exit 1
  fi
fi

# Validate project path exists
if [[ ! -d "$PROJECT_PATH" ]]; then
  echo "Error: Project path does not exist: $PROJECT_PATH" >&2
  exit 1
fi

# Create state file
mkdir -p .claude

cat > .claude/metis-ralph.local.md <<EOF
---
active: true
mode: task
short_code: "$SHORT_CODE"
project_path: "$PROJECT_PATH"
iteration: 1
max_iterations: $MAX_ITERATIONS
completion_promise: "TASK COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
---

Execute Metis task $SHORT_CODE
EOF

# Output setup message
cat <<EOF
Metis Ralph activated for task execution

Task: $SHORT_CODE
Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo $MAX_ITERATIONS; else echo "unlimited"; fi)

INSTRUCTIONS:
1. Read the task using mcp__metis__read_document
2. Transition the task to "active" using mcp__metis__transition_phase
3. Implement what the task describes
4. When FULLY complete:
   - Transition to "completed" using mcp__metis__transition_phase
   - Output: <promise>TASK COMPLETE</promise>

The loop will continue until you output the promise after completing the task.

To cancel: /cancel-metis-ralph
EOF
