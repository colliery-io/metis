---
name: ste100-writing
description: This skill should be used whenever Claude writes or edits the content of a Metis document that designs work - a vision, initiative, task, ADR, or specification - including acceptance criteria, implementation plans, design sections, decision rationale, requirements tables, and status updates. Also use it when the user says "STE", "ASD-STE100", "Simplified Technical English", "plain technical English", "controlled language", or asks to make a document clearer, shorter, or easier to translate. It covers the ASD-STE100 (Issue 9) writing rules that Metis uses for all design text.
---

# Writing Design Documents in ASD-STE100

Metis documents are read by people and agents who were not in the conversation: a reviewer next week, an agent after a compaction, a colleague who reads English as a second language. Write all design text in **ASD-STE100 Simplified Technical English (STE), Issue 9**. STE is a controlled language: a dictionary of approved words, each with one meaning and one part of speech, and a set of writing rules. The result is text with one possible interpretation.

Rule numbers below refer to ASD-STE100 Issue 9 (2025-01-15), Part 1.

## Scope

**Write in STE:**
- All content that you write into a vision, initiative, task, ADR, or specification.
- Acceptance criteria, implementation plans, design and alternatives sections, decision rationale, requirements tables, risks, and status updates.
- Proposals for work that you put to the user for approval (for example, a draft task breakdown before `create_document`).

**Do not change** (these are technical nouns, or they are not yours):
- Metis short codes, `REQ-`/`NFR-` IDs, document types, phases, and backlog categories.
- Code, identifiers, file paths, commands, configuration keys, API names, and error messages. Put them in backticks.
- Text that you quote from the user or from another source. Mark it as a quotation.

Ordinary chat replies do not have to be STE, but the text that goes into a document must be.

## The Core Rules

### Words (Section 1, Section 9)

1. **Use only approved words, technical nouns, and technical verbs** (Rule 1.1). Use an approved word only as its approved part of speech and with its approved meaning (Rules 1.2, 1.3). For example, "use" is approved and "utilize" is not; "do" is approved and "perform" is not. See `references/ste100-rules.md` for verified substitutions.
2. **Technical nouns and technical verbs from your subject field are permitted** (Rules 1.5, 1.6, 1.8). In software, `repository`, `schema`, `cache`, `endpoint`, `merge`, `commit`, and `deploy` are technical nouns or verbs. Use them exactly as the code and the other Metis documents use them. Do not use a technical noun as a verb (Rule 1.7).
3. **One item, one technical noun** (Rule 1.11, Rule 9.4). When you select a term for a thing, use only that term in all the documents. Do not change between "user", "customer", and "account holder" for the same thing. Record project terms in the specification or initiative.
4. **Do not use slang or jargon as technical nouns** (Rule 1.10).
5. **Do not make phrasal verbs** (Rule 9.3). Use "start", not "kick off". Use "do", not "carry out". Use "examine", not "look into".
6. **Do not use Latin abbreviations** (GR-6). Write "for example", not "e.g.". Write "that is", not "i.e.". Do not write "etc.": give the full list or omit it.

### Verbs (Section 3)

7. **Use only these verb forms** (Rule 3.2): infinitive, imperative, simple present, simple past, simple future, and the past participle as an adjective. Do not write "has been done" or "will have been migrated". Write "the migration is complete".
8. **Do not use auxiliary verbs to make complex verb constructions** (Rule 3.4).
9. **Use the "-ing" form only as a technical noun or in a technical noun** (Rule 3.5). Write "Before you merge", not "Before merging".
10. **Use the active voice** (Rule 3.6). Write "The server rejects the edit", not "The edit is rejected by the server". In descriptive text, use the passive voice only when the agent is unknown.
11. **Use a verb to describe an action, not a noun** (Rule 3.7). Write "Examine the logs", not "Do an examination of the logs".

### Sentences (Sections 2, 4, 5, 6, 8)

12. **Procedural sentences: 20 words or fewer** (Rule 5.1). **Descriptive sentences: 25 words or fewer** (Rule 6.3). Procedural text tells the reader what to do (steps, acceptance criteria). Descriptive text gives information (context, rationale, design).
13. **Write instructions in the imperative** (Rule 5.3), **one instruction in each sentence** (Rule 5.2), unless two actions occur at the same time.
14. **If there is a condition, put it first** (Rule 5.4). Write "If the index is stale, run `metis index`."
15. **One topic in each paragraph, and 6 sentences or fewer in a paragraph** (Rules 6.5, 6.6). Give information gradually (Rule 6.1).
16. **Do not omit words or use contractions** (Rule 4.2). Keep the articles "a", "an", and "the" (Rule 4.5). "Update config, restart" is not STE. Write "Update the configuration file. Then restart the server." Write "do not", not "don't".
17. **Do not use multi-word nouns of more than three words** (Rule 2.1). "Document phase transition validation error" becomes "a validation error in a phase transition".
18. **Use a vertical list for complex text** (Rule 4.3). Use connecting words ("then", "but", "because", "thus") to connect related sentences (Rule 4.4).
19. **Do not use the semicolon** (Rule 8.1). Write two sentences.

### Precision and Safety (Section 7)

20. **Be accurate.** Use numbers and units, not "some", "several", "fast", or "soon". Write "less than 200 ms", not "quickly".
21. **Start a warning or caution with a clear command or condition, then give the risk** (Rules 7.2, 7.3): "Do not force the transition. The exit criteria are not complete."

## How to Apply the Rules to Metis Documents

| Section | Text type | What to do |
|---------|-----------|------------|
| Vision objectives, initiative context, ADR context | Descriptive | 25 words or fewer in each sentence. Give facts and causes. |
| Design, alternatives, decision rationale | Descriptive | One alternative in each paragraph. Give each trade-off as a fact, with a number where possible. |
| Acceptance criteria, implementation plan steps | Procedural | Imperative. 20 words or fewer. One action or one result in each item. |
| Requirements tables (`REQ-`/`NFR-`) | Procedural | One requirement in each row. Use "must" for the requirement. Give the cause in one sentence. |
| Status updates | Descriptive | Simple past for what occurred. Simple present for the current condition. Short codes for all references. |

### Example

Not STE:

> We should probably look into utilizing a caching layer in order to ensure that the dashboard is loading reasonably fast, since users have been complaining about performance issues on large projects.

STE:

> The dashboard is slow on large projects. On a project with 500 documents, the load time is 4 s. Users sent reports about this problem.
>
> Add a cache for document summaries. The load time must be less than 1 s for a project with 500 documents.

## Checklist Before You Save a Document

- Are all sentences in the length limit (20 procedural, 25 descriptive)?
- Is each instruction in the imperative, with one action in each sentence?
- Did you use the same technical noun for the same item in the full document?
- Did you remove phrasal verbs, perfect tenses, "-ing" verbs, contractions, semicolons, and Latin abbreviations?
- Did you replace vague words ("some", "fast", "better") with numbers or with a result that a reader can examine?
- Did you keep all short codes, code, and technical nouns as they are?

## Additional Resources

- **`references/ste100-rules.md`** - Word substitutions verified against the Issue 9 dictionary, and more examples for acceptance criteria, decision rationale, and status updates.
- The rules and substitutions here come from ASD-STE100 Issue 9 (2025-01-15), from the ASD Simplified Technical English Maintenance Group. The reference table is the word list for Metis documents. Do not look for or download the full standard.
