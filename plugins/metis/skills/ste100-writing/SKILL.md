---
name: ste100-writing
description: This skill should be used whenever Claude writes or edits the content of a Metis document that designs work - a vision, initiative, task, ADR, or specification - including acceptance criteria, implementation plans, design sections, decision rationale, requirements tables, and status updates. Also use it when the user says "STE", "ASD-STE100", "Simplified Technical English", "plain technical English", "controlled language", or asks to make a document clearer, shorter, or easier to translate. It covers the ASD-STE100 writing rules that Metis uses for all design text.
---

# Writing Design Documents in ASD-STE100

Metis documents are read by people and agents who were not in the conversation: a reviewer next week, an agent after a compaction, a colleague who reads English as a second language. Write all design text in **ASD-STE100 Simplified Technical English (STE)**. STE is a controlled language: a small set of approved words, each with one meaning, and a small set of writing rules. The result is text with one possible interpretation.

## Scope

**Write in STE:**
- All content that you write into a vision, initiative, task, ADR, or specification.
- Acceptance criteria, implementation plans, design and alternatives sections, decision rationale, requirements tables, risks, and status updates.
- Proposals for work that you put to the user for approval (for example, a draft task breakdown before `create_document`).

**Do not change** (these are technical names, or they are not yours):
- Metis short codes, `REQ-`/`NFR-` IDs, document types, phases, and backlog categories.
- Code, identifiers, file paths, commands, configuration keys, API names, and error messages. Put them in backticks.
- Text that you quote from the user or from another source. Mark it as a quotation.

Ordinary chat replies do not have to be STE, but the text that goes into a document must be.

## The Core Rules

### Words

1. **Use approved words, with their approved meaning.** Prefer short, common words. Use `use`, not "utilize". Use `do`, not "perform". Use `before`, not "prior to". See `references/ste100-rules.md` for common substitutions.
2. **One word, one meaning. One meaning, one word.** When you select a term for a thing, use only that term in all the documents. Do not change between "user", "customer", and "account holder" for the same thing. Record project terms in the specification or initiative glossary.
3. **Technical names and technical verbs are permitted.** A technical name is a name of a thing in this project or domain (`transition_phase`, "code index", "exit criteria"). Use them exactly as the code and the other Metis documents use them.
4. **Do not use phrasal verbs or idioms.** Use "start", not "kick off". Use "make", not "set up" when "make" is correct. Use "examine", not "look into".

### Verbs

5. **Use only simple verb tenses:** simple present, simple past, and simple future. Do not write "has been done" or "will have been migrated". Write "the migration is complete".
6. **Use the active voice.** Write "The server rejects the edit", not "The edit is rejected by the server". In descriptive text, use the passive voice only when the agent is unknown or not important.
7. **Do not use the -ing form as a verb or noun.** Write "Before you merge", not "Before merging". Technical names that contain -ing are permitted.
8. **Use the imperative for instructions.** Acceptance criteria and implementation steps tell the reader what to do: "Add a retry limit of 3 to `SyncClient`."

### Sentences

9. **Procedural sentences: 20 words or fewer. Descriptive sentences: 25 words or fewer.** Procedural text says what to do (steps, acceptance criteria). Descriptive text says what a thing is or why (context, rationale, design).
10. **One instruction per sentence.** If two actions must occur at the same time, you can put them in one sentence.
11. **One topic per paragraph, and 6 sentences or fewer in a paragraph.**
12. **Do not omit words to make text shorter.** Keep articles ("a", "the") and verbs. "Update config, restart" is not STE. Write "Update the configuration file. Then restart the server."
13. **Do not use noun clusters of more than three words.** "Document phase transition validation error handler" becomes "the handler for validation errors in phase transitions".
14. **Use vertical lists** for sequences and for complex text. Use a numbered list when the sequence is important.

### Precision

15. **Be specific.** Use numbers and units, not "some", "a few", "fast", or "soon". Write "less than 200 ms", not "quickly".
16. **State conditions before actions.** Write "If the index is stale, run `metis index`", not "Run `metis index` if the index is stale".
17. **Start a warning or caution with a clear instruction**, then give the reason: "Do not force the transition. The exit criteria are not complete."

## How to Apply the Rules to Metis Documents

| Section | Text type | What to do |
|---------|-----------|------------|
| Vision objectives, initiative context, ADR context | Descriptive | 25 words or fewer per sentence. State facts and reasons. |
| Design, alternatives, decision rationale | Descriptive | One option per paragraph. Give each trade-off as a fact with a number where possible. |
| Acceptance criteria, implementation plan steps | Procedural | Imperative. 20 words or fewer. One verifiable action or result per item. |
| Requirements tables (`REQ-`/`NFR-`) | Procedural | One requirement per row. Use "must" for the requirement. Give the rationale in one sentence. |
| Status updates | Descriptive | Simple past for what occurred. Simple present for the current state. Short codes for all references. |

### Example

Not STE:

> We should probably look into utilizing a caching layer in order to ensure that the dashboard is loading reasonably fast, since users have been complaining about performance issues on large projects.

STE:

> The dashboard is slow on large projects. On a project with 500 documents, the load time is 4 s. Users reported this problem.
>
> Add a cache for document summaries. The load time must be less than 1 s for a project with 500 documents.

## Checklist Before You Save a Document

- Are all sentences in the length limit (20 procedural, 25 descriptive)?
- Is each instruction in the imperative, with one action per sentence?
- Did you use the same term for the same thing in the full document?
- Did you remove phrasal verbs, idioms, "-ing" verbs, and perfect tenses?
- Did you replace vague words ("some", "fast", "better") with numbers or with a verifiable result?
- Did you keep all short codes, code, and technical names exactly as they are?

## Additional Resources

- **`references/ste100-rules.md`** - Common word substitutions, and more examples for acceptance criteria and decision rationale.
- The official specification is ASD-STE100, "Simplified Technical English", from the ASD Simplified Technical English Maintenance Group (https://www.asd-ste100.org). Its dictionary is the authority for which words are approved.
