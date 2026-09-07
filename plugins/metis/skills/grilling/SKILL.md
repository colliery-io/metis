---
name: grilling
description: This skill should be used when the user asks to "grill me", "stress-test this plan", "interview me about this design", "poke holes in this", "sharpen this vision", "sharpen this initiative", or wants to be questioned relentlessly about a plan, decision, or idea until a shared understanding is reached. It is the interview engine behind the grill-vision and grill-initiative skills.
---

# Grilling

Interview the user relentlessly until you reach a shared understanding. Map the conversation as a **design tree**: every decision branches into the decisions that hang off it. Nothing is left silently assumed.

This is the interview engine. The `grill-vision`, `grill-initiative`, and `grill-decomposition` skills tell you *which* Metis document you are sharpening and which sections its branches map to. Read them for the document-specific parts; this file covers how to run the interview.

## Rounds and the Frontier

Work the tree in **rounds**. The **frontier** is every decision whose prerequisites are already settled: the questions you can ask *now* without guessing at answers you haven't heard yet.

Each round:
1. Ask the whole frontier at once. Number each question and give your recommended answer.
2. Wait for the user's answers. Do not answer for them.
3. Settled decisions push the frontier outward and unblock questions that depended on them. Recompute the frontier and ask the next round.

A question whose answer depends on another question still open in *this* round belongs to a *later* round, not this one.

Format a round like so:

```
❓ **Q1** - **<question title>**: <question body, may be several paragraphs, may offer choices>

➡️ <your recommended answer>

---

❓ **Q2** - **<question title>**: <question body>

➡️ <your recommended answer>
```

## Facts vs Decisions

Finding **facts** is your job, never the user's. When a frontier question needs a fact from the environment (the codebase, `.metis/code-index.md`, existing Metis documents, tests, configs), go and look it up. Prefer `.metis/code-index.md` and `mcp__metis__search_documents` before exploring from scratch. Never ask the user for anything you could find yourself.

Don't block on a lookup: a running exploration is an unsettled prerequisite, so only the questions downstream of it wait. Ask the rest of the frontier now.

**Decisions** are the user's. Put each to them, with your recommendation, and wait.

## Write As You Go

This is the "with docs" half. The document you are grilling is a Metis document, and it is the record of the interview. Do not batch edits to the end.

- When a decision settles, immediately `mcp__metis__edit_document` the section it belongs to. Replace template placeholders with the actual answer. Always `read_document` before `edit_document`.
- When a decision reverses an earlier one, update the section so it reads as the current truth, not as a changelog.
- Add a short entry to the document's status updates section per round: which questions closed, what changed.

The document should be readable at any point during the session by someone who was not in the conversation.

## Sharpen Language

- When the user uses a vague or overloaded term, propose a precise one. "You said 'account'. Do you mean the Customer or the User? Those are different things."
- When a term conflicts with how existing Metis documents or the code use it, call it out immediately and ask which is right.
- When relationships between concepts are being discussed, invent concrete scenarios that probe edge cases and force the boundary to be stated.
- When the user states how something works, check whether the code agrees. Surface contradictions.

## Offer ADRs Sparingly

Only offer to record an ADR when all three are true:

1. **Hard to reverse**: changing your mind later costs something real.
2. **Surprising without context**: a future reader will ask "why did they do it this way?"
3. **A real trade-off**: there were genuine alternatives and one was picked for specific reasons.

If any one is missing, skip the ADR. When all three hold, ask the user; if they agree, create it with `mcp__metis__create_document(document_type="adr", ...)`, fill in Context, Decision, Rationale, and Consequences from the interview, and reference it from the document being grilled.

## Ending the Session

The session is done when the frontier is empty: every branch of the design tree visited, nothing left silently assumed. Then:

1. Read the document back in full and check it against its exit criteria checklist. Tick what is genuinely met.
2. Summarise the settled design in a few lines and ask the user to confirm you have reached a shared understanding.
3. Do **not** transition the document's phase and do **not** start acting on the design until the user confirms. Phase transitions on visions and initiatives are human decisions.
