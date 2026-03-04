---
id: remove-strategy-domain-model
level: task
title: "Remove Strategy domain model"
short_code: "METIS-T-0091"
created_at: 2026-03-03T19:10:47.392443+00:00
updated_at: 2026-03-03T20:33:53.832111+00:00
parent: METIS-I-0024
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0024
---

# Remove Strategy domain model

## Parent Initiative

[[METIS-I-0024]]

## Objective

Delete the Strategy document type from the domain model. This is the foundational task — everything else depends on the type being gone.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `crates/metis-docs-core/src/domain/documents/strategy/` directory deleted (mod.rs, frontmatter.yaml, content.md)
- [x] `DocumentType::Strategy` variant removed from `types.rs`
- [x] `Phase::Shaping` removed from `types.rs` (strategy-only phase)
- [x] Strategy phase sequence removed from `phase_sequence()`
- [x] Strategy valid transitions removed
- [x] `DocumentFactory` match arms for strategy removed from `factory.rs`
- [x] `strategy_id: Option<DocumentId>` removed from `DocumentCore` in `traits.rs`
- [x] `pub mod strategy;` removed from `documents/mod.rs`
- [x] `strategy::{RiskLevel, Strategy}` exports removed from `lib.rs`
- [x] `RiskLevel` enum removed entirely
- [x] Strategy constants removed from `constants.rs` (`STRATEGY_DIR`, `STRATEGY_TEMPLATE`, phase constants, risk level constants)
- [x] `crates/metis-docs-core/src/templates/strategy/` directory deleted
- [x] Code compiles (other crates will have errors — those are handled by downstream tasks)

## Implementation Notes

Start here. Other tasks will not compile until this is done, but this task itself should focus only on the core crate's domain layer. Expect downstream compilation failures in CLI, MCP, and service layers — those are separate tasks.

## Status Updates

### Session 1 — 2026-03-03

**All acceptance criteria met. Core crate compiles cleanly. All 174 tests pass.**

#### Deleted
- `crates/metis-docs-core/src/domain/documents/strategy/` directory (mod.rs, frontmatter.yaml, content.md, acceptance_criteria.md)
- `crates/metis-docs-core/src/templates/strategy/` directory (content.md, postmatter.md, frontmatter.yaml)

#### Domain Layer Changes
- `documents/mod.rs` — removed `pub mod strategy;`
- `lib.rs` — removed `strategy::{RiskLevel, Strategy}` from public exports
- `traits.rs` — removed `strategy_id: Option<DocumentId>` from `DocumentCore` struct
- `types.rs` — removed `DocumentType::Strategy`, `Phase::Shaping`, all related match arms, transitions, phase sequences, Display/FromStr impls
- `factory.rs` — removed Strategy imports and match arms, updated tests
- `constants.rs` — removed `STRATEGY_DIR`, `STRATEGY_TEMPLATE`, strategy phase constants, entire `risk` module
- `initiative/mod.rs` — removed `strategy_id` param from all constructors/parsers/serializers, updated validation message to reference "Vision" instead of "Strategy", updated all tests
- `task/mod.rs` — removed `strategy_id` param from all constructors/parsers/serializers, updated all tests
- `vision/mod.rs` — removed `strategy_id: None` from DocumentCore construction, removed context insertion
- `adr/mod.rs` — same as vision
- All 4 frontmatter.yaml templates — removed `strategy_id: {{ strategy_id }}` line

#### Services/DAL Layer (same crate, required for compilation)
- `template.rs` — removed strategy embedded defaults, match arms, sample context
- `document/creation.rs` — removed `create_strategy`, `RiskLevel`, `strategy_id` params
- `document/discovery.rs` — removed Strategy references and methods
- `document/validation.rs` — removed Strategy match arms
- `document/deletion.rs` — removed strategy references, rewrote tests
- `workspace/archive.rs` — removed Strategy match arms
- `workspace/transition.rs` — removed Strategy match arms and tests
- `database.rs` — removed `find_by_strategy_id`, `find_strategy_hierarchy` methods
- `synchronization.rs` — simplified `extract_lineage_from_path`, removed strategy path patterns
- `dal/database/models.rs` — removed `strategy_id` from structs
- `dal/database/schema.rs` — removed `strategy_id` column
- `dal/database/repository.rs` — removed strategy query methods
- `domain/configuration.rs` — removed Strategy from match arms
- Integration tests updated

#### Scope Note
Task originally scoped to "domain model only" but services/DAL are in the same crate (`metis-docs-core`). Since acceptance criteria requires "Code compiles", all layers in the crate were updated. This significantly reduces work for METIS-T-0092 (Remove Strategy from services) which may now be mostly complete.

#### Verification
- `cargo check -p metis-docs-core` — compiles cleanly
- `angreal test-core` — 174 tests pass (147 unit + 27 integration)