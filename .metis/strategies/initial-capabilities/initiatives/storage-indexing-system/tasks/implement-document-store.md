---
id: task-implement-document-store
title: "Implement DocumentStore CRUD Operations"
level: task
status: todo
created_at: 2025-07-02T18:50:00Z
updated_at: 2025-07-02T18:50:00Z
parent: initiative-storage-indexing-system
blocked_by: 
  - "[[Create Database Schema & Migrations]]"
phase: todo
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
assignee: 
estimated_hours: 12
pr_links: []
archived: false
---

# Implement DocumentStore CRUD Operations

## Parent Initiative

[[Storage & Indexing System]]

## Objective

Implement the core DocumentStore struct with basic CRUD operations for documents, following the async Rust patterns established in the existing Metis codebase.

## Acceptance Criteria

- [ ] `DocumentStore` struct created with SQLx pool management
- [ ] `Document` model struct with all required fields and proper serialization
- [ ] `store_document()` function with INSERT OR REPLACE logic
- [ ] `get_document()` function with optional document retrieval
- [ ] `update_document()` function for document modifications
- [ ] `delete_document()` function with proper cleanup
- [ ] Frontmatter property extraction and storage in `document_properties` table
- [ ] Content hash calculation for change detection
- [ ] Proper error handling with custom error types
- [ ] All database operations use transactions where appropriate
- [ ] Integration with SQLx compile-time checked queries
- [ ] Unit tests for all CRUD operations
- [ ] Documentation with usage examples

## Decision Made During Implementation

**Interface Design Decision**: DocumentStore will accept file paths only, not pre-parsed content. This provides a cleaner interface where DocumentStore handles all file parsing internally:

```rust
async fn store_document(filepath: &Path) -> Result<Document>
```

**Rationale**:
- Cleaner interface - caller doesn't need to parse markdown
- Better fits "filesystem as source of truth" principle
- Enables future file watcher integration (just send filepath on change)
- DocumentStore controls the entire parse->store pipeline

**Implementation Impact**:
- Need frontmatter parsing dependency (gray_matter or similar)
- DocumentStore handles markdown parsing, frontmatter extraction, content separation
- Content hash calculated from file contents

## Implementation Notes

Key components to implement:

1. **DocumentStore Structure**:
   ```rust
   pub struct DocumentStore {
       pool: SqlitePool,
   }
   
   impl DocumentStore {
       pub async fn new(database_url: &str) -> Result<Self>
       pub async fn store_document(filepath: &Path) -> Result<Document>
       // Other CRUD operations...
   }
   ```

2. **Document Model**:
   - Based on the schema design from Create Database Schema task
   - Include all fields: id, filepath, document_type, phase, content, etc.
   - Proper serde derives for JSON serialization

3. **Core Operations**:
   - Use sqlx::query! macro for compile-time validation
   - Parse markdown files to separate frontmatter from content body
   - Extract and store frontmatter properties for efficient querying
   - Calculate content hash from file contents

4. **File Parsing**:
   - Add frontmatter parsing dependency
   - Handle YAML frontmatter extraction
   - Separate markdown body content

5. **Error Handling**:
   - Custom error types using thiserror
   - Proper error propagation from SQLx and file I/O

6. **Testing**:
   - Unit tests with in-memory SQLite database
   - Test all CRUD scenarios including edge cases
   - Test frontmatter parsing edge cases

## Status Updates

### 2025-07-03 - Implementation Complete

**CRUD Operations Implemented:**
- ✅ `store_document(filepath)` - Reads, parses, and stores documents from filesystem
- ✅ `get_document(id)` - Retrieves documents by ID with full deserialization
- ✅ `update_document(filepath)` - Re-reads and updates existing documents
- ✅ `delete_document(id)` - Removes documents and related data

**Key Features:**
- ✅ Frontmatter parsing using gray_matter crate with YAML engine
- ✅ Content hash calculation using SHA256 for change detection
- ✅ Automatic property extraction from frontmatter to `document_properties` table
- ✅ Full error handling with custom MetisError types
- ✅ Comprehensive test suite with 9 passing tests
- ✅ SQLx compile-time query validation with prepared query cache

**Technical Implementation:**
- Filepath-based interface as decided: `store_document(filepath: &Path)`
- Gray_matter for frontmatter parsing with Pod → JSON conversion
- SHA256 content hashing for change detection
- Automatic property extraction for efficient querying
- SQLx migrations with compile-time checked queries
- Complete CRUD test coverage including edge cases

**Files Modified:**
- Added gray_matter and sha2 dependencies to Cargo.toml
- Implemented all CRUD operations in src/database/mod.rs
- Created comprehensive test suite covering all functionality
- Generated SQLx query metadata cache (.sqlx directory)

**Testing Process:**
To run tests for this module, follow these steps:

1. **Create development database:**
   ```bash
   DATABASE_URL=sqlite:metis_dev.db cargo sqlx database create
   ```

2. **Run migrations:**
   ```bash
   DATABASE_URL=sqlite:metis_dev.db sqlx migrate run --source ./src/migrations
   ```

3. **Generate query cache (required for compile-time checking):**
   ```bash
   DATABASE_URL=sqlite:metis_dev.db cargo sqlx prepare
   ```

4. **Run tests:**
   ```bash
   cargo test database::tests
   ```

**Note:** Steps 1-3 only need to be run once or when schema changes. The `.sqlx` directory contains cached query metadata and should be committed to version control for offline compilation.