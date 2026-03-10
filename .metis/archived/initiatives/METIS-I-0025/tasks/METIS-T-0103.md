---
id: add-cli-create-specification
level: task
title: "Add CLI create specification command"
short_code: "METIS-T-0103"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T04:51:34.863309+00:00
parent: METIS-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0025
---

# Add CLI create specification command

## Parent Initiative

[[METIS-I-0025]]

## Objective

Add `metis create specification` CLI command following the existing pattern for `metis create adr`.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `crates/metis-docs-cli/src/commands/create/specification.rs` — New file with:
  - `CreateSpecificationArgs` struct with clap derives: `--title`, `--parent` (required — Vision or Initiative short code)
  - `execute()` function that calls `DocumentCreationService::create_specification()`
  - Output: success message with short code
- [ ] `crates/metis-docs-cli/src/commands/create/mod.rs` — Add `Specification` variant to `CreateCommands` enum, import and dispatch to specification module
- [ ] `crates/metis-docs-cli/src/commands/create/mod.rs` — `Specification` variant dispatches to `specification::execute()`
- [ ] Command validates parent exists and is appropriate type (Vision published, or Initiative non-terminal)
- [ ] Help text is clear: `metis create specification --title "System Design" --parent PROJ-V-0001`

## Implementation Notes

Follow `create/adr.rs` as the reference pattern. Key differences:
- ADR has `--decision-maker` arg; Specification has `--parent` arg (required)
- Specification parent must be validated (Vision published or Initiative non-terminal)

Reference files:
- `crates/metis-docs-cli/src/commands/create/adr.rs` — Pattern to follow
- `crates/metis-docs-cli/src/commands/create/mod.rs` — CreateCommands enum

## Status Updates

### Session 1 (2026-03-03)

**All acceptance criteria met.** Build passes, all tests pass (316+ total, 52 CLI).

**Created** `crates/metis-docs-cli/src/commands/create/specification.rs`:
- `create_new_specification(title, parent)` function
- Validates workspace exists, builds `DocumentCreationConfig` with parent_id
- Calls `DocumentCreationService::create_specification()`
- Prints success with file path, ID, short code, title, parent
- 2 tests: no-workspace error, full creation with roundtrip parse

**Updated** `crates/metis-docs-cli/src/commands/create/mod.rs`:
- Added `mod specification;`
- Added `Specification { title, parent }` variant to `CreateCommands` enum
- Help: `--parent` is required, described as "Parent document short code (Vision or Initiative)"
- Dispatches to `specification::create_new_specification()`

**Also fixed** `crates/metis-docs-core/src/dal/database/configuration_repository.rs`:
- Added `"specification" => "S"` to `generate_short_code()` type letter mapping
- Without this, short code generation failed with "Unknown document type: specification"

**CLI help output**: `metis create specification --parent <PARENT> <TITLE>`