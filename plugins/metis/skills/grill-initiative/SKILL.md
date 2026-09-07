---
name: grill-initiative
description: A relentless interview to sharpen a Metis initiative, writing settled answers into the initiative document as you go and recording hard-to-reverse trade-offs as ADRs. Use when the user wants to create, challenge, or design an initiative before it is decomposed. Pass an initiative short code (e.g. PROJ-I-0001) or a title for a new initiative.
disable-model-invocation: true
---

# Grill Initiative

Run the `grilling` skill against a Metis initiative document. Read the `grilling` skill first; it defines how to run the interview. This file defines what you are sharpening.

## Step 1: Locate or Create the Initiative

Parse `$ARGUMENTS`:

- **A short code** matching `PREFIX-I-NNNN`: `mcp__metis__read_document` it. If it is not found, say so and stop.
- **Anything else** (a title or a one-line idea): confirm the title and the parent vision (`mcp__metis__list_documents(document_type="vision")`; initiatives hang off a published vision), then `mcp__metis__create_document(document_type="initiative", title=..., parent_id=...)` and read it back.
- **Nothing**: `mcp__metis__list_documents(document_type="initiative")`. If exactly one initiative is in `discovery` or `design`, use it. Otherwise ask.

Also read the parent vision. Every answer in the interview should trace back to it; when one does not, say so.

## Phase Awareness

The initiative's phase sets the depth of the interview:

| Phase | Focus of the interview |
|-------|------------------------|
| `discovery` | Context, goals, non-goals, and whether the problem is understood well enough to design |
| `design` | Detailed design, alternatives considered, testing strategy, implementation plan |
| `ready` / `decompose` | The design is meant to be settled. Grill only what the user asks to reopen, and flag that answers may invalidate tasks already created |
| `active` / `completed` | Do not grill. Tell the user the initiative is past design and offer to open a new one |

Never transition the phase yourself. Initiatives are strategic; the human decides when discovery is done and when design is done.

## Step 2: Seed the Design Tree

The initiative template has these sections. Each is a branch; the questions below are the roots. Facts about the codebase, existing documents, and prior decisions are yours to look up before asking. Use `.metis/code-index.md` first.

**Context**
- What is true today that makes this initiative necessary now? Verify against the code and the vision.
- Who is asking for it, and who is affected but not asking?
- What has been tried before, in this codebase or elsewhere, and why did it stop?

**Goals & Non-Goals**
- What capability exists after this initiative that does not exist before? One sentence per goal.
- For each goal: how would you tell it was met? If you cannot, it is not a goal yet.
- What adjacent thing will people assume is included that is not? Write each one down as a non-goal.
- Which goal is the one you would keep if you had to drop the rest?

**Detailed Design** (in `design` and later)
- What are the moving parts, and where do they live in the current codebase? Look this up.
- What is the interface between this and what already exists? Data shapes, APIs, ownership.
- Walk through the main scenario end to end. Then walk through the failure scenario. Then the concurrent one.
- Which term in the design is ambiguous? Pin it down before moving on.

**Alternatives Considered**
- What is the simplest thing that could work, and why is it not enough?
- What would a different team have built here? Why not that?
- For each rejected alternative: what specific fact or value ruled it out? "Felt wrong" is not an answer.

**Implementation Plan**
- What is the first vertical slice that proves the approach? What would it demonstrate?
- What must be true before decomposition can start?
- Where are the dependencies on other initiatives, teams, or systems? Are they resolved?

**Testing Strategy**
- How does the user know it works? How does the code know?
- What would you be embarrassed to have shipped broken? Test that first.
- What cannot be tested automatically, and who checks it?

Expect new branches to appear as answers land. Recompute the frontier every round.

## Step 3: Interview and Write

Follow the `grilling` skill. As each decision settles, `edit_document` the matching section immediately. Record a short status update per round.

Two extra disciplines for initiatives:

- **ADRs.** When an alternative is rejected for a reason that is hard to reverse, surprising without context, and a real trade-off, offer an ADR. If accepted, create it with `mcp__metis__create_document(document_type="adr", ...)`, fill it in from the interview, and link it from Alternatives Considered. Most rejected alternatives do not need one.
- **Scope creep.** When an answer describes work that is its own capability increment, say so and suggest it be a separate initiative. Note it in the status updates rather than growing this one.

## Step 4: Close

When the frontier is empty:

1. Read the initiative back in full. Check its exit criteria (context and goals defined, technical approach designed, implementation plan phased, testing strategy defined, dependencies identified).
2. Summarise the settled design in a few lines and ask the user to confirm shared understanding.
3. Do not transition the phase. Tell the user which phase the document is in and what the next one is, and let them decide.
4. If the user confirms and the initiative is in `design` or later, offer the `decomposition` skill for breaking it into tasks. Decomposition stays human-in-the-loop.
