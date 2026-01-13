# Docker Sandbox Setup for Autonomous Execution

Run Metis Ralph loops in an isolated Docker sandbox for autonomous task execution without permission prompts.

## Prerequisites

- Docker Desktop with sandbox feature enabled
- A Metis project initialized in your workspace

## Quick Start

### 1. Start the Sandbox

```bash
docker sandbox run -w "$(pwd)" claude
```

This opens OAuth authentication in your browser on first run.

### 2. Install Metis CLI

Once inside Claude, ask it to install the Metis CLI:

```
Download and install the metis CLI from https://github.com/colliery-io/metis/releases/latest to /usr/local/bin/metis
```

### 3. Install Plugin

```
/plugin marketplace add colliery-io/metis
```

```
/plugin install metis@colliery-io-metis
```

### 4. Add MCP Server

```
!claude mcp add --scope user metis metis mcp
```

### 5. Verify Setup

```
/mcp
```

Should show:
```
metis: metis mcp - Connected
```

## Running Ralph in the Sandbox

Once configured, you can run autonomous task execution:

```
/metis-ralph PROJ-T-0001
```

Or execute all tasks under an initiative:

```
/metis-ralph-initiative PROJ-I-0001
```

The sandbox provides:
- **Isolation**: Changes are contained within the sandbox
- **No permission prompts**: Bypass permissions mode is available
- **OAuth authentication**: Credentials handled by Docker sandbox
- **Workspace mounting**: Your project is accessible at the original path

## Notes

- Configuration (plugin, MCP) needs to be re-done each time you start a fresh sandbox
- Consider keeping a sandbox running for repeated use
- Progress is logged to Metis documents which persist in your mounted workspace
