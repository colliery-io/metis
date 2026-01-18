---
description: "Decompose a Metis initiative into tasks"
argument-hint: "SHORT_CODE [--max-iterations N]"
allowed-tools: ["Bash(${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-decompose.sh:*)", "mcp__metis__read_document"]
hide-from-slash-command-tool: "true"
---

# Metis Decompose - Initiative to Tasks

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
"${CLAUDE_PLUGIN_ROOT}/scripts/setup-metis-decompose.sh" $ARGUMENTS
```

## Step 3: Decompose Initiative

You are now in a Metis Ralph loop for initiative decomposition. Follow these steps:

### Initialize
- The initiative content was already read in Step 1
- Use `mcp__metis__transition_phase` to transition the initiative to "decompose"

### Decompose
- Analyze the initiative requirements thoroughly
- Break down the work into discrete, actionable tasks
- Use `mcp__metis__create_document` to create each task under this initiative
- Each task should be:
  - Independently completable (1-14 days of work)
  - Clear and specific
  - Testable/verifiable

### Iterate
- Review the tasks you've created
- Check for gaps in coverage
- Add additional tasks as needed
- Refine task descriptions for clarity

### Complete
When decomposition is FULLY complete:
- Do **NOT** transition to "active" - that requires user approval
- Output: `<promise>DECOMPOSITION COMPLETE</promise>` to signal ready for review

## Critical Rules

- **ONLY** output the promise when decomposition is genuinely complete
- **DO NOT** transition to "active" - the user will do this after review
- Create meaningful tasks that cover all aspects of the initiative
- Do NOT create overly granular tasks - aim for substantive work items
- Do NOT lie or output false promises to escape the loop

The loop will continue until you output the promise after completing decomposition.
