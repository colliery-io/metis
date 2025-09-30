# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2025-01-30

### Added
- Shared workspace detection service in core for consistent behavior across all crates
- Constants module with centralized configuration values and magic strings
- Error conversion utilities with user-friendly messaging and categorization
- Shared test utilities framework with optional `test-utils` feature
- Comprehensive error context traits for better error handling

### Changed
- **BREAKING**: Consolidated workspace detection logic - all crates now use core service
- Standardized error handling patterns across TUI, CLI, and MCP crates
- Unified test infrastructure reduces boilerplate by ~60%
- Improved code maintainability with ~30% reduction in duplication

### Fixed
- Eliminated duplicate workspace finding implementations
- Consistent `.metis` directory detection and validation
- Standardized error messaging across all interfaces

## [0.2.6] - 2025-07-31

### Changed
- Version bump for release preparation

## [0.2.5] - 2025-07-30

### Added
- Comprehensive regression tests for ID/path consistency bug
- Archive cascade tests for both TUI and MCP interfaces
- Directory merging capability for archive operations

### Fixed
- **CRITICAL**: TUI workspace discovery now recursively searches parent directories (like Git)
- **CRITICAL**: "Directory not empty" error when archiving initiatives with tasks
- Archive process now handles existing archive directories by merging contents
- Archive cascading for strategies/initiatives preserves directory structure
- All child documents are marked as archived before parent directory moves

### Changed
- TUI can now be started from any subdirectory within a Metis workspace
- Archive operations are more robust with proper error handling

## [0.2.4] - 2025-07-29

### Added
- Automated testing infrastructure for critical functionality
- Comprehensive test coverage improvements

### Fixed
- Unicode character boundary issues in document ID generation
- Complexity validation to accept all valid values (XS, S, M, L, XL)
- ID/path mismatch between database records and filesystem structure

## [0.2.3] - 2025-07-28

### Added
- Visual feedback for TUI sync operation: Shows "⟳ Syncing database..." when 'y' is pressed
- TUI status bar now displays 'y: Sync' key binding for better discoverability

### Fixed
- Documentation inaccuracies: Corrected CLI command syntax, removed deleted tools, added missing features
- TUI key bindings now properly documented in status bar

## [0.2.2] - 2025-07-28

### Added
- TUI sync functionality: Press 'Y' to sync database and reload boards
- `archive_document` tool to MCP server for programmatic archiving
- Document move detection in sync process to handle relocated files

### Fixed
- **CRITICAL**: Database constraint violations when syncing after archiving documents
- Archived documents no longer appear in TUI board displays
- Sync process now properly handles document moves without ID conflicts
- CLI sync command displays moved document operations with "↻ Moved" indicator

### Changed
- Enhanced sync logic to detect and handle document relocations (e.g., to archived folder)
- Improved error handling for archive operations in all interfaces

## [0.2.1] - 2025-07-15

### Added
- TUI archive functionality: Press 'r' to archive selected documents

### Changed
- **BREAKING**: MCP server tool argument names updated for consistency with CLI/TUI:
  - `create_document`: Changed `parent_title` to `parent_id` to match CLI/TUI patterns
  - `transition_phase`: Changed `new_phase` to `phase` (optional) with auto-transition support
- Removed `check_phase_transition` tool as redundant with `transition_phase`
- Removed `hello_world` tool to streamline the API surface

### Fixed
- MCP tool arguments now consistently use document IDs instead of titles for parent references
- Auto-transition logic added to `transition_phase` tool when phase is not specified
- TUI document sorting comparison operator to implement proper total ordering

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