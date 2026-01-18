---
description: "Execute all tasks under a decomposed initiative with Ralph loop"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph-initiative.sh:*)", "mcp__metis__read_document"]
hide-from-slash-command-tool: "true"
---

# Metis Ralph Initiative - Execute All Tasks

## Step 1: Verify Initiative Exists

**BEFORE starting the Ralph loop**, you MUST verify the initiative exists.

Parse the SHORT_CODE from: `$ARGUMENTS`

Use `mcp__metis__read_document` to verify the initiative exists:
- `project_path`: Auto-detect by finding `.metis` directory (usually `$PWD/.metis` or parent)
- `short_code`: The SHORT_CODE from arguments (e.g., PROJ-I-0001)

**If the document is NOT found** (error response):
- Do NOT proceed with the loop
- Tell the user: "Initiative {SHORT_CODE} was not found. Please verify the short code is correct using `mcp__metis__list_documents`."
- Stop here

**If the document IS found**, proceed to Step 2.

## Step 2: Initialize Loop

Only after successful verification, execute the setup script:

```bash
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph-initiative.sh" $ARGUMENTS
```

## Step 3: Execute All Tasks

You are now in a Metis Ralph Initiative loop. You will work through ALL tasks under this initiative.

### Initialize
- The initiative content was already read in Step 1
- Use `mcp__metis__list_documents` to find all tasks under the initiative
- Identify tasks in "todo" phase that need work

### Work Through Tasks
For each task in "todo" phase:
1. Transition it to "active" using `mcp__metis__transition_phase`
2. Read the task content
3. Implement what it describes
4. Log progress to the task's "Status Updates" section
5. **Transition the task to "completed"** when done
6. Move to the next "todo" task

### Log Progress
- Update each task's "Status Updates" section as you work
- Also update the initiative's "Status Updates" with overall progress
- This creates a permanent record that survives session interruptions

### Signal Ready for Review
When ALL tasks are complete (no more "todo" or "active" tasks remain):
- Do **NOT** transition initiative to "completed" - user will review
- Output: `<promise>INITIATIVE COMPLETE</promise>`

## Critical Rules

- **ONLY** output the promise when ALL tasks are genuinely complete
- **DO** transition each task to "completed" as you finish it
- **DO NOT** transition the initiative - user reviews the whole thing at the end
- **ALWAYS** log progress to task and initiative documents
- Work on ONE task at a time until it's done
- If stuck on a task, log the blocker and move to the next task
- If all remaining tasks are blocked, output the promise and explain in the initiative

The loop exits when you output the promise. User reviews the completed initiative.
