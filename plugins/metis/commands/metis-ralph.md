---
description: "Execute a Metis task with Ralph loop"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph.sh:*)", "mcp__metis__read_document"]
hide-from-slash-command-tool: "true"
---

# Metis Ralph - Task Execution

## Step 1: Verify Task Exists

**BEFORE starting the Ralph loop**, you MUST verify the task exists.

Parse the SHORT_CODE from: `$ARGUMENTS`

Use `mcp__metis__read_document` to verify the task exists:
- `project_path`: Auto-detect by finding `.metis` directory (usually `$PWD/.metis` or parent)
- `short_code`: The SHORT_CODE from arguments (e.g., PROJ-T-0001)

**If the document is NOT found** (error response):
- Do NOT proceed with the loop
- Tell the user: "Task {SHORT_CODE} was not found. Please verify the short code is correct using `mcp__metis__list_documents`."
- Stop here

**If the document IS found**, proceed to Step 2.

## Step 2: Initialize Loop

Only after successful verification, execute the setup script:

```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph.sh" $ARGUMENTS
```

## Step 3: Execute Task

You are now in a Metis Ralph loop. Follow these steps:

### Initialize
- The task content was already read in Step 1
- Use `mcp__metis__transition_phase` to transition the task to "active"

### Execute
- Implement what the task describes
- Write code, create files, run tests as needed
- The task content defines your success criteria

### Log Progress
- **IMPORTANT**: Log your progress to the task's "Status Updates" section using `mcp__metis__edit_document`
- Record what you've done, findings, decisions, and next steps
- This creates a permanent record that survives session interruptions

### Signal Ready for Review
When you believe the task is FULLY complete:
- Do **NOT** transition the task to "completed" - that requires user approval
- Output: `<promise>TASK COMPLETE</promise>` to signal you're ready for review

## Critical Rules

- **ONLY** output the promise when the task is genuinely complete
- **DO NOT** transition to "completed" - the user will do this after review
- **ALWAYS** log progress to the task document - this is your working memory
- Do NOT lie or output false promises to escape the loop
- If stuck, continue iterating - the loop is designed for persistence

The loop exits when you output the promise. The user will review your work and transition the task to "completed" if satisfied.
