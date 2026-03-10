---
id: add-and-fix-tests-across-all-crates
level: task
title: "Add and fix tests across all crates"
short_code: "METIS-T-0105"
created_at: 2026-03-03T21:00:00+00:00
updated_at: 2026-03-04T13:10:17.225550+00:00
parent: METIS-I-0025
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0025
---

# Add and fix tests across all crates

## Parent Initiative

[[METIS-I-0025]]

## Objective

Add tests for the new Specification document type across all crates and ensure all existing tests still pass.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] **Core crate inline tests**:
  - `specification/mod.rs` — Tests for Specification struct creation, phase sequence, `from_file()`, `to_markdown()` roundtrip
  - `types.rs` — Tests for `DocumentType::Specification` variant, `Phase::Drafting` transitions, specification phase sequence
  - `factory.rs` — Test for Specification factory extraction from file
- [ ] **Core integration tests**:
  - Test specification creation with Vision parent (published)
  - Test specification creation with Initiative parent (non-terminal)
  - Test specification creation fails with invalid parent (unpublished Vision, completed Initiative)
  - Test specification phase transitions: Discovery → Drafting → Review → Published
  - Test specification archive (no cascade)
- [ ] **CLI tests**:
  - Test `create specification` command with valid parent
  - Test `create specification` command with missing parent (should fail)
- [ ] **MCP tests**:
  - Test CreateDocumentTool with `document_type = "specification"`
  - Test specification appears in ListDocumentsTool output
  - Test specification phase transitions via TransitionPhaseTool
- [ ] `angreal test` passes with zero failures across all crates

## Implementation Notes

This is the "make it green" task. Run `angreal test` iteratively until clean. Focus on both new tests for specification functionality AND ensuring existing tests aren't broken by the additions.

Reference test files:
- `crates/metis-docs-core/src/domain/documents/adr/mod.rs` — ADR inline test pattern
- `crates/metis-docs-core/tests/` — Integration test patterns
- `crates/metis-docs-cli/tests/comprehensive_functional_test.rs` — CLI functional tests
- `crates/metis-docs-mcp/tests/` — MCP functional tests

## Status Updates

### Session 2
- Root cause: `FlightLevelConfig::enabled_document_types()` didn't include Specification → MCP `test_specification_workflow` failed
- Fixed `configuration.rs`: added `DocumentType::Specification` to always-available types (like ADR)
- Updated unit test assertions for streamlined and direct configs
- **All tests pass**: core (166 unit + 5 integration), CLI (52), MCP (5 unit + 12 integration)
- `angreal test` exits cleanly: "All crate tests completed successfully!"

### Session 1
- Created `crates/metis-docs-core/tests/specification_test.rs` (5 integration tests)
- Updated `crates/metis-docs-mcp/tests/comprehensive_functional_test.rs` (fixed regex, added spec workflow test)
- Fixed 3 bugs: discovery path (flat→directory), discovery scanning (subdirs), config repo short code gen