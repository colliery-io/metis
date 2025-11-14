---
id: gui-project-switching
level: task
title: "GUI Project Switching"
short_code: "METIS-T-0001"
created_at: 2025-10-16T00:49:18.154457+00:00
updated_at: 2025-10-16T20:20:06.885875+00:00
parent: 
blocked_by: []
archived: true

tags:
  - "#task"
  - "#bug"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# GUI Project Switching

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[Parent Initiative]]

## Objective **[REQUIRED]**

Enable multiple developers to work on different branches creating tasks independently without permanent short code collisions. When branches are merged and multiple documents share the same short code, the sync process will automatically detect and resolve collisions using lazy renumbering.

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

### Type
- [ ] Bug - Production issue that needs fixing
- [x] Feature - New functionality or enhancement  
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [x] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: Enable team collaboration without manual ID coordination. Developers can work independently on separate branches without worrying about ID conflicts.
- **Business Value**: Reduces friction in multi-developer workflows. Prevents merge conflicts and database errors that currently block team collaboration.
- **Effort Estimate**: M - Requires database migration, sync logic changes, and reference update logic

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [ ] Database UNIQUE constraint removed from short_code, replaced with non-unique index
- [ ] Sync process detects duplicate short codes across different file paths
- [ ] Collision resolution uses deterministic ordering (alphabetical filepath) to choose which document keeps original short code
- [ ] Renumbered documents have updated frontmatter with new short_code
- [ ] Renumbered documents have updated filenames (e.g., T-0013.md → T-0014.md)
- [ ] Child documents have parent references updated when parent is renumbered
- [ ] Cross-references within sibling documents are updated (same directory group)
- [ ] Counter recovery sets counters to max seen value + 1
- [ ] Collision resolutions are logged/reported to user
- [ ] Integration tests verify multi-branch collision scenarios work correctly



## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Core Insight**: Filepath is already the primary key in the database. Git prevents file path collisions due to hierarchical nesting. Short code collisions can exist temporarily and be resolved lazily during sync.

**Architecture**:
1. **Remove UNIQUE constraint** on `short_code`, make it a non-unique index for performance
2. **Lazy renumbering** during `sync_directory()` - every sync detects and resolves collisions
3. **Deterministic resolution** - alphabetical filepath sorting determines which document keeps original short code
4. **Sibling-scoped reference fixing** - update cross-references only within same directory group using regex

**Implementation Flow**:
```rust
// During sync_directory():
1. Scan all documents from filesystem
2. Group by short_code (HashMap<String, Vec<Document>>)
3. For each collision group:
   a. Sort by filepath (alphabetical)
   b. First keeps original short code
   c. Rest get renumbered sequentially
4. Update counters to max seen + 1
5. For each renumbered document:
   a. Update frontmatter
   b. Rename file
   c. Update children's parent references
   d. Fix cross-references in siblings using regex
```

**Sibling Groups** (for reference fixing):
- Tasks under same initiative: `.metis/strategies/S-X/initiatives/I-Y/tasks/*.md`
- Initiatives under same strategy: `.metis/strategies/S-X/initiatives/*.md`
- All strategies: `.metis/strategies/*/strategy.md`
- All ADRs: `.metis/adrs/*.md`

**Reference Update Regex**:
```rust
let pattern = format!(r"\b{}\b", regex::escape(&old_code));
// Matches whole word only, works in:
// - Frontmatter: blocked_by: [METIS-T-0013]
// - Content: "See METIS-T-0013 for details"
```

### Dependencies

- Database migration to remove UNIQUE constraint
- `SyncService::sync_directory()` in `crates/metis-docs-core/src/application/services/synchronization.rs`
- `ConfigurationRepository::generate_short_code()` in `crates/metis-docs-core/src/dal/database/configuration_repository.rs`
- Frontmatter parsing/updating via `gray_matter` crate
- File renaming operations via `FilesystemService`

### Key Files to Modify

- `crates/metis-docs-core/src/dal/database/migrations/` - New migration to drop UNIQUE constraint
- `crates/metis-docs-core/src/application/services/synchronization.rs` - Collision detection and resolution
- `crates/metis-docs-core/src/application/services/document/update.rs` - Helper for renumbering documents
- `crates/metis-docs-core/tests/` - Integration tests for collision scenarios

### Risk Considerations

**Risk 1: Cross-reference breakage**
- **Issue**: References in markdown content could break after renumbering
- **Mitigation**: Sibling-scoped regex replacement updates most references automatically
- **Residual Risk**: Cross-references outside sibling groups won't auto-update
- **Acceptance**: User manually fixes edge cases; expected to be rare

**Risk 2: Renumbering surprises users**
- **Issue**: Silent renumbering might confuse developers
- **Mitigation**: Log all resolutions clearly; consider adding `_original_short_code` in frontmatter for audit trail
- **Decision**: Log but don't preserve original (keeps it simple)

**Risk 3: Counter gaps**
- **Issue**: After resolution, might have T-0001, T-0002, T-0015 (gap)
- **Mitigation**: Acceptable; short codes aren't meant to be sequential
- **Acceptance**: Gaps are fine, counters only increment

**Risk 4: Nested collisions**
- **Issue**: Both parent initiative and child task have colliding short codes
- **Mitigation**: Resolve parents first in dependency order, then children
- **Implementation**: Sort by path depth before processing collisions

**Risk 5: Concurrent sync operations**
- **Issue**: Multiple sync operations at same time could cause issues
- **Mitigation**: Existing database transactions provide some protection
- **Acceptance**: System assumes single-process model (current architecture)

## Status Updates **[REQUIRED]**

*To be added during implementation*