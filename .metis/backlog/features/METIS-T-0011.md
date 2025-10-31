---
id: syncronize-more-often
level: task
title: "Syncronize More Often"
short_code: "METIS-T-0011"
created_at: 2025-10-30T20:15:41.037892+00:00
updated_at: 2025-10-31T02:00:13.399981+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#feature"
  - "#feature"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# Syncronize More Often

## Objective

Ensure database synchronization happens automatically on application startup (MCP server, TUI, GUI, CLI) and potentially at the beginning/end of service calls to keep database and filesystem in sync without requiring manual intervention.

## Backlog Item Details

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

### Business Justification

- **User Value**: Users shouldn't have to think about syncing - the system should automatically detect filesystem changes and keep the database up-to-date. This prevents stale data issues and improves reliability.
- **Business Value**: Reduces user friction and support issues related to "documents not showing up" or "changes not appearing" problems. Improves trust in the system.
- **Effort Estimate**: M (Medium - requires changes across multiple entry points but sync infrastructure already exists)

## Acceptance Criteria

## Acceptance Criteria

- [ ] MCP server performs sync before read operations
- [ ] GUI performs sync before read operations
- [ ] CLI performs sync before read operations
- [ ] TUI continues to sync on startup (already implemented)
- [ ] All read operations (list, read, search) sync before querying database
- [ ] Write operations continue to sync after completion (already implemented across MCP, most of CLI/GUI)

## Test Cases

### Test Case 1: MCP Server Lazy Sync
- **Test ID**: TC-001
- **Preconditions**: Metis project with documents on filesystem, MCP server not yet started
- **Steps**:
  1. Manually edit a document markdown file
  2. Start MCP server
  3. Call list_documents tool (first tool call)
  4. Verify the edited content appears in the results
- **Expected Results**: First tool call triggers sync, edited content is visible
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: GUI Project Load Sync
- **Test ID**: TC-002
- **Preconditions**: Metis project with documents on filesystem
- **Steps**:
  1. Manually edit a document markdown file
  2. Open/load the project in GUI
  3. View document list
  4. Verify the edited content appears
- **Expected Results**: Project load triggers sync, edited content is visible immediately
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 3: CLI Pre-Command Sync
- **Test ID**: TC-003
- **Preconditions**: Metis project with documents on filesystem
- **Steps**:
  1. Manually edit a document markdown file
  2. Run `metis list` command
  3. Verify the edited content appears in the list
- **Expected Results**: List command syncs before reading database
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 4: External Edit Detection
- **Test ID**: TC-004
- **Preconditions**: Running application (MCP/TUI/GUI)
- **Steps**:
  1. With application running, externally edit a document markdown file
  2. Perform an operation that reads from database
  3. Verify the external edit is reflected
- **Expected Results**: Sync before read operation detects and imports external changes
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Implementation Notes

### Current State Analysis

**Already Syncing:**
- TUI: Syncs on startup (App::initialize, line 61-64 in app/mod.rs)
- MCP: Syncs after ALL write operations (create, edit, transition, archive)
- CLI Archive: Syncs before archiving (archive.rs:32-38)
- GUI Transition: Syncs after phase transitions (services/transition.rs:68-77)

**Missing Startup Sync:**
- MCP Server: No sync on startup (lib.rs:134)
- GUI: No sync on project load (services/project.rs)
- CLI: No pre-command sync hook (main.rs)

**Inconsistent Operation Sync:**
- CLI Transition: Has TODO comment but doesn't sync (transition.rs:51)
- GUI Create/Edit: Doesn't sync after document operations
- Read operations: Don't sync before reading (may cause stale data)

### Technical Approach

**Phase 1: Add Sync Before All Read Operations**

1. **MCP Server - Sync Before Reads**
   - Add sync call at the beginning of read tool handlers (list_documents, read_document, search_documents)
   - Reuse existing `sync_workspace` helper
   - Location: `crates/metis-docs-mcp/src/tools/list_documents.rs`, `read_document.rs`, `search_documents.rs`

2. **GUI - Sync Before Reads**
   - Add sync call at the beginning of read service methods
   - List documents: sync before querying
   - Read document: sync before fetching
   - Search: sync before searching
   - Location: `crates/metis-docs-gui/src-tauri/src/services/*.rs`

3. **CLI - Sync Before Reads**
   - Add sync call at the beginning of read commands (list, read, search)
   - Reuse pattern from archive command
   - Location: `crates/metis-docs-cli/src/commands/list.rs`, `read.rs`, `search.rs`

**Implementation Pattern:**
```rust
// Standard pattern for read operations
pub async fn read_operation(&self) -> Result<Data> {
    // Sync first to catch any external edits
    let database = Database::new(db_path)?;
    let app = Application::new(database);
    app.sync_directory(&metis_dir).await?;

    // Then perform read
    let result = app.read_from_database()?;
    Ok(result)
}
```

**Phase 2: Complete Existing TODOs**

1. CLI Transition: Add sync after transition (transition.rs:51)
2. GUI Create/Edit: Add sync after document operations (if missing)
3. Standardize sync pattern across all write operations

### Key Files to Modify

**MCP Read Operations:**
1. **crates/metis-docs-mcp/src/tools/list_documents.rs** - Add sync before listing
2. **crates/metis-docs-mcp/src/tools/read_document.rs** - Add sync before reading
3. **crates/metis-docs-mcp/src/tools/search_documents.rs** - Add sync before searching

**CLI Read Operations:**
4. **crates/metis-docs-cli/src/commands/list.rs** - Add sync before listing
5. **crates/metis-docs-cli/src/commands/read.rs** - Add sync before reading
6. **crates/metis-docs-cli/src/commands/search.rs** - Add sync before searching

**GUI Read Operations:**
7. **crates/metis-docs-gui/src-tauri/src/services/*.rs** - Add sync before read operations

**Write Operation TODOs:**
8. **crates/metis-docs-cli/src/commands/transition.rs** - Complete TODO: add sync after transition
9. **crates/metis-docs-gui/src-tauri/src/services/document.rs** - Add sync after create/update if missing

### Dependencies

- Existing sync infrastructure (Application::sync_directory)
- Workspace detection logic (workspace.rs)
- TUI sync pattern can serve as reference implementation

### Decision: Sync Before Every Read

**Rationale:**
- Sync is fast even for large projects (performance is not a concern)
- Ensures external edits are always visible immediately
- Prevents stale data issues across all interfaces
- Sync on write is already handled (MCP, most CLI/GUI operations)

**Implementation:**
- Add sync call at the start of every read operation (list, read, search)
- Reuse existing `sync_directory` infrastructure
- No caching or TTL needed - just sync every time

### Risk Considerations

**Low Risk:**
- Sync conflicts or race conditions with concurrent operations
  - Mitigation: Sync operations are idempotent (safe to run multiple times)
  - Mitigation: Database operations are serialized through connection pool

## Status Updates **[REQUIRED]**

*To be added during implementation*