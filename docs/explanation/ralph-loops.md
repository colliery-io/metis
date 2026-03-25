# The Ralph Loop Pattern

Ralph loops enable autonomous AI task execution by intercepting Claude Code's exit attempts and feeding the same prompt back until the work is genuinely complete. This document explains how the mechanism works, its design decisions, and when it's appropriate to use.

## Origin

The Ralph loop is based on the [Ralph Wiggum technique](https://ghuntley.com/ralph/), named after the Simpsons character who famously declares "I'm helping!" The core idea: instead of one-shot prompting, keep the AI in a loop where it repeatedly re-reads the task requirements and checks its own work.

## How It Works

### The Loop Mechanism

```
┌───────────────────────────┐
│                           │
│  1. Claude reads task     │
│  2. Claude works on it    │
│  3. Claude tries to stop  │
│         │                 │
│    ┌────┴────┐            │
│    │ Stop    │            │
│    │ Hook    │            │
│    └────┬────┘            │
│         │                 │
│    Has completion     No  │
│    promise? ──────────────┘
│         │
│        Yes
│         │
│    Loop exits
└───────────────────────────┘
```

1. **Setup:** The `/metis-ralph` command creates a state file (`.claude/metis-ralph.local.md`) with the task short code, iteration count, and a completion promise string.

2. **First iteration:** Claude reads the task document, transitions it to `active`, and starts working.

3. **Exit interception:** When Claude decides it's done and tries to stop, the Stop hook fires. It reads the state file, checks Claude's output for the completion promise tag (`<promise>TASK COMPLETE</promise>`), and:
   - **Promise found:** Allows the exit. Loop ends.
   - **Promise not found:** Blocks the exit. Reads the task content and feeds it back as a new prompt. Increments the iteration counter.

4. **Subsequent iterations:** Claude re-reads the task requirements. It sees the files it already modified. It checks its work against the acceptance criteria. If something is wrong or incomplete, it continues working. If everything is done, it outputs the promise.

### The State File

```yaml
# .claude/metis-ralph.local.md
---
active: true
mode: task
short_code: "PROJ-T-0001"
project_path: "/path/to/.metis"
iteration: 3
max_iterations: 20
completion_promise: "TASK COMPLETE"
started_at: "2026-03-25T14:30:00Z"
---
```

The state file persists across iterations because it's outside Claude's context — it's a file on disk that the hook reads. The hook increments `iteration` each time it intercepts an exit.

### The Stop Hook

The stop hook (`plugins/metis/hooks/stop-hook.sh`) runs when Claude Code fires the `Stop` event:

1. Checks if `.claude/metis-ralph.local.md` exists and `active: true`
2. If not active, allows normal exit
3. If active, searches Claude's recent output for the completion promise
4. If `max_iterations` is reached, allows exit regardless
5. Otherwise, reads the task document and returns it as the next prompt

The hook uses `jq` to parse Claude's output from the event payload.

### Completion Signals

The promise tag is the exit condition:

**Task execution:**
```
<promise>TASK COMPLETE</promise>
```

**Initiative decomposition:**
```
<promise>DECOMPOSITION COMPLETE</promise>
```

**Multi-task execution:**
```
<promise>ALL TASKS COMPLETE</promise>
```

Claude must include this exact text in its output. The hook does a simple string match — no fuzzy matching.

## Why This Design?

### Why not just ask Claude to loop itself?

Claude doesn't have persistent memory across its own output. If you ask it to "keep working until done," it has no way to re-evaluate its work from scratch. Each turn builds on the previous context, which means early mistakes compound.

The Ralph loop solves this by re-injecting the original task requirements. Claude re-reads the acceptance criteria fresh, checks the actual files, and can catch issues it missed earlier.

### Why a stop hook and not a prompt loop?

A prompt-based loop (where Claude just keeps talking to itself) would consume context window rapidly. The stop hook resets the conversation context while preserving the task requirements and the code changes on disk.

### Why a file-based state?

The state file is outside Claude's context window. It survives context compaction, session restarts, and memory loss. The hook reads it independently — Claude doesn't need to remember the loop state.

### Why a fixed completion promise?

A simple string match is unambiguous. No parsing, no LLM interpretation, no false positives. Claude either outputs the promise or it doesn't. This makes the loop behavior predictable and debuggable.

## Self-Correction

The power of Ralph loops comes from self-correction. On each iteration, Claude:

1. Re-reads the task's acceptance criteria
2. Examines the actual code on disk (not its memory of what it wrote)
3. Runs tests if they exist
4. Identifies gaps between the criteria and the current state
5. Fixes issues and tries again

This is particularly effective for tasks with clear, testable criteria:
- "All tests pass" — Claude can run the test suite and verify
- "Function handles edge case X" — Claude can check the code
- "API returns 400 for invalid input" — Claude can test the endpoint

## When to Use Ralph Loops

**Good fit:**
- Tasks with objective, testable acceptance criteria
- Iterative work (getting tests to pass, fixing bugs)
- Systematic initiative decomposition
- Well-scoped implementation tasks

**Poor fit:**
- Tasks requiring human judgment ("does this look right?")
- Exploratory work with unclear scope
- One-shot operations (rename a variable, add a dependency)
- Creative work (write documentation, design an API)

**Safety tips:**
- Always set `--max-iterations` as a safety net (10-20 for focused tasks)
- Write specific acceptance criteria with checkboxes
- Keep tasks small (2-3 iterations ideal, not 15)
- Monitor the first few iterations to ensure Claude is on track

## Modes

### Task Mode (`/metis-ralph`)

Single task execution. Claude works on one task document until complete. The task should be in `todo` phase — Claude transitions it to `active` at the start.

### Decompose Mode (`/metis-decompose`)

Initiative decomposition. Claude reads the initiative, transitions it to `decompose`, and creates tasks. The loop continues until Claude believes the decomposition covers all requirements.

### Multi-Task Mode (`/metis-ralph-tasks`)

Serial execution of multiple tasks. Claude completes one task, then moves to the next. Each task is transitioned independently.

### Initiative Mode (`/metis-ralph-initiative`)

Finds all tasks under an initiative and executes them serially using task mode. Combines decomposition awareness with sequential execution.

## Progress Tracking

Claude logs progress to the task document's "Status Updates" section using `edit_document`. This creates a persistent record that:

- Survives context compaction
- Is visible to other agents or humans
- Captures decisions, findings, and approach changes
- Enables resume-after-interruption

This is why active Metis tasks serve as "working memory" — they're the one place where progress is durably recorded.
