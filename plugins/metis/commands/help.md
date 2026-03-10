---
description: "Explain Metis Ralph plugin and available commands"
---

# Metis Ralph Plugin Help

Please explain the following to the user:

## What is Metis Ralph?

Metis Ralph integrates the Ralph Wiggum technique with Metis document management for iterative AI-driven work loops.

**Three modes:**

1. **Task Execution** (`/metis-ralph`) - Execute a single Metis task iteratively until complete
2. **Multi-Task Execution** (`/metis-ralph-tasks`) - Execute a list of tasks serially, auto-completing each
3. **Initiative Decomposition** (`/metis-decompose`) - Break down an initiative into tasks

**Core concept:**
- Claude receives a Metis document as the prompt
- Works on the task/decomposition
- Stop hook intercepts exit attempts
- Same prompt fed back for next iteration
- Claude sees previous work in files
- Continues until genuinely complete

## Available Commands

### /metis-ralph <SHORT_CODE> [OPTIONS]

Execute a Metis task with a Ralph loop.

**Usage:**
```
/metis-ralph PROJ-T-0001
/metis-ralph PROJ-T-0001 --max-iterations 20
```

**What happens:**
1. Task content read from Metis
2. Task transitioned to "active"
3. Claude works on implementation
4. Loop continues until complete
5. Task transitioned to "completed"

**Completion:** Output `<promise>TASK COMPLETE</promise>` after transitioning to completed.

---

### /metis-ralph-tasks <SHORT_CODE> [SHORT_CODE...] [OPTIONS]

Execute multiple Metis tasks serially with a Ralph loop. Each task is auto-completed before moving to the next.

**Usage:**
```
/metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 PROJ-T-0003
/metis-ralph-tasks PROJ-T-0001 PROJ-T-0002 --max-iterations 50
```

**What happens:**
1. All tasks verified to exist
2. For each task in order: read, activate, implement, log progress, complete
3. Loop continues until all tasks are done
4. User reviews all work at the end

**Completion:** Output `<promise>ALL TASKS COMPLETE</promise>` when all tasks are done.

**Best for:** Well-understood tasks that can be executed autonomously without human review between each one.

---

### /metis-decompose <SHORT_CODE> [OPTIONS]

Decompose a Metis initiative into tasks.

**Usage:**
```
/metis-decompose PROJ-I-0001
/metis-decompose PROJ-I-0001 --max-iterations 15
```

**What happens:**
1. Initiative content read from Metis
2. Initiative transitioned to "decompose"
3. Claude analyzes and creates tasks
4. Loop continues until decomposition complete
5. Initiative transitioned to "active"

**Completion:** Output `<promise>DECOMPOSITION COMPLETE</promise>` after transitioning to active.

---

### /cancel-metis-ralph

Cancel an active Metis Ralph loop.

**Usage:**
```
/cancel-metis-ralph
```

Note: This removes the loop state but does NOT revert Metis document phase transitions.

---

## Options

| Option | Description |
|--------|-------------|
| `--max-iterations <n>` | Stop after N iterations (default: unlimited) |
| `--project-path <path>` | Path to .metis folder (default: auto-detect) |

## When to Use

**Good for:**
- Tasks with clear success criteria
- Iterative work (getting tests to pass)
- Systematic initiative breakdown
- Autonomous execution with Metis tracking

**Not good for:**
- Tasks requiring human judgment
- Exploratory work
- One-shot operations

---

## Docker Sandbox (Autonomous Execution)

For fully autonomous execution without permission prompts, run Ralph in a Docker sandbox:

```bash
docker sandbox run -w "$(pwd)" claude
```

Then install the plugin and MCP inside the sandbox:
```
/plugin marketplace add colliery-io/metis
/plugin install metis@colliery-io-metis
!claude mcp add --scope user metis metis mcp
```

See full instructions: https://github.com/colliery-io/metis/blob/main/docs/docker-sandbox.md
