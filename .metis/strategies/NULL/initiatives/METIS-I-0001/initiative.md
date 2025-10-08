---
id: multi-project-gui-application
level: initiative
title: "Multi-Project GUI Application"
short_code: "METIS-I-0001"
created_at: 2025-10-08T01:05:01.962967+00:00
updated_at: 2025-10-08T14:47:16.738625+00:00
parent: metis
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: multi-project-gui-application
---

# Multi-Project GUI Application Initiative

## Context **[REQUIRED]**

Current Metis interfaces (CLI and TUI) are designed for single-project management, requiring users to switch contexts when working across multiple projects. As Metis adoption grows, users increasingly need to manage multiple related projects or compare work across different Metis workspaces. The existing TUI provides excellent functionality but lacks modern GUI conveniences expected in desktop applications. This initiative addresses the gap by creating a unified desktop GUI that maintains the familiar Metis workflow while adding multi-project support and enhanced usability features.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Create a Tauri-based desktop GUI for multi-project Metis management
- Achieve feature parity with existing TUI functionality
- Enable seamless switching between multiple projects in a single interface
- Implement drag-and-drop document organization between phases
- Provide rich text editing via Tiptap integration
- Support multiple themes (dark/light/hyper) with persistent preferences
- Integrate directly with metis-docs-core for optimal performance

**Non-Goals:**
- Project workspace or grouping concepts (future consideration)
- Cross-project reporting or comparison features
- Web-based or browser interface
- Mobile application support
- Real-time collaboration features

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

### Functional Requirements
- **REQ-001**: Application shall provide file picker dialog to add new Metis projects (.metis directories)
- **REQ-002**: Application shall display list of configured projects in sidebar for easy switching
- **REQ-003**: Application shall validate project directories contain valid .metis structure and metis.db
- **REQ-004**: Application shall display documents organized by type (Vision, Initiative, Task, ADR) in separate board views
- **REQ-005**: Application shall organize documents within boards by phase in column layout (matching TUI behavior)
- **REQ-006**: Application shall provide document creation functionality with appropriate parent-child relationships
- **REQ-007**: Application shall integrate Tiptap rich text editor for document content editing with markdown support
- **REQ-008**: Application shall handle YAML frontmatter parsing and editing within document editor
- **REQ-009**: Application shall support drag-and-drop document movement between phases with validation
- **REQ-010**: Application shall support drag-and-drop document reordering within phases
- **REQ-011**: Application shall provide theme switching between dark, light, and hyper themes
- **REQ-012**: Application shall persist user preferences (projects, theme) between sessions
- **REQ-013**: Application shall auto-save document changes with debounced updates
- **REQ-014**: Application shall provide document deletion and archiving capabilities
- **REQ-015**: Application shall integrate directly with metis-docs-core library (not CLI wrapper)

### Platform Requirements
- **PLAT-001**: Application shall compile to single binary executable for macOS
- **PLAT-002**: Application shall compile to single binary executable for Linux
- **PLAT-003**: Application shall compile to single binary executable for Windows (if Tauri supports)

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

### Use Case 1: Multi-Project Management
- **Actor**: Project Manager or Developer
- **Scenario**: 
  1. Launch GUI application
  2. Click "Add Project" button to open file picker
  3. Navigate to and select .metis directory
  4. Project appears in sidebar with project name
  5. Switch between projects via sidebar selection
  6. Remove projects via context menu or settings
- **Expected Outcome**: Seamless management of multiple Metis projects in single interface

### Use Case 2: Document Lifecycle Management
- **Actor**: Project Manager or Developer
- **Scenario**:
  1. Select project from sidebar
  2. Choose document type board (Vision, Initiative, Task, ADR)
  3. View documents organized in phase-based columns
  4. Click document to open Tiptap editor
  5. Edit document content with rich text features
  6. Save changes automatically
  7. Drag document between phase columns to transition
- **Expected Outcome**: Intuitive document management with visual feedback and smooth transitions

### Use Case 3: Document Creation and Organization
- **Actor**: Project Manager or Developer
- **Scenario**:
  1. Navigate to appropriate board (Initiative, Task, etc.)
  2. Click "Create Document" button
  3. Fill in document details in creation dialog
  4. Document appears in appropriate phase column
  5. Drag documents to reorder within phases
  6. Use drag-and-drop to move between phases
- **Expected Outcome**: Efficient document creation with immediate visual organization

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

### Overview
Tauri-based desktop application with Rust backend and TypeScript/React frontend. The architecture follows Tauri's recommended patterns with the frontend handling UI/UX and the backend providing secure access to metis-docs-core functionality.

### Component Architecture
- **Frontend Layer**: React/TypeScript SPA with component-based architecture
  - Project Manager: Sidebar with project list and switching logic
  - Board View: Kanban-style layout with phase-based columns
  - Document Editor: Tiptap integration with markdown support
  - Theme Manager: CSS-in-JS with theme switching
  - Settings Manager: User preferences and project configuration

- **Backend Layer**: Rust Tauri commands exposing core functionality
  - Project Service: Multi-project management and configuration
  - Document Service: CRUD operations via metis-docs-core
  - File Service: Project discovery and validation
  - Settings Service: User preference persistence

- **Data Layer**: 
  - metis-docs-core integration for document operations
  - SQLite databases (one per project)
  - Application settings in platform-appropriate location

### Key Interactions
1. **Project Loading**: Frontend requests project list → Backend scans configured paths → Returns project metadata
2. **Document Operations**: Frontend triggers document action → Backend calls metis-docs-core → Database updated → Frontend refreshed
3. **Drag-and-Drop**: Frontend handles drag events → Backend validates transition → Core library updates document phase

## Detailed Design **[REQUIRED]**

**Technology Stack:**
- **Framework**: Tauri v1.x with Rust backend and React frontend
- **Frontend**: React 18+ with TypeScript, CSS-in-JS (styled-components or emotion)
- **Editor**: Tiptap v2 with markdown extensions and YAML frontmatter handling
- **Drag-and-Drop**: React DnD or @dnd-kit for smooth interactions
- **State Management**: React Context/useReducer or Zustand for application state
- **Build Tools**: Vite for frontend bundling, integrated with Tauri build process

**Core Implementation Details:**

1. **Project Management System**:
   - Application settings stored in platform-appropriate directory (AppData/Application Support/config)
   - Project list persisted as JSON with project paths and metadata
   - File picker integration using Tauri's dialog API
   - Project validation by checking for .metis directory and metis.db

2. **Document Visualization**:
   - Kanban board layout with CSS Grid for responsive column sizing
   - Phase-based columns matching current TUI behavior (draft, review, published, etc.)
   - Document cards showing title, short code, and key metadata
   - Color coding by document type and phase status

3. **Rich Text Editor Integration**:
   - Tiptap editor with custom markdown parser for YAML frontmatter
   - Live preview mode for markdown rendering
   - Custom extensions for Metis-specific syntax and document structure
   - Auto-save functionality with debounced updates to backend

4. **Drag-and-Drop Implementation**:
   - Document cards as draggable items with visual feedback
   - Phase columns as drop zones with hover states
   - Phase transition validation through metis-docs-core before allowing drops
   - Smooth animations and error handling for invalid moves

5. **Theme System**:
   - CSS custom properties for theme switching
   - Three themes: light (default), dark, and hyper (high contrast)
   - Theme preference persisted in application settings
   - Immediate theme switching without restart required

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

The GUI will follow the existing TUI layout patterns and workflow, providing familiar interaction patterns for current Metis users while adding modern GUI conveniences. The TUI serves as the effective "mockup" for the basic layout and organization.

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

**Primary Success Criteria**: "Does it work and does it make me happy"

### Cross-Platform Testing
- Manual testing on macOS, Linux, and Windows (if supported by Tauri)
- Verify single binary compilation and execution on each platform
- Test file picker and native OS integration points

### Core Functionality Validation
- All functional requirements (REQ-001 through REQ-015) working as specified
- Integration with metis-docs-core behaves identically to TUI
- Drag-and-drop operations don't corrupt data or state

### User Experience Validation
- Application feels responsive and intuitive
- Theme switching works smoothly
- Document editing workflow matches or improves upon TUI experience

## Alternatives Considered **[REQUIRED]**

1. **Electron-based Application**: Rejected due to larger bundle size and resource usage compared to Tauri
2. **Web Application with Local Server**: Rejected due to complexity of local server management and browser dependency
3. **Native Desktop (Qt/GTK)**: Rejected due to UI development complexity and limited cross-platform consistency
4. **Extending TUI Interface**: Rejected as it doesn't address multi-project requirement and lacks modern GUI capabilities
5. **VS Code Extension**: Rejected as it ties functionality to specific editor and doesn't provide standalone experience

## Implementation Plan **[REQUIRED]**

**Phase 1: Foundation (Weeks 1-2)**
- Set up Tauri project structure in metis-docs-gui crate
- Configure build pipeline and development environment
- Implement basic application shell with project sidebar
- Add project management (add/remove/switch) with file picker

**Phase 2: Core Integration (Weeks 3-4)**
- Integrate metis-docs-core into Tauri backend
- Implement project loading and validation
- Create basic document listing and board structure
- Add document type navigation (Vision, Initiative, Task, ADR)

**Phase 3: Document Visualization (Weeks 5-6)**
- Implement kanban board layout with phase columns
- Create document cards with metadata display
- Add document creation and basic CRUD operations
- Implement theme system with dark/light/hyper options

**Phase 4: Rich Text Editor (Weeks 7-8)**
- Integrate Tiptap editor with markdown support
- Handle YAML frontmatter parsing and editing
- Implement auto-save functionality
- Add live preview capabilities

**Phase 5: Drag-and-Drop (Weeks 9-10)**
- Implement drag-and-drop for document organization
- Add phase transition via drag-and-drop
- Implement visual feedback and animations
- Add error handling for invalid transitions

**Phase 6: Cross-Platform Polish (Weeks 11-12)**
- Cross-platform compilation testing (macOS, Linux, Windows)
- Single binary distribution setup for each platform
- Platform-specific integration testing (file picker, native dialogs)
- Bug fixes and final polish
- Basic user documentation