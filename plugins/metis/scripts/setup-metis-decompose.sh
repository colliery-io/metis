#!/bin/bash

# Metis Ralph Setup Script - Initiative Decomposition Mode
# Creates state file for Ralph loop to decompose an initiative into tasks

set -euo pipefail

# Parse arguments
SHORT_CODE=""
MAX_ITERATIONS=0
PROJECT_PATH=""

show_help() {
  cat << 'HELP_EOF'
Metis Decompose - Break down an initiative into tasks with Ralph loop

USAGE:
  /metis-decompose <SHORT_CODE> [OPTIONS]

ARGUMENTS:
  SHORT_CODE    Metis initiative short code (e.g., PROJ-I-0001)

OPTIONS:
  --project-path <path>    Path to .metis folder (default: auto-detect)
  --max-iterations <n>     Maximum iterations before auto-stop (default: unlimited)
  -h, --help               Show this help message

DESCRIPTION:
  Starts a Ralph loop to decompose a Metis initiative into tasks. Claude will:
  1. Read the initiative content from Metis
  2. Transition the initiative to "decompose" phase
  3. Analyze requirements and create tasks under the initiative
  4. Iterate until decomposition is complete
  5. Transition initiative to "active" and exit

EXAMPLES:
  /metis-decompose PROJ-I-0001
  /metis-decompose PROJ-I-0001 --max-iterations 15
  /metis-decompose PROJ-I-0001 --project-path /path/to/.metis

STOPPING:
  The loop stops when Claude outputs <promise>DECOMPOSITION COMPLETE</promise>
  after transitioning the initiative to the active phase.

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
  echo "Error: No initiative short code provided" >&2
  echo "" >&2
  echo "Usage: /metis-decompose <SHORT_CODE> [OPTIONS]" >&2
  echo "" >&2
  echo "Examples:" >&2
  echo "  /metis-decompose PROJ-I-0001" >&2
  echo "  /metis-decompose PROJ-I-0001 --max-iterations 15" >&2
  exit 1
fi

# Validate short code format (should be an initiative: PREFIX-I-NNNN)
if ! [[ "$SHORT_CODE" =~ ^[A-Z]+-I-[0-9]+$ ]]; then
  echo "Error: Invalid initiative short code format: $SHORT_CODE" >&2
  echo "" >&2
  echo "Expected format: PREFIX-I-NNNN (e.g., PROJ-I-0001)" >&2
  echo "Note: /metis-decompose is for initiatives. Use /metis-ralph for tasks." >&2
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
# Document access goes through MCP - no file path needed
mkdir -p .claude

cat > .claude/metis-ralph-active.yaml <<EOF
# Metis Ralph Loop State
# Progress is logged to the initiative document via MCP
short_code: "$SHORT_CODE"
project_path: "$PROJECT_PATH"
mode: decompose
iteration: 1
max_iterations: $MAX_ITERATIONS
completion_promise: "DECOMPOSITION COMPLETE"
started_at: "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
EOF

# Output setup message
cat <<EOF
Metis Ralph activated for initiative decomposition

Initiative: $SHORT_CODE
Project: $PROJECT_PATH
Max iterations: $(if [[ $MAX_ITERATIONS -gt 0 ]]; then echo $MAX_ITERATIONS; else echo "unlimited"; fi)

INSTRUCTIONS:
1. Read the initiative using mcp__metis__read_document
2. Transition the initiative to "decompose" using mcp__metis__transition_phase
3. Analyze the initiative and create tasks using mcp__metis__create_document
4. Log progress to the initiative's Status Updates section using mcp__metis__edit_document
5. When decomposition is complete:
   - Do NOT transition to "active" (user will do this after review)
   - Output: <promise>DECOMPOSITION COMPLETE</promise>

Progress is logged to the initiative document. User reviews and approves the final transition.

To cancel: /cancel-metis-ralph
EOF
