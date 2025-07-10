---
id: phase-1-document-domain-model
title: "Phase 1: Document Domain Model"
level: task
status: todo
created_at: 2025-07-06T17:00:00Z
updated_at: 2025-07-06T17:00:00Z
parent: core-library-refactor
blocked_by: []
archived: false

# Phase progression for tasks
tags:
  - "#task"
  - "#phase/todo"
  # - "#phase/doing"
  # - "#phase/completed"

exit_criteria_met: false
---

# Phase 1: Document Domain Model

## Objective

Extract and refine the core Document domain model as the foundation for all other refactoring work.

## Acceptance Criteria

- [ ] Clean `Document` domain model with no infrastructure dependencies
- [ ] Document-related types and enums properly organized
- [ ] Document behavior (methods) separated from data
- [ ] Clear value objects for document properties
- [ ] Document validation rules as domain logic
- [ ] No database or file system concerns in document model

## Implementation Details

### 1. Create Document Domain Structure
```
src/
└── domain/
    ├── mod.rs
    └── documents/
        ├── mod.rs
        ├── document.rs      // Core Document entity
        ├── types.rs         // DocumentType, Phase, etc.
        ├── context.rs       // DocumentContext value object
        ├── metadata.rs      // Document metadata value objects
        └── rules.rs         // Business rules for documents
```

### 2. Extract Pure Document Model
```rust
// domain/documents/document.rs
pub struct Document {
    pub id: DocumentId,
    pub title: String,
    pub document_type: DocumentType,
    pub content: DocumentContent,
    pub metadata: DocumentMetadata,
}

// No SQLx types, no file paths, no rendering logic
```

### 3. Define Document Behaviors
- Document creation with validation
- Phase transitions as document methods
- Exit criteria checking as document behavior
- Parent-child relationship rules

### 4. Create Value Objects
- `DocumentId` wrapper type with validation
- `DocumentContent` with frontmatter/body separation
- `DocumentMetadata` for timestamps, status, etc.

## Exit Criteria

- [ ] Document model has zero infrastructure imports
- [ ] All document business rules captured in domain
- [ ] Document tests work without database/files
- [ ] Clear separation between document data and behavior