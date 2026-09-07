# Metis Plugin

Flight Levels methodology plugin for Metis work management. Includes methodology skills, design interviews for visions and initiatives, iterative Ralph loops for task execution, and a Flight Levels agent for guidance.

## Components

| Component | Description |
|-----------|-------------|
| **Skills** | Flight Levels methodology guidance (decomposition, phases, patterns) and design interviews (`/grill-vision`, `/grill-initiative`, `/grill-decomposition`) |
| **Agent** | `flight-levels` - Methodology expert for document selection and best practices |
| **Commands** | `/metis-ralph`, `/metis-ralph-tasks`, `/cancel-metis-ralph` - Iterative work loops |
| **Hooks** | SessionStart (project detection), Stop (Ralph loop control) |
| **MCP** | Metis MCP server configuration |

## Skills: Flight Levels Methodology

Four focused skills provide targeted methodology guidance, and four more run design interviews:

### document-selection
**Triggers:** "what document type", "create a bug ticket", "should this be a task or initiative", "when to use ADR"

Helps choose the right document type (vision, initiative, task, backlog, ADR, specification) and maps user terminology to Metis documents.

### decomposition
**Triggers:** "break down this initiative", "decompose into tasks", "how to size tasks", "vertical slices"

Guides breaking higher-level work into actionable lower-level items with patterns like vertical slices, risk-first, and milestone-based decomposition.

### phase-transitions
**Triggers:** "when to transition", "move to active", "exit criteria", "how to complete a task"

Explains phase sequences, exit criteria, and the critical rule that phases cannot be skipped.

### project-patterns
**Triggers:** "start a new project", "greenfield", "tech debt campaign", "incident response", "which preset"

Provides patterns for different work types: greenfield projects, tech debt campaigns, incident response, and feature development. Also covers preset selection and anti-patterns.

## Skills: Design Interviews

The Metis equivalent of [grill-with-docs](https://github.com/mattpocock/skills). Claude interviews you relentlessly about a vision or initiative, and the Metis document is the record: every settled decision is written into its sections as it lands, and trade-offs that are hard to reverse are offered as Metis ADRs. Phase transitions stay with you.

### grilling
**Triggers:** "grill me", "stress-test this plan", "poke holes in this", "sharpen this initiative"

The interview engine. Maps the conversation as a design tree and asks the whole frontier of answerable questions each round, numbered, each with a recommended answer. Facts (what the code does, what documents exist) are Claude's job to look up; decisions are yours. Ends only when nothing is left silently assumed, and never acts on the design until you confirm.

### grill-vision
**Invoke:** `/grill-vision PROJ-V-0001` or `/grill-vision "A title for a new vision"`

Grills a vision across Purpose, Current State, Future State, Success Criteria, Principles, and Constraints. Pushes back on anything that is really an initiative and notes it as a candidate for later.

### grill-initiative
**Invoke:** `/grill-initiative PROJ-I-0001` or `/grill-initiative "A title for a new initiative"`

Grills an initiative with depth set by its phase: context, goals, and non-goals in `discovery`; detailed design, alternatives, implementation plan, and testing strategy in `design`. Flags scope that should be its own initiative. Hands off to the `decomposition` skill once you confirm the design.

### grill-decomposition
**Invoke:** `/grill-decomposition PROJ-I-0001`

One bounded pass over an initiative's drafted tasks, taken as a set, not one interview per task. Checks slice boundaries, ordering and risk, whether each acceptance criterion is something a Ralph loop can verify, and scope leaks against the initiative's non-goals. Writes the answers into the task documents. If it keeps hitting design questions, it stops and sends you back to `grill-initiative`.

## Agent: Flight Levels

The `flight-levels` agent provides methodology guidance when working with Metis documents:

- Document type selection based on work scope
- Work decomposition patterns and timing
- Phase transitions and exit criteria
- Anti-pattern identification
- User terminology mapping (e.g., "bug ticket" → backlog item)

The agent is triggered automatically when methodology guidance is needed.

## Commands: Metis Ralph Loops

Iterative AI loops based on the [Ralph Wiggum technique](https://ghuntley.com/ralph/).

### /metis-ralph

Execute a Metis task with a Ralph loop.

```bash
/metis-ralph PROJ-T-0001
/metis-ralph PROJ-T-0001 --max-iterations 20
/metis-ralph PROJ-T-0001 --project-path /path/to/.metis
```

**Flow:**
1. Reads task content from Metis
2. Transitions task to "active"
3. Works on implementation
4. Iterates until complete
5. Transitions to "completed"
6. Outputs completion promise to exit

### /cancel-metis-ralph

Cancel an active Metis Ralph loop.

```bash
/cancel-metis-ralph
```

## Options

| Option | Description |
|--------|-------------|
| `--max-iterations <n>` | Stop after N iterations (default: unlimited) |
| `--project-path <path>` | Path to .metis folder (default: auto-detect) |

## How Ralph Loops Work

### State File

Loop state is stored in `.claude/metis-ralph.local.md`:

```yaml
---
active: true
mode: task  # or "decompose"
short_code: "PROJ-T-0001"
project_path: "/path/to/.metis"
iteration: 1
max_iterations: 20
completion_promise: "TASK COMPLETE"
started_at: "2026-01-10T..."
---

Execute Metis task PROJ-T-0001
```

### Stop Hook

The stop hook intercepts Claude's exit attempts:

1. Checks if loop is active
2. Looks for completion promise in Claude's output
3. If found: allows exit
4. If not found: blocks exit, feeds prompt back, increments iteration

### Completion Signals

To complete a loop, Claude must:

1. Finish the actual work
2. Transition the Metis document to the appropriate phase
3. Output the promise tag:

**Task execution:**
```
<promise>TASK COMPLETE</promise>
```

**Multi-task execution:**
```
<promise>ALL TASKS COMPLETE</promise>
```

## When to Use Ralph Loops

**Good for:**
- Well-defined tasks with clear success criteria
- Tasks requiring iteration (tests to pass, bugs to fix)
- Breaking down complex initiatives systematically
- Autonomous work with automatic Metis state updates

**Not good for:**
- Tasks requiring human judgment
- Exploratory work with unclear scope
- One-shot operations

## Installation

Add this plugin to your Claude Code configuration:

```json
{
  "plugins": [
    "/path/to/metis/plugins/metis"
  ]
}
```

Or symlink to your plugins directory:

```bash
ln -s /path/to/metis/plugins/metis ~/.claude/plugins/metis
```

## Dependencies

- Metis MCP server must be connected and available
- `jq` command-line tool (for stop hook JSON parsing)

## Safety

- Always use `--max-iterations` as a safety net for Ralph loops
- The loop cannot be stopped manually except via `/cancel-metis-ralph`
- Document phase transitions persist even if the loop is cancelled
