---
name: metis-vocabulary
description: This skill should be used whenever work is named, referred to, numbered, or grouped in a Metis project - when the user or Claude says "task 1", "slice 2", "D3", "the third decision", "the first task", "epic", "story", "sub-task", "ticket", "workstream", "phase 2", "mark it done", "in progress", "what should I call this", "what's the ID for this" - and whenever writing an implementation plan, a decision log, requirements, a commit message, or a status summary that refers to Metis work. Metis assigns every document a short code; this skill covers using Metis's own identifiers, document types, phases, and categories instead of inventing shorthand, and the one carve-out for numbered requirements (REQ/NFR) on a specification.
---

# Metis Vocabulary

Metis has names for everything it tracks. **Use them. Never invent your own.**

An invented name -- "Task 3", "the auth epic", "Slice 2", "D3", "phase 2 work" -- looks harmless in one message and is unrecoverable two messages later. It does not resolve in `list_documents`, it does not appear in the user's GUI or `metis status`, and after a compaction nobody (including you) can tell which document it meant.

## The Test

Not every identifier is invented. The question is always the same:

> **Can a reader find where this identifier is defined?**

- A **Metis short code** resolves through `read_document`. Legitimate.
- A **`REQ-2.1.1` in a specification's requirements table** resolves to a row in that table. Legitimate, and required -- but only there; see "The One Carve-Out" below.
- **`D3`, `Slice 2`, `Task 1`, "the third decision"** resolve to nothing. They were introduced in passing and then reused as if they had been defined. Invented.

So there are two kinds of identifier, with two different rules:

| Kind | Owned by | Rule |
|------|----------|------|
| **Cross-document identity** -- anything referred to from outside the document it lives in | Metis | The short code, minted by `create_document`, never invented |
| **Requirements on a specification** -- and nothing else | That specification | Use the template's scheme (`REQ-x.y.z`, `NFR-x.y.z`); qualify with the short code when citing from outside |

Everything else is invented, and the fact that it looks structured (`D3`, `S-2`, `Slice 1`) makes it worse, not better: a reader assumes a scheme exists and goes looking for it.

## The One Rule

**A work item's name is its Metis short code, and nothing else.**

Short codes look like `PREFIX-T-0042`: the project prefix, a type letter (`V` vision, `I` initiative, `T` task, `A` ADR, `S` specification), and a four-digit number. The prefix comes from project configuration and the number from a database counter.

That means:

- **Never guess, predict, reserve, or "continue the sequence" of a short code.** Only `create_document` mints one. Read it back from the tool result, and use exactly that string.
- **Never renumber.** `PROJ-T-0007` stays `PROJ-T-0007` even if it is the second task you created and the fourth one to run.
- **Never abbreviate.** Not `T-7`, not `#7`, not "the 0007 task". The full short code, every time.
- If you don't know an item's short code, find it -- `mcp__metis__list_documents` or `mcp__metis__search_documents`. Do not describe it instead.

**A short code you write into a document must be one you have just created or just read back.** This is the rule that gets broken quietly: an initiative's Implementation Plan fills up with references like "handled by `PROJ-T-0012`" for tasks that were never created, or that turn out to be something else entirely. A short code recalled from memory is a guess. Before it goes in a document, either it came back from `create_document` in this session, or you resolved it with `read_document`/`list_documents`. Work that doesn't exist yet is referenced by quoted title, never by a forward-looking code.

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
- **Numbering schemes**: there is exactly one, and it belongs to a specification's requirements tables. See "The One Carve-Out" below.

`references/canonical-vocabulary.md` has the complete tables, including phases per document type and the translation table for terms users bring from other trackers.

## The One Carve-Out: Requirements On a Specification

There is exactly one place where an identifier that isn't a Metis short code is correct, and it is **a specification document, in its requirements tables**. A specification is where PRD-style content lives, and PRD requirements are numbered so they can be cited, tested, and traced. That numbering stays.

The carve-out reaches no further. It does not apply to initiatives, tasks, ADRs, or visions -- those have no requirement tables, and nothing in them gets a numbering scheme of your own.

The template gives both requirement tables an `ID` column:

```
### Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| REQ-1.1.1 | The API rejects a token older than 24h | Limits blast radius of a leaked token |

### Non-Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| NFR-1.1.1 | p99 auth latency stays under 50ms | Login is on the critical path |
```

- Use the template's prefixes: **`REQ-x.y.z`** for functional, **`NFR-x.y.z`** for non-functional. Don't substitute a scheme of your own (`FR-1`, `F1`, `R.1`) -- the template's is the convention the project already reads.
- Every requirement gets an ID and a rationale. A requirements table with a blank ID column is incomplete in the same way a template placeholder is.
- IDs are stable. Adding a requirement means a new number, never renumbering the existing rows -- something may already cite them.
- **Citing one from outside the specification, qualify it with the short code**: "`METIS-S-0002` REQ-2.1.1". `REQ-2.1.1` alone is only unique inside its own document, so on its own -- in an initiative, a task, or a commit -- it is exactly the dangling reference this skill is about.

Everywhere else, resist the pull to number things:

- An **initiative's** goals, non-goals, and detailed design are prose. They do not become `G1`, `NG2`, `DD-3`.
- A **task's** acceptance criteria are the checklist the template gives it. Cite one as "`PROJ-T-0042`, third criterion" -- not `AC-3`.
- An **ADR's** decision is the ADR. It is `PROJ-A-0003`, not `D3`.

## Plans Are Written In Short Codes

A plan describes work, and every piece of work it describes already has a Metis name. So a plan -- an initiative's Implementation Plan, a specification's Decision Log, a status summary, anything that sequences or assigns -- is written in short codes:

- Work to be done: the task's short code, or a quoted title if the task does not exist yet.
- The capability it belongs to: the initiative's short code.
- The requirements it satisfies: the specification's short code, qualified (`METIS-S-0002` REQ-2.1.1).
- The decision it follows: the ADR's short code.

Every one of those must resolve. A plan whose steps are "Slice 2", "D3", and "phase 1 work" describes nothing that can be opened, worked, or completed -- and a plan citing `PROJ-T-0012` for a task that was never created is worse, because it looks like it resolves.

## Decisions Are ADRs, Not D-Numbers

A design conversation produces decisions, and the tempting shorthand is to number them: `D1`, `D2`, `D3`, "the third decision". Two messages later "let's revisit D3" means nothing, and nothing in Metis ever closed it.

Metis already has the identity for a decision, and the specification template already says so:

- **Architecture Framing** describes a *Decision Area* -- named by its area name, with context, constraints, required capabilities, and an **ADR** field for the decision once made.
- **Decision Log** is a table keyed by **ADR short code** (`PROJ-A-0001`), with title, status, and a one-line summary.

So:

- A decision that has been made is `PROJ-A-0003`. That is its name, in the decision log, in the initiative that referenced it, and in conversation.
- A decision area still open is named by its **area name** ("Session storage backend"), with the ADR field marked pending. Not `D3`.
- Not every decision earns an ADR -- most settle into the Detailed Design section as prose and need no identifier at all. Writing it down as prose is the alternative to numbering it, not a reason to number it.
- Numbers you used to run a conversation (`Q1`-`Q6` in a grill round, "option 2") stay in that message. They never enter a document, a commit, or a later message as a name.

## Slices Are a Shape, Not a Name

"Vertical slice" describes *how* work was cut, the way "horizontal layer" or "risk-first" does. It is not an identifier, and a slice is not a thing Metis tracks.

The failure looks like this: you cut an initiative into five slices, label them "Slice 1" through "Slice 5" while drafting, write those labels into the initiative's Implementation Plan, and now the plan refers to five entities that exist nowhere -- no short code, no phase, nothing that can be completed.

- A slice **becomes a task**. Once created, its name is the task's short code.
- Before it is created, it is a **quoted title** -- "Login flow" -- marked not yet created.
- An implementation plan sequences **short codes**, not slice numbers.
- The word "slice" belongs in a sentence about the shape of the decomposition ("cut as vertical slices so each proves something on its own"), never as a label attached to a number.

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

**A short code already written into a document gets verified, not trusted.** When you read an initiative whose plan cites `PROJ-T-0012`, `read_document` it before building on it. A code that resolves to nothing, or to a document that isn't what the sentence claims, is a broken reference: fix the document to name the real work, or replace the code with a quoted title if the work was never created. Do not propagate it into new documents, tasks, or commits on the assumption that a previous session got it right.

## Additional Resources

- **`references/canonical-vocabulary.md`** -- complete tables: short code grammar, document types, phases per type, backlog categories, template section headings, the specification requirement-ID convention, and translations for outside terminology.
