---
id: fix-tests-across-all-crates
level: task
title: "Fix tests across all crates"
short_code: "METIS-T-0098"
created_at: 2026-03-03T19:10:53.693741+00:00
updated_at: 2026-03-04T01:18:23.852782+00:00
parent: METIS-I-0024
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: METIS-I-0024
---

# Fix tests across all crates

## Parent Initiative

[[METIS-I-0024]]

## Objective

Update or remove all tests that reference strategy across all three crates. Ensure `angreal test` passes clean.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Inline tests in `strategy/mod.rs` deleted (module is gone)
- [ ] Inline tests in `configuration.rs` updated — no strategy preset tests, no strategy enablement tests
- [ ] Inline tests in `types.rs` updated — strategy transition tests removed, `Phase::Shaping` tests removed
- [ ] Inline tests in `factory.rs` updated — strategy extraction test removed
- [ ] `crates/metis-docs-core/tests/reassignment_test.rs` — strategy references removed
- [ ] `crates/metis-docs-core/tests/collision_resolution_test.rs` — strategy references removed
- [ ] `crates/metis-docs-core/tests/database_reconstruction_test.rs` — strategy references removed
- [ ] `crates/metis-docs-core/tests/id_path_consistency_test.rs` — strategy references removed
- [ ] `crates/metis-docs-cli/tests/comprehensive_functional_test.rs` — strategy test cases removed
- [ ] `crates/metis-docs-mcp/tests/functional_test.rs` — strategy references removed
- [ ] `crates/metis-docs-mcp/tests/comprehensive_functional_test.rs` — strategy references removed
- [ ] `crates/metis-docs-mcp/tests/configuration_scenarios_test.rs` — strategy config scenarios removed/updated
- [ ] `angreal test` passes with zero failures

## Implementation Notes

This is the "make it green" task. Depends on all other tasks being complete. Run `angreal test` iteratively until clean. Some tests will need strategy logic replaced with the new flat layout, not just deleted.

## Status Updates

### Session 2 (2026-03-03)

**All 308 tests pass with zero failures across all crates.**

Files fixed:
- `crates/metis-docs-cli/tests/comprehensive_functional_test.rs` — removed `strategies: None`, rewrote "full" preset test to "direct"
- `crates/metis-docs-cli/src/commands/create/initiative.rs` — made `find_vision` async with filesystem fallback (init doesn't sync vision to DB), fixed CWD race conditions in tests
- `crates/metis-docs-mcp/tests/configuration_scenarios_test.rs` — complete rewrite removing all strategy tests
- `crates/metis-docs-mcp/tests/functional_test.rs` — complete rewrite replacing strategy creation with initiative/ADR
- `crates/metis-docs-mcp/tests/comprehensive_functional_test.rs` — removed full config test, removed risk_level fields
- `crates/metis-docs-mcp/tests/mcp_archive_test.rs` — rewrote to test Vision→Initiative→Task cascade instead of strategy cascade

Root causes:
1. Strategy document type removed but test files still referenced it (risk_level, FlightLevelConfig::full, strategy creation, strategies_enabled)
2. `find_vision` only checked DB but init doesn't sync vision.md to documents table — added filesystem fallback
3. CWD race conditions from parallel tests using `set_current_dir` — added graceful handling