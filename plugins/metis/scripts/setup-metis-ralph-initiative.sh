#!/bin/bash

# Metis Ralph Initiative Setup Script
# Creates state file for Ralph loop to execute all tasks under an initiative

set -euo pipefail

# Parse arguments
SHORT_CODE=""
MAX_ITERATIONS=0
PROJECT_PATH=""

show_help() {
  cat << 'HELP_EOF'
Metis Ralph Initiative - Execute all tasks under a decomposed initiative

USAGE:
  /metis-ralph-initiative <SHORT_CODE> [OPTIONS]

ARGUMENTS:
  SHORT_CODE    Metis initiative short code (e.g., PROJ-I-0001)

OPTIONS:
  --project-path <path>    Path to .metis folder (default: auto-detect)
  --max-iterations <n>     Maximum iterations before auto-stop (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Starts a Ralph loop to execute ALL tasks under a decomposed initiative. Claude will:
  1. Read the initiative and list all its tasks
  2. Work through each task in "todo" phase
  3. Implement each task, logging progress
  4. Continue until all tasks are complete
  5. Signal ready for user review

PREREQUISITES:
  The initiative should be in "decompose" or "active" phase with tasks created.
  Use /metis-decompose first if the initiative hasn't been broken down yet.

EXAMPLES:
  /metis-ralph-initiative PROJ-I-0001
  /metis-ralph-initiative PROJ-I-0001 --max-iterations 50

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
  echo "Error: No initiative short code provided" >&2
  echo "" >&2
  echo "Usage: /metis-ralph-initiative <SHORT_CODE> [OPTIONS]" >&2
  echo "" >&2
  echo "Examples:" >&2
  echo "  /metis-ralph-initiative PROJ-I-0001" >&2
  echo "  /metis-ralph-initiative PROJ-I-0001 --max-iterations 50" >&2
  exit 1
fi

# Validate short code format (should be an initiative: PREFIX-I-NNNN)
if ! [[ "$SHORT_CODE" =~ ^[A-Z]+-I-[0-9]+$ ]]; then
  echo "Error: Invalid initiative short code format: $SHORT_CODE" >&2
  echo "" >&2
  echo "Expected format: PREFIX-I-NNNN (e.g., PROJ-I-0001)" >&2
  echo "Note: /metis-ralph-initiative is for initiatives. Use /metis-ralph for single tasks." >&2
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

# Create pointer file with loop state
mkdir -p .claude

cat > .claude/metis-ralph-active.yaml <<EOF
# Metis Ralph Loop State - Initiative Execution Mode
# Working through all tasks under an initiative
short_code: "$SHORT_CODE"
project_path: "$PROJECT_PATH"
mode: initiative
iteration: 1
max_iterations: $MAX_ITERATIONS
completion_promise: "INITIATIVE COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EOF

# Output setup message
cat <<EOF
Metis Ralph activated for initiative execution

Initiative: $SHORT_CODE
Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo $MAX_ITERATIONS; else echo "unlimited"; fi)

INSTRUCTIONS:
1. Read the initiative using mcp__metis__read_document
2. List tasks under it using mcp__metis__list_documents
3. For each task in "todo" phase:
   - Transition to "active"
   - Implement the task
   - Log progress to the task's Status Updates
   - Transition to "completed" when done
4. When ALL tasks are complete:
   - Do NOT transition the initiative (user reviews)
   - Output: <promise>INITIATIVE COMPLETE</promise>

Tasks are auto-completed. User reviews the initiative as a whole at the end.

To cancel: /cancel-metis-ralph
EOF
