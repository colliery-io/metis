---
id: task-restructure-rust-workspace
level: task
status: completed
created_at: 2025-07-03T19:50:00Z
updated_at: 2025-07-04T00:00:00Z
parent: initiative-mcp-server-implementation
blocked_by: 
tags:
  - "#task"
  - "#phase/completed"
  # - "#phase/doing"
  # - "#phase/completed"
exit_criteria_met: true
assignee: 
estimated_hours: 2
pr_links: []
---

# Restructure Project as Rust Workspace

## Parent Initiative
[[MCP Server Implementation Initiative]]

## Objective
Restructure the current single-crate Metis project into a Rust workspace containing the core library and a separate MCP server crate, enabling modular development and clean separation of concerns.

## Acceptance Criteria
- [ ] Create workspace `Cargo.toml` at project root
- [ ] Move existing core library code to `metis-core/` crate
- [ ] Create new `metis-mcp-server/` crate for MCP implementation
- [ ] Update all import paths and dependencies
- [ ] Ensure all existing tests pass after restructuring
- [ ] Update project documentation to reflect workspace structure
- [ ] Verify both crates build independently and together
- [ ] Maintain compatibility with existing `.metis.db` and document structure

## Implementation Notes

### Workspace Structure
```
metis/
├── Cargo.toml          # Workspace manifest
├── metis-core/         # Core library crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── database.rs
│   │   ├── models.rs
│   │   ├── project.rs
│   │   ├── updates.rs
│   │   └── ...
│   └── tests/
├── metis-mcp-server/   # MCP server crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   └── tools/
│   └── tests/
├── .metis.db          # Project database (unchanged)
└── docs/              # Documentation (unchanged)
```

### Workspace Cargo.toml
```toml
[workspace]
members = [
    "metis-core",
    "metis-mcp-server"
]
resolver = "2"

[workspace.dependencies]
# Shared dependencies
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
# ... other common deps
```

### Core Crate Dependencies
- Move existing dependencies from root `Cargo.toml` to `metis-core/Cargo.toml`
- Keep all current functionality: SQLite, templates, validation, updates
- Export public API through `lib.rs`

### MCP Server Crate Dependencies
- Add `metis-core` as workspace dependency
- Add MCP-specific dependencies: `rust_mcp_sdk`, etc.
- Create binary target for server executable

### Migration Steps
1. **Create workspace structure**:
   - Create `metis-core/` and `metis-mcp-server/` directories
   - Create workspace `Cargo.toml`

2. **Move core library**:
   - Move `src/` to `metis-core/src/`
   - Move existing `Cargo.toml` to `metis-core/Cargo.toml`
   - Update package name to `metis-core`

3. **Create MCP server crate**:
   - Create `metis-mcp-server/Cargo.toml` with dependencies
   - Add placeholder `src/main.rs` and `src/lib.rs`
   - Add `metis-core` as dependency

4. **Update imports and tests**:
   - Verify all internal imports work within `metis-core`
   - Update any absolute paths or references
   - Ensure test suite runs from workspace root

5. **Validate migration**:
   - Run `cargo build` from workspace root
   - Run `cargo test` to ensure all tests pass
   - Test project initialization still works with new structure

### Backup Strategy
- Create `src-bu/` backup before migration (already exists)
- Commit intermediate states during restructuring
- Maintain ability to rollback if issues arise

### Import Path Changes
Most imports should remain unchanged since they're relative within the core crate. External consumers will now use:
```rust
use metis_core::{initialize_project, update_document_content, ...};
```

### Testing Considerations
- All existing integration tests should continue to work
- Workspace-level testing: `cargo test --workspace`
- Individual crate testing: `cargo test -p metis-core`
- Verify database operations work from both crates

## Error Scenarios
- Build failures due to import path issues
- Test failures from moved modules
- Dependency resolution conflicts
- Path assumptions in existing code

## Status Updates
*To be added during implementation*

## Exit Criteria
- [ ] Workspace builds successfully with `cargo build`
- [ ] All existing tests pass with `cargo test --workspace`
- [ ] Core library functionality unchanged and accessible
- [ ] MCP server crate structure ready for implementation
- [ ] Project initialization and document operations work as before
- [ ] Clean separation between core library and MCP concerns