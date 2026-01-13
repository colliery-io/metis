# Metis Plugin for Claude Code

The Metis plugin teaches Claude Code *when* and *why* to use Metis tools, providing methodology guidance for the Flight Levels approach to project management.

## What the Plugin Provides

- **Skills**: Guidance on document selection, decomposition patterns, phase transitions, and project patterns
- **Commands**: `/metis-ralph` for autonomous task execution, `/metis-decompose` for breaking down initiatives
- **Agents**: Flight Levels methodology expert for document type selection and work decomposition
- **MCP Integration**: Automatic Metis MCP server configuration

## Installation

### 1. Add the Metis Marketplace

```
/plugin marketplace add colliery-io/metis
```

### 2. Install the Plugin

```
/plugin install metis@colliery-io-metis
```

### 3. Add MCP Server

```bash
claude mcp add --scope user metis metis mcp
```

Or via the `!` prefix inside Claude Code:
```
!claude mcp add --scope user metis metis mcp
```

### 4. Verify Setup

```
/mcp
```

Should show:
```
metis: metis mcp - Connected
```

## Available Commands

### `/metis-ralph <SHORT_CODE>`

Executes a Metis task in a loop until completion. Works with your existing permission settings.

```
/metis-ralph PROJ-T-0001
/metis-ralph PROJ-T-0001 --max-iterations 20
```

### `/metis-ralph-initiative <SHORT_CODE>`

Executes all tasks under a decomposed initiative.

```
/metis-ralph-initiative PROJ-I-0001
```

### `/metis-decompose <SHORT_CODE>`

Decomposes an initiative into tasks.

```
/metis-decompose PROJ-I-0001
```

### `/cancel-metis-ralph`

Cancels an active Ralph loop.

## Available Skills

The plugin provides methodology guidance through skills:

- **document-selection**: Helps choose the right document type (vision, initiative, task, ADR)
- **decomposition**: Patterns for breaking down work into tasks
- **phase-transitions**: Guidance on advancing documents through their lifecycle
- **project-patterns**: Common patterns for greenfield projects, tech debt, incident response

## Sandboxed Execution

For autonomous, isolated execution without permission prompts, see [Docker Sandbox Setup](./docker-sandbox.md).

## Troubleshooting

### Plugin not showing up

Verify the marketplace is added:
```
/plugin marketplace list
```

Re-install if needed:
```
/plugin install metis@colliery-io-metis
```

### MCP server not connecting

Check MCP status:
```
/mcp
```

Re-add the server:
```
!claude mcp add --scope user metis metis mcp
```

### Commands not available

Restart Claude Code after plugin installation for commands to become available.
