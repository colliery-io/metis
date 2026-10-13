---
id: lazy-short-code-finalization-core
level: initiative
title: "Lazy Short Code Finalization (core, CLI, MCP)"
short_code: "METIS-I-0030"
created_at: 2026-09-21T20:42:46.739660+00:00
updated_at: 2026-09-21T20:42:46.739660+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: M
initiative_id: lazy-short-code-finalization-core
---

# Lazy Short Code Finalization (core, CLI, MCP) Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context **[REQUIRED]**

METIS-A-0008 decided that Metis will separate a document's durable identity from its display short code: creation assigns a permanent `uid` and defers `short_code` assignment to sync time, so two branches creating unrelated documents while diverged no longer collide on the same file path during a git merge. This initiative plans the implementation of that decision.

Scope, per the ADR's Implementation scope section: `metis-docs-core`, the CLI, and the MCP tools, in one pass. The Tauri GUI is explicitly deferred to a follow-up initiative — its `create_document` command and TypeScript types assume a short code exists immediately after creation, and that's a separate design question from the core mechanism.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Implement the `uid` field and optional `short_code` across `DocumentMetadata` and all five document types' frontmatter (de)serialization.
- Change `DocumentCreationService` to write pending documents (`short_code: null`, filename derived from the full `uid`) instead of eagerly generating a short code.
- Add a finalize-at-sync step to `SyncService::sync_directory` that assigns real short codes to pending documents in `created_at` order, reusing the existing `renumber_document` machinery.
- Broaden `update_sibling_references` to a tree-wide reference sweep, per METIS-A-0008's Neutral consequence, and update `collision_resolution_test.rs` accordingly.
- Add the automatic sync-time `uid` backfill for documents that already have a short code but no `uid`.
- Verify end-to-end that two branches creating unrelated documents while diverged merge without a git conflict, and that `metis sync` finalizes both correctly afterward.

**Non-Goals:**
- Tauri GUI changes (`crates/metis-docs-gui`) — tracked separately, per the ADR's deferred scope.
- Any change to the short code format (`PREFIX-TYPE-NNNN`) itself — METIS-A-0006 stands.
- A schema/version gate for old-binary compatibility — METIS-A-0008 explicitly relies on existing per-file error isolation in `sync_directory` instead.

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

### Overview
Three layers change, all inside `metis-docs-core`, consumed as-is by the CLI and MCP tool:

1. **Domain** (`crates/metis-docs-core/src/domain/documents/metadata.rs` and each of `vision/`, `initiative/`, `task/`, `adr/`, `specification/mod.rs`): `DocumentMetadata` gains `uid: Uuid`, and `short_code` becomes `Option<String>`.
2. **Creation** (`crates/metis-docs-core/src/application/services/document/creation.rs`): `DocumentCreationService` stops calling `generate_short_code` synchronously; it generates a `uid`, writes `short_code: null`, and derives the filename from the full `uid` instead of a short code.
3. **Sync** (`crates/metis-docs-core/src/application/services/synchronization.rs`): `sync_directory` gains a finalize-pending step (find `short_code: null`, order by `created_at`, assign + rename via the same path `renumber_document` already uses), the `uid` backfill for pre-existing documents, and `update_sibling_references` becomes tree-wide.

The CLI (`crates/metis-docs-cli`) and MCP tools (`crates/metis-docs-mcp`) call into `Application::sync_directory`/`DocumentCreationService` already, so no changes are anticipated there beyond whatever surfaces during implementation (e.g. any place that assumes `CreationResult.short_code` is non-optional).

## Detailed Design **[REQUIRED]**

- `DocumentMetadata::new` generates a `uid` (UUID v4) and takes `short_code: Option<String>`; `from_frontmatter` parses `short_code: null` as `None` and a missing `uid` field (pre-existing documents) as a signal for the sync-time backfill to fill in.
- `DocumentCreationService::create_*` methods stop calling `self.generate_short_code(doc_type)` and instead call a new `uid`-generation step; the file path changes from `{short_code}.md` to `_pending-{uid}.md` (full UUID v4, per METIS-A-0008's pending-filename decision).
- `SyncService::sync_directory` gains a step before the existing `resolve_short_code_collisions` (or after — sequencing needs to be decided during implementation so a pending document that also happens to collide, e.g. two documents both missing a short code with an unrelated existing collision elsewhere, resolves correctly): scan for `short_code: null`, sort by `created_at`, and for each call `generate_short_code` + rename + update references — the same three steps `renumber_document` already performs, refactored so both callers (collision resolution and finalization) share the implementation.
- `update_sibling_references` changes its `find_markdown_files` call from the renumbered file's parent directory to the workspace root, becoming a tree-wide sweep. `collision_resolution_test.rs::test_sibling_cross_reference_update` needs updating to assert cross-directory references are also caught.
- The `uid` backfill reuses the shape of `recover_counters_from_filesystem`: scan every markdown file during `sync_directory`, and for any document with a short code but no `uid`, assign one and write it back.

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

### Unit Testing
- **Strategy**: Cover `DocumentMetadata`'s new optional `short_code` (de)serialization for each of the five document types, and `uid` generation/persistence.
- **Tools**: existing Rust `#[test]`/`#[tokio::test]` conventions already used throughout `metis-docs-core`.

### Integration Testing
- **Strategy**: Extend `collision_resolution_test.rs` for the broadened tree-wide reference sweep; add a new test exercising the full finalize-at-sync flow (create two pending documents out of `created_at` order, run sync, assert codes assigned in creation order); add a backfill test (short-coded document with no `uid`, run sync, assert `uid` appears and is stable across repeated syncs).
- **Test Environment**: `tempfile::TempDir`-based workspaces, matching existing test setup in `synchronization.rs` and `id_path_consistency_test.rs`.

### Test Selection
- Any test currently asserting eager short-code assignment at creation time (e.g. in `id_path_consistency_test.rs`, `specification_test.rs`) needs review — those assertions describe the exact behavior this initiative changes.

## Alternatives Considered **[REQUIRED]**

Covered in full in METIS-A-0008's Alternatives Analysis (status quo, permanent UUID filenames, a centralized/locked counter service) — not repeated here. This initiative implements the option METIS-A-0008 decided on; it does not revisit that choice.

## Implementation Plan **[REQUIRED]**

Not yet decomposed into tasks — decomposition requires a separate check-in per this project's process. Proposed sequencing for that decomposition (work titles, not yet created):

1. "Add uid and optional short_code to DocumentMetadata and all five document types" (domain layer; foundation for everything else).
2. "Change DocumentCreationService to write pending documents instead of eager short codes" (creation layer; depends on 1).
3. "Add finalize-at-sync step for pending documents, ordered by created_at" (sync layer; depends on 1).
4. "Broaden update_sibling_references to a tree-wide sweep and update collision_resolution_test.rs" (can proceed in parallel with 2–3; touches the same file as 3, so sequencing with it needs care).
5. "Add automatic uid backfill for pre-existing documents during sync" (depends on 1 and 3, since it shares the finalize-at-sync pass).
6. "End-to-end verification: two branches creating documents while diverged merge without conflict and finalize correctly" (depends on all of the above).