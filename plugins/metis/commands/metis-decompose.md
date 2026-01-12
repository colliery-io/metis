---
description: "Decompose a Metis initiative into tasks"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-decompose.sh:*)"]
hide-from-slash-command-tool: "true"
---

# Metis Decompose - Initiative to Tasks

Execute the setup script to initialize the loop:

```!
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-decompose.sh" $ARGUMENTS
```

## Your Task

You are now in a Metis Ralph loop for initiative decomposition. Follow these steps:

### 1. Initialize
- Use `mcp__metis__read_document` to read the initiative content
- Use `mcp__metis__transition_phase` to transition the initiative to "decompose"

### 2. Decompose
- Analyze the initiative requirements thoroughly
- Break down the work into discrete, actionable tasks
- Use `mcp__metis__create_document` to create each task under this initiative
- Each task should be:
  - Independently completable (1-14 days of work)
  - Clear and specific
  - Testable/verifiable

### 3. Iterate
- Review the tasks you've created
- Check for gaps in coverage
- Add additional tasks as needed
- Refine task descriptions for clarity

### 4. Complete
When decomposition is FULLY complete:
- Use `mcp__metis__transition_phase` to transition the initiative to "active"
- Output: `<promise>DECOMPOSITION COMPLETE</promise>`

## Critical Rules

- **ONLY** output the promise when decomposition is genuinely complete
- **ALWAYS** transition to "active" before outputting the promise
- Create meaningful tasks that cover all aspects of the initiative
- Do NOT create overly granular tasks - aim for substantive work items
- Do NOT lie or output false promises to escape the loop

The loop will continue until you output the promise after completing decomposition.
