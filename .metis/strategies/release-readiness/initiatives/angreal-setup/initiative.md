---
id: initiative-angreal-setup
title: Angreal Setup and Configuration Initiative
level: initiative
status: completed
created_at: 2025-07-04T23:05:00Z
updated_at: 2025-07-04T16:35:00Z
parent: strategy-release-readiness
blocked_by: 
tags:
  - "#initiative"
  - "#phase/completed"
  # - "#phase/design"
  # - "#phase/ready"
  # - "#phase/decompose"
  # - "#phase/active"
  # - "#phase/completed"
exit_criteria_met: true
technical_lead: 
estimated_complexity: m
related_adrs: 
archived: false
---

# Angreal Setup and Configuration Initiative

## Context

Angreal (https://angreal.github.io/angreal/) is a task automation framework that can help standardize development workflows. We need to research and set up angreal to provide consistent commands for common development tasks in the Metis project.

## Goals & Non-Goals

**Goals:**
- Research angreal functionality and best practices
- Install and configure angreal for the Metis project
- Create standard commands for development workflows
- Document angreal usage for developers
- Establish consistent command interface across the project

**Non-Goals:**
- Complex workflow automation or orchestration
- Integration with external CI/CD systems (separate initiative)
- Advanced angreal features beyond basic task automation

## Desired Command Interface

The goal is to have simple, consistent commands for common tasks:

```bash
# Development workflow
angreal test           # Run all tests across workspace
angreal coverage       # Generate coverage report using tarpaulin
angreal lint          # Run clippy and rustfmt
angreal build         # Build all crates in workspace
angreal clean         # Clean build artifacts

# Documentation
angreal docs          # Generate and view documentation

# Quality checks
angreal check         # Run comprehensive quality checks (test + lint + build)
```

## Implementation Plan

1. **Research Phase** - Study angreal documentation and examples
2. **Installation** - Install angreal and understand project setup
3. **Basic Configuration** - Initialize angreal in Metis project
4. **Core Commands** - Implement the desired command interface
5. **Testing** - Verify commands work correctly across workspace
6. **Documentation** - Document angreal usage for developers

## Exit Criteria

- [x] Angreal researched and understood
- [x] Angreal installed and project configured
- [x] Desired commands implemented and working
- [x] Commands properly handle workspace structure
- [x] Documentation explaining available commands
- [x] Consistent command interface established

## Completion Summary

### 2025-07-04 - Initiative Completed

**Objective**: Set up angreal task automation framework to provide consistent commands for common development tasks in the Metis project.

**Results Achieved**:
- **Angreal Setup**: Successfully created `.angreal` directory with task definitions
- **Command Implementation**: Implemented 5 essential development commands:
  - `angreal test` - Run all tests across workspace using cargo test
  - `angreal build` - Build all crates in workspace using cargo build
  - `angreal clean` - Clean build artifacts using cargo clean
  - `angreal coverage` - Generate coverage report using cargo tarpaulin
  - `angreal check` - Run comprehensive checks (clippy + format + check)
- **Workspace Integration**: All commands properly handle the multi-crate workspace structure
- **Error Handling**: Proper exit codes and error messages for CI/automation use
- **Documentation**: Commands include comprehensive `when_to_use` and `when_not_to_use` guidance

**Key Deliverables**:
1. **Task Definition File** (`.angreal/task_dev.py`) with all 5 commands
2. **Working Command Interface** - Verified all commands execute successfully
3. **Workspace Support** - Commands work across metis-core and metis-mcp-server crates
4. **Developer Documentation** - Built-in command descriptions and usage guidelines

**Impact**: Developers now have a consistent command interface (`angreal <command>`) for all common development tasks, reducing cognitive overhead and ensuring standardized workflows across the project.