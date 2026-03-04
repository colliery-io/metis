---
id: simplify-configuration-and-presets
level: task
title: "Simplify configuration and presets"
short_code: "METIS-T-0095"
created_at: 2026-03-03T19:10:50.978361+00:00
updated_at: 2026-03-04T00:53:45.083328+00:00
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

# Simplify configuration and presets

## Parent Initiative

[[METIS-I-0024]]

## Objective

Remove `strategies_enabled` from configuration, collapse the "full" preset (which is now identical to "streamlined"), and ensure existing `config.toml` files with the old field parse without error.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] `strategies_enabled` field removed from `FlightLevelConfig` struct
- [x] `FlightLevelConfig::new()` simplified to take only `initiatives_enabled`
- [x] `full()` deprecated with `#[deprecated]` attribute, aliased to `streamlined()`
- [x] Two presets remain: `streamlined` (Vision→Initiative→Task) and `direct` (Vision→Task)
- [x] `is_document_type_allowed()` no longer checks for Strategy (done in T-0091)
- [x] `get_parent_type()` no longer returns Vision as parent for Strategy (done in T-0091)
- [x] `preset_name()` updated — returns "streamlined" instead of "full"
- [x] `enabled_document_types()` never includes Strategy (done in T-0091)
- [x] `hierarchy_display()` never includes "Strategy" (done in T-0091)
- [x] Existing `config.toml` files with `strategies_enabled = true/false` parse without error (serde ignores unknown fields by default)
- [ ] CLI `metis config set --preset` only offers streamlined/direct (T-0096 scope)
- [ ] CLI `metis config set --strategies` flag removed (T-0096 scope)

## Implementation Notes

The config parser needs to tolerate the old `strategies_enabled` field in existing config files. Either add `#[serde(deny_unknown_fields)]` removal or use `#[serde(default)]` on the field temporarily and silently ignore it. Alternatively, strip unknown keys before parsing.

Consider whether "streamlined" should be renamed now that it's the only full-hierarchy preset, or if that's a separate concern.

## Status Updates

### Session 1 (2026-03-03)
- Removed `strategies_enabled` field from `FlightLevelConfig` struct in `configuration.rs`
- Simplified `FlightLevelConfig::new()` to take only `initiatives_enabled: bool`
- Deprecated `full()` with `#[deprecated]` attribute, aliased to `streamlined()`
- Updated `preset_name()` to return "streamlined" instead of "full"
- Added backward-compat tests for old JSON (`{"strategies_enabled":false,"initiatives_enabled":true}`) and old TOML files
- New TOML format test verifies `strategies_enabled` is NOT written to new config files
- Updated callers in core crate:
  - `creation.rs`: `FlightLevelConfig::full()` → `FlightLevelConfig::streamlined()` (3 occurrences)
  - `configuration_repository.rs`: default JSON updated to `{"initiatives_enabled":true}`
  - `configuration_recovery_test.rs`: rewrote to test streamlined→direct transition (old test was trivially identical after full→streamlined alias)
- All 184 tests pass (157 unit + 27 integration)
- Remaining 2 acceptance criteria (CLI preset/flag changes) are T-0096 scope