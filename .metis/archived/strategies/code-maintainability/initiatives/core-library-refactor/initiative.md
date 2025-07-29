---
id: core-library-refactor
level: initiative
title: "Core Library Refactor"
created_at: 2025-07-06T16:55:16+00:00
updated_at: 2025-07-28T21:06:32.998161+00:00
parent: code-maintainability
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/completed"


exit_criteria_met: false
estimated_complexity: L
---

# Core Library Refactor Initiative

## Context

The metis-docs-core library has grown organically into a flat, disorganized structure mixing domain logic, infrastructure concerns, and utilities. Analysis reveals mixed concerns across modules, tight coupling, and lack of clear domain boundaries. This refactor aims to establish a proper domain-driven architecture as the foundation for future maintainability improvements.

## Goals & Non-Goals

**Goals:**
- Establish clear domain boundaries and module hierarchy
- Separate business logic from infrastructure concerns
- Implement proper service layer abstractions
- Create testable, loosely coupled components
- Maintain backward compatibility during refactor

**Non-Goals:**
- Database technology changes (SQLx to Diesel is separate initiative)
- Performance optimizations
- External API contract changes
- MCP server architectural changes

## Detailed Design

### Target Architecture

**Domain Layer** (`domain/`):
- `models/` - Core domain entities (Document, DocumentType, etc.)
- `services/` - Domain services (validation, phase transitions)
- `events/` - Domain events and event handlers

**Application Layer** (`application/`):
- `commands/` - Command handlers (create, update, delete)
- `queries/` - Query handlers (search, retrieve)
- `services/` - Application services orchestrating domain operations

**Infrastructure Layer** (`infrastructure/`):
- `persistence/` - Database operations and repositories
- `filesystem/` - File system operations
- `templates/` - Template engine and rendering

### Migration Strategy

1. **Create new module structure** alongside existing code
2. **Move domain models** first (Document, DocumentType, etc.)
3. **Extract business logic** into domain services
4. **Implement repository pattern** for data access
5. **Create application services** to orchestrate operations
6. **Update public API** to use new structure
7. **Remove old modules** once fully migrated

## Alternatives Considered

**Big Bang Refactor**: Rewrite entire library at once
- Rejected: Too risky, would break existing functionality

**Feature-based Modules**: Organize by features instead of layers
- Rejected: Would mix concerns within modules, doesn't address core issues

**Keep Current Structure**: Minimal changes to existing organization
- Rejected: Doesn't solve maintainability issues, technical debt continues to grow

## Implementation Plan

### Phase 1: Foundation (Week 1-2)
- Create new module structure directories
- Move domain models to `domain/models/`
- Extract core types and enums
- Update imports and ensure tests pass

### Phase 2: Domain Logic (Week 3-4)
- Extract validation logic to `domain/services/`
- Move phase transition logic to domain
- Create domain events system
- Implement domain service interfaces

### Phase 3: Application Layer (Week 5-6)
- Create command/query handlers
- Implement application services
- Extract orchestration logic from current modules
- Update public API to use application layer

### Phase 4: Infrastructure (Week 7-8)
- Move database operations to `infrastructure/persistence/`
- Extract file system operations
- Implement repository pattern
- Create infrastructure service implementations

### Phase 5: Cleanup (Week 9-10)
- Remove old modules
- Update documentation
- Verify all tests pass
- Performance validation

## Testing Strategy

- **Unit Tests**: Each domain service and application service gets comprehensive unit tests
- **Integration Tests**: Verify end-to-end workflows still function
- **Migration Tests**: Ensure refactored code produces identical results
- **Performance Tests**: Validate no performance regressions
- **Backward Compatibility**: Ensure existing MCP server integration continues to work

## Exit Criteria

- [ ] Context and goals are clearly defined
- [ ] Technical approach is designed at sufficient detail
- [ ] Implementation plan shows clear phases
- [ ] Testing strategy is defined
- [ ] Dependencies are identified and resolved