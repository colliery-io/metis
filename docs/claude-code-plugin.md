# Metis Plugin for Claude Code

The Metis plugin teaches Claude Code *when* and *why* to use Metis tools, providing methodology guidance for the Flight Levels approach to project management.

## What the Plugin Provides

- **Skills**: Guidance on Metis vocabulary, document selection, decomposition patterns, phase transitions, and project patterns, plus design interviews (`/grill-vision`, `/grill-initiative`, `/grill-decomposition`) that sharpen visions, initiatives, and their task breakdown before work starts
- **Commands**: `/metis-ralph` and `/metis-ralph-tasks` for autonomous task execution
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

### `/metis-ralph-tasks <SHORT_CODE> [SHORT_CODE...]`

Executes a list of tasks serially, completing each before moving to the next.

```
/metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 PROJ-T-0003
```

### `/cancel-metis-ralph`

Cancels an active Ralph loop.

## Available Skills

The plugin provides methodology guidance through skills:

- **metis-vocabulary**: Keeps Claude on Metis's own names — short codes for work items, closed sets for document types, phases, and backlog categories — instead of shorthand it invented ("Task 3", "Slice 2", "D3", "the auth epic", "in progress"). Plans are written in short codes that have been created or read back; the one carve-out is a specification's `REQ-`/`NFR-` requirement IDs
- **document-selection**: Helps choose the right document type (vision, initiative, task, ADR, specification)
- **decomposition**: Patterns for breaking down work into tasks
- **phase-transitions**: Guidance on advancing documents through their lifecycle
- **project-patterns**: Common patterns for greenfield projects, tech debt, incident response
- **grilling**: The interview engine. Claude questions you in rounds, recommends an answer for each question, looks up facts itself, and leaves decisions to you
- **grill-vision** (`/grill-vision PROJ-V-0001` or `/grill-vision "A title"`): Grills a vision and writes settled answers into its Purpose, Current State, Future State, Success Criteria, Principles, and Constraints sections as they land
- **grill-initiative** (`/grill-initiative PROJ-I-0001` or `/grill-initiative "A title"`): Grills an initiative with depth set by its phase, fills its sections as decisions settle, and offers a Metis ADR for trade-offs that are hard to reverse
- **grill-decomposition** (`/grill-decomposition PROJ-I-0001`): One bounded pass over an initiative's drafted tasks as a set. Sharpens slice boundaries, ordering, and acceptance criteria until each task is something a Ralph loop can verify, and writes the results into the task documents

The grill skills are the Metis equivalent of [grill-with-docs](https://github.com/mattpocock/skills): the same relentless interview, but the document being sharpened is the Metis vision or initiative itself, and decisions worth keeping become Metis ADRs. Phase transitions stay with you; the skills never advance a document on their own.

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
