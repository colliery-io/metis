---
id: external-document-viewer
level: initiative
title: "External Document Viewer Integration"
short_code: "METIS-I-0027"
created_at: 2026-03-26T14:16:23.728984+00:00
updated_at: 2026-03-26T14:58:49.641711+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: L
initiative_id: external-document-viewer
---

# External Document Viewer Integration Initiative

## Context

Reviewing and editing Metis documents through MCP tool calls (read_document, edit_document) is functional but clunky. Users see raw tool output rather than rendered markdown, and editing requires dictating search/replace operations to an agent, manually opening text files, or remembering to ask. This friction discourages document review during active work — the moment when review is most valuable.

Both the Metis Tauri GUI (with tiptap markdown editing) and VSCode already handle markdown rendering and editing well. Rather than building yet another viewer, this initiative connects Metis to these existing tools so agents and users can pop open documents in a proper editor during workflow.

Since all three systems (Metis MCP, VSCode, Tauri GUI) read and write directly to disk, bidirectional sync is effectively free — no sync protocol needed. Convention (don't edit the same file simultaneously as the agent) is sufficient to avoid conflicts for now.

## Goals & Non-Goals

**Goals:**
- Enable agents to open Metis documents in an external viewer/editor during workflow (e.g., "open this ticket for review")
- Support opening multiple related documents at once (initiative + child tasks for decomposition review)
- Integrate with VSCode as a primary viewer/editor path 
    - investigate fall back to system configured default editor as well. May need to force opening a new terminal to do this though. 
- ~~GUI work moved to METIS-I-0028 (Metis GUI Improvements)~~
- Add a `default_viewer` configuration to arawn.toml so users can direct behavior to their preferred tool
    - if the default system editor is a viable path also, make that the default behavior when not configured
- Keep the architecture open so additional viewers could be added later

**Non-Goals:**
- Real-time collaborative editing or conflict resolution — convention handles this
- Building a new editor inside the MCP server or Claude Code
- Replacing the MCP read/edit tools — those remain the programmatic interface for agents
- Enterprise SSO or multi-user access control for the viewer

**Design Notes**:
- The MCP server must enforce a read-before-edit guard: track `last_read` timestamps per document and compare against file `mtime` on edit. If the file has changed since the agent's last read, reject the edit and require a fresh read. This prevents agents from silently overwriting user changes made in external viewers.

## Use Cases

### UC-1: Agent Opens Tickets During Decomposition
- **Actor**: AI agent performing initiative decomposition
- **Scenario**: Agent decomposes METIS-I-0027 into tasks. After creating tasks, agent calls `open_document` with the initiative short code and `include_children: true`. The configured viewer opens with the initiative and all child tasks available for review.
- **Expected Outcome**: User can read through the full decomposition in rendered markdown, make edits directly, and confirm with the agent when satisfied.

### UC-2: User Requests Document Review
- **Actor**: User working with an agent
- **Scenario**: User says "open this ticket for review in metis." Agent calls `open_document` with the short code. The document opens in the user's configured default viewer (VSCode or Metis GUI).
- **Expected Outcome**: Document opens in the preferred viewer, rendered and editable.

### UC-3: Agent Working on Task Opens It for Visibility
- **Actor**: AI agent actively working a task
- **Scenario**: Agent transitions a task to active and opens it in the viewer so the user can follow along as the agent updates progress in the document.
- **Expected Outcome**: User sees a live view of the document that reflects on-disk changes as the agent writes them.

### UC-4: User Edits in Viewer, Agent Reads Back
- **Actor**: User editing in VSCode/GUI, then returning to agent conversation
- **Scenario**: User opens a document, makes edits in the viewer, saves. Returns to the agent and says "I've updated the requirements, take a look." Agent calls `read_document` and sees the updated content.
- **Expected Outcome**: Agent picks up changes seamlessly since both read from disk.

### UC-5: Agent Auto-Opens After Creation/Edit
- **Actor**: AI agent creating or editing documents
- **Scenario**: Agent creates a new initiative via `create_document`. The MCP server automatically opens it in the configured viewer. Agent tells the user "I've opened the initiative in VSCode — review the content and let me know when you're ready to proceed." Agent waits for confirmation, then re-reads the document to pick up any user edits before continuing.
- **Expected Outcome**: Every create/edit naturally flows into a review checkpoint without the user having to ask. The agent knows to pause and re-read after the user confirms.

## Architecture

### Overview

The design introduces a **viewer abstraction layer** in the MCP server that dispatches open requests to the configured viewer backend. The MCP server gains a new `open_document` tool that resolves a short code to a file path (or set of paths for parent+children), then delegates to the appropriate viewer.

```
Agent/User
    │
    ▼
MCP Server (open_document tool)
    │
    ├── reads `default_viewer` from arawn.toml
    │
    ├──► VSCode Backend
    │      └── `code --goto <file>` / VSCode URI protocol
    │
    └──► Metis GUI Backend
           └── deep-link URL or IPC command to navigate to document
```

All viewers read/write the same markdown files on disk. No sync protocol needed.

### Key Components

1. **`open_document` MCP tool** — New tool in metis-mcp. Accepts short code(s), resolves to file paths, dispatches to configured viewer. Supports `include_children: true` to open an initiative with all its tasks.

2. **Viewer trait/abstraction** — A `DocumentViewer` trait with `open(&self, paths: &[PathBuf])` that each backend implements. Makes it straightforward to add new viewers later.

3. **`default_viewer` config** — New field in arawn.toml: `default_viewer = "sys_editor" | "code" | "gui"`. Falls back to `$EDITOR` if not configured. No `auto` mode — explicit choice or env var fallback.

4. **VSCode backend** — Shells out to `code` CLI to open files. For multi-file opens, could use a workspace or just open each file. May also explore the VSCode URI scheme for richer integration.

5. **GUI backend** — Launches the Tauri app (if not running) and navigates to the document. Requires the GUI to accept a deep-link or CLI argument for navigation. Also requires replacing the manual refresh button with a file watcher (notify crate) so edits appear immediately.

## Detailed Design

### MCP Tool: `open_document`

```
open_document:
  short_code: string        # e.g., "METIS-I-0027"
  include_children: bool    # open child tasks alongside parent (default: false)
  viewer: string?           # override default_viewer for this call
```

Resolution logic:
1. Resolve short_code → file path via existing document index
2. If `include_children`, gather child document paths
3. Read `viewer` param or fall back to `default_viewer` from config
4. Dispatch to viewer backend

### VSCode Integration

- Primary mechanism: `code --goto <file>:<line>` for single files, `code <file1> <file2> ...` for multi-file
- For richer integration, explore `vscode://file/<path>` URI scheme
- Consider opening files in a specific VSCode window/workspace if one is already associated with the project

### GUI Integration

- **File watching**: Replace manual refresh with `notify`-based file watcher on the `.metis` directory. On change events, the GUI reloads affected documents automatically.
- **Deep linking via custom URL scheme**: The Tauri app registers a `metis://` URL scheme (e.g., `metis://open/METIS-I-0027`). This is the primary mechanism for the GUI backend to navigate to a document. Tauri supports custom protocol registration via its deep-link plugin. The URL scheme handles both "app not running" (launches and navigates) and "app already running" (brings to front and navigates) cases.
  - Format: `metis://open/<short-code>` — opens a single document
  - Format: `metis://open/<short-code>?children=true` — accepted but GUI only opens the parent document (known limitation: one document at a time)
  - **Research needed**: Tauri deep-link plugin specifics, platform differences (macOS `Info.plist` vs Linux `.desktop` file registration), and how to handle the case where multiple `.metis` projects exist.
- **Launch management**: Handled by the OS URL scheme dispatch — if the app is running, the OS routes to the existing instance. No custom IPC needed.

### Read-Before-Edit Guard

The MCP server maintains an in-memory `HashMap<ShortCode, SystemTime>` tracking the last time each document was read via `read_document`. On every `edit_document` call:

1. Check if the short code has a `last_read` entry — if not, reject ("must read before editing")
2. Stat the file's `mtime` — if `mtime > last_read`, reject with an error message indicating the file was modified externally since the last read, and instruct the agent to re-read
3. On successful edit, update `last_read` to current time

This is lightweight (no persistence needed — the map lives for the MCP server session) and catches the primary failure mode: agent overwrites changes the user just made in a viewer.

### Proactive Opening Behavior

The MCP server proactively opens documents in the viewer after `create_document` and `edit_document` calls, unless suppressed. This makes review a default part of the workflow — agents don't need to remember to call `open_document` separately.

The server uses a "look before you leap" approach: before dispatching to a viewer backend, it checks whether the document is already open. Each backend implements its own detection (e.g., VSCode: check if the file is in an active editor tab; GUI: query app state; sys_editor: track PIDs or session state). If already open, the server skips the open and returns success. This prevents tab/window sprawl from repeated create/edit cycles.

### Configuration (arawn.toml)

```toml
[viewer]
default = "code"                        # "sys_editor" | "code" | "gui" — falls back to $EDITOR if not set
suppress_proactive_ticket_opening = false  # set to true to disable auto-open on create/edit
```



## Alternatives Considered

### 1. Build a viewer into the Claude Code plugin itself
Rejected because it would duplicate effort — VSCode and the Tauri GUI already render and edit markdown well. The MCP read/edit tools remain available for programmatic access. Adding a third rendering surface inside the agent conversation doesn't improve the experience.

### 2. Web-based viewer (localhost HTTP server)
Considered a standalone web server that renders documents in a browser. Viable but adds another moving part (server process, port management) when VSCode and the GUI already exist. Could revisit if there's demand for a browser-based workflow.

### 3. Sync protocol between viewers
Rejected. Since all tools read/write the same files on disk, a sync protocol adds complexity for no gain. Convention (one editor at a time per document) is sufficient.

## Implementation Plan

### Phase 1: Foundation
- Add `[viewer]` config section to arawn.toml schema and parsing
- Define the `DocumentViewer` trait and dispatcher
- Implement the `open_document` MCP tool with short code → path resolution and `include_children` support
- Implement read-before-edit guard: `last_read` timestamp tracking per document in the MCP server, `mtime` comparison on `edit_document`, reject with error + require re-read if stale

### Phase 2: VSCode Backend
- Implement VSCode viewer backend using `code` CLI
- Handle single and multi-file opens
- Test with various VSCode configurations (workspaces, remote, etc.)

### ~~Phase 3 & 4: GUI work — moved to METIS-I-0028~~

### Phase 5: Plugin & Workflow Integration
- Update MCP tools (`create_document`, `edit_document`) to auto-open documents in the viewer after creation/edit if not already open
- Update Claude Code plugin skills (Ralph loop, decomposition) to open documents at natural review points and prompt the user to review externally before proceeding
- Update agent system prompts / skill instructions to make the agent aware of the viewer workflow: open for review → wait for user confirmation → re-read before continuing
- Ensure human-in-the-loop checkpoints leverage the viewer (e.g., "I've opened the initiative in VSCode — review and let me know when you're ready to proceed")
- Document the feature and configuration