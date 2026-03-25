# MCP Tools Reference

The Metis MCP server exposes 10 tools via the Model Context Protocol (JSON-RPC over stdio). These tools are available to AI agents when the server is connected.

**Server binary:** `metis mcp`
**Transport:** stdio (JSON-RPC 2.0)
**Logs:** `.metis/metis-mcp-server.log`

All tools require a `project_path` parameter pointing to the `.metis` directory (e.g., `/path/to/project/.metis`).

---

## initialize_project

Create a new Metis workspace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path where `.metis/` will be created |
| `prefix` | string | no | Short code prefix, 2-8 uppercase ASCII letters. Default: `"PROJ"` |

**Hints:** idempotent, not destructive, not read-only

**Returns:** Table with Metis Directory, Database path, Vision path, Project Prefix.

**Notes:**
- Creates `.metis/` directory, SQLite database, vision document, config.toml, and .gitignore
- Safe to re-run on existing workspace
- Prefix is uppercased automatically

---

## list_documents

List all documents with optional archive filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `include_archived` | boolean | no | Include archived documents. Default: `false` |

**Hints:** idempotent, read-only

**Returns:** Table with columns: Type, Code, Title, Phase. Sorted by type (vision → specification → initiative → task → adr), then by short code.

**Notes:**
- Auto-syncs workspace before listing
- Queries all document types from database

---

## search_documents

Full-text search across document content, titles, and types.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `query` | string | yes | Search query text |
| `document_type` | string | no | Filter by type: `vision`, `initiative`, `task`, `adr`, `specification` |
| `limit` | u32 | no | Maximum number of results |
| `include_archived` | boolean | no | Include archived documents. Default: `false` |

**Hints:** idempotent, read-only

**Returns:** Header with result count, table with columns: Code, Title, Type.

**Notes:**
- Uses SQLite FTS5 full-text search with Porter stemmer
- Special characters in queries are automatically quoted to prevent FTS5 syntax errors
- Short queries (2 characters or fewer) are wrapped in quotes

---

## read_document

Read a document's full content and metadata.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `short_code` | string | yes | Document identifier (e.g., `PROJ-V-0001`) |

**Hints:** idempotent, read-only

**Returns:** Header with `{short_code}: {title} ({type}, {phase})`, followed by full markdown content.

**Notes:**
- Resolves short code to filesystem path via database
- Reads raw file content
- Extracts metadata from YAML frontmatter: type, phase, created_at, archived, title

---

## create_document

Create a new Metis document.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `document_type` | string | yes | `vision`, `initiative`, `task`, `adr`, `specification` |
| `title` | string | yes | Document title |
| `parent_id` | string | no | Parent short code. Required for initiative, task (non-backlog), specification |
| `complexity` | string | no | For initiatives: `xs`, `s`, `m`, `l`, `xl` |
| `stakeholders` | string[] | no | List of stakeholder names |
| `decision_maker` | string | no | For ADRs: who made the decision |
| `backlog_category` | string | no | For standalone tasks: `bug`, `feature`, `tech-debt` (also accepts `techdebt`, `tech_debt`) |

**Hints:** not idempotent, not destructive, not read-only

**Returns:** Success message with short code, table with: Title, Type, Short Code, Parent, Path.

**Validation rules:**
- Document type must be enabled in current flight level configuration
- Vision documents cannot have a parent
- Tasks:
  - With `backlog_category` → creates backlog item (no parent required)
  - With `parent_id` → creates task under initiative
  - Without parent in streamlined config → error (must specify parent or backlog_category)
  - In direct config → creates task with no parent
- Initiative requires vision parent
- Specification requires vision or initiative parent

---

## edit_document

Perform search-and-replace edits on document content.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `short_code` | string | yes | Document identifier |
| `search` | string | yes | Text to find |
| `replace` | string | yes | Replacement text |
| `replace_all` | boolean | no | Replace all occurrences. Default: `false` (first only) |

**Hints:** not idempotent, not destructive, not read-only

**Returns:** Success message with replacement count and diff block showing changes.

**Notes:**
- Returns error if search text is not found
- When `replace_all` is false, only the first occurrence is replaced

---

## transition_phase

Move a document to a new phase.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `short_code` | string | yes | Document identifier |
| `phase` | string | no | Target phase name. If omitted, auto-advances to next phase |
| `force` | boolean | no | Force transition bypassing validation. Default: `false` |

**Hints:** not idempotent, not destructive, not read-only

**Returns:** Header "Phase Transition", text showing `{short_code}: {from} -> {to}`, phase progress indicator.

**Valid phase names:** `draft`, `review`, `published`, `discussion`, `decided`, `superseded`, `backlog`, `todo`, `active`, `blocked`, `completed`, `design`, `ready`, `decompose`, `discovery`, `drafting`

**Rules:**
- Only adjacent transitions are valid (cannot skip phases)
- Phase names are case-insensitive
- Omitting `phase` auto-advances to the next sequential phase

See [Phase Lifecycle Reference](./phase-lifecycle.md) for valid transitions per document type.

---

## archive_document

Archive a document and all its children.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `short_code` | string | yes | Document identifier |

**Hints:** idempotent, destructive, not read-only

**Returns:** Success message with count of archived documents.

**Notes:**
- Moves document files to `.metis/archived/`
- Archives all child documents recursively
- Returns error if document is already archived
- Updates database to mark documents as archived

---

## reassign_parent

Move a task to a different parent initiative or to/from the backlog.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `short_code` | string | yes | Task short code |
| `new_parent_id` | string | no | Target initiative short code. Omit to move to backlog |
| `backlog_category` | string | no | Required when moving to backlog: `bug`, `feature`, `tech-debt` |

**Hints:** not idempotent, not destructive, not read-only

**Returns:** Success message with new assignment and file path.

**Rules:**
- Only tasks can be reassigned
- Target initiative must be in `decompose` or `active` phase
- When moving to backlog, `backlog_category` is required

---

## index_code

Generate a code index for AI agent navigation.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_path` | string | yes | Path to `.metis` folder |
| `structure_only` | boolean | no | Skip symbol extraction. Default: `false` |
| `incremental` | boolean | no | Only re-index changed files. Default: `false` |

**Hints:** idempotent, not destructive, not read-only

**Returns:** Table with: Files indexed, Symbols extracted, Time, Output path, Parse errors. Subheader "Languages Detected" with language and file count.

**Supported languages:** Rust, Python, TypeScript, JavaScript, Go

**Notes:**
- Writes output to `.metis/code-index.md`
- Incremental mode uses `.metis/code-index-hashes.json` and `.metis/code-index-symbols.json`
- Preserves existing AI-authored semantic summaries
- Skips `target/`, `node_modules/`, `__pycache__/`, `.git/`, and other build directories
