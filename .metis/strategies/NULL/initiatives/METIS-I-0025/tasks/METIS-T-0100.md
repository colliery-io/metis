---
id: add-specification-domain-model
level: task
title: "Add Specification domain model"
short_code: "METIS-T-0100"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T03:49:26.962560+00:00
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

# Add Specification domain model

## Parent Initiative

[[METIS-I-0025]]

## Objective

Add the Specification document type to the domain model in `metis-docs-core`. This is the foundation that all other tasks build on.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/metis-docs-core/src/domain/documents/specification/mod.rs` — New `Specification` struct with:
  - Fields mirroring ADR pattern: `id`, `metadata`, `content`, `phases`
  - `SpecificationPhase` enum: `Discovery`, `Drafting`, `Review`, `Published`
  - `Document` trait implementation (`id()`, `metadata()`, `content()`, `phases()`, `from_file()`, `to_markdown()`)
  - Phase sequence: `[Discovery, Drafting, Review, Published]`
  - Parent validation: must be Vision (published) or Initiative (non-terminal phase)
- [ ] `crates/metis-docs-core/src/domain/documents/types.rs` — Add `DocumentType::Specification` variant, `Phase::Discovery` (already exists for Initiative), `Phase::Drafting`, map specification phases in `phase_sequence()`
- [ ] `crates/metis-docs-core/src/domain/documents/factory.rs` — Add `Specification` match arm in `DocumentFactory::from_file()` and `from_content()`
- [ ] `crates/metis-docs-core/src/domain/documents/traits.rs` — No changes needed (trait is generic)
- [ ] `crates/metis-docs-core/src/domain/documents/mod.rs` — Add `pub mod specification;`
- [ ] `crates/metis-docs-core/src/lib.rs` — Export `Specification` type
- [ ] `crates/metis-docs-core/src/constants.rs` — Add `SPECIFICATION_DIR`, `SPECIFICATION_TEMPLATE`, phase constants (`SPECIFICATION_DISCOVERY`, `SPECIFICATION_DRAFTING`, `SPECIFICATION_REVIEW`, `SPECIFICATION_PUBLISHED`)

## Implementation Notes

Follow the ADR pattern closely — ADR is an attached document type (not a hierarchy node), which matches Specification's design. Key differences from ADR:
- ADR has phases: Draft → Discussion → Decided → Superseded
- Specification has phases: Discovery → Drafting → Review → Published
- ADR has no parent requirement; Specification requires Vision or Initiative parent
- Specification uses `S` short code prefix

Reference files:
- `crates/metis-docs-core/src/domain/documents/adr/mod.rs` — Pattern to follow
- `crates/metis-docs-core/src/domain/documents/types.rs` — DocumentType enum, Phase enum
- `crates/metis-docs-core/src/domain/documents/factory.rs` — Factory match arms
- `crates/metis-docs-core/src/constants.rs` — Constants pattern

Note: `Phase::Discovery` already exists (used by Initiative). Check if `Phase::Drafting` needs to be added or if an existing phase can be reused. `Phase::Review` also already exists (used by Vision). `Phase::Published` already exists (used by Vision). So only `Phase::Drafting` is new.

## Status Updates

### Session 1 (2026-03-03)

**All acceptance criteria met. Domain model complete, all tests pass.**

Files created:
- `crates/metis-docs-core/src/domain/documents/specification/mod.rs` — Specification struct, Document trait impl, phase sequence (Discovery→Drafting→Review→Published), from_content/from_file/to_content/to_file, 9 inline tests
- `crates/metis-docs-core/src/domain/documents/specification/frontmatter.yaml` — Tera frontmatter template
- `crates/metis-docs-core/src/domain/documents/specification/content.md` — Content template with Overview, System Context, Requirements (REQ-x.x.x/NFR-x.x.x), Architecture Framing, Decision Log, Constraints, Changelog sections
- `crates/metis-docs-core/src/domain/documents/specification/acceptance_criteria.md` — Default acceptance criteria

Files modified:
- `crates/metis-docs-core/src/domain/documents/types.rs` — Added `DocumentType::Specification`, `Phase::Drafting`, specification phase transitions, phase_sequence, tag parsing for "drafting"
- `crates/metis-docs-core/src/domain/documents/factory.rs` — Added Specification match arms in from_file and from_content
- `crates/metis-docs-core/src/domain/documents/mod.rs` — Added `pub mod specification;`
- `crates/metis-docs-core/src/lib.rs` — Exported `Specification` type
- `crates/metis-docs-core/src/constants.rs` — Added SPECIFICATION_DIR, SPECIFICATION_TEMPLATE, phase constants
- `crates/metis-docs-core/src/domain/configuration.rs` — Added Specification to is_document_type_allowed (always allowed) and get_parent_type (None — attached doc)
- `crates/metis-docs-core/src/application/services/document/discovery.rs` — Added Specification to find_document_of_type, find_all_documents_of_type, document_type_from_short_code ("S"), construct_path_from_short_code
- `crates/metis-docs-core/src/application/services/document/validation.rs` — Added Specification match arm
- `crates/metis-docs-core/src/application/services/workspace/archive.rs` — Added Specification to mark_as_archived_helper, archive_document, get_document_id, determine_document_type
- `crates/metis-docs-core/src/application/services/workspace/transition.rs` — Added Specification to get_current_phase and perform_transition
- `crates/metis-docs-mcp/src/tools/create_document.rs` — Added stub returning error (full MCP support is T-0104)

All 308+ tests pass across all crates.