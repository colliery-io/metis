---
description: "Execute a list of Metis tasks serially with Ralph loop"
argument-hint: "SHORT_CODE [SHORT_CODE...] [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph-tasks.sh:*)", "mcp__metis__read_document"]
hide-from-slash-command-tool: "true"
---

# Metis Ralph Tasks - Serial Multi-Task Execution

## Step 1: Verify All Tasks Exist

**BEFORE starting the Ralph loop**, you MUST verify every task exists.

Parse the SHORT_CODEs from: `$ARGUMENTS`

Use `mcp__metis__read_document` to verify **each** task exists:
- `project_path`: Auto-detect by finding `.metis` directory (usually `$PWD/.metis` or parent)
- `short_code`: Each SHORT_CODE from arguments

**If ANY document is NOT found** (error response):
- Do NOT proceed with the loop
- Tell the user which task(s) were not found
- Stop here

**If ALL documents are found**, proceed to Step 2.

## Step 2: Initialize Loop

Only after successful verification of all tasks, execute the setup script:

```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph-tasks.sh" $ARGUMENTS
```

## Step 3: Execute Tasks Serially

You are now in a Metis Ralph multi-task loop. Work through each task in order.

### Orient (only if needed)
- If the tasks already specify which files/modules to modify, skip this — go straight to execution
- If you need to discover where code lives, **read `.metis/code-index.md` first** — it has the project structure, key symbols, and module summaries. Use it to target your searches instead of exploring the codebase from scratch

### For Each Task

1. **Read** the task content using `mcp__metis__read_document`
2. **Activate** it using `mcp__metis__transition_phase` (transition to "active")
3. **Implement** what the task describes — write code, create files, run tests as needed
4. **Log progress** to the task's "Status Updates" section using `mcp__metis__edit_document`
5. **Complete** it using `mcp__metis__transition_phase` (transition to "completed")
6. **Move to the next task**

### Signal Ready for Review
When ALL tasks in the list are complete:
- Output: `<promise>ALL TASKS COMPLETE</promise>`

## Critical Rules

- **ONLY** output the promise when ALL tasks are genuinely complete
- **DO** transition each task to "completed" as you finish it
- **ALWAYS** log progress to each task's document as you work
- Work on **ONE task at a time** until it's done before moving to the next
- If stuck on a task, log the blocker, transition it to "blocked", and move to the next task
- If all remaining tasks are blocked, output the promise and explain what's blocked
- Do NOT lie or output false promises to escape the loop
