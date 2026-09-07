# Canonical Metis Vocabulary

Every enumeration Metis uses, in full. If a word is not on these lists, it is not a Metis term -- translate it before you write it down.

## Short Codes

```
PREFIX-L-NNNN
   |    |   |
   |    |   +-- four-digit counter, assigned by Metis, never reused or renumbered
   |    +------ type letter
   +----------- project prefix, set at `metis init`
```

| Letter | Document type |
|--------|---------------|
| `V` | vision |
| `I` | initiative |
| `T` | task (including backlog items) |
| `A` | ADR |
| `S` | specification |

The counter is per type and lives in the project database. It is allocated by `create_document` at creation time, and the result of that call is the only place a new short code comes from. Two documents created in the same session are not necessarily consecutive; documents may be archived, leaving gaps. Never infer the next number.

Backlog items are tasks -- they get a `T` code like any other task.

## Document Types

| Type | What it is | Parent |
|------|------------|--------|
| `vision` | North star: why the project exists | None |
| `initiative` | A capability increment | Vision (published) |
| `task` | An atomic unit of work | Initiative (in `decompose` or `active`) |
| `task` + `backlog_category` | Standalone bug / feature / debt, no initiative | None |
| `adr` | A recorded architectural decision | None |
| `specification` | Living system or feature spec | Vision or initiative |

Availability depends on the project's flight level preset -- `direct` mode has no initiatives, for instance. `create_document` rejects a type the preset disables; read the error rather than substituting a different type.

## Phases, By Document Type

Phases are forward-only. The only backward move in the system is a task returning from `blocked`.

| Type | Phase sequence | Also |
|------|----------------|------|
| Vision | `draft` -> `review` -> `published` | |
| Initiative | `discovery` -> `design` -> `ready` -> `decompose` -> `active` -> `completed` | |
| Task | `todo` -> `active` -> `completed` | `backlog` (before `todo`, for backlog items); `blocked` from `todo` or `active`, and back |
| ADR | `draft` -> `discussion` -> `decided` -> `superseded` | |
| Specification | `discovery` -> `drafting` -> `review` -> `published` | |

A task created under an initiative starts in `todo`. A task created with a `backlog_category` starts in `backlog`.

Report status using these exact words:

| Don't say | Say |
|-----------|-----|
| in progress, WIP, started, underway | `active` |
| done, shipped, closed, finished, resolved | `completed` |
| open, not started, queued, up next | `todo` |
| on hold, waiting, stuck, parked | `blocked` |
| icebox, someday, unprioritized | `backlog` |
| approved, signed off, final | `published` (vision/spec) or `decided` (ADR) |
| planning, scoping | `discovery` or `design` (initiative) |

## Backlog Categories

Exactly three: `bug`, `feature`, `tech-debt`.

Not `enhancement`, `chore`, `refactor`, `docs`, `spike`, `incident`, `p0`. Pick the closest of the three -- a refactor is `tech-debt`, a spike is a `feature` or a task under an initiative -- and put the nuance in the title and body, where it is readable, rather than in a category that doesn't exist.

## Terms From Other Trackers

Users bring vocabulary from Jira, Linear, GitHub, and Shortcut. Translate on the way in; answer using the Metis term so the mapping stays visible.

| They say | Metis |
|----------|-------|
| epic, project, workstream | initiative |
| story, ticket, issue, card, work item | task (with a parent) or backlog item (standalone) |
| sub-task, checklist item, step | an acceptance criterion inside a task -- not its own document |
| bug, defect, regression | task with `backlog_category="bug"` |
| feature request, enhancement | task with `backlog_category="feature"` |
| tech debt, refactor, cleanup | task with `backlog_category="tech-debt"` |
| spike, investigation, research | task (title it "Spike: ...") |
| design doc, RFC, tech spec | specification |
| decision record, ADR, "the why" | adr |
| roadmap, OKR, north star | vision |
| sprint, milestone, release, phase 2 | no Metis equivalent -- this is ordering between tasks, not a document. Record it as a dependency in the initiative's implementation plan |
| backlog grooming, refinement | the `decompose` phase of an initiative |
| status, state, column, swimlane | phase |
| slice, vertical slice, workstream slice | a shape of decomposition, not a name -- the slice becomes a task with a short code |
| D1 / D2 / D3, "decision 3" | the ADR's short code, or the decision area's name while it is still open |
| FR-1, F1, R.1 | `REQ-x.y.z` in a specification's Functional Requirements table |
| assignee, owner | not tracked as a field -- name people in the document body if it matters |

The last few rows matter most: they are the terms with **no** Metis document behind them. Naming one anyway is how a phantom entity enters a project.

## Template Section Headings

Fill in the headings the template already has instead of adding your own. If content has nowhere to go, it usually belongs in a different document type.

| Type | Headings |
|------|----------|
| Vision | Purpose, Current State, Future State, Success Criteria, Principles, Constraints |
| Initiative | Context, Goals & Non-Goals, Detailed Design, Alternatives Considered, Implementation Plan, Testing Strategy |
| Task | Parent Initiative, Objective, Acceptance Criteria, Implementation Notes, Status Updates |
| ADR | Context, Decision, Rationale, Consequences (Positive / Negative / Neutral) |
| Specification | Overview, System Context, Requirements (Functional / Non-Functional, each an ID table), Architecture Framing, Decision Log, Constraints, Changelog |

Read the document after creating it -- the template is the source of truth, and presets or project-local template overrides can change it. Placeholder text in `{braces}` is not content; replace all of it.

## Numbered Requirements: Specifications Only

A specification is the one document type with a numbering scheme of its own, because it is where PRD-style requirements live and those need to be citable and traceable. The template provides it:

| Table | ID prefix | Example |
|-------|-----------|---------|
| Functional Requirements | `REQ-` | `REQ-1.1.1` |
| Non-Functional Requirements | `NFR-` | `NFR-1.1.1` |

Rules:

- Every row gets an ID and a rationale. A blank ID column is an incomplete document.
- Use the template's prefixes. Not `FR-`, `F1`, or `R.1` -- the project reads the template's convention.
- IDs never get renumbered; a new requirement takes a new number, because something may already cite the old ones.
- Cited from outside the specification, an ID is qualified with the short code: `METIS-S-0002` REQ-2.1.1.

**No other document type gets a numbering scheme.** Initiative goals and design are prose. Task acceptance criteria are the template's checklist, cited as "`PROJ-T-0042`, third criterion". A decision is an ADR short code, never `D3`.

## Where Decisions Live

| Situation | Its name |
|-----------|----------|
| Decision made, recorded as an ADR | The ADR short code, e.g. `PROJ-A-0003` |
| Decision area open on a specification | Its area name ("Session storage backend"), ADR field pending |
| Design choice not worth an ADR | Prose in the Detailed Design section -- no identifier at all |
| A round of interview questions | `Q1`-`Q6`, valid only inside that one message |

The specification template's Decision Log is a table keyed by ADR short code. A `D1`/`D2`/`D3` scheme is a substitute for that key, and it resolves to nothing.

## Progress Goes In Status Updates

A task's `Status Updates` section is the running log: what you did, what you found, what you decided, what's next. It is the section that survives compaction.

Do not invent a parallel place for it -- no `NOTES.md`, no `progress.md`, no scratch file beside the code, no section you added yourself called "Progress" or "Log". One task, one log, under the heading the template gave you.
