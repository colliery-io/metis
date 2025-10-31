---
id: paths-should-be-relative-to-metis
level: task
title: "paths should be relative to .metis in the database."
short_code: "METIS-T-0010"
created_at: 2025-10-30T20:12:19.361895+00:00
updated_at: 2025-10-31T01:56:57.535418+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# paths should be relative to .metis in the database.

## Objective

Convert document path storage from absolute paths to paths relative to the .metis directory to enable proper synchronization between different computers and user accounts.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [x] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment

- **Affected Users**: All users syncing Metis databases between different computers or user accounts
- **Reproduction Steps**:
  1. Initialize a Metis project on Computer A (e.g., /Users/alice/project/.metis/)
  2. Create documents and let them sync to the database
  3. Copy the .metis directory to Computer B at a different location (e.g., /Users/bob/project/.metis/)
  4. Attempt to use Metis commands (list, search, read) on Computer B
  5. Observe that documents cannot be found or loaded correctly
- **Expected vs Actual**:
  - **Expected**: Documents should be found using their short codes regardless of the absolute location of the .metis directory
  - **Actual**: Document lookups fail because the database contains absolute paths from Computer A that don't exist on Computer B

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Database stores document paths relative to .metis directory (e.g., "strategies/PROJ-S-0001/strategy.md" instead of "/Users/alice/project/.metis/strategies/PROJ-S-0001/strategy.md")
- [ ] Document lookup by short code works correctly regardless of .metis directory location
- [ ] Synchronization correctly imports files using relative paths
- [ ] All document operations (read, edit, search, list) work with relative paths
- [ ] Tests verify path resolution works across different base directory locations
- [ ] Documentation updated to instruct users to delete metis.db and re-sync after upgrading

## Test Cases

### Test Case 1: Import Document with Relative Path Storage
- **Test ID**: TC-001
- **Preconditions**: Clean Metis database, test document exists
- **Steps**:
  1. Initialize Metis project at /tmp/test-project/.metis
  2. Create a document at /tmp/test-project/.metis/strategies/TEST-S-0001/strategy.md
  3. Run sync to import the document
  4. Query the database to verify the stored filepath
- **Expected Results**: Database contains "strategies/TEST-S-0001/strategy.md" (relative path)
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: Cross-Location Document Lookup
- **Test ID**: TC-002
- **Preconditions**: Database with documents stored using relative paths
- **Steps**:
  1. Create test project at /tmp/location-a/.metis with documents
  2. Copy .metis directory to /tmp/location-b/.metis
  3. Change working directory to /tmp/location-b
  4. Run metis read <short_code> for existing document
- **Expected Results**: Document is found and displayed correctly
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 3: Re-sync After Database Reset
- **Test ID**: TC-003
- **Preconditions**: Existing markdown documents in .metis directory structure
- **Steps**:
  1. Create test project with documents in .metis/strategies/, .metis/initiatives/, etc.
  2. Delete metis.db file
  3. Run metis sync to re-import all documents
  4. Query database to verify all paths are relative
  5. Verify all documents are accessible via short codes
- **Expected Results**: All documents imported with relative paths, all relationships reconstructed
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Implementation Notes

### Root Cause Analysis

The database schema uses `filepath` as the PRIMARY KEY in the documents table (see `001_initial_schema/up.sql:6`). Currently, absolute paths are stored at several critical points:

1. **FilesystemService::find_markdown_files** (filesystem.rs:77) - Returns absolute paths from WalkDir
2. **SynchronizationService::import_from_file** (synchronization.rs:33) - Stores the full path string via `to_string_lossy()`
3. **SynchronizationService::sync_file** (synchronization.rs:252) - Uses full paths for lookups

Related tables use filepath as foreign keys:
- `document_relationships`: Uses `child_filepath` and `parent_filepath`
- `document_tags`: Uses `document_filepath`
- `document_search`: Uses `document_filepath` for FTS

### Technical Approach

**Phase 1: Path Conversion Logic**
- Add a function to strip workspace directory prefix from paths before storage
- Update `import_from_file` to convert absolute paths to relative before creating `NewDocument`
- Update `sync_file` to work with relative paths
- Add helper function to resolve relative paths to absolute when accessing filesystem

**Phase 2: Update Database Operations**
- Modify `find_by_filepath` and related queries to work with relative paths
- Add path resolution layer that converts relative DB paths to absolute filesystem paths when needed
- Update all join operations to work with relative paths

**Phase 3: Validation**
- Add unit tests for path conversion functions
- Add integration tests for cross-location functionality
- Test delete + re-sync workflow with sample data

**Phase 4: User Communication**
- Update changelog/release notes about the breaking change
- Add clear instructions for users to delete metis.db and run `metis sync` after upgrading
- Consider adding a version check that warns users if they need to re-sync

### Key Files to Modify

1. **crates/metis-docs-core/src/application/services/synchronization.rs**
   - Line 33: Convert path before storing in `import_from_file`
   - Line 252: Update path handling in `sync_file`
   - Add path conversion helper functions

2. **crates/metis-docs-core/src/application/services/filesystem.rs**
   - Line 77: Convert discovered paths to relative in `find_markdown_files`
   - Accept base directory parameter for path stripping

3. **crates/metis-docs-core/src/dal/database/repository.rs**
   - Add path resolution layer for filesystem access
   - Update queries to handle relative paths

4. **Documentation files (CHANGELOG, README, etc.)**
   - Add upgrade instructions for users
   - Document the breaking change and re-sync requirement

### Dependencies

- Workspace detection logic (workspace.rs:8-32) must be available before path resolution
- Existing lineage extraction logic (synchronization.rs:134-198) already uses relative paths - can be referenced as a pattern

### Upgrade Path for Users

Users with existing databases will need to:
1. Back up any custom edits to documents (if not already in markdown files)
2. Delete the `metis.db` file in their .metis directory
3. Run `metis sync` to re-import all documents with relative paths
4. Verify documents are accessible

This is a breaking change but acceptable because:
- The source of truth is the markdown files, not the database
- Re-sync is a simple, safe operation
- No data loss occurs (all document content remains in markdown files)

### Risk Considerations

**Medium Risk:**
- Users unaware of breaking change may experience lookup failures
  - Mitigation: Clear release notes and upgrade instructions
  - Mitigation: Consider adding version detection that warns/auto-prompts for re-sync
  - Mitigation: Error messages should hint at the need to re-sync

**Low Risk:**
- Edge cases with symlinks or special path characters
  - Mitigation: Use canonical path resolution
  - Mitigation: Add path validation before storage

## Status Updates **[REQUIRED]**

*To be added during implementation*