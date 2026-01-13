---
description: "Execute a Metis task with Ralph loop"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph.sh:*)"]
hide-from-slash-command-tool: "true"
---

# Metis Ralph - Task Execution

Execute the setup script to initialize the loop:

```!
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-ralph.sh" $ARGUMENTS
```

## Your Task

You are now in a Metis Ralph loop. Follow these steps:

### 1. Initialize
- Use `mcp__metis__read_document` to read the task content
- Use `mcp__metis__transition_phase` to transition the task to "active"

### 2. Execute
- Implement what the task describes
- Write code, create files, run tests as needed
- The task content defines your success criteria

### 3. Log Progress
- **IMPORTANT**: Log your progress to the task's "Status Updates" section using `mcp__metis__edit_document`
- Record what you've done, findings, decisions, and next steps
- This creates a permanent record that survives session interruptions

### 4. Signal Ready for Review
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
