# How to Run Metis in a Docker Sandbox

Run Ralph loops in an isolated Docker sandbox for autonomous task execution without permission prompts.

## Prerequisites

- Docker Desktop with the sandbox feature enabled
- A Metis project initialized in your workspace
- Claude Code installed

## Start the Sandbox

```bash
docker sandbox run -w "$(pwd)" claude
```

This mounts your current directory into the sandbox. On first run, it opens OAuth authentication in your browser.

## Install Metis Inside the Sandbox

Once inside the sandbox, install the CLI:

```
Download and install the metis CLI from https://github.com/colliery-io/metis/releases/latest to /usr/local/bin/metis
```

## Install the Plugin

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
```

## Add the MCP Server

```
!claude mcp add --scope user metis metis mcp
```

## Verify

```
/mcp
```

Expected:
```
metis: metis mcp - Connected
```

## Run Ralph Loops

Execute a single task:
```
/metis-ralph PROJ-T-0001
```

Execute all tasks under an initiative:
```
/metis-ralph-initiative PROJ-I-0001
```

## What the Sandbox Provides

- **Isolation** — Changes are contained within the sandbox filesystem
- **No permission prompts** — Bypass permissions mode is available for fully autonomous execution
- **OAuth authentication** — Credentials handled by Docker sandbox
- **Workspace mounting** — Your project files are accessible at the original path

## Important Notes

- Plugin and MCP configuration must be re-done each time you start a fresh sandbox
- Consider keeping a sandbox running for repeated use
- Progress logged to Metis documents persists in your mounted workspace (since `.metis/` is part of the mounted directory)
- Document phase transitions persist even if the sandbox is destroyed
