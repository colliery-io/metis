#!/bin/bash
# SessionStart hook for Metis projects
# Detects .metis directory and provides project context

# Exit silently if not in a Metis project
if [ ! -d "$CLAUDE_PROJECT_DIR/.metis" ]; then
    exit 0
fi

# Check if metis-mcp is installed
if ! command -v metis-mcp &> /dev/null; then
    cat << 'ENDJSON'
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "WARNING: This is a Metis project (`.metis` directory found) but the `metis-mcp` command is not installed or not in PATH. Install it from: https://github.com/colliery-io/metis"
    }
}
ENDJSON
    exit 0
fi

# Build context message for active Metis project
read -r -d '' CONTEXT << 'EOF'
This is a **Metis project** (detected `.metis` directory).

## Quick Reference
- Use `mcp__metis__list_documents` to see current project state
- Check active work items and their phases
- Use Metis skills for methodology guidance

## Available Skills
When working with this project:
- **document-selection** - Guidance on choosing the right document type (vision, strategy, initiative, task, ADR)
- **phase-transitions** - How to transition documents through phases correctly
- **decomposition** - Patterns for breaking down work into tasks
- **project-patterns** - Common project patterns (greenfield, tech debt, incident response, feature development)

## MCP Server
The `metis` MCP server should be connected. If Metis tools are not available, the MCP server may need to be started.
EOF

# Output JSON for Claude
cat << ENDJSON
{
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": "$(echo "$CONTEXT" | sed 's/"/\\"/g' | tr '\n' ' ')"
    }
}
ENDJSON

exit 0
