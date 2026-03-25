# Getting Started with Metis

This tutorial walks you through creating your first Metis project, from initialization to completing a task. By the end, you'll understand the core workflow: create a project, define initiatives, break them into tasks, and track progress through phase transitions.

## Prerequisites

- Metis installed (see [How to Install Metis](../how-to/install.md))
- A terminal with `metis` available on your PATH
- A project directory to work in

## Step 1: Initialize a Metis Project

Navigate to your project directory and initialize Metis:

```bash
cd ~/my-project
metis init --name "My First Project" --prefix "MFP"
```

You'll see output like:

```
[+] Initialized Metis workspace in /Users/you/my-project
[+] Created vision.md with project template
[+] Created config.toml with project settings
[+] Set project prefix: MFP
[+] Set flight level configuration: streamlined
```

This creates a `.metis/` directory inside your project with configuration, a database, and your first document. See [Project Structure Reference](../reference/project-structure.md) for the full layout.

The `--prefix` flag sets the prefix for short codes — unique identifiers like `MFP-V-0001` that you'll use to reference documents throughout the project.

## Step 2: Edit Your Vision

Open `.metis/vision.md` in your editor. You'll see a template with sections for Purpose, Current State, Future State, Success Criteria, Principles, and Constraints. Fill these in to describe your project's direction.

For example:

```markdown
# My First Project Vision

## Purpose
Build a REST API for managing a book collection.

## Current State
No existing system — books are tracked in a spreadsheet.

## Future State
A deployed API with CRUD operations, search, and user authentication.

## Success Criteria
- API handles 100 concurrent users
- All endpoints have integration tests
- Deployed to production with CI/CD

## Principles
- Keep the API surface small and focused
- Prefer standard libraries over custom solutions

## Constraints
- Must use PostgreSQL for persistence
- Timeline: 6 weeks
```

After editing, sync the workspace so the database reflects your file changes:

```bash
metis sync
```

## Step 3: Transition the Vision to Published

Your vision starts in `draft` phase. Move it through its lifecycle:

```bash
metis transition MFP-V-0001 review
```

After review (even if it's just you reviewing), publish it:

```bash
metis transition MFP-V-0001 published
```

## Step 4: Create an Initiative

With a published vision, create an initiative — a concrete project aligned with that vision:

```bash
metis create initiative "Build Core API" --vision MFP-V-0001
```

You'll be prompted to select a complexity level:

```
? Select complexity level:
  S - Small (1-3 days)
> M - Medium (1-2 weeks)
  L - Large (2-4 weeks)
  XL - Extra Large (1+ months)
```

Select `M` and press Enter. Metis creates the initiative:

```
✓ Created initiative: .metis/initiatives/MFP-I-0001/initiative.md
  Short Code: MFP-I-0001
  Title: Build Core API
  Parent Vision: MFP-V-0001
  Complexity: M
```

Open `.metis/initiatives/MFP-I-0001/initiative.md` and fill in the Context, Goals & Non-Goals, and other sections.

## Step 5: Advance the Initiative to Decompose

Initiatives follow a longer lifecycle: Discovery → Design → Ready → Decompose → Active → Completed.

Move your initiative through the early planning phases. Each phase represents a stage of readiness — discovery (understanding the problem), design (planning the approach), ready (design reviewed), and decompose (breaking into tasks):

```bash
metis transition MFP-I-0001 design
metis transition MFP-I-0001 ready
metis transition MFP-I-0001 decompose
```

In a real project you'd spend time in each phase. For this tutorial, we're advancing quickly to reach the `decompose` phase, where you break the initiative into concrete tasks.

## Step 6: Create Tasks

Create tasks under the initiative:

```bash
metis create task "Set up project structure" --initiative MFP-I-0001
metis create task "Implement book CRUD endpoints" --initiative MFP-I-0001
metis create task "Add search functionality" --initiative MFP-I-0001
metis create task "Write integration tests" --initiative MFP-I-0001
```

Each task gets a short code (MFP-T-0001, MFP-T-0002, etc.) and starts in the `todo` phase.

## Step 7: Work on a Task

Pick a task and start working:

```bash
metis transition MFP-T-0001 active
```

Open the task file and fill in the Implementation Notes and Acceptance Criteria sections. As you work, update the Status Updates section with your progress.

When you're done:

```bash
metis transition MFP-T-0001 completed
```

## Step 8: Check Project Status

See the state of all your work:

```bash
metis status
```

```
Code           Title                        Type        Phase       Updated
MFP-T-0001    Set up project structure     task        completed   2min ago
MFP-T-0002    Implement book CRUD          task        todo        1h ago
MFP-T-0003    Add search functionality     task        todo        1h ago
MFP-T-0004    Write integration tests      task        todo        1h ago
MFP-I-0001    Build Core API               initiative  decompose   1h ago

Phase Insights:
  Active: 0 | Todo: 3 | Completed: 1
```

You can also filter by type or phase:

```bash
metis list -t task -p todo           # Show only todo tasks
metis list -t initiative             # Show all initiatives
metis search "CRUD"                  # Full-text search
```

## Step 9: Complete the Initiative

Once all tasks are done (in a real project you'd complete the remaining three), transition the initiative:

```bash
metis transition MFP-I-0001 active
metis transition MFP-I-0001 completed
```

## Step 10: Archive Completed Work

Clean up completed documents:

```bash
metis archive MFP-I-0001
```

This moves the initiative and its tasks to `.metis/archived/`.

## What You've Learned

- **Initialize** a Metis project with `metis init`
- **Create** documents: visions, initiatives, tasks
- **Transition** documents through their phase lifecycle
- **Track** progress with `metis status` and `metis list`
- **Search** across all documents with `metis search`
- **Archive** completed work

## Next Steps

- [Using Metis with Claude Code](./using-with-claude-code.md) — Set up AI integration
- [Document Types Reference](../reference/document-types.md) — All document types and their schemas
- [Phase Lifecycle Reference](../reference/phase-lifecycle.md) — Complete phase transition rules
- [Flight Levels Methodology](../explanation/flight-levels.md) — Why Metis uses this hierarchy
