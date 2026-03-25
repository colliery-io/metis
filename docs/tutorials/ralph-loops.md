# Running Autonomous Tasks with Ralph Loops

This tutorial walks you through setting up and running a Ralph loop — an autonomous AI execution cycle where Claude Code works on a Metis task iteratively until it's complete. By the end, you'll understand how to launch, monitor, and cancel Ralph loops for both individual tasks and full initiative execution.

## Prerequisites

- Metis project initialized with at least one initiative and task
- Claude Code connected to Metis (see [Using Metis with Claude Code](./using-with-claude-code.md))
- The Metis plugin installed (`/plugin install metis@colliery-io-metis`)
- `jq` installed on your system (`brew install jq` on macOS, `apt install jq` on Linux)

## What is a Ralph Loop?

A Ralph loop is an autonomous execution cycle where Claude Code works on a task iteratively until it's complete. Claude reads the task, works on it, and when it tries to stop, a hook (a script that runs automatically in Claude Code) sends the task back. This repeats until the work genuinely meets the acceptance criteria. See [The Ralph Loop Pattern](../explanation/ralph-loops.md) for the full conceptual explanation.

## Step 1: Prepare a Task

You need a task with clear, testable acceptance criteria. Create one:

```bash
metis create task "Add input validation to create-book endpoint" --initiative MFP-I-0001
```

Open the task file and define clear acceptance criteria:

```markdown
## Acceptance Criteria
- [ ] POST /books validates required fields (title, author, isbn)
- [ ] Returns 400 with descriptive error messages for invalid input
- [ ] ISBN format validated against ISBN-13 pattern
- [ ] Title length limited to 500 characters
- [ ] Unit tests cover all validation rules
- [ ] Integration test confirms 400 response for invalid requests
```

The more specific your criteria, the better the Ralph loop performs.

## Step 2: Launch a Task Ralph Loop

In Claude Code, run:

```
/metis-ralph MFP-T-0005
```

You can optionally set a maximum iteration count as a safety net:

```
/metis-ralph MFP-T-0005 --max-iterations 20
```

Here's what happens:

1. Claude verifies the task exists by reading it from Metis
2. A setup script creates a state file at `.claude/metis-ralph.local.md`
3. Claude transitions the task to `active`
4. Claude reads the code index (`.metis/code-index.md`) to orient itself
5. Claude starts implementing the task

## Step 3: Monitor Progress

While the loop runs, Claude logs progress to the task's Status Updates section. You can check progress in another terminal:

```bash
metis search "MFP-T-0005"
```

Or read the task directly:

```bash
cat .metis/initiatives/MFP-I-0001/tasks/MFP-T-0005/task.md
```

The loop state file at `.claude/metis-ralph.local.md` tracks the current iteration and other loop metadata. See [Configuration Reference](../reference/configuration.md) for the full state file format.

## Step 4: Completion

When Claude believes the task is complete, it outputs a completion signal (`<promise>TASK COMPLETE</promise>`). The hook sees this and allows Claude to exit the loop. The task remains in `active` phase so you can review the work. If satisfied, transition to completed:

```bash
metis transition MFP-T-0005 completed
```

## Step 5: Cancel a Loop

If you need to stop a loop early:

```
/cancel-metis-ralph
```

This removes the state file and stops the hook from intercepting exits. The task remains in whatever phase it's in — you can resume later or transition manually.

Note: Document phase transitions persist even after cancellation. If Claude already moved the task to `active`, it stays `active`.

## Running Initiative Decomposition

Ralph loops also work for breaking initiatives into tasks:

```
/metis-decompose MFP-I-0002
```

This loop:

1. Reads the initiative content
2. Transitions it to `decompose` phase
3. Analyzes requirements and creates tasks
4. Iterates: reviews coverage, adds missing tasks, refines descriptions
5. Transitions the initiative to `active`
6. Exits with `<promise>DECOMPOSITION COMPLETE</promise>`

## Running Multiple Tasks

Execute a list of tasks serially:

```
/metis-ralph-tasks MFP-T-0001 MFP-T-0002 MFP-T-0003
```

Claude works through each task in order, completing one before moving to the next. The completion signal for multi-task execution is `<promise>ALL TASKS COMPLETE</promise>`.

## Running an Initiative's Tasks

Execute all tasks under a decomposed initiative:

```
/metis-ralph-initiative MFP-I-0001
```

This reads the initiative, finds all its child tasks, and executes them serially using the Ralph loop pattern.

## Running in a Docker Sandbox

For fully autonomous execution without permission prompts, use a Docker sandbox:

```bash
docker sandbox run -w "$(pwd)" claude
```

Inside the sandbox, install the plugin and MCP server:

```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
!claude mcp add --scope user metis metis mcp
```

Then run Ralph as normal. The sandbox provides isolation and bypasses permission prompts. See [How to Run Metis in a Docker Sandbox](../how-to/docker-sandbox.md) for full details.

## Tips for Effective Ralph Loops

**Write clear acceptance criteria.** The loop works best when Claude can objectively verify completion. Checkboxes and testable requirements are ideal.

**Always use `--max-iterations`.** This prevents runaway loops. Start with 10-20 for focused tasks, 30-50 for complex ones.

**Keep tasks small.** A task that takes Claude 2-3 iterations is better than one that takes 15. Break large work into smaller tasks.

**Use the code index.** Run `metis index --incremental` before starting a loop so Claude can navigate your codebase efficiently.

**Log progress.** Claude writes status updates to the task document. If a loop is cancelled, you can see exactly where it left off.

## What You've Learned

- **Launch** a Ralph loop with `/metis-ralph`
- **Monitor** progress through task status updates
- **Cancel** loops with `/cancel-metis-ralph`
- **Decompose** initiatives with `/metis-decompose`
- **Execute** multiple tasks with `/metis-ralph-tasks`
- **Run** in Docker sandbox for autonomous execution
- **Write** effective tasks that work well with Ralph loops

## Next Steps

- [The Ralph Loop Pattern](../explanation/ralph-loops.md) — How the stop hook and completion signals work
- [How to Run Metis in a Docker Sandbox](../how-to/docker-sandbox.md) — Full sandbox setup
- [CLI Reference](../reference/cli.md) — All CLI commands
