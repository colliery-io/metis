---
id: phase-2-document-operations
level: task
status: todo
created_at: 2025-07-06T17:00:00Z
updated_at: 2025-07-06T17:00:00Z
parent: core-library-refactor
blocked_by: ["phase-1-document-domain-model"]

# Phase progression for tasks
tags:
  - "#task"
  - "#phase/todo"
  # - "#phase/doing"
  # - "#phase/completed"

exit_criteria_met: false
---

# Phase 2: Document Operations & Services

## Objective

Build document-centric domain services that operate on the clean Document model from Phase 1.

## Acceptance Criteria

- [ ] Document lifecycle operations defined as domain services
- [ ] Template rendering as a document operation
- [ ] Document relationship management
- [ ] Document search/query specifications
- [ ] Content processing services
- [ ] All operations work on pure domain models

## Implementation Details

### 1. Document Lifecycle Services
```rust
// domain/documents/services/lifecycle.rs
pub trait DocumentLifecycle {
    fn create(context: DocumentContext) -> Result<Document>;
    fn update(document: &mut Document, changes: DocumentChanges) -> Result<()>;
    fn transition_phase(document: &mut Document, new_phase: Phase) -> Result<()>;
}
```

### 2. Content Processing
```rust
// domain/documents/services/content.rs
pub trait ContentProcessor {
    fn parse_frontmatter(content: &str) -> Result<DocumentMetadata>;
    fn render_content(document: &Document, template: &Template) -> Result<String>;
    fn validate_content(content: &DocumentContent) -> Result<()>;
}
```

### 3. Document Relationships
```rust
// domain/documents/services/relationships.rs
pub struct DocumentHierarchy {
    pub parent: Option<DocumentId>,
    pub children: Vec<DocumentId>,
    pub blocked_by: Vec<DocumentId>,
}
```

### 4. Query Specifications
```rust
// domain/documents/specifications/mod.rs
pub trait DocumentSpecification {
    fn is_satisfied_by(&self, document: &Document) -> bool;
}
```

## Dependencies

- Phase 1 must be complete (Document model available)

## Exit Criteria

- [ ] Document operations defined without infrastructure
- [ ] Services operate on pure domain models
- [ ] Clear interfaces for all document operations
- [ ] Template rendering abstracted as domain concern
- [ ] Relationship management in domain layer