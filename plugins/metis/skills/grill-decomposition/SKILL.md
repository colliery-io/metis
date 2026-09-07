---
name: grill-decomposition
description: One bounded interview over an initiative's drafted tasks, taken as a set, that sharpens slice boundaries, ordering, and acceptance criteria and writes the answers into the task documents. Use after tasks are drafted and before the initiative goes active. Pass an initiative short code (e.g. PROJ-I-0001).
disable-model-invocation: true
---

# Grill Decomposition

Run the `grilling` skill against the *set* of tasks under an initiative. Read the `grilling` skill first; it defines how to run the interview. This file defines what you are sharpening and, just as importantly, what you are not.

This is not a per-task grill. By the time an initiative has been through `grill-initiative`, most task-level questions are facts (which file, which test, which module) and facts are yours to look up. The decisions that remain live at the boundaries between tasks. Expect one or two rounds, not a long session.

## Step 1: Load the Set

Parse `$ARGUMENTS` for a short code matching `PREFIX-I-NNNN`. If none is given, `mcp__metis__list_documents(document_type="initiative")` and pick the one in `decompose`; ask if there is more than one.

Then:

1. `mcp__metis__read_document` the initiative. Note its phase, goals, non-goals, design, and implementation plan.
2. `mcp__metis__list_documents(document_type="task")` and keep the tasks whose parent is this initiative. Read each one.
3. Read `.metis/code-index.md` so you can check claims about where code lives without asking.

**Phase check.** This skill expects the initiative in `decompose` (or `active` with tasks still in `todo`). If it is in `discovery`, `design`, or `ready` with no tasks, there is nothing to grill yet; point the user at `grill-initiative` or the `decomposition` skill and stop. If there are no child tasks, say so and stop.

## Step 2: The Four Questions

Each branch below is a root of the design tree. Ask across the whole set at once, naming tasks by short code. Give a recommended answer for every question.

Name every existing task by its short code, never by position ("the second task") or a label of your own. A task you are proposing to add does not have a short code yet: give it a full quoted title and mark it as not yet created, and switch to the real code the moment `create_document` returns one. Question numbers belong to this round only — they never become names for work.

**Slice boundaries.** For each task: is it a vertical slice that proves something on its own, or a horizontal layer that proves nothing until others land? Where a task is a layer, propose the merge or split that makes it a slice, and ask.

**Ordering and risk.** Which task retires the most uncertainty? Recommend it go first. Where one task cannot start until another finishes, say so and ask whether the dependency is real or an artefact of how the work was cut.

**Done criteria.** For each acceptance criterion: could a Ralph loop verify it without a human? Criteria like "works well" or "is clean" are not verifiable. Propose a concrete replacement (a command that passes, a response code, a file that exists, a metric under a threshold) and ask the user to confirm or correct it. Criteria that genuinely need a human eye should say so explicitly.

**Scope leaks.** Compare every task against the initiative's non-goals and goals. Flag any task that does work the initiative ruled out, and any goal no task covers. Ask whether to cut, add, or accept the gap.

## Step 3: Interview and Write

Follow the `grilling` skill. As each answer settles, write it into the task it belongs to: `read_document`, then `edit_document` the Acceptance Criteria, Objective, or Implementation Notes section. When a task is split, merged, or cut, do it with `create_document`, `edit_document`, and `archive_document` immediately, then read the set back so the next round starts from the real state.

Record one status update on the initiative per round: which tasks changed and why.

## Going Back Up

If a question turns out to be a design question rather than a decomposition question ("should this use a queue or a cron job?", "is partial cancellation allowed?"), stop. Do not absorb it into a task. Tell the user the initiative's design has a hole, name it, and offer `grill-initiative`. Tasks written on top of an unsettled design are the anti-pattern the `decomposition` skill warns about.

## Step 4: Close

When the frontier is empty:

1. Read every task back. Each should have criteria a loop can verify, one clear objective, and an obvious link to the parent.
2. Summarise the final set in a few lines: task order, dependencies, and anything cut or added. Ask the user to confirm.
3. Do not transition the initiative or any task. The user decides when the initiative goes `active` and when tasks move to `todo`.
4. Once confirmed, the tasks are ready for `/metis-ralph` or `/metis-ralph-tasks`.
