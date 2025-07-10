---
id: phase-3-application-layer
title: "Phase 3: Build Application Layer"
level: task
status: todo
created_at: 2025-07-06T17:00:00Z
updated_at: 2025-07-06T17:00:00Z
parent: core-library-refactor
blocked_by: ["phase-2-document-operations"]
archived: false

# Phase progression for tasks
tags:
  - "#task"
  - "#phase/todo"
  # - "#phase/doing"
  # - "#phase/completed"

exit_criteria_met: false
---

# Phase 3: Build Application Layer

## Objective

Create application layer with command/query handlers and services that orchestrate domain operations.

## Acceptance Criteria

- [ ] Command handlers for all document operations
- [ ] Query handlers for search and retrieval
- [ ] Application services orchestrating workflows
- [ ] Clear separation between commands and queries
- [ ] Public API updated to use application layer
- [ ] Error handling centralized at application boundary

## Implementation Details

### 1. Create Command Handlers
```rust
// application/commands/create_document.rs
pub struct CreateDocumentCommand {
    pub title: String,
    pub doc_type: DocumentType,
    pub parent_id: Option<String>,
}

pub trait CreateDocumentHandler {
    async fn handle(&self, command: CreateDocumentCommand) -> Result<Document>;
}
```

### 2. Create Query Handlers
```rust
// application/queries/get_document.rs
pub struct GetDocumentQuery {
    pub id: String,
    pub include_content: bool,
}

pub trait GetDocumentHandler {
    async fn handle(&self, query: GetDocumentQuery) -> Result<Option<Document>>;
}
```

### 3. Application Services
```rust
// application/services/document_service.rs
pub struct DocumentService {
    validator: Box<dyn DocumentValidator>,
    repository: Box<dyn DocumentRepository>,
    event_bus: Box<dyn EventBus>,
}
```

### 4. Update Public API
- Modify `core.rs` to use application services
- Maintain backward compatibility with facade pattern
- Route all operations through application layer

## Dependencies

- Phase 2 must be complete (domain services available)

## Exit Criteria

- [ ] All document operations have command/query handlers
- [ ] Application services orchestrate domain operations
- [ ] Public API routes through application layer
- [ ] Clear command/query separation
- [ ] Integration tests pass with new structure