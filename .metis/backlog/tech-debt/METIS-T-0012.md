---
id: database-as-cache-only
level: task
title: "Database as cache only"
short_code: "METIS-T-0012"
created_at: 2025-11-06T10:02:34.158101+00:00
updated_at: 2025-12-06T23:15:08.158649+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Database as cache only

## Objective **\[REQUIRED\]**

Currently, metis uses the sqlite database as a way to determine if a folder is a metis project and fails when the database is missing. The database should be treated as a cache that is always synchronized from the filesystem.

The new behavior should be:
1. Detect valid metis project by checking for `.metis` directory (not database)
2. ALWAYS run a sync operation during workspace validation to ensure database is up-to-date
3. This handles missing database, corrupted database, and out-of-sync database automatically
4. Database becomes a true cache - always fresh, always regenerable

This requires changing the project detection logic to rely on filesystem indicators and making sync a standard part of workspace validation, not a separate command users must remember to run. 

## Backlog Item Details **\[CONDITIONAL: Backlog Item\]**

### Type

- \[ \] Bug - Production issue that needs fixing
- \[ \] Feature - New functionality or enhancement
- \[X\] Tech Debt - Code improvement or refactoring
- \[ \] Chore - Maintenance or setup work

### Priority

- \[ \] P0 - Critical (blocks users/revenue)
- \[X\] P1 - High (important for user experience)
- \[ \] P2 - Medium (nice to have)
- \[ \] P3 - Low (when time permits)

### Technical Debt Impact **\[CONDITIONAL: Tech Debt\]**

- **Current Problems**:
  - Database in git causes merge conflicts and synchronization issues
  - Metis fails when database is missing or out of sync
  - Users must remember to run `metis sync` after git operations
  - Multi-developer workflows are painful due to database conflicts
  - Manual changes to markdown files don't reflect in database until explicit sync

- **Benefits of Fixing**:
  - Database can be gitignored - no more merge conflicts
  - Database always stays in sync with filesystem - no manual sync needed
  - Self-healing behavior: missing, corrupted, or stale database auto-fixed
  - Filesystem (markdown files) becomes clear source of truth
  - Seamless multi-developer collaboration
  - Direct markdown edits immediately reflected (on next command)

- **Risk Assessment**:
  - Current git workflow is severely hampered by database conflicts
  - Users frequently forget to sync, leading to stale data issues
  - Without this fix, multi-developer collaboration remains problematic

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **\[REQUIRED\]**

- \[ \] Project detection works based on `.metis` directory presence, not database presence
- \[ \] Database sync runs automatically on every workspace validation (every command)
- \[ \] Sync is silent/fast when no changes detected (up-to-date case)
- \[ \] Missing database is automatically created and populated from filesystem
- \[ \] Corrupted database is handled gracefully (recreated if needed)
- \[ \] Manual markdown edits are reflected in database on next command execution
- \[ \] Database file can be added to `.gitignore` without breaking functionality
- \[ \] After git pull/merge, changes are automatically synced without manual intervention

## Implementation Notes **\[CONDITIONAL: Technical Task\]**

### Technical Approach

The implementation requires changes in three key areas:

1. **Workspace Detection** (`crates/metis-docs-core/src/application/services/workspace/detection.rs`)
   - Modify `validate_workspace()` (line 56-70) to check for `.metis` directory only
   - Always run sync operation as part of validation, not just when database missing
   - Return validated workspace after sync completes

2. **Always-Sync Strategy**
   - `validate_workspace()` becomes: check `.metis` exists → initialize/verify database → run sync → return workspace
   - Use existing `Database::new()` to create/migrate database (handles missing/new database)
   - Use existing `app.sync_directory()` to keep database fresh
   - Sync should be fast when no changes (existing behavior already optimized)
   - Log sync results only if changes detected (keep output clean)

3. **Update is_workspace checks** (`crates/metis-docs-core/src/application/services/workspace/initialization.rs`)
   - Modify `is_workspace()` (line 157-162) to only check for `.metis` directory
   - Remove database existence requirement

### Code Locations to Modify

- `crates/metis-docs-core/src/application/services/workspace/detection.rs:56-70` - validate_workspace method (add sync here)
- `crates/metis-docs-core/src/application/services/workspace/initialization.rs:157-162` - is_workspace method
- Remove scattered sync calls from commands (now handled in workspace detection)
- Tests in both files that verify database-required behavior
- Add `.gitignore` entry for `metis.db` in init command
- Deprecate/remove `metis sync` command (now automatic)

### Required Integration Test

Create a functional test that validates the core behavior:
1. Initialize a workspace (metis init)
2. Add some documents/data to the workspace
3. Delete the SQLite database file
4. Call workspace detection/validation (e.g., any metis command)
5. Verify the database was automatically reformed with all data from filesystem
6. Validate data integrity - all documents present and correct

### Dependencies

- Existing sync functionality (`Application::sync_directory()`)
- Existing database initialization (`Database::new()`)
- No new external dependencies needed

### Risk Considerations

- **None identified**: This is actually a simplification
  - Commands already run syncs before/after invocation
  - This consolidates sync logic into workspace detection
  - Can remove scattered sync calls throughout codebase
  - Partial sync failures self-heal on next command (next sync)
  - System is single-user, concurrency not a concern

## Status Updates **\[REQUIRED\]**

*To be added during implementation*