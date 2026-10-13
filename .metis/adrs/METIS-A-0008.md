---
id: 001-lazy-short-code-finalization-via
level: adr
title: "Lazy Short Code Finalization via UUID Provisional Identity"
number: 1
short_code: "METIS-A-0008"
created_at: 2026-09-21T20:01:38.628134+00:00
updated_at: 2026-09-21T20:42:39.980698+00:00
decision_date: 
decision_maker: 
parent: 
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
initiative_id: NULL
---

# ADR-1: Lazy Short Code Finalization via UUID Provisional Identity

*This template includes sections for various types of architectural decisions. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

METIS-A-0006 established short codes (`PREFIX-TYPE-NNNN`) as Metis's document identifier, assigned automatically and immediately at `metis create` time by incrementing a counter in `metis.db`. It explicitly claimed "automatic conflict-free document creation" as a benefit.

That claim only holds when one branch creates documents at a time. Metis is git-native and offline-first: two branches can each run `metis create` while diverged, and each computes its "next free number" from its own local `metis.db`/filesystem state, with no coordination between them. When both branches later merge:

- If the two new documents land at **different file paths** but the same `short_code` (e.g. one nested under an initiative, the other in `backlog/`), `SyncService::resolve_short_code_collisions` (`crates/metis-docs-core/src/application/services/synchronization.rs:296`) detects the duplicate short code post-merge and renumbers one of them automatically. This case is already handled.
- If the two new documents land at the **same file path** (same type, same counter value, e.g. both computed `RFQ-T-0091.md`), git sees a conflicting add on that path and stops the merge with a conflict marker before any Metis code runs. No sync-time logic ever gets a chance to fire.

This second case is what actually happened twice on a real branch: `RFQ-A-0018.md` (an ADR both branches independently created) and `RFQ-T-0091.md`/`RFQ-T-0092.md` (a task collision that also masqueraded as a rename-vs-modify conflict). Both required manual archaeology (including reading raw git blobs) to untangle, and one produced a silent renumbering to a short code (`RFQ-T-0121`) that had to be re-verified against every cross-reference by hand. The root cause in both cases was the same: short code assignment is a decision made independently, with incomplete information, by whichever branch runs `metis create` first — and the filename is derived directly from that guess.

## Decision **[REQUIRED]**

We will separate a document's durable identity from its display short code:

- `DocumentMetadata` gains a `uid` field — a UUID v4 assigned once at creation, written to frontmatter, and never changed. It sits alongside the existing `id` (title slug, `crates/metis-docs-core/src/domain/documents/traits.rs:9`) and `short_code` (`crates/metis-docs-core/src/domain/documents/metadata.rs:10`) as a third identity concept: `id` is a human-readable slug, `short_code` is the display code, `uid` is the collision-proof anchor that lets any later process recognize "this is still the same document" independent of what it's named.
- `short_code` becomes optional (`Option<String>`, serialized as `short_code: null` when unset).
- `DocumentCreationService` (`crates/metis-docs-core/src/application/services/document/creation.rs`) no longer calls `generate_short_code` synchronously during creation. It assigns a `uid`, writes `short_code: null` into frontmatter, and names the file from the full uid (e.g. `_pending-3f9a1c2e-6b7d-4a1f-9e21-8d4b6a2c9f10.md`) instead of from a short code that doesn't exist yet.
- `SyncService::sync_directory` (`crates/metis-docs-core/src/application/services/synchronization.rs:621`) gains a finalization step: find documents with `short_code: null`, order them by `created_at`, and assign each the next short code in that order — reusing the exact same `generate_short_code` + file-rename + `update_sibling_references` machinery that `renumber_document` (`:380`) already implements for collision resolution, just triggered by "code missing" instead of "code duplicated."
- `metis create` (CLI, MCP tool, and the Tauri GUI command) already auto-runs sync immediately after creating a document, in every interface. Day-to-day single-branch usage is unaffected: a document still gets its real, permanent short code within the same command invocation. The only behavior change is what happens when two branches each create a document while diverged — they now write to two different uid-derived paths, git merges that cleanly with no conflict, and the next `metis sync` (already required practice after any merge) hands out real short codes in creation order on the already-merged tree, with no other branch racing it.

This does not change the short code format or retire METIS-A-0006's decision to use `PREFIX-TYPE-NNNN` — it changes *when* and *from what state* a short code is assigned, to fix the specific failure mode METIS-A-0006 didn't anticipate.

**Migration path.** Metis is already in active use across multiple repos, so this cannot assume a clean-slate rollout:

- *Backfilling existing documents*: every already-short-coded document on disk has no `uid`. This is handled the same way counter recovery already works — `recover_counters_from_filesystem` (`crates/metis-docs-core/src/application/services/synchronization.rs:753`) scans every markdown file on each `sync_directory` call and repairs counters that are behind; the finalization pass gains the same shape of step, assigning a fresh `uid` to any document that doesn't have one yet. It's automatic, idempotent, and requires no user-run command — the same property `WorkspaceMigrationService::migrate` (`crates/metis-docs-core/src/application/services/workspace/migration.rs:34`) already relies on for its v1→v2 layout migration. Because `uid` is purely additive and YAML frontmatter parsers ignore unrecognized fields, an old binary reading an already-backfilled document is unaffected either way.
- *Pending filename format*: the provisional path uses the **full UUID v4** — `_pending-3f9a1c2e-6b7d-4a1f-9e21-8d4b6a2c9f10.md` — not a truncated prefix. A truncated prefix (e.g. 8 hex chars) would reintroduce a nonzero collision probability into a design whose entire purpose is eliminating that probability, and specifically under the one condition (diverged branches that can't coordinate before merging) where there's no safety net to catch a near-collision before it happens. The full UUID also matches the `uid` frontmatter field verbatim, so the same string greps to both places. The only cost is a longer, less readable filename during the pending window, which in the common case lasts only until the auto-sync that immediately follows every `metis create` call.
- *Version skew within a repo*: the actual incompatibility is an old binary (expecting `short_code` as a required, non-optional string) encountering a `_pending-*.md` file a new binary created. We are explicitly not adding a schema/version gate for this. `sync_directory` already isolates a per-file parse failure into a single `SyncResult::Error` (`crates/metis-docs-core/src/application/services/synchronization.rs:634-641`) rather than aborting the whole sync, so an old binary hitting a pending file today already fails loud but contained on that one document — the rest of the workspace keeps syncing. That existing behavior is judged sufficient; collaborators on an old binary see a clear per-document error until they upgrade, rather than a corrupted workspace.

**Implementation scope.** This ADR's implementation covers `metis-docs-core`, the CLI, and the MCP tools in one pass — those three already call straight into `Application::sync_directory`/`DocumentCreationService`, so they pick up the fix together. The Tauri GUI (`crates/metis-docs-gui`) is explicitly deferred to a follow-up: its own `create_document` command and TypeScript types (`CreateDocumentResult.short_code: string`) assume a short code exists immediately after creation, and showing a still-pending document sensibly in the Kanban/viewer UI is its own design question, not something to rush alongside the core mechanism. Until the GUI follow-up lands, GUI-created documents keep working exactly as they do today (eager assignment) — the GUI is simply not yet a source of the provisional-path behavior described above.

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| Status quo (eager assignment + post-hoc `resolve_short_code_collisions`) | No code change; already handles same-code/different-path collisions | Does nothing for same-path collisions, which is the case that actually causes unmergeable conflicts | High (recurring manual merge archaeology) | None |
| UUID filenames permanently (never finalize to short codes) | Maximal collision-proofing; simplest sync logic | Loses the whole point of METIS-A-0006 — short codes stop being the thing people actually reference in conversation, commits, and cross-references | Medium (regresses readability) | Low |
| Lazy short code via provisional uid path, finalized at sync (chosen) | Keeps short codes as the durable, referenced identifier; makes unrelated same-branch-window creates merge without conflict; reuses existing renumber/collision machinery | Adds a `uid` field and a finalize-at-sync pass; a short code is not knowable until after the next sync (mitigated by auto-sync-on-create) | Medium | Medium |
| Central/locked counter service (e.g. require network round-trip to allocate a number) | Prevents duplicate numbers outright | Breaks the offline-first, git-native design goal that is the whole reason Metis works without a server | High (violates a core design goal) | High |

## Rationale **[REQUIRED]**

Lazy finalization is the only option that fixes the actual failure (a git-level path conflict that happens before Metis code runs) without abandoning either of Metis's two governing constraints: offline/git-native operation (rules out a locked central counter) and short codes as the readable, citable identifier (rules out permanent UUID filenames). It also does not introduce new machinery — `resolve_short_code_collisions` already contains the rename/renumber/cross-reference-update logic this needs; finalization is a second caller of the same primitives, not a new subsystem. And because every creation path already triggers an immediate sync, the change is invisible in the common single-branch case, which is most of Metis's actual usage.

## Consequences **[REQUIRED]**

### Positive
- Two branches creating unrelated documents while diverged no longer produce a git merge conflict on the document's file path — the path is unique (uid-derived) before either branch's short code is decided.
- Short code assignment order becomes deterministic and meaningful (`created_at` order on the merged tree) instead of "whichever branch's counter got there first."
- Reuses existing renumber/rename/cross-reference-update code paths rather than adding a parallel mechanism.
- No change to the short code format itself, so every existing reference, document, and integration built against `PREFIX-TYPE-NNNN` keeps working unchanged.

### Negative
- `DocumentMetadata` and all five document types (vision/initiative/task/adr/specification) need their frontmatter (de)serialization updated to handle `short_code: null` and a new `uid` field — a real, if mechanical, migration across the domain layer.
- A document's short code is technically unknown between `metis create`'s file-write and its auto-sync call, a window that did not exist before. Any tooling that reads a just-created file before sync completes needs to tolerate `short_code: null`.
- Collaborators on an old binary who hit a `_pending-*.md` file created by a colleague on the new binary will see a per-document sync error until they upgrade (see Migration path above) — an acceptable, contained failure mode rather than a blocker, but still a rollout cost across the multiple repos already using Metis.

### Neutral
- `update_sibling_references` (`crates/metis-docs-core/src/application/services/synchronization.rs:488`) currently scans only the renumbered file's parent directory (literally same-folder siblings — e.g. all ADRs, or all tasks under one initiative), not the workspace tree. The transcript that motivated this decision needed 3 cross-reference fixes outside that scope (a task cited from a different initiative's plan). This decision broadens it to a tree-wide sweep — `find_markdown_files` runs against the workspace root instead of the parent directory — so both collision-resolution renumbering and the new finalize-at-sync step correctly update every reference, not just same-directory ones. This changes the existing, tested scope of collision resolution too; `collision_resolution_test.rs`'s `test_sibling_cross_reference_update` will need updating to assert the wider scope.

## Review Schedule **[CONDITIONAL: Temporary Decision]**

### Review Triggers
- The `_pending-*` provisional naming scheme proves confusing or leaks into user-visible surfaces (GUI, CLI output) in ways that need smoothing over.
- The automatic sync-time `uid` backfill for pre-existing documents turns out to be unsafe or lossy for any document type.
- Real-world use surfaces a merge scenario this design does not cover (e.g. three-way merges, cherry-picks across many diverged branches).

### Scheduled Review
- **Next Review Date**: 2026-12-21
- **Review Criteria**: Whether merge conflicts on document creation have actually stopped occurring in practice; whether the `uid` backfill completed cleanly across existing projects.
- **Sunset Date**: N/A — this is a foundational fix to the identification system established in METIS-A-0006.