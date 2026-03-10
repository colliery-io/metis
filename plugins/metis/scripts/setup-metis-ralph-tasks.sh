#!/bin/bash

# Metis Ralph Tasks Setup Script
# Creates state file for Ralph loop to execute a list of tasks serially

set -euo pipefail

# Parse arguments
SHORT_CODES=()
MAX_ITERATIONS=0
PROJECT_PATH=""

show_help() {
  cat << 'HELP_EOF'
Metis Ralph Tasks - Execute a list of tasks serially with Ralph loop

USAGE:
  /metis-ralph-tasks <SHORT_CODE> [SHORT_CODE...] [OPTIONS]

ARGUMENTS:
  SHORT_CODE    One or more Metis task short codes (e.g., PROJ-T-0001 PROJ-T-0002)

OPTIONS:
  --project-path <path>    Path to .metis folder (default: auto-detect)
  --max-iterations <n>     Maximum iterations before auto-stop (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Starts a Ralph loop to execute multiple Metis tasks serially. Claude will:
  1. Work through each task in order
  2. For each task: read, activate, implement, log progress, complete
  3. Auto-complete each task and move to the next
  4. Signal ready for user review when all tasks are done

EXAMPLES:
  /metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 PROJ-T-0003
  /metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 --max-iterations 50

STOPPING:
  Use /cancel-metis-ralph to stop the loop.
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
      # Validate task short code format
      if ! [[ "$1" =~ ^[A-Z]+-T-[0-9]+$ ]]; then
        echo "Error: Invalid task short code format: $1" >&2
        echo "" >&2
        echo "Expected format: PREFIX-T-NNNN (e.g., PROJ-T-0001)" >&2
        exit 1
      fi
      SHORT_CODES+=("$1")
      shift
      ;;
  esac
done

# Validate at least one short code
if [[ ${#SHORT_CODES[@]} -eq 0 ]]; then
  echo "Error: No task short codes provided" >&2
  echo "" >&2
  echo "Usage: /metis-ralph-tasks <SHORT_CODE> [SHORT_CODE...] [OPTIONS]" >&2
  echo "" >&2
  echo "Examples:" >&2
  echo "  /metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 PROJ-T-0003" >&2
  exit 1
fi

# Auto-detect project path if not provided
if [[ -z "$PROJECT_PATH" ]]; then
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

# Build YAML list of short codes
TASK_LIST=""
for code in "${SHORT_CODES[@]}"; do
  TASK_LIST="${TASK_LIST}  - \"${code}\"\n"
done

# Create pointer file with loop state
mkdir -p .claude

cat > .claude/metis-ralph-active.yaml <<EOF
# Metis Ralph Loop State - Multi-Task Serial Execution
# Working through an explicit list of tasks
project_path: "$PROJECT_PATH"
mode: tasks
tasks:
$(for code in "${SHORT_CODES[@]}"; do echo "  - \"${code}\""; done)
current_task_index: 0
iteration: 1
max_iterations: $MAX_ITERATIONS
completion_promise: "ALL TASKS COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EOF

# Output setup message
cat <<EOF
Metis Ralph activated for multi-task serial execution

Tasks (${#SHORT_CODES[@]}):
$(for i in "${!SHORT_CODES[@]}"; do echo "  $((i+1)). ${SHORT_CODES[$i]}"; done)
Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo $MAX_ITERATIONS; else echo "unlimited"; fi)

INSTRUCTIONS:
1. Verify all tasks exist using mcp__metis__read_document
2. For each task in order:
   - Transition to "active"
   - Implement what it describes
   - Log progress to the task's Status Updates
   - Transition to "completed" when done
   - Move to the next task
3. When ALL tasks are complete:
   - Output: <promise>ALL TASKS COMPLETE</promise>

Each task is auto-completed. User reviews all work at the end.

To cancel: /cancel-metis-ralph
EOF
