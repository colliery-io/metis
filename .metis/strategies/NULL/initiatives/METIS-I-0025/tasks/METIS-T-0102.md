---
id: add-specification-to-core-services
level: task
title: "Add Specification to core services"
short_code: "METIS-T-0102"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T04:08:50.326860+00:00
parent: METIS-I-0025
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0025
---

# Add Specification to core services

## Parent Initiative

[[METIS-I-0025]]

## Objective

Wire Specification into all core service layers — creation, discovery, phase transition, archive, synchronization, and template loading.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Creation service** (`crates/metis-docs-core/src/services/creation.rs`):
  - Add `create_specification()` method following `create_adr()` pattern
  - Parent validation: must be Vision (published) or Initiative (non-terminal phase)
  - Filesystem: creates under `.metis/specifications/{SHORT_CODE}/specification.md`
  - Database: inserts with `level = 'specification'`
- [ ] **Discovery service** (`crates/metis-docs-core/src/services/discovery.rs`):
  - Add `DocumentType::Specification` to `find_document_of_type()` match arms
  - Add `"S"` → `DocumentType::Specification` mapping in `document_type_from_short_code()`
  - Add specification path logic to discovery scanning
  - Handle `specifications/` directory in filesystem traversal
- [ ] **Phase transition service** (`crates/metis-docs-core/src/services/transition.rs`):
  - Add `DocumentType::Specification` match arm in `get_current_phase()` and `perform_transition()`
  - Phase sequence: Discovery → Drafting → Review → Published
- [ ] **Archive service** (`crates/metis-docs-core/src/services/archive.rs`):
  - Add `DocumentType::Specification` match arm in archive logic
  - Specification archival should NOT cascade (it has no children — it's an attached document)
- [ ] **Synchronization service** (`crates/metis-docs-core/src/services/synchronization.rs`):
  - Add `"specification"` to `extract_lineage()` level matching
  - Add `specifications/` directory scanning in `sync_directory()`
  - Add counter recovery for `S` short code prefix
- [ ] **Template service** (`crates/metis-docs-core/src/services/template.rs`):
  - Add embedded template defaults for specification (frontmatter, content, postmatter)
  - Add specification to template fallback chain
- [ ] **DocumentCreationConfig** — Add specification variant if needed (check if `complexity` or other fields apply — likely not, similar to ADR)

## Implementation Notes

Follow the ADR pattern for creation (attached document, not hierarchy node). Key differences:
- ADR has no parent requirement; Specification DOES require a parent
- ADR creates under `.metis/adrs/`; Specification creates under `.metis/specifications/`
- Specification discovery needs to scan `specifications/` directory
- Archive does NOT cascade (unlike Initiative which cascades to tasks)

Reference files:
- `crates/metis-docs-core/src/services/creation.rs` — `create_adr()` method
- `crates/metis-docs-core/src/services/discovery.rs` — ADR discovery match arms
- `crates/metis-docs-core/src/services/transition.rs` — ADR transition match arms
- `crates/metis-docs-core/src/services/synchronization.rs` — `extract_lineage()`, directory scanning

## Status Updates

### Session 2 (2026-03-03)

**All acceptance criteria met.** Build passes, all 310+ tests pass.

Most service changes were already done in T-0100 to fix compilation after adding `DocumentType::Specification`. This task completed the remaining gaps:

**Creation service** (`creation.rs`):
- Added `create_specification()` method following `create_adr()` pattern
- Validates parent is provided (Vision or Initiative)
- Creates under `.metis/specifications/{SHORT_CODE}/specification.md`
- Uses `generate_short_code("specification")` for DB-backed counter
- Added `Specification` import

**Discovery service** — Already complete from T-0100:
- `DocumentType::Specification` in `find_document_of_type()`, `find_all_documents_of_type()`
- `"S"` mapping in `document_type_from_short_code()`
- `specifications/` path logic in `construct_path_from_short_code()`

**Phase transition service** — Already complete from T-0100:
- `get_current_phase()` and `perform_transition()` match arms
- Default phase: `Phase::Discovery`

**Archive service** — Already complete from T-0100:
- All match arms added; grouped with Vision/Task/Adr as leaf docs (no cascade)

**Synchronization service** (`synchronization.rs`):
- Added `["specifications", _, "specification.md"]` path pattern to `extract_lineage_from_path()` (no lineage — attached document)
- Added `"S" => "specification"` to counter recovery type letter mapping
- Added `"S"` to `is_valid_short_code_format()` allowed set

**Template service** (`template.rs`):
- Added `defaults::specification` module with `include_str!()` for content and exit criteria
- Added `("specification", TemplateType::Content/ExitCriteria)` to `get_embedded_template()`
- Added `"specification" => 'S'` to `doc_type_letter()`
- Added specification sample context to `sample_context_for_type()`
- Updated `test_load_embedded_templates` to include specification

**DocumentCreationConfig** — No changes needed; existing fields suffice (like ADR, no complexity/risk fields)