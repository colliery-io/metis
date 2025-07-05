---
id: initiative-project-initialization-management
level: initiative
status: completed
created_at: 2025-07-03T16:00:00Z
updated_at: 2025-07-03T18:00:00Z
parent: strategy-mcp-interface
blocked_by: 
tags:
  - "#initiative"
  # - "#phase/discovery"
  # - "#phase/design"
  # - "#phase/ready"
  # - "#phase/decompose"
  # - "#phase/active"
  - "#phase/completed"
exit_criteria_met: true
technical_lead: 
estimated_complexity: s
related_adrs: 
---

# Project Initialization Management Initiative

## Context

Currently, there's no way for agents (or any users) to initialize a new Metis project. The system assumes an existing directory structure and database, but provides no method to create this structure from scratch. This is a critical gap - agents need to set up new projects before they can create any documents.

Without project initialization, agents must rely on manual setup or pre-existing project structures, severely limiting autonomous project creation and management capabilities.

## Goals & Non-Goals

**Goals:**
- Implement project initialization in core library
- Create standard Metis directory structure
- Initialize database with proper schema
- Create initial vision.md document template
- Support configuration options (project name, description)
- Validate project doesn't already exist

**Non-Goals:**
- Complex project templates or customization
- Migration from other documentation systems
- Multi-tenant or workspace management
- Project deletion or cleanup (separate concern)
- Git repository initialization
- Cloud storage setup

## Detailed Design

### Core Library Function

```rust
// Project initialization configuration
pub struct ProjectConfig {
    pub name: String,
    pub description: Option<String>,
    pub root_path: PathBuf,
}

// Initialize a new Metis project
pub async fn initialize_project(config: ProjectConfig) -> Result<ProjectMetadata> {
    // 1. Validate project doesn't exist
    // 2. Create directory structure
    // 3. Initialize database
    // 4. Create initial vision.md
    // 5. Return project metadata
}

pub struct ProjectMetadata {
    pub project_path: PathBuf,
    pub database_path: PathBuf,
}
```

### Directory Structure

```
project-root/
├── .metis.db          # SQLite database (hidden file)
├── vision.md          # Root vision document
├── strategies/        # Strategy documents
└── decisions/         # ADR documents
```

### Initial Vision Document

The initial vision document will be generated using the existing template system in `src/templates/vision/`. The template engine will create a complete vision.md file with:

- Proper frontmatter with project-specific ID (`vision-{slug}`)
- Template content structure (Purpose, Current State, Future State, Success Criteria, Principles, Constraints)
- Exit criteria checkboxes for phase transitions
- Optional project description insertion if provided

### MCP Tool Interface

```rust
#[mcp_tool(name = "initialize_project")]
pub struct InitializeProjectTool {
    pub project_name: String,
    pub description: Option<String>,
    pub root_path: String,
}
```

### Validation Rules

- Project path must not already contain a `.metis.db` file
- Project name must be valid for filesystem (no special chars)
- Parent directory must exist and be writable
- Database initialization must complete successfully
- Database schema migration must complete successfully

## Alternatives Considered

1. **Manual Setup Instructions**
   - Pros: No code needed
   - Cons: Error-prone, not agent-friendly
   - Decision: Rejected - automation is essential

2. **Template Repository Cloning**
   - Pros: Rich templates possible
   - Cons: Requires git, more complex
   - Decision: Rejected - keep it simple

3. **Configuration File Only**
   - Pros: Very minimal
   - Cons: Still requires directory creation
   - Decision: Rejected - need full initialization

## Implementation Plan

1. **Phase 1: Core Function** (Week 1)
   - Implement initialize_project() in core library
   - Directory structure creation
   - Database initialization with schema

2. **Phase 2: Initial Documents** (Week 1)
   - Vision document template creation
   - Project metadata tracking

3. **Phase 3: MCP Integration** (Week 2)
   - Create InitializeProjectTool
   - Add to MCP server tools
   - Error handling for agent feedback

4. **Phase 4: Testing & Polish** (Week 2)
   - Comprehensive testing
   - Edge case handling
   - Documentation

## Testing Strategy

- **Unit Tests**: Project creation with various configurations
- **Integration Tests**: Full initialization flow with database
- **Error Tests**: Existing projects, invalid paths, permissions
- **MCP Tests**: Agent tool invocation and responses

## Exit Criteria

- [ ] initialize_project() creates complete project structure
- [ ] Database initialized with proper schema
- [ ] Initial vision document created with correct template
- [ ] Project validation prevents overwriting existing projects
- [ ] MCP tool successfully exposes initialization
- [ ] Clear error messages for all failure scenarios
- [ ] Comprehensive test coverage
- [ ] Documentation for project setup