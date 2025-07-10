---
id: task-implement-phase-transitions
title: "Implement Phase Transition Management"
level: task
status: todo
created_at: 2025-07-03T12:45:00Z
updated_at: 2025-07-03T12:45:00Z
parent: initiative-core-document-management
blocked_by: 
  - "[[Implement Document Validation]]"
phase: todo
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
assignee: 
estimated_hours: 6
pr_links: []
archived: false
---

# Implement Phase Transition Management

## Parent Initiative

[[Core Document Management Library]]

## Objective

Implement phase transition functions that enforce business rules and update document phase and tags using the comment/uncomment approach.

## Acceptance Criteria

- [ ] transition_phase() function with validation and file updates
- [ ] can_transition_to_phase() function for checking validity
- [ ] Phase flow rules for each document type
- [ ] Comment/uncomment logic for phase and tag updates
- [ ] Force override option for emergency transitions
- [ ] Error handling for invalid transitions
- [ ] Unit tests for all phase transition scenarios
- [ ] Integration tests with real document files

## Implementation Details

### Function Signatures

```rust
pub fn transition_phase(
    document_path: &str,
    new_phase: &str,
    force: bool,
) -> Result<String>;

pub fn can_transition_to_phase(
    document_path: &str,
    target_phase: &str,
) -> Result<bool>;
```

### Phase Flow Rules

- **Vision:** draft → review → published
- **Strategy:** shaping → design → ready → active → completed
- **Initiative:** discovery → design → ready → decompose → active → completed
- **Task:** todo → doing → completed
- **ADR:** draft → discussion → decided → superseded

### Comment/Uncomment Logic

Update frontmatter by commenting current phase and uncommenting target phase, plus corresponding tags.

## Status Updates

*To be added during implementation*