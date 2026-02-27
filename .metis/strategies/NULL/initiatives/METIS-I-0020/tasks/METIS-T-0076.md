---
id: config-toml-multi-workspace-schema
level: task
title: "config.toml multi-workspace schema"
short_code: "METIS-T-0076"
created_at: 2026-02-26T01:32:04.301299+00:00
updated_at: 2026-02-26T02:03:06.628564+00:00
parent: METIS-I-0020
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: METIS-I-0020
---

# config.toml multi-workspace schema

## Objective

Extend `config.toml` to support multi-workspace sync. Define and implement the schema for workspace identity, upstream configuration, and sync state.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `config.toml` supports `upstream_url` field (central repo URL, optional — absent means single-workspace mode)
- [ ] `config.toml` supports `workspace_prefix` field (this workspace's owned prefix in central, e.g. "api")
- [ ] `config.toml` supports `team_label` field (grouping key for multi-workspace views, e.g. "platform")
- [ ] `config.toml` supports `last_synced_commit` field (SHA of last successful sync, updated by sync engine)
- [ ] Configuration parser reads and writes all new fields
- [ ] Existing single-workspace configs remain valid (new fields are optional)
- [ ] Validation: `workspace_prefix` must be lowercase alphanumeric + hyphens, 2-20 chars
- [ ] Validation: `upstream_url` must be a valid git remote URL (SSH or HTTPS)

## Implementation Notes

### Schema

```toml
[workspace]
prefix = "api"          # owned folder name in central — also determines short code prefix
team = "platform"       # grouping label for multi-workspace views

[sync]
upstream_url = "git@github.com:org/metis-central.git"
last_synced_commit = "abc123..."   # updated after each successful sync
```

### Technical Approach

- Extend the existing `Configuration` struct in `metis-docs-core/src/domain/configuration.rs`
- New fields are `Option<T>` — absent means single-workspace mode (backward compatible)
- `workspace_prefix` is immutable after first sync (it's an identifier, not an identity)
- Add serialization/deserialization for the new TOML sections

### Dependencies

- None — this is foundation work

## Test Scenarios

### Unit Tests — Schema Parsing

1. **Parse complete config**: config.toml with all fields populated (`prefix`, `team`, `upstream_url`, `last_synced_commit`) → all fields parsed correctly
2. **Parse minimal config**: config.toml with no `[workspace]` or `[sync]` sections → all new fields are `None`, existing behavior unchanged
3. **Parse partial config — workspace only**: `[workspace]` present with `prefix` but no `[sync]` section → workspace fields populated, sync fields `None`
4. **Parse partial config — sync only**: `[sync]` present but no `[workspace]` → sync fields populated, workspace fields `None`
5. **Roundtrip serialization**: parse → serialize → parse produces identical config (no field loss, no reordering that breaks semantics)
6. **Existing config preservation**: config with existing fields (preset, strategies_enabled, etc.) plus new fields → all fields preserved after read/write cycle
7. **Unknown fields ignored**: config with extra fields not in the schema → parsed without error, unknown fields preserved on write (forward compatibility)

### Unit Tests — Validation

8. **Valid prefix — simple**: `"api"` → accepted
9. **Valid prefix — with hyphens**: `"api-team"` → accepted
10. **Valid prefix — min length**: `"ab"` → accepted (2 chars)
11. **Valid prefix — max length**: `"abcdefghijklmnopqrst"` → accepted (20 chars)
12. **Invalid prefix — too short**: `"a"` → rejected with clear error
13. **Invalid prefix — too long**: 21+ characters → rejected
14. **Invalid prefix — uppercase**: `"API"` → rejected (must be lowercase)
15. **Invalid prefix — spaces**: `"api team"` → rejected
16. **Invalid prefix — special chars**: `"api_team"`, `"api.team"`, `"api/team"` → all rejected (only hyphens allowed)
17. **Invalid prefix — starts with hyphen**: `"-api"` → rejected
18. **Invalid prefix — empty string**: `""` → rejected
19. **Valid upstream URL — SSH**: `"git@github.com:org/repo.git"` → accepted
20. **Valid upstream URL — HTTPS**: `"https://github.com/org/repo.git"` → accepted
21. **Invalid upstream URL — empty**: `""` → rejected
22. **Invalid upstream URL — not a git URL**: `"not-a-url"` → rejected with helpful error
23. **Valid last_synced_commit**: 40-char hex SHA → accepted
24. **Invalid last_synced_commit**: non-hex string → rejected

### Integration Tests — Backward Compatibility

25. **Existing project upgrade**: initialize a project with current Metis (no sync fields), then read config with the new parser → no errors, all sync fields `None`, project works normally
26. **Single-workspace mode preserved**: config with no `upstream_url` → all sync operations are no-ops, existing document operations unchanged
27. **Config migration idempotent**: read → write → read a legacy config → no fields lost, no fields added that weren't there

### Edge Cases

28. **Concurrent config writes**: two processes write config.toml simultaneously → file not corrupted (use atomic write or file lock)
29. **Config file missing**: `.metis/config.toml` doesn't exist → graceful error, not a panic
30. **Config file empty**: 0 bytes → parsed as empty config, all fields None/default
31. **Config file with BOM**: UTF-8 BOM at start → parsed correctly or clear error
32. **Very long upstream URL**: 500+ character URL → accepted if structurally valid

## Status Updates

### Implementation Complete

**File modified**: `crates/metis-docs-core/src/domain/configuration.rs`

**What was added**:
- `WorkspaceConfig` struct — workspace identity (prefix + optional team label)
- `SyncConfig` struct — upstream URL + optional last_synced_commit SHA
- Both added as `Option<T>` fields on `ConfigFile` with `#[serde(skip_serializing_if)]`
- Validation functions: `validate_workspace_prefix()`, `validate_upstream_url()`, `validate_commit_sha()`
- Accessor methods on `ConfigFile`: `workspace()`, `sync_config()`, `workspace_prefix()`, `upstream_url()`, `last_synced_commit()`, `is_multi_workspace()`
- Mutation methods: `set_workspace()`, `set_sync()`, `update_last_synced_commit()`

**Test results**: 57 unit tests pass (up from 14), zero regressions across entire workspace.

**Backward compatibility verified**:
- Legacy config.toml (no workspace/sync sections) parses without error
- Save → load roundtrip of legacy config does NOT add new sections
- `ConfigFile::new()` signature unchanged — all existing callers compile
- All existing MCP, CLI, and core tests pass unchanged