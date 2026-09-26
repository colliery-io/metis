# ASD-STE100 Reference for Metis Documents

Every substitution in this file was checked against the dictionary in ASD-STE100 Issue 9 (2025-01-15), Part 2. The dictionary is the authority. Get it free from https://www.asd-ste100.org/STE_downloads.html. It is in the PDF, not on a web page.

In the dictionary, approved words are in uppercase. Unapproved words are in lowercase and show one or more approved alternatives. An approved word is approved for one part of speech only. For example, `CHANGE (v)` is approved, `COMPLETE (v)` is approved, but `complete (adj)` is not: use `FULL (adj)`.

## Verified Word Substitutions

### General

| Do not use | Use (Issue 9 alternative) |
|------------|---------------------------|
| utilize, employ | USE |
| perform, conduct, carry out, implement, execute | DO |
| prior to | BEFORE |
| ensure, verify | MAKE SURE |
| commence, initiate | START |
| terminate, cease | STOP |
| obtain, acquire, achieve | GET |
| modify, alter, amend | CHANGE |
| indicate | SHOW |
| locate, detect, determine | FIND |
| assist, facilitate | HELP |
| investigate, evaluate | EXAMINE |
| create | MAKE |
| delete | REMOVE, ERASE |
| reduce | DECREASE |
| improve | make BETTER |
| maintain | KEEP |
| avoid | PREVENT |
| fix (a defect) | REPAIR |
| run (a system) | OPERATE (in software, "run" as a technical verb is also acceptable) |
| provide | GIVE, SUPPLY |
| enable, allow | LET |
| explain | TELL |
| describe, define | GIVE |
| consider | THINK |
| require, need | NECESSARY ("X is necessary"), MUST |
| should, shall | MUST |
| may (possibility) | CAN, POSSIBLY |
| enough, adequate | SUFFICIENT |
| additional | MORE |
| several | SOME (or the accurate number) |
| various | DIFFERENT |
| exact | ACCURATE, CORRECT |
| specific | APPROVED, SPECIFIED |
| appropriate | APPLICABLE |
| following (adj) | THESE, or a clause with FOLLOW (v) |
| however | BUT |
| therefore | THUS, AS A RESULT |
| whether | IF |
| once, whenever | WHEN |
| now | AT THIS TIME |
| via | THROUGH |
| in the event of | IF |
| state (v) | TELL |
| reason (n) | CAUSE |
| option (n) | ALTERNATIVE |
| decide | SELECT |
| remain | STAY |
| fail | IF ... NOT, UNSATISFACTORY |
| failed (adj) | DEFECTIVE |

### Approved Words That People Often Remove by Mistake

These words are approved in Issue 9. You can use them: SUBSEQUENT (adj), SUFFICIENT (adj), APPROXIMATELY (adv), INCLUDE (v), CONTAIN (v), IDENTIFY (v), PREVENT (v), RECOMMEND (v), SELECT (v), REJECT (v), ACCEPT (v), BECAUSE (conj), SINCE (conj), WHILE (conj), UNLESS (conj), UNTIL (prep), THUS (adv), THEN (adv).

### Technical Nouns and Technical Verbs

Many words in software design are not in the dictionary. Rules 1.5, 1.6, and 1.8 permit them as technical nouns or technical verbs of your subject field. Examples: `repository`, `schema`, `cache`, `endpoint`, `requirement`, `document`, `merge`, `commit`, `deploy`, `index`, `archive`. Use each with one meaning only. Do not use a technical noun as a verb (Rule 1.7).

## Acceptance Criteria

Each criterion is one procedural sentence (20 words or fewer, imperative) or one result that a reader can examine.

| Not STE | STE |
|---------|-----|
| Make sure things work properly with large inputs | `import_documents` accepts a file of 10,000 rows in less than 5 s. |
| Error handling should be improved | If the database file is locked, the CLI shows the path of the file and stops with exit code 2. |
| Tests have been written and are passing | Add unit tests for `transition_phase` with a blocked task. Make sure that all tests are satisfactory. |
| Look into refactoring the parser | Move the frontmatter parser from `lib.rs` to `parser.rs`. Do not change its public API. |

## Decision Rationale (ADRs and Design Sections)

Give each alternative in its own paragraph. For each alternative, tell what it is and give its trade-offs as facts.

Not STE:

> After having considered a number of options, we've decided to go with SQLite since it's basically zero-config and should scale well enough for what we're doing.

STE:

> We selected SQLite for the document store.
>
> SQLite does not use a server process. Each workspace has one database file in `.metis/`.
>
> The largest workspace at this time has 3,000 documents. SQLite reads this workspace in less than 50 ms.
>
> We rejected PostgreSQL because it must have a server process on each developer computer.

## Status Updates

Use the simple past for what occurred. Use the simple present for the current condition. Refer to all Metis documents by short code.

Not STE:

> Been working through the migration, mostly done, still need to sort out the edge cases with archived docs.

STE:

> We migrated 42 of 45 document types to the new schema.
>
> Archived documents do not migrate because the `archived` flag is not in the new schema. Next step: add the flag. Then migrate the 3 remaining types.

## Terms

STE requires one technical noun for one item (Rule 1.11). In a design, new terms occur quickly. When a term is settled:

1. Select one term. Use the term that the code already uses, if there is one.
2. Write a one-sentence definition of the term in the specification or the initiative.
3. Use only that term in all the documents. If an earlier document uses a different term, change it.
