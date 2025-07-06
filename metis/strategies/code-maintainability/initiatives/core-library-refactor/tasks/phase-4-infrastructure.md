---
id: phase-4-infrastructure
level: task
status: todo
created_at: 2025-07-06T17:00:00Z
updated_at: 2025-07-06T17:00:00Z
parent: core-library-refactor
blocked_by: ["phase-3-application-layer"]

# Phase progression for tasks
tags:
  - "#task"
  - "#phase/todo"
  # - "#phase/doing"
  # - "#phase/completed"

exit_criteria_met: false
---

# Phase 4: Reorganize Infrastructure

## Objective

Move all infrastructure concerns to dedicated layer with proper abstractions and repository pattern.

## Acceptance Criteria

- [ ] Database operations moved to `infrastructure/persistence/`
- [ ] Repository pattern implemented for data access
- [ ] File system operations in `infrastructure/filesystem/`
- [ ] Template engine in `infrastructure/templates/`
- [ ] All infrastructure behind interfaces
- [ ] No domain logic in infrastructure layer

## Implementation Details

### 1. Implement Repository Pattern
```rust
// domain/repositories/document_repository.rs
pub trait DocumentRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<Document>>;
    async fn save(&self, document: &Document) -> Result<()>;
    async fn delete(&self, id: &str) -> Result<bool>;
}

// infrastructure/persistence/sqlx_document_repository.rs
pub struct SqlxDocumentRepository {
    pool: SqlitePool,
}
```

### 2. Extract File System Operations
```rust
// infrastructure/filesystem/sync_engine.rs
pub struct FileSyncEngine {
    watcher: Box<dyn FileWatcher>,
    repository: Box<dyn DocumentRepository>,
}
```

### 3. Move Template Engine
- Extract template rendering from current location
- Create infrastructure service for templates
- Separate template loading from rendering logic

### 4. Database Migrations
- Move migrations to `infrastructure/persistence/migrations/`
- Create migration runner service
- Ensure clean separation from domain

## Dependencies

- Phase 3 must be complete (application layer using repositories)

## Exit Criteria

- [ ] All infrastructure code isolated in infrastructure layer
- [ ] Repository pattern fully implemented
- [ ] Infrastructure services behind interfaces
- [ ] No circular dependencies between layers
- [ ] Database operations abstracted from domain