# How to Use the Desktop GUI

The Metis desktop application provides a visual Kanban board interface for managing your Flight Levels documents. This guide covers the main workflows.

## Open a Project

Launch Metis from your Applications folder (macOS), start menu (Windows), or `~/.local/bin/metis` (Linux).

From the sidebar, click "Open Project" and navigate to a directory containing a `.metis/` folder. Recently opened projects appear in the sidebar for quick access.

## Navigate the Boards

The GUI provides seven board types, shown as tabs at the top:

### Vision Board
Columns: Draft → Review → Published

Shows your project vision document. The vision uses a full-page editor rather than cards.

### Initiative Board
Columns: Discovery → Design → Ready → Decompose → Active → Completed

Shows all initiatives as Kanban cards. Drag cards between columns to transition phases.

### Task Board
Columns: Blocked → Todo → Active → Completed

Shows tasks for a specific initiative. Use the initiative dropdown at the top to filter which tasks are displayed.

### Backlog Board
Columns: General → Bug → Feature → Tech Debt

Shows standalone tasks not assigned to any initiative. Tasks are categorized by their backlog tags.

### ADR Board
Columns: Draft → Discussion → Decided → Superseded

Shows architecture decision records.

### Specification Board
Table view (not Kanban) with columns: Short Code, Title, Phase, Actions.

### Strategy Board
Columns: Shaping → Design → Ready → Active → Completed

Only visible when strategies are enabled in configuration. This feature is currently reserved for future use.

## Drag and Drop

Drag cards between columns to transition phases. When you drop a card in a new column, the GUI:

1. Calls the backend to validate the transition
2. Updates the document's frontmatter
3. Syncs the database
4. Refreshes the board

Phase rules are enforced — you can only drop cards in adjacent columns.

## Create Documents

Click the "+" button or "Create" in the toolbar. A dialog appears asking for:

- **Document type** — Vision, Initiative, Task, ADR, Specification
- **Title** — Document name
- **Parent** — Required for initiatives (select a vision), tasks (select an initiative), and specifications (select a vision or initiative)
- **Complexity** — For initiatives: XS, S, M, L, XL
- **Backlog category** — For standalone tasks: Bug, Feature, Tech Debt

## View and Edit Documents

Click a card to open the Document Viewer modal. This shows:

- Full document content in a rich text editor
- Phase transition buttons
- Archive button
- Related documents

The editor supports:
- Bold, italic, strikethrough
- Headings (H1-H6)
- Bullet and numbered lists
- Blockquotes
- Tables (insert, modify, delete rows/columns)
- Horizontal rules
- Undo/redo

## Search

Use the search bar at the top to find documents by title or content. Results appear in a dropdown — click a result to open it in the Document Viewer.

Search is debounced (300ms delay) and performs full-text search across all non-archived documents.

## Change Themes

Click the theme toggle (or use the Settings menu) to cycle between three themes:

- **Light** — Warm off-white with sage accents ("Editorial")
- **Dark** — Deep black with blue undertones ("Midnight Observatory")
- **Hyper** — Pure black with neon accents ("Neon Cyberpunk")

Your theme choice is saved automatically between sessions.

## Sync with External Changes

If you edit documents outside the GUI (via CLI, text editor, or Claude Code), click the Refresh button to sync. The GUI runs `sync_project` which:

- Scans the `.metis/` directory for changes
- Updates the database to match the filesystem
- Reports imported, updated, deleted, and error counts

Most operations (create, transition, archive) auto-sync after completing.

## Archive Documents

Right-click a card or use the archive button in the Document Viewer. Archiving moves the document and all its children to `.metis/archived/`. Archived documents are hidden from boards by default.

## Install the CLI from the GUI

On first launch, the GUI offers to install the CLI binary. This copies the bundled `metis` binary to:

- macOS: `~/Library/Application Support/io.colliery.metis/bin/metis` (symlinked to `~/.local/bin/metis`)
- Linux: `~/.local/bin/metis`

You can also trigger this from Settings > Install CLI. The GUI tracks CLI version and offers upgrades when the GUI is updated.
