# How to Manage Documents

This guide covers creating, editing, searching, and archiving Metis documents through both the CLI and MCP tools.

## Create Documents

### Create an Initiative

Initiatives require a parent vision:

```bash
metis create initiative "Migrate to PostgreSQL" --vision PROJ-V-0001
```

You'll be prompted to select complexity (S/M/L/XL). The initiative is created in `.metis/initiatives/PROJ-I-0001/initiative.md`.

### Create a Task

Tasks require a parent initiative:

```bash
metis create task "Write migration scripts" --initiative PROJ-I-0001
```

Tasks start in the `todo` phase.

### Create a Backlog Item

Standalone tasks without an initiative use the backlog. Via MCP (in Claude Code):

```
Create a backlog task called "Fix login timeout" with category "bug"
```

This creates a task with a `#bug` tag in `.metis/tasks/backlog/`.

Backlog categories: `bug`, `feature`, `tech-debt`.

### Create an ADR

Architecture Decision Records don't require a parent:

```bash
metis create adr "Use event sourcing for audit log"
```

ADRs are numbered automatically (PROJ-A-0001, PROJ-A-0002, etc.) and start in `draft` phase.

### Create a Specification

Specifications require a parent (vision or initiative):

```bash
metis create specification "API Contract v2" --parent PROJ-V-0001
```

## Edit Documents

### Edit in Your Editor

Documents are standard Markdown files with YAML frontmatter. Open them directly in any editor:

```bash
# Vision
$EDITOR .metis/vision.md

# Initiative
$EDITOR .metis/initiatives/PROJ-I-0001/initiative.md

# Task
$EDITOR .metis/initiatives/PROJ-I-0001/tasks/PROJ-T-0001/task.md

# ADR
$EDITOR .metis/adrs/PROJ-A-0001-use-event-sourcing.md
```

After editing, sync to update the database:

```bash
metis sync
```

### Edit via MCP (Claude Code)

Ask Claude to make targeted edits using search-and-replace:

> "In task PROJ-T-0001, replace 'TBD' with 'Use the repository pattern for data access'"

Claude uses the `edit_document` MCP tool, which finds the text and replaces it. The `replace_all` option replaces every occurrence.

## Search Documents

### Full-Text Search

```bash
metis search "authentication"
```

Returns matching documents across titles and content:

```
Code           Title                                    Type
PROJ-T-0003    Implement authentication middleware      task
PROJ-I-0002    Authentication Infrastructure            initiative

Found 2 document(s) for "authentication"
```

### Filter by Type or Phase

```bash
metis list -t task                    # All tasks
metis list -t task -p active          # Active tasks only
metis list -t initiative -p design    # Initiatives in design
metis list --include-archived         # Include archived documents
```

### Output Formats

```bash
metis list -f table                   # Human-readable table (default)
metis list -f compact                 # One-line-per-document for scripts
metis list -f json                    # JSON array for programmatic use
metis search "api" -f json -l 5      # JSON, limit to 5 results
```

### Search via MCP

Ask Claude:

> "Search for documents about database migration"

## Check Project Status

```bash
metis status
```

Shows all documents sorted by actionability priority:
1. Active (highest priority)
2. Todo
3. Blocked
4. Other phases

Includes phase insights (counts by phase) and blocked-by information.

## Archive Documents

Archive completed documents to clean up your workspace:

```bash
metis archive PROJ-T-0001
```

When you archive an initiative, all its child tasks are archived too:

```bash
metis archive PROJ-I-0001
```

Archived documents move to `.metis/archived/` and are hidden from `metis list` and `metis status` by default. Use `--include-archived` to see them.

## Reassign Tasks

### Move a Task to a Different Initiative

Via MCP (ask Claude):

> "Move task PROJ-T-0005 to initiative PROJ-I-0002"

### Move a Task to the Backlog

> "Move task PROJ-T-0005 to the backlog as a bug"

### Move a Backlog Item to an Initiative

> "Assign backlog item PROJ-T-0010 to initiative PROJ-I-0001"

## Validate a Document

Check that a document's structure is valid:

```bash
metis validate .metis/vision.md
metis validate .metis/initiatives/PROJ-I-0001/initiative.md
```

See [CLI Reference](../reference/cli.md) (`metis validate`) for details on what is checked.

For full details on document schemas and fields, see [Document Types Reference](../reference/document-types.md). For output format options, see [CLI Reference](../reference/cli.md).
