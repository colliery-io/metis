---
name: code-index
description: This skill should be used when the user asks to "create a code index", "index this codebase", "update the code index", "generate code index", "build code index", "refresh module summaries", or needs guidance on generating or updating the .metis/code-index.md file for AI agent codebase navigation.
model: sonnet
---

# Code Index Generation

This skill guides the process of generating and maintaining `.metis/code-index.md` — a structured codebase map for AI agent navigation.

## Overview

The code index has two layers:

1. **Layer 1 (Structural)**: Auto-generated file tree and symbol extraction via tree-sitter. Fast, deterministic.
2. **Layer 2 (Summaries)**: AI-generated module summaries that explain what each directory does. Requires reading source code.

## Quick Start

### Generate a fresh index

```
1. Run structural index:     mcp__metis__index_code(project_path, structure_only=false)
2. Read the generated index: Read .metis/code-index.md
3. Generate module summaries: Follow "Writing Module Summaries" below
```

### Update an existing index

```
1. Re-run structural index:  mcp__metis__index_code(project_path, structure_only=false)
2. Read the updated index:   Read .metis/code-index.md
3. Review Module Summaries:  Update any changed directories
```

## Writing Module Summaries

After the structural index exists, populate the `## Module Summaries` section. Use Sonnet for this work to keep costs low.

### Process

1. Read `.metis/code-index.md` to see the project structure and key symbols
2. For each top-level source directory (e.g., `src/`, `crates/foo/src/`, `lib/`):
   - Read 2-3 representative files to understand the directory's purpose
   - Look at imports/exports to understand dependencies
   - Write a concise summary
3. Edit `.metis/code-index.md` to replace the Module Summaries placeholder

### Summary Format

For each directory, write:

```markdown
### path/to/directory

**Purpose**: One sentence describing what this module does.

**Key files**:
- `file.rs` — Brief description of what it contains
- `other.rs` — Brief description

**Dependencies**: What this module depends on (other modules, external crates/packages).
```

### Example

```markdown
## Module Summaries

### crates/metis-code-index/src

**Purpose**: Tree-sitter based code indexing library that walks source files, parses them, extracts symbols, and formats results as markdown.

**Key files**:
- `parser.rs` — Multi-language parser with lazy-loaded tree-sitter grammars
- `walker.rs` — Gitignore-aware file discovery using the `ignore` crate
- `formatter.rs` — Markdown output with ASCII tree, module summaries, and symbol listings
- `symbols.rs` — Symbol types (SymbolKind, Visibility) shared across extractors

**Dependencies**: tree-sitter grammars (rust, python, typescript, javascript, go), ignore crate, serde.

### crates/metis-code-index/src/lang

**Purpose**: Language-specific symbol extractors using tree-sitter queries.

**Key files**:
- `rust.rs` — Rust extractor (functions, structs, traits, impls, macros)
- `typescript.rs` — TypeScript/JavaScript extractor (functions, classes, interfaces)
- `go.rs` — Go extractor (functions, methods, structs, interfaces)
- `python.rs` — Python extractor (functions, classes, decorators)

**Dependencies**: tree-sitter queries, parent module's Symbol types.
```

## Guidelines

### Keep summaries concise
- One sentence for Purpose (what, not how)
- 3-5 key files maximum per directory
- Dependencies as a comma-separated list

### Skip directories that don't need summaries
- Test directories (unless they contain important fixtures)
- Generated code directories
- Vendor/third-party directories
- Directories with only one file (the file is self-explanatory)

### When to regenerate

Re-run the full index (Layer 1 + Layer 2) when:
- Starting work on an unfamiliar project
- After major refactors that change directory structure
- When significant new modules are added
- When module summaries feel stale or inaccurate

Re-run Layer 1 only (structural) when:
- New files were added but directory structure is the same
- You just need updated symbol listings

## Model Selection

Use **Sonnet** for generating module summaries. It produces good quality summaries at lower cost. The structural index (Layer 1) is deterministic and doesn't use any model.
