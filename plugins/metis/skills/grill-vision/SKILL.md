---
name: grill-vision
description: A relentless interview to sharpen a Metis vision, writing settled answers into the vision document as you go. Use when the user wants to create, challenge, or finish a vision. Pass a vision short code (e.g. PROJ-V-0001) or a title for a new vision.
disable-model-invocation: true
---

# Grill Vision

Run the `grilling` skill against a Metis vision document. Read the `grilling` skill first; it defines how to run the interview. This file defines what you are sharpening.

## Step 1: Locate or Create the Vision

Parse `$ARGUMENTS`:

- **A short code** matching `PREFIX-V-NNNN`: `mcp__metis__read_document` it. If it is not found, say so and stop.
- **Anything else** (a title or a one-line idea): confirm the title with the user, then `mcp__metis__create_document(document_type="vision", title=...)` and read it back.
- **Nothing**: `mcp__metis__list_documents(document_type="vision")`. If there is exactly one non-published vision, use it. Otherwise ask which one, or whether to create a new one.

Note the vision's phase. Grilling is normal in `draft` and `review`. A `published` vision is terminal; if the user wants to change one, tell them a new vision is the path and ask before creating it.

## Step 2: Seed the Design Tree

The vision template has these sections. Each is a branch of the tree; the questions below are the roots of each branch. Facts about the current codebase, existing initiatives, and prior visions are yours to look up before asking.

**Purpose** (why this exists)
- Who is this for, and what changes for them when the vision is achieved?
- What problem is painful enough today that it justifies this effort?
- What is the one sentence a stranger could repeat back?

**Current State** (the baseline)
- What exists now? Check the codebase, existing Metis documents, and any prior vision before asking.
- What is working that must not be broken?
- What is the honest gap between now and the future state?

**Future State** (the target)
- Describe a concrete day-in-the-life once the vision is realised. What is different?
- What is explicitly *not* part of this future? Draw the boundary.
- How far out is this? What horizon does "vision" mean here?

**Success Criteria** (how we know)
- What would you measure, or observe, to say it is done?
- Which criteria are leading indicators and which are lagging?
- What would make you declare the vision a failure even if the work shipped?

**Principles** (how we decide)
- When two good options conflict, which value wins? Ask for concrete tie-breakers, not slogans.
- Which principle would you keep even if it slowed you down?
- Is any principle already contradicted by the code or by an existing initiative?

**Constraints** (the fences)
- What is fixed: budget, people, timeline, platform, compatibility, regulation?
- Which constraints are real and which are assumptions? Challenge each one.
- What existing initiatives or ADRs bound this vision?

Expect new branches to appear as answers land. Recompute the frontier every round.

## Step 3: Interview and Write

Follow the `grilling` skill. As each decision settles, `edit_document` the matching section immediately. Record a short status update per round.

Push on anything that reads like an initiative rather than a vision ("build feature X", "Q3 goals"). Those belong under the vision, not in it; note them as candidate initiatives in the status updates and move on.

## Step 4: Close

When the frontier is empty:

1. Read the vision back in full. Check its exit criteria (purpose and success criteria defined, current and future states documented, principles established, stakeholder review).
2. Summarise the settled vision in a few lines and ask the user to confirm shared understanding.
3. Do not transition the vision's phase. If the user wants it moved to `review`, they say so; `mcp__metis__transition_phase` only on explicit instruction.
4. If candidate initiatives surfaced, list them and offer to grill the first one with `grill-initiative` once the vision is published.
