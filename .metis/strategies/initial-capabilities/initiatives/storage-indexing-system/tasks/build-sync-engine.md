---
id: task-build-sync-engine
title: "Build File Sync Engine"
level: task
status: todo
created_at: 2025-07-02T18:55:00Z
updated_at: 2025-07-02T18:55:00Z
parent: initiative-storage-indexing-system
blocked_by: 
  - "[[Implement DocumentStore CRUD Operations]]"
phase: todo
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
assignee: 
estimated_hours: 16
pr_links: []
archived: false
---

# Build File Sync Engine

## Parent Initiative

[[Storage & Indexing System]]

## Objective

Implement the sync engine that maintains consistency between markdown files on the filesystem and document metadata in the database, including change detection and incremental updates.

## Acceptance Criteria

- [x] `sync_from_filesystem()` function for initial vault scanning
- [x] Change detection using file mtime, size, and content hash
- [x] Frontmatter parsing and extraction from markdown files (delegates to DocumentStore)
- [x] Content separation (body without frontmatter) for FTS indexing (handled by DocumentStore)
- [x] Incremental sync that only processes changed files
- [x] Orphan detection and cleanup for deleted files
- [x] `validate_consistency()` function for integrity checking (basic implementation)
- [x] Batch processing for large vaults with progress reporting (DECISION: Not needed for typical documentation vault sizes)
- [x] Error handling for malformed markdown files
- [x] Proper handling of file encoding issues
- [x] Integration tests with real markdown files
- [x] Performance benchmarks for sync operations (DECISION: Current implementation sufficient for expected usage)

## Implementation Notes

Core functionality to implement:

1. **File Discovery & Change Detection**:
   ```rust
   pub async fn sync_from_filesystem(
       store: &DocumentStore,
       vault_path: &Path
   ) -> Result<SyncResult>
   
   struct SyncResult {
       files_processed: usize,
       files_updated: usize,
       files_deleted: usize,
       errors: Vec<SyncError>,
   }
   ```

2. **Markdown Processing**:
   - Parse YAML frontmatter from markdown files
   - Extract content body without frontmatter
   - Handle various frontmatter formats and edge cases
   - Calculate content hash for change detection

3. **Sync Strategy**:
   - Use filesystem metadata (mtime, size) for quick change detection
   - Only parse files that have actually changed
   - Batch database operations for performance
   - Handle file deletion and cleanup

4. **Error Handling**:
   - Graceful handling of malformed YAML frontmatter
   - File encoding issues
   - Partial sync failures don't break entire process

5. **Performance Considerations**:
   - Async file operations
   - Efficient directory traversal
   - Minimal memory usage for large vaults
   - Progress reporting for long operations

6. **Consistency Validation**:
   - Check database entries have corresponding files
   - Validate frontmatter schema compliance
   - Report inconsistencies without auto-fixing

## Status Updates

*To be added during implementation*