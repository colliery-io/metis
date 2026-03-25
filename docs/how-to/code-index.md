# How to Use the Code Index

The code index generates a structured map of your codebase at `.metis/code-index.md`. It extracts symbols (functions, classes, structs, types) from source files using tree-sitter (a parsing library that understands source code structure), producing a document optimized for AI agent navigation.

## Generate a Full Index

```bash
metis index
```

```
Scanning source files...
  Found 42 source files
    Rust: 25 files
    TypeScript: 10 files
    Python: 7 files
  Extracted 234 symbols from 42 files

Index written to .metis/code-index.md (2.3s)
```

## Incremental Indexing

After the first full index, use incremental mode to only re-parse changed files:

```bash
metis index --incremental
```

```
Scanning source files...
  Found 42 source files
  Incremental: 3 changed, 39 unchanged, 0 deleted
  Extracted 18 symbols from 3 files

Index written to .metis/code-index.md (0.4s)
```

Incremental indexing uses content hashes stored in `.metis/code-index-hashes.json` to detect changes. Unchanged files reuse cached symbols from `.metis/code-index-symbols.json`.

## Structure-Only Mode

Generate just the directory tree without symbol extraction:

```bash
metis index --structure-only
```

This is faster and useful when you only need the project layout.

## Combine Flags

```bash
metis index --incremental --structure-only
```

## Supported Languages

Rust, Python, TypeScript, JavaScript, and Go. See [CLI Reference](../reference/cli.md) (`metis index`) for the full list of supported file extensions and extracted symbol types.

The walker automatically skips build and dependency directories (`target/`, `node_modules/`, `__pycache__/`, `.git/`, `vendor/`, `dist/`, `build/`, `.venv/`, `.next/`, and others) and respects `.gitignore` rules.

## Add Semantic Summaries

The code index includes placeholder summaries for each module. AI agents can generate richer summaries:

In Claude Code, the Metis plugin includes a `code-index-summarizer` agent that reads the source files in each module and writes semantic descriptions. This runs automatically when summaries are missing.

To manually trigger:

> "Update the code index summaries"

Summaries are preserved across re-indexes — only placeholder summaries are overwritten.

## Via MCP (Claude Code)

The `index_code` MCP tool provides the same functionality:

> "Generate the code index for this project"

Parameters match the CLI flags: `structure_only` and `incremental`.

## When to Re-Index

- After adding new source files or significant refactoring
- Before starting a Ralph loop (so Claude can navigate efficiently)
- The Metis plugin's `PreCompact` hook automatically re-indexes before context compaction
- The `PostToolUse` hook tracks file changes and marks the index as dirty
