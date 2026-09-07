---
name: metis-vocabulary
description: This skill should be used whenever work is named, referred to, or grouped in a Metis project - when the user or Claude says "task 1", "the first task", "epic", "story", "sub-task", "ticket", "workstream", "phase 2", "mark it done", "in progress", "what should I call this", "what's the ID for this", or when writing a plan, commit message, branch name, or status summary that refers to Metis work. Metis assigns every document a short code; this skill covers using Metis's own identifiers, document types, phases, and categories instead of inventing shorthand.
---

# Metis Vocabulary

Metis has names for everything it tracks. **Use them. Never invent your own.**

An invented name -- "Task 3", "the auth epic", "AUTH-01", "phase 2 work" -- looks harmless in one message and is unrecoverable two messages later. It does not resolve in `list_documents`, it does not appear in the user's GUI or `metis status`, and after a compaction nobody (including you) can tell which document it meant. Every reference to work must be something the user can paste into Metis and get a document back.

## The One Rule

**A work item's name is its Metis short code, and nothing else.**

Short codes look like `PREFIX-T-0042`: the project prefix, a type letter (`V` vision, `I` initiative, `T` task, `A` ADR, `S` specification), and a four-digit number. The prefix comes from project configuration and the number from a database counter.

That means:

- **Never guess, predict, reserve, or "continue the sequence" of a short code.** Only `create_document` mints one. Read it back from the tool result, and use exactly that string.
- **Never renumber.** `PROJ-T-0007` stays `PROJ-T-0007` even if it is the second task you created and the fourth one to run.
- **Never abbreviate.** Not `T-7`, not `#7`, not "the 0007 task". The full short code, every time.
- If you don't know an item's short code, find it -- `mcp__metis__list_documents` or `mcp__metis__search_documents`. Do not describe it instead.

## Naming Work That Doesn't Exist Yet

Proposals, decomposition drafts, and grill rounds all discuss work before it is created. That gap is where invented IDs come from.

Until the document exists, refer to it by its **full title in quotes**, and say it isn't created yet:

> Proposed (not yet created): "Add refresh-token rotation", "Expire sessions on password change".

- **Do not assign an interim ID** -- no "T1", no "task A", no placeholder short code to be swapped later.
- Numbered lists are fine *within one message* for the user to answer against ("re: 2, yes"). The numbers are question numbers, not identifiers: don't carry them into a later message, a document, or a commit.
- The moment the document is created, drop the working title as an identifier and switch to the short code the tool returned.

## Use Metis's Types, Phases, and Categories

Metis's enumerations are closed sets. If a word isn't in them, it isn't a Metis concept -- translate it, don't adopt it.

- **Document types**: `vision`, `initiative`, `task`, `adr`, `specification`. There is no epic, story, sub-task, chore, milestone, or bug type. A spike is a `task`. A bug is a `task` with `backlog_category="bug"`.
- **Phases**: each type has its own set (tasks: `backlog`, `todo`, `active`, `blocked`, `completed`). Report status with the phase word Metis uses -- a task is `active`, not "in progress"; `completed`, not "done" or "shipped".
- **Backlog categories**: `bug`, `feature`, `tech-debt`. Nothing else.
- **Section headings**: come from the document template. Write your content under the heading that already exists rather than adding a heading of your own.

`references/canonical-vocabulary.md` has the complete tables, including phases per document type and the translation table for terms users bring from other trackers.

## Don't Invent Grouping Entities

"Workstream A", "batch 2", "phase 1 tasks", "the migration epic" -- these read like tracked things and aren't. Nothing in Metis holds them, so they die with the conversation while people go on referring to them.

Group work the way Metis groups it:

- Work that belongs together is **children of the same initiative**. That is the grouping.
- Sequencing is stated as a dependency between short codes ("`PROJ-T-0007` before `PROJ-T-0008`"), recorded in the initiative's implementation plan or the tasks' notes.
- If a grouping is real and durable, it deserves a document -- usually its own initiative. Propose creating one instead of naming a phantom.

## Outside the Conversation, Too

Anything that outlives the session carries the short code:

- **Commits and PRs**: reference `PROJ-T-0042`, never a made-up ticket ID.
- **Branch names**: derive from the short code when you name one.
- **Code comments and TODOs**: `// TODO(PROJ-T-0042)`, or no ID at all -- never an invented one.
- **Status summaries to the user**: short code plus title, so the user can look it up.
- **TodoWrite entries**: if a todo corresponds to Metis work, put the short code in the text. A todo list of invented labels is a shadow backlog with no way home.

## When You Catch an Invented Name

You will sometimes notice one -- yours, an earlier session's, or the user's. Repair it as soon as you see it; don't quietly keep using it.

1. **Resolve it.** `mcp__metis__search_documents` on the words in the name, or `mcp__metis__list_documents` filtered by type and parent, to find the document it meant.
2. **If it resolves**, say so plainly and switch: "'the auth epic' is `PROJ-I-0003`." Then fix the references you still control -- `read_document`, `edit_document` the documents that carry the invented name.
3. **If it doesn't resolve**, the work isn't tracked. Say so, and offer to create the document. Don't guess at a match.
4. **If the user used it**, they get an answer, not a correction lecture: answer the question, and use the short code in your reply so the mapping is visible.

## Additional Resources

- **`references/canonical-vocabulary.md`** -- complete tables: short code grammar, document types, phases per type, backlog categories, template section headings, and translations for outside terminology.
