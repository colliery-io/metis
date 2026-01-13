#!/bin/bash
# Run Metis Ralph in a sandboxed Docker container
#
# Usage:
#   ./run-sandboxed-ralph.sh <SHORT_CODE> [OPTIONS]
#
# Options:
#   --task                Run single task (auto-detected from short code)
#   --max-iterations N    Maximum iterations (default: unlimited)
#   --attach              Attach to container instead of background
#
# Uses Docker's official sandbox feature which:
#   - Handles OAuth authentication automatically
#   - Stores credentials in persistent Docker volume
#   - Provides isolated execution environment
#   - Includes Claude Code and common dev tools

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Parse arguments
SHORT_CODE=""
MAX_ITERATIONS=""
ATTACH_MODE=false
TASK_MODE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --max-iterations)
            MAX_ITERATIONS="$2"
            shift 2
            ;;
        --attach)
            ATTACH_MODE=true
            shift
            ;;
        --task)
            TASK_MODE=true
            shift
            ;;
        -h|--help)
            cat << 'EOF'
Run Metis Ralph in a sandboxed Docker container

USAGE:
    ./run-sandboxed-ralph.sh <SHORT_CODE> [OPTIONS]

ARGUMENTS:
    SHORT_CODE    Task (PROJ-T-NNNN) or Initiative (PROJ-I-NNNN) short code

OPTIONS:
    --task                Execute a single task (auto-detected from code format)
    --max-iterations N    Maximum iterations before auto-stop
    --attach              Attach to container (default: background)
    -h, --help            Show this help

AUTHENTICATION:
    Uses Docker's official sandbox which handles OAuth automatically.
    On first run, you'll be prompted to authenticate via browser.
    Credentials are stored in Docker volume 'docker-claude-sandbox-data'.

EXAMPLES:
    # Run all tasks under an initiative
    ./run-sandboxed-ralph.sh PROJ-I-0001

    # Run a single task
    ./run-sandboxed-ralph.sh PROJ-T-0001

    # Attach to see output in real-time
    ./run-sandboxed-ralph.sh PROJ-T-0001 --attach

COMPLETION:
    Progress is logged to Metis documents in .metis/
EOF
            exit 0
            ;;
        -*)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
        *)
            if [[ -z "$SHORT_CODE" ]]; then
                SHORT_CODE="$1"
            fi
            shift
            ;;
    esac
done

# Validate short code
if [[ -z "$SHORT_CODE" ]]; then
    echo "Error: No short code provided" >&2
    echo "Usage: ./run-sandboxed-ralph.sh <SHORT_CODE> [OPTIONS]" >&2
    exit 1
fi

# Determine mode from short code if not explicitly set
if [[ "$SHORT_CODE" =~ ^[A-Z]+-T-[0-9]+$ ]]; then
    TASK_MODE=true
    CONTAINER_PREFIX="ralph-task"
    MODE_NAME="Task"
    RALPH_CMD="/metis-ralph"
elif [[ "$SHORT_CODE" =~ ^[A-Z]+-I-[0-9]+$ ]]; then
    CONTAINER_PREFIX="ralph-initiative"
    MODE_NAME="Initiative"
    RALPH_CMD="/metis-ralph-initiative"
else
    echo "Error: Invalid short code format: $SHORT_CODE" >&2
    echo "Expected: PREFIX-T-NNNN (task) or PREFIX-I-NNNN (initiative)" >&2
    exit 1
fi

# Find project root (look for .metis directory)
PROJECT_DIR="$(pwd)"
while [[ "$PROJECT_DIR" != "/" ]]; do
    if [[ -d "$PROJECT_DIR/.metis" ]]; then
        break
    fi
    PROJECT_DIR="$(dirname "$PROJECT_DIR")"
done

if [[ ! -d "$PROJECT_DIR/.metis" ]]; then
    echo "Error: Could not find .metis directory" >&2
    echo "Run from within a Metis project directory" >&2
    exit 1
fi

echo "========================================"
echo "  Sandboxed Ralph Launcher"
echo "========================================"
echo ""
echo "Mode:       $MODE_NAME"
echo "Short Code: $SHORT_CODE"
echo "Project:    $PROJECT_DIR"
echo ""

# Build the prompt for Claude
RALPH_PROMPT="Execute $RALPH_CMD $SHORT_CODE"
if [[ -n "$MAX_ITERATIONS" ]]; then
    RALPH_PROMPT="$RALPH_PROMPT --max-iterations $MAX_ITERATIONS"
fi

# Check if docker sandbox is available
if ! docker sandbox --help &>/dev/null 2>&1; then
    echo "Error: 'docker sandbox' command not available" >&2
    echo "" >&2
    echo "Docker sandbox requires Docker Desktop with AI features enabled." >&2
    echo "See: https://docs.docker.com/ai/sandboxes/get-started/" >&2
    exit 1
fi

# Check if sandbox is authenticated
# Credentials are stored inside the sandbox container at /home/agent/.claude/.credentials.json
check_sandbox_auth() {
    # Find any running claude sandbox for this workspace
    local container_name
    container_name=$(docker ps --filter "label=docker/sandbox=true" --format "{{.Names}}" 2>/dev/null | head -1)

    if [[ -n "$container_name" ]]; then
        # Check if credentials exist in the running container
        local has_creds
        has_creds=$(docker exec "$container_name" test -f /home/agent/.claude/.credentials.json 2>/dev/null && echo "yes")
        if [[ "$has_creds" == "yes" ]]; then
            echo "$container_name"  # Return container name for reuse
            return 0
        fi
    fi

    return 1
}

echo "Checking sandbox authentication..."
EXISTING_CONTAINER=$(check_sandbox_auth)
if [[ -z "$EXISTING_CONTAINER" ]]; then
    echo ""
    echo "========================================"
    echo "  Authentication Required"
    echo "========================================"
    echo ""
    echo "No authenticated Docker sandbox found."
    echo ""
    echo "Please run this command in your terminal:"
    echo ""
    echo "  docker sandbox run -w $PROJECT_DIR claude"
    echo ""
    echo "This will:"
    echo "  1. Open a browser for OAuth login"
    echo "  2. Start an interactive Claude session"
    echo "  3. Keep the sandbox running for Ralph to use"
    echo ""
    echo "IMPORTANT: Keep the sandbox running (don't exit)."
    echo "Then re-run this script in another terminal."
    echo ""
    echo "Alternatively, use an API key:"
    echo ""
    echo "  export ANTHROPIC_API_KEY='sk-ant-...'"
    echo "  docker sandbox run -e ANTHROPIC_API_KEY -w $PROJECT_DIR claude"
    echo ""
    exit 1
fi
echo "Authentication: OK"
echo "Using sandbox: $EXISTING_CONTAINER"
echo ""

echo "Launching Ralph..."
echo "Prompt: $RALPH_PROMPT"
echo ""

LOG_FILE="$PROJECT_DIR/.metis/ralph-sandbox-$(date +%Y%m%d-%H%M%S).log"

# Run with docker sandbox
if [[ "$ATTACH_MODE" == "true" ]]; then
    # Attached mode - run in foreground
    echo "Running in foreground..."
    docker exec -it "$EXISTING_CONTAINER" claude --print "$RALPH_PROMPT"
else
    # Detached mode - run in background
    echo "Running in background..."
    echo ""

    # Run the command inside the existing sandbox container
    nohup docker exec "$EXISTING_CONTAINER" claude --print "$RALPH_PROMPT" > "$LOG_FILE" 2>&1 &
    EXEC_PID=$!

    echo "Ralph started!"
    echo ""
    echo "  Container: $EXISTING_CONTAINER"
    echo "  PID:       $EXEC_PID"
    echo "  Log:       $LOG_FILE"
    echo ""
    echo "MONITORING:"
    echo "  tail -f $LOG_FILE"
    echo ""
    echo "PROGRESS:"
    echo "  Progress is logged to Metis documents in:"
    echo "  $PROJECT_DIR/.metis/"
    echo ""
    echo "STOP:"
    echo "  kill $EXEC_PID"
    echo ""
fi
