#!/bin/bash
# Run Claude Code in isolated container
#
# Usage:
#   ./claude-sandbox.sh              # Normal sandboxed session (no network)
#   ./claude-sandbox.sh --auth       # Enable network for OAuth login
#   ./claude-sandbox.sh -- <args>    # Pass args to claude
#
# OAuth: Tokens are stored in ~/.claude on your host. The container mounts
# this directory, so once authenticated, tokens persist across runs.
# If you need to re-authenticate, use --auth flag.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"
IMAGE_NAME="claude-sandbox-mimir"

# Parse arguments
NETWORK_MODE="none"
CLAUDE_ARGS=()

while [[ $# -gt 0 ]]; do
    case $1 in
        --auth)
            NETWORK_MODE="host"
            echo "⚠️  Network enabled for OAuth authentication"
            shift
            ;;
        --)
            shift
            CLAUDE_ARGS=("$@")
            break
            ;;
        *)
            CLAUDE_ARGS+=("$1")
            shift
            ;;
    esac
done

# Build if image doesn't exist
if ! docker image inspect "$IMAGE_NAME" &>/dev/null; then
    echo "Building sandbox image (this may take a few minutes)..."
    docker build -t "$IMAGE_NAME" -f "$SCRIPT_DIR/Dockerfile.claude-sandbox" "$SCRIPT_DIR"
fi

echo "Starting Claude Code sandbox..."
echo "  Project: $PROJECT_DIR"
echo "  Network: $NETWORK_MODE"
echo ""

# Run container
docker run -it --rm \
    -v "$PROJECT_DIR:/workspace" \
    -v "$HOME/.claude:/root/.claude" \
    --network "$NETWORK_MODE" \
    --workdir /workspace \
    "$IMAGE_NAME" \
    claude "${CLAUDE_ARGS[@]}"
