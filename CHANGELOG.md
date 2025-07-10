# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-07-10

### Added
- Complete migration from `metis/` to `.metis/` directory structure
- Direct CLI access to TUI and MCP server via `metis tui` and `metis mcp` commands
- Comprehensive MCP server with 11 tools for AI assistant integration:
  - `initialize_project` - Set up new Metis projects
  - `create_document` - Create documents at any level
  - `validate_document` - Validate document structure
  - `update_document_content` - Update document sections
  - `update_exit_criterion` - Manage exit criteria
  - `update_blocked_by` - Handle dependencies
  - `transition_phase` - Move documents through workflows
  - `check_phase_transition` - Validate phase transitions
  - `validate_exit_criteria` - Check completion status
  - `list_documents` - Find documents
  - `search_documents` - Full-text search
- Standardized document front matter with required fields (title, archived)
- Comprehensive functional testing for MCP endpoints
- Fallback versions for local crate dependencies

### Changed
- Migrated all project documentation to new `.metis/` structure
- Updated all document front matter to include standardized metadata
- Fixed validation issues with tags, phase transitions, and missing fields
- Improved GitHub Actions workflows for correct crate publishing
- Enhanced workspace configuration with proper metadata

### Fixed
- Document validation errors across all migrated files
- Gitignore typos and missing coverage report entries
- GitHub workflow crate paths for proper publishing
- Phase tag inconsistencies in task documents
- Missing required fields in document front matter

## [0.1.0] - 2025-07-05

### Added
- Initial release with core Metis functionality
- Basic CLI interface for document management
- TUI (Terminal User Interface) for interactive document management
- Core library with Flight Levels methodology implementation
- Document templates for Vision, Strategy, Initiative, Task, and ADR
- SQLite database with full-text search capabilities
- Phase transition management and validation
- Exit criteria tracking and validation
- Basic MCP server implementation