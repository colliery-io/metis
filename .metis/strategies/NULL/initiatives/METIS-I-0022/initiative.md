---
id: workspace-roles-central-folder
level: initiative
title: "Workspace Roles & Central Folder Architecture"
short_code: "METIS-I-0022"
created_at: 2026-03-01T01:08:04.053025+00:00
updated_at: 2026-03-01T01:08:04.053025+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: workspace-roles-central-folder
---

# Workspace Roles & Central Folder Architecture Initiative

*This template includes sections for various types of initiatives. Delete sections that don't apply to your specific use case.*

## Context

In Flight Levels methodology, strategies (FL3) and initiatives (FL2) are **central, cross-team concerns** — not owned by individual delivery workspaces. Currently, all documents are pushed to `{workspace_prefix}/` in the central repo, making strategies local to whichever workspace creates them. This contradicts the Flight Levels model where:

- **Strategies** are owned by a central **strategy group** (FL3)
- **Initiatives** are owned by an **initiative working group** (FL2)
- **Tasks** are local to **delivery teams** (FL1)
- Delivery teams can also have their own **local initiatives**
- A single group can hold **multiple roles** (e.g., both strategy_group and initiative_group)

## Goals & Non-Goals

**Goals:**
- Add workspace roles (`strategy_group`, `initiative_group`, `delivery`) to config.toml
- Route strategies to shared `strategies/` folder in central repo during dehydration
- Route shared initiatives to `initiatives/` folder in central repo during dehydration
- All workspaces hydrate `strategies/` and `initiatives/` as read-only
- Role holders skip hydrating the central folders they own
- Backward compatible: existing workspaces default to `delivery` role and work unchanged

**Non-Goals:**
- GUI changes (separate follow-up)
- MCP tool changes for role-aware document creation (separate follow-up)
- Write-protection enforcement (trust-based for now — role determines routing, not access control)

## Central Repo Layout (Target)

```
central.git/
├── strategies/         # Shared strategies (written by strategy_group role)
│   ├── PROJ-S-0001.md
│   └── PROJ-S-0002.md
├── initiatives/        # Shared initiatives (written by initiative_group role)
│   ├── PROJ-I-0001.md
│   └── PROJ-I-0002.md
├── api/                # Delivery workspace "api"
│   ├── API-V-0001.md
│   ├── API-I-0003.md   # Local initiative (delivery team's own)
│   └── API-T-0001.md
└── sre/                # Delivery workspace "sre"
    ├── SRE-V-0001.md
    └── SRE-T-0001.md
```

## Document Routing Rules

| Document Type | Role Required | Central Path |
|---------------|---------------|--------------|
| Strategy | `strategy_group` | `strategies/{filename}` |
| Initiative | `initiative_group` | `initiatives/{filename}` |
| Initiative (no role) | `delivery` | `{prefix}/{filename}` (local initiative) |
| All others | any | `{prefix}/{filename}` |

## Architecture

### WorkspaceRole Enum

```rust
enum WorkspaceRole {
    StrategyGroup,    // Can write to strategies/
    InitiativeGroup,  // Can write to initiatives/
    Delivery,         // Default — writes only to {prefix}/
}
```

Stored in `config.toml` as:
```toml
[workspace]
prefix = "api"
roles = ["delivery"]  # or ["strategy_group", "initiative_group"]
```

Default: `["delivery"]` (backward compatible).

### Dehydration Plan

```rust
struct DehydrationPlan {
    workspace_docs: Vec<FlatDoc>,      // → {prefix}/
    strategy_docs: Vec<FlatDoc>,       // → strategies/
    initiative_docs: Vec<FlatDoc>,     // → initiatives/
}
```

### SyncContext Multi-Prefix Writes

`SyncContext` gains `additional_write_prefixes: Vec<String>` to relax path validation in `commit_update()` — allows writing to `strategies/` and `initiatives/` alongside `{prefix}/` in a single atomic commit.

## Detailed Design

### Files Modified

| File | Change |
|------|--------|
| `crates/metis-docs-core/src/domain/configuration.rs` | `WorkspaceRole` enum, roles on `WorkspaceConfig`, validation |
| `crates/metis-sync/src/lib.rs` | `additional_write_prefixes` on `SyncContext`, relaxed path validation |
| `crates/metis-sync/src/dehydration.rs` | `DehydrationPlan`, `plan_dehydration()`, `dehydrate_multi()` |
| `crates/metis-sync/src/hydration.rs` | Multi-prefix owned skip, central folder hydration |
| `crates/metis-sync/src/orchestration.rs` | Role-aware `sync()` wiring |
| `crates/metis-sync/src/projection.rs` | Add "initiatives" to reserved names, role-aware owned flag |
| `crates/metis-docs-cli/src/commands/init.rs` | `--roles` CLI argument |

## Testing Strategy

- Unit tests for role routing, multi-prefix path validation, owned-prefix skip logic
- Integration test: strategy_group workspace pushes to `strategies/`, delivery workspace hydrates from `strategies/`
- Integration test: two workspaces with different roles both sync to same central repo
- Verify existing single-workspace and streamlined configs work unchanged (default `delivery` role)
- `angreal test` — all existing tests pass (backward compatible)

## Alternatives Considered

- **Per-document routing tags**: Instead of workspace roles, tag individual documents for central routing. Rejected: too granular, doesn't match Flight Levels ownership model.
- **Separate repos for each flight level**: Strategies in one repo, initiatives in another. Rejected: over-engineered, complicates sync.
- **Single flat namespace**: All documents in a single folder with metadata-based ownership. Rejected: loses the clear folder-based ownership semantics.

## Implementation Plan

1. `WorkspaceRole` enum + roles on `WorkspaceConfig` (configuration.rs)
2. `SyncContext` multi-prefix write support (lib.rs)
3. `DehydrationPlan` + `dehydrate_multi()` (dehydration.rs)
4. Role-aware hydration with multi-prefix skip (hydration.rs)
5. Wire roles into orchestration (orchestration.rs)
6. Update projection cache for central folders (projection.rs)
7. `--roles` CLI argument for init (init.rs)
8. `angreal test` — verify all tests pass