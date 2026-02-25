---
id: code-index-memory-for-ai-agent
level: initiative
title: "Code Index Memory for AI Agent Navigation"
short_code: "METIS-I-0021"
created_at: 2026-02-20T14:31:12.026384+00:00
updated_at: 2026-02-20T14:52:07.402592+00:00
parent: METIS-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/active"


exit_criteria_met: false
estimated_complexity: L
strategy_id: NULL
initiative_id: code-index-memory-for-ai-agent
---

# Code Index Memory for AI Agent Navigation

## Context

AI coding agents spend significant time orienting themselves in codebases -- running dozens of Glob, Grep, and Read calls just to understand where things are before they can do real work. This is especially costly in large, complex projects, and worse in codebases with inconsistent organization or sparse documentation.

Metis already provides persistent memory for *work tracking* (tasks, initiatives, decisions). This initiative extends that to *codebase knowledge* -- a persistent, compressed index of the project's code structure that agents can read to navigate efficiently from the start of any session.

### Prior Art

- **Aider's Repo Map**: Tree-sitter AST extraction → dependency graph → PageRank ranking → compressed ~1k token representation of key symbols and signatures. Dynamic, regenerated per query to fit token budget.
- **AiDex**: Persistent SQLite index of tree-sitter symbols, served via MCP. ~50 tokens per query vs 2000+ for grep. Exact identifier matching, no embeddings.
- **Repomix**: Packs entire repo into a single AI-friendly file. Works for small projects, blows up context for large ones.

## Goals & Non-Goals

**Goals:**
- Reduce agent orientation time in large/complex codebases by providing a pre-built structural index
- Generate a flat-file code index in `.metis/` that agents can read for project navigation
- Support two layers: structural index (tree-sitter, always available) and AI-generated module summaries (optional enrichment)
- Provide a `metis index` CLI command and MCP tool for index generation
- Integrate with the Metis plugin via a skill (how to create/update the index) and hook (detect when index is missing)
- Support incremental updates when files change
- Respect `.gitignore` for scope

**Non-Goals:**
- Replacing full-text code search (agents still use Grep/Glob for specific queries)
- Embedding-based semantic code search (that's a potential future enhancement, not this initiative)
- Cross-repository indexing
- Real-time file watching (index updates are triggered, not automatic)
- `.metisignore` support (add later if needed)

## Use Cases

### UC-1: First Session in a New Project
- **Actor**: AI agent starting a session in a Metis project with no code index
- **Scenario**: SessionStart hook detects no `.metis/code-index.md`. Informs agent the index is missing and suggests creating one. Agent (or user) triggers index creation. Tree-sitter generates structural data (Layer 1). Agent reads structural data and code to generate module summaries (Layer 2, via Sonnet). Combined output written to `.metis/code-index.md`.
- **Expected Outcome**: Agent has a comprehensive code map available for navigation in all future sessions.

### UC-2: Navigating an Unfamiliar Codebase
- **Actor**: AI agent asked to work on a feature in a large project
- **Scenario**: Agent reads `.metis/code-index.md` to understand project structure, module purposes, and key symbols. Identifies the relevant files without exploratory Glob/Grep rounds. Goes directly to the right code.
- **Expected Outcome**: Agent reaches the correct files in 1-2 tool calls instead of 10-20.

### UC-3: Incremental Update After Code Changes
- **Actor**: AI agent or user after significant code changes
- **Scenario**: User runs `metis index` or agent triggers update via MCP tool. Only changed files are re-parsed (tree-sitter) and re-summarized (LLM). Index file is updated in place.
- **Expected Outcome**: Index stays current without full regeneration cost.

### UC-4: Codebase with Poor Documentation
- **Actor**: AI agent working in a legacy codebase with minimal docs
- **Scenario**: Layer 1 provides structural navigation (symbols, signatures). Layer 2 summaries describe what each module *does* in plain language, compensating for missing documentation.
- **Expected Outcome**: Agent can reason about module purposes even when the code doesn't self-document.

## Detailed Design

### Output Format

A single flat markdown file at `.metis/code-index.md`:

```markdown
# Code Index
Generated: 2026-02-20T14:30:00Z
Files indexed: 142
Languages: Rust, TypeScript, Vue

## Project Structure
src/
├── auth/
│   ├── handler.rs
│   ├── middleware.rs
│   └── models.rs
├── db/
│   ├── pool.rs
│   └── queries/
...

## Module Summaries
### src/auth/
Purpose: JWT authentication middleware chain
Key files: handler.rs (login/refresh route handlers), middleware.rs (token validation), models.rs (User, Session, Claims types)
Dependencies: src/db/, src/config/

### src/db/
Purpose: Database connection pooling and query layer
Key files: pool.rs (connection management), queries/ (per-table SQL modules)
...

## Key Symbols
### src/auth/handler.rs
  pub async fn login(req: LoginRequest) -> Result<TokenPair>
  pub async fn refresh(token: RefreshToken) -> Result<TokenPair>
  pub async fn logout(claims: Claims) -> Result<()>

### src/auth/middleware.rs
  pub struct AuthMiddleware
  pub async fn validate_token(token: &str) -> Result<Claims>
...
```

### Vendored Code from muninn-graph

The `colliery-io/muninn` repo contains a `muninn-graph` crate with most of the tree-sitter infrastructure already built. We vendor the relevant modules into a new Metis crate rather than taking a dependency (to avoid pulling in graphqlite).

**Vendor (as-is or with minor mods):**
- `parser.rs` -- Multi-language tree-sitter parser with lazy init
- `symbols.rs` -- `Symbol`, `SymbolKind`, `Visibility` types
- `lang/rust.rs` + `lang/python.rs` -- Full extractors (symbols, imports, calls)
- `queries/*.scm` -- Tree-sitter query files

**Drop (not needed):**
- `store.rs` -- graphqlite storage (we output flat markdown, not a graph DB)
- `graph.rs` -- stub, unused
- `edges.rs` -- graph edges, not needed for flat file output
- `embeddings.rs` -- stub, unused
- `watcher.rs` -- file watching (incremental updates are Phase 4, simpler hash-based approach)

**Add:**
- TypeScript/JavaScript extractor + tree-sitter grammar
- Go extractor + tree-sitter grammar
- `ignore` crate integration in the directory walker (already a dependency in muninn, just not used in the builder)
- Markdown output formatter

### Two-Layer Architecture

**Layer 1: Structural Index (tree-sitter)**
- File tree (source files only, `.gitignore`-aware)
- Symbol extraction: function/method signatures, struct/class definitions, type aliases, trait/interface definitions
- Generated purely by `metis index` CLI / MCP tool
- No LLM required, fast, deterministic

**Layer 2: Module Summaries (AI-generated)**
- Per-directory summaries: purpose, key files, dependencies
- Generated by Claude (default: Sonnet) reading the structural index + source code
- Optional -- Layer 1 is useful on its own
- Triggered as part of the same index command, but can be skipped with a flag

### Generation Flow

1. `metis index` (CLI) or `index_code` (MCP tool) is invoked
2. Walk file tree, respecting `.gitignore`
3. Parse each source file with tree-sitter, extract symbols
4. Write Layer 1 (structure + symbols) to `.metis/code-index.md`
5. If Layer 2 requested: agent reads structural index + source files, generates module summaries, appends to the file

For incremental updates:
- Track file content hashes in `.metis/code-index-hashes.json`
- On re-index, only re-parse files whose hash changed
- Regenerate summaries only for directories with changed files

### Plugin Integration

**SessionStart hook (updated):**
- Detect whether `.metis/code-index.md` exists
- If missing: inform agent "No code index found. Consider running `metis index` or asking Claude to generate one for better codebase navigation."
- If present: inform agent "Code index available at `.metis/code-index.md` -- read it when you need to orient yourself in the codebase."

**Skill: `code-index`**
- Guides Claude on how to create and update the index
- Covers: when to generate (new project, after major refactors), how to write good module summaries, how to keep it current
- Triggered by: "create a code index", "index this codebase", "update the code index"

### Tree-sitter Language Support

Supported languages for this initiative:
- Rust, TypeScript/JavaScript, Python, Go

Additional languages (Java, C/C++, Vue/Svelte, etc.) are out of scope -- future feature requests.

Use tree-sitter grammars to extract:
- Function/method definitions with signatures
- Struct/class/interface/trait definitions
- Type aliases and enums
- Module/package declarations
- Import/dependency statements (for cross-file edges)

### CLI Command

```bash
metis index                        # Full index (Layer 1 + Layer 2 if agent available)
metis index --structure-only       # Layer 1 only, no AI summaries
metis index --incremental          # Only re-index changed files
```

### MCP Tool

```json
{
  "name": "index_code",
  "arguments": {
    "project_path": "/path/to/project/.metis",
    "structure_only": false,
    "incremental": true
  }
}
```

## Alternatives Considered

### Alt 1: MCP Query Server (AiDex-style)
A persistent index queried on demand via MCP tools rather than a flat file.
- **Rejected for now**: Adds complexity (persistent server, query protocol). A flat file is simpler, works with any MCP client, and can be evolved into a queryable index later.

### Alt 2: Inject Index into Context (Repomix-style)
Pack the index directly into the session context at start.
- **Rejected**: Too expensive for large projects. Better to inform the agent where the file is and let it read on demand.

### Alt 3: Embedding-based Semantic Code Search
Use vector embeddings for code search rather than structural indexing.
- **Rejected for this initiative**: Higher complexity, overlaps with I-0018's vector search work. Structural index solves the navigation problem; semantic search can be layered on later.

### Alt 4: External LLM for Summaries
Require an API key for an external LLM to generate module summaries.
- **Rejected**: Claude can generate the summaries itself during the indexing flow. No external dependency needed.

## Implementation Plan

### Phase 1: Tree-sitter Structural Index
- Add tree-sitter dependency to metis-docs-core (or new crate)
- Implement file tree walker (`.gitignore`-aware)
- Symbol extraction for Rust, TypeScript/JavaScript, Python, Go
- `metis index --structure-only` CLI command
- Output flat file to `.metis/code-index.md`

### Phase 2: MCP Tool + Plugin Hook
- Add `index_code` MCP tool
- Update SessionStart hook to detect code index presence
- Basic plugin skill for code index creation guidance

### Phase 3: AI-Generated Module Summaries (Layer 2)
- Plugin skill guides Claude through summary generation
- Claude reads structural index + source code, writes summaries
- Summaries appended to the index file

### Phase 4: Incremental Updates
- Content hash tracking in `.metis/code-index-hashes.json`
- Incremental re-indexing of changed files only
- Partial summary regeneration for affected directories