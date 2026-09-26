# ASD-STE100 Reference for Metis Documents

This file gives common substitutions and more examples. The ASD-STE100 dictionary (https://www.asd-ste100.org) is the authority for approved words. If a word is not in this table and you are not sure about it, use a shorter, more common word.

## Common Word Substitutions

| Do not use | Use |
|------------|-----|
| utilize, leverage, employ | use |
| perform, carry out, conduct | do |
| prior to | before |
| subsequent to, following | after |
| in order to | to |
| ensure, verify | make sure |
| commence, initiate, kick off | start |
| terminate, cease | stop |
| obtain, acquire | get |
| modify, alter, amend | change |
| indicate, demonstrate, point out | show |
| locate, find out | find |
| look into, investigate | examine |
| assist, facilitate | help |
| numerous, a number of | many (or the exact number) |
| sufficient, adequate | enough (or the exact quantity) |
| X requires Y | X must have Y / Y is necessary for X |
| may, might (possibility) | can, possibly |
| should (for a requirement) | must |
| via | through, with |
| e.g. / i.e. / etc. | for example / that is / write the full list |
| in the event that | if |
| at this point in time | now |

**Technical verbs are permitted.** Verbs such as `merge`, `commit`, `deploy`, `index`, `archive`, and `roll back` have a specific meaning in this domain. Use them as technical verbs, with one meaning each.

## Acceptance Criteria

Each criterion is one procedural sentence: an action or a result that a reader can verify.

| Not STE | STE |
|---------|-----|
| Make sure things work properly with large inputs | `import_documents` accepts a file of 10,000 rows in less than 5 s. |
| Error handling should be improved | If the database file is locked, the CLI shows the path of the file and stops with exit code 2. |
| Tests have been written and are passing | Add unit tests for `transition_phase` with a blocked task. All tests pass. |
| Look into refactoring the parser | Move the frontmatter parser from `lib.rs` to `parser.rs`. Do not change its public API. |

## Decision Rationale (ADRs and Design Sections)

Give each option in its own paragraph. For each option, state what it is, one advantage, and one disadvantage, as facts.

Not STE:

> After having considered a number of options, we've decided to go with SQLite since it's basically zero-config and should scale well enough for what we're doing.

STE:

> We selected SQLite for the document store.
>
> SQLite needs no server process. Each workspace has one database file in `.metis/`.
>
> The largest known workspace has 3,000 documents. SQLite reads this workspace in less than 50 ms.
>
> We rejected PostgreSQL. It needs a server process on each developer machine.

## Status Updates

Use the simple past for what occurred and the simple present for the current state. Refer to all Metis documents by short code.

Not STE:

> Been working through the migration, mostly done, still need to sort out the edge cases with archived docs.

STE:

> Migrated 42 of 45 document types to the new schema.
>
> Archived documents do not migrate. The `archived` flag is not in the new schema. Next step: add the flag, then migrate the 3 remaining types.

## Terms and Glossaries

STE requires one word for one meaning. In a design, new terms appear fast. When a term settles:

1. Select one term. Prefer the term that the code already uses.
2. Write a one-sentence definition in the glossary or the design section of the specification or initiative.
3. Use only that term in all the documents. If an earlier document uses a different term, change it.
