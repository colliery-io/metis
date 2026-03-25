# Using Metis with Claude Code

This tutorial shows you how to connect Metis to Claude Code so that Claude can read, create, and manage your project documents directly. By the end, you'll have Claude Code fully integrated with Metis, able to create tasks, transition phases, and search documents through natural conversation.

## Prerequisites

- Metis installed with the CLI available on your PATH (see [How to Install Metis](../how-to/install.md))
- Claude Code installed (`npm install -g @anthropic-ai/claude-code` or via the desktop app)
- A Metis project already initialized (see [Getting Started](./getting-started.md))

## Step 1: Install the Metis Plugin

The Metis plugin teaches Claude Code *when* and *why* to use Metis tools. It provides methodology guidance, automated hooks, and slash commands.

Inside Claude Code, add the plugin marketplace and install:

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
```

If you installed Metis via the install script, the plugin was already installed automatically.

## Step 2: Add the MCP Server

The MCP (Model Context Protocol) server is a standard protocol that lets AI assistants use external tools — in this case, Metis's document management capabilities. Add it as a user-scoped server:

```
!claude mcp add --scope user metis metis mcp
```

The `!` prefix runs this as a shell command within Claude Code. This registers the `metis mcp` command as an MCP server that starts automatically in each session.

## Step 3: Verify the Connection

Check that everything is connected:

```
/mcp
```

You should see:

```
metis: metis mcp - Connected
```

If it shows "Disconnected", restart Claude Code and try again.

## Step 4: Try Basic Operations

Now Claude Code can work with your Metis project directly. Start a conversation in your project directory:

**Ask Claude to list your documents:**

> "Show me all my Metis documents"

Claude will use the `list_documents` MCP tool and display a table of your documents with their short codes, titles, types, and phases.

**Ask Claude to create a task:**

> "Create a task called 'Add rate limiting to API' under initiative MFP-I-0001"

Claude creates the task and shows you the new short code.

**Ask Claude to read a document:**

> "Read task MFP-T-0005"

Claude fetches the full content and metadata.

**Ask Claude to transition a phase:**

> "Move MFP-T-0005 to active"

Claude transitions the document and confirms the change.

## Step 5: Understand Automatic Behaviors

The Metis plugin includes hooks — scripts that run automatically at key moments. When you start a session, the plugin detects your Metis project and shows active tasks. When Claude edits files, the plugin tracks changes to keep the code index current. These work behind the scenes without any action from you.

## Step 6: Try Methodology Guidance

The plugin includes skills that help Claude give better advice about Metis workflows. Try asking:

> "What document type should I use for this bug?"

Claude will automatically activate the document-selection skill and recommend the right document type. You can also ask about breaking down initiatives, when to transition phases, or how to set up different project types.

## Step 7: Use the Code Index

Metis generates a code index at `.metis/code-index.md` that maps your project's structure, symbols, and module relationships. This helps Claude navigate your codebase efficiently.

Generate or update the index:

> "Generate the code index"

Or from the CLI:

```bash
metis index --incremental
```

The code index is used automatically by Ralph loops and other AI-driven workflows. It supports Rust, Python, TypeScript, JavaScript, and Go.

## What You've Learned

- **Install** the Metis plugin and MCP server for Claude Code
- **Verify** the connection with `/mcp`
- **Use** natural language to manage Metis documents through Claude
- **Understand** automatic hooks (session start, pre-compact, post-tool tracking)
- **Leverage** methodology skills for workflow guidance
- **Generate** code indexes for AI navigation

## Next Steps

- [Running Autonomous Tasks with Ralph Loops](./ralph-loops.md) — Set up autonomous AI execution
- [MCP Tools Reference](../reference/mcp-tools.md) — All MCP tools available to Claude
- [The Ralph Loop Pattern](../explanation/ralph-loops.md) — How autonomous execution works
