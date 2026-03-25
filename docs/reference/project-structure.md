# Project Structure Reference

This document describes the filesystem layout, database schema, and document conventions used by Metis.

## Workspace Layout

A Metis workspace is a `.metis/` directory at the root of your project:

```
my-project/
├── .metis/
│   ├── config.toml                  # Project configuration
│   ├── metis.db                     # SQLite database (gitignored)
│   ├── metis-mcp-server.log         # MCP server logs (gitignored)
│   ├── .gitignore                   # Auto-generated ignore rules
│   ├── vision.md                    # Vision document
│   ├── initiatives/
│   │   ├── PROJ-I-0001/
│   │   │   ├── initiative.md
│   │   │   └── tasks/
│   │   │       ├── PROJ-T-0001.md
│   │   │       └── PROJ-T-0002.md
│   │   └── PROJ-I-0002/
│   │       ├── initiative.md
│   │       └── tasks/
│   ├── backlog/
│   │   ├── PROJ-T-0010.md
│   │   ├── bugs/
│   │   │   └── PROJ-T-0011.md
│   │   ├── features/
│   │   └── tech-debt/
│   ├── adrs/
│   │   ├── PROJ-A-0001.md
│   │   └── PROJ-A-0002.md
│   ├── specifications/
│   │   └── PROJ-S-0001/
│   │       └── specification.md
│   ├── archived/
│   │   └── (archived documents moved here)
│   ├── code-index.md                # Generated code index (gitignored)
│   ├── code-index-hashes.json       # File hashes for incremental indexing (gitignored)
│   └── code-index-symbols.json      # Cached symbols (gitignored)
└── (your project files)
```

## Document File Structure

Every document is a Markdown file with YAML frontmatter:

```markdown
---
id: implement-authentication
level: task
status: active
created_at: "2026-03-25T14:00:00Z"
updated_at: "2026-03-25T14:00:00Z"
parent: "[[build-core-api]]"
blocked_by: []
tags:
  - "#task"
  - "#phase/todo"
exit_criteria_met: false
short_code: "PROJ-T-0001"
archived: false
initiative_id: "build-core-api"
---

# Implement Authentication

## Objective
...
```

### Frontmatter Fields (All Document Types)

| Field | Type | Description |
|-------|------|-------------|
| `id` | String | Title-derived slug (max 35 chars). Generated from `title_to_slug()`. |
| `level` | String | Document type: `vision`, `initiative`, `task`, `adr`, `specification` |
| `status` | String | Always `"active"` for non-archived documents |
| `created_at` | ISO 8601 | Creation timestamp |
| `updated_at` | ISO 8601 | Last modification timestamp |
| `parent` | String or null | Wiki-link to parent document ID: `"[[parent-id]]"` |
| `blocked_by` | String[] | List of document IDs blocking this document |
| `tags` | String[] | Phase tags (`#phase/active`) and label tags (`#initiative`, `#bug`) |
| `exit_criteria_met` | Boolean | Whether acceptance criteria are complete |
| `short_code` | String | Unique identifier: `PREFIX-TYPE-NNNN` |
| `archived` | Boolean | Whether the document is archived |
| `initiative_id` | String or null | Initiative this document belongs to (for lineage tracking) |

### Additional Fields by Type

**Initiative:**
| Field | Type | Description |
|-------|------|-------------|
| `estimated_complexity` | String | `xs`, `s`, `m`, `l`, `xl` |
| `technical_lead` | String | Lead engineer |
| `related_adrs` | String[] | References to related ADRs |

**ADR:**
| Field | Type | Description |
|-------|------|-------------|
| `decision_maker` | String | Who made the decision |
| `decision_date` | ISO 8601 or null | When the decision was made |

## ID Generation

Document IDs are deterministically generated from the title:

1. Title is lowercased
2. Non-alphanumeric characters removed
3. Words joined with hyphens
4. Truncated to 35 characters

Example: `"Implement User Authentication"` → `"implement-user-authentication"`

## Database Schema

### documents

Primary table storing all document metadata.

| Column | Type | Description |
|--------|------|-------------|
| `filepath` | TEXT (PK) | Relative file path from workspace root |
| `id` | TEXT | Title-derived ID |
| `title` | TEXT | Document title |
| `document_type` | TEXT | `vision`, `initiative`, `task`, `adr`, `specification` |
| `created_at` | REAL | Unix timestamp (float64) |
| `updated_at` | REAL | Unix timestamp (float64) |
| `archived` | BOOLEAN | Archive status |
| `exit_criteria_met` | BOOLEAN | Acceptance criteria status |
| `file_hash` | TEXT | Content hash for change detection |
| `frontmatter_json` | TEXT | Parsed YAML frontmatter as JSON |
| `content` | TEXT | Document body (without frontmatter) |
| `phase` | TEXT | Current phase string |
| `initiative_id` | TEXT | Lineage tracking |
| `short_code` | TEXT | Unique identifier |
| `parent_id` | TEXT | Parent document ID |

### document_relationships

Parent-child links between documents.

| Column | Type | Description |
|--------|------|-------------|
| `child_id` | TEXT | Child document ID |
| `parent_id` | TEXT | Parent document ID |
| `child_filepath` | TEXT | Child file path |
| `parent_filepath` | TEXT | Parent file path |

Primary key: `(child_filepath, parent_filepath)`

### document_tags

Tags associated with documents.

| Column | Type | Description |
|--------|------|-------------|
| `document_filepath` | TEXT | Document file path |
| `tag` | TEXT | Tag text (e.g., `#phase/active`, `#bug`) |

Primary key: `(document_filepath, tag)`

### document_search

FTS5 virtual table for full-text search.

| Column | Type | Description |
|--------|------|-------------|
| `document_filepath` | TEXT (UNINDEXED) | Reference to document |
| `content` | TEXT | Indexed body text |
| `title` | TEXT | Indexed title |
| `document_type` | TEXT | Indexed type |

Tokenizer: `porter unicode61` (English stemming with Unicode support)

### configuration

Key-value configuration store.

| Column | Type | Description |
|--------|------|-------------|
| `key` | TEXT (PK) | Configuration name |
| `value` | TEXT | Configuration value |
| `updated_at` | REAL | Last update timestamp |

## Migration History

| Migration | Description |
|-----------|-------------|
| 001 | Initial schema: documents, relationships, tags, FTS5 search, triggers |
| 002 | Add `phase` column to documents |
| 003 | Add `configuration` table |
| 004 | Add `initiative_id` for lineage tracking |
| 005 | Add `short_code` column |
| 006 | Remove old unique constraint on short codes |
| 007 | Allow duplicate short codes per project |
| 008 | Remove `strategy_id` column |
| 009 | Add `parent_id` column |

## Dual Storage Model

Metis uses a dual storage architecture:

1. **Filesystem (source of truth):** Documents are Markdown files with YAML frontmatter. These are version-controlled and human-editable.

2. **SQLite database (index):** Provides search, relationship queries, and fast lookups. Rebuilt from the filesystem via `metis sync`.

If the database is deleted or corrupted, running `metis sync` reconstructs it from the files. The filesystem always wins in case of conflict. See [Architecture Overview](../explanation/architecture.md) for the design rationale.

## Migrations

Database migrations run automatically when the database is opened. There is no manual migration step.

Workspaces created with Metis v1 (which used a `strategies/` directory nesting layer) are automatically migrated to the v2 layout during workspace detection. This migration is transparent — the directory structure is reorganized and the database is rebuilt.

## Database Recovery

If the database is missing or corrupted, `metis sync` triggers automatic recovery:

1. Recreates the database from filesystem documents
2. Rebuilds short code counters from existing document short codes
3. Syncs `config.toml` settings to the database
4. Reconstructs parent-child relationships

Recovery is automatic and requires no manual intervention.

## Workspace Detection

Metis finds the workspace by walking up the directory tree from the current working directory, looking for a `.metis/` directory. This means you can run `metis` commands from any subdirectory of your project.

The MCP server uses `WorkspaceDetectionService` which applies the same algorithm.
