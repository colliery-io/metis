# Code Index

> Generated: 2026-03-04T19:38:52Z | 144 files | JavaScript, Rust, TypeScript

## Project Structure

```
├── crates/
│   ├── metis-code-index/
│   │   └── src/
│   │       ├── formatter.rs
│   │       ├── hasher.rs
│   │       ├── lang/
│   │       │   ├── go.rs
│   │       │   ├── mod.rs
│   │       │   ├── python.rs
│   │       │   ├── rust.rs
│   │       │   └── typescript.rs
│   │       ├── lib.rs
│   │       ├── parser.rs
│   │       ├── symbols.rs
│   │       └── walker.rs
│   ├── metis-docs-cli/
│   │   ├── src/
│   │   │   ├── cli.rs
│   │   │   ├── commands/
│   │   │   │   ├── archive.rs
│   │   │   │   ├── config.rs
│   │   │   │   ├── create/
│   │   │   │   │   ├── adr.rs
│   │   │   │   │   ├── initiative.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── specification.rs
│   │   │   │   │   └── task.rs
│   │   │   │   ├── index.rs
│   │   │   │   ├── init.rs
│   │   │   │   ├── list.rs
│   │   │   │   ├── mcp.rs
│   │   │   │   ├── mod.rs
│   │   │   │   ├── search.rs
│   │   │   │   ├── status.rs
│   │   │   │   ├── sync.rs
│   │   │   │   ├── transition.rs
│   │   │   │   └── validate.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   ├── utils.rs
│   │   │   └── workspace.rs
│   │   └── tests/
│   │       └── comprehensive_functional_test.rs
│   ├── metis-docs-core/
│   │   ├── src/
│   │   │   ├── application/
│   │   │   │   ├── mod.rs
│   │   │   │   └── services/
│   │   │   │       ├── database.rs
│   │   │   │       ├── document/
│   │   │   │       │   ├── creation.rs
│   │   │   │       │   ├── deletion.rs
│   │   │   │       │   ├── discovery.rs
│   │   │   │       │   ├── mod.rs
│   │   │   │       │   └── validation.rs
│   │   │   │       ├── filesystem.rs
│   │   │   │       ├── mod.rs
│   │   │   │       ├── synchronization.rs
│   │   │   │       ├── template.rs
│   │   │   │       └── workspace/
│   │   │   │           ├── archive.rs
│   │   │   │           ├── detection.rs
│   │   │   │           ├── initialization.rs
│   │   │   │           ├── migration.rs
│   │   │   │           ├── mod.rs
│   │   │   │           ├── reassignment.rs
│   │   │   │           ├── recovery.rs
│   │   │   │           └── transition.rs
│   │   │   ├── constants.rs
│   │   │   ├── dal/
│   │   │   │   ├── database/
│   │   │   │   │   ├── configuration_repository.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── models.rs
│   │   │   │   │   ├── repository.rs
│   │   │   │   │   └── schema.rs
│   │   │   │   ├── filesystem/
│   │   │   │   │   └── mod.rs
│   │   │   │   └── mod.rs
│   │   │   ├── domain/
│   │   │   │   ├── configuration.rs
│   │   │   │   ├── documents/
│   │   │   │   │   ├── adr/
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── content.rs
│   │   │   │   │   ├── factory.rs
│   │   │   │   │   ├── helpers.rs
│   │   │   │   │   ├── initiative/
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── metadata.rs
│   │   │   │   │   ├── mod.rs
│   │   │   │   │   ├── specification/
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── task/
│   │   │   │   │   │   └── mod.rs
│   │   │   │   │   ├── traits.rs
│   │   │   │   │   ├── types.rs
│   │   │   │   │   └── vision/
│   │   │   │   │       └── mod.rs
│   │   │   │   └── mod.rs
│   │   │   ├── error/
│   │   │   │   └── conversions.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── main.rs
│   │   │   └── tests/
│   │   │       ├── common.rs
│   │   │       └── mod.rs
│   │   └── tests/
│   │       ├── collision_resolution_test.rs
│   │       ├── configuration_recovery_test.rs
│   │       ├── database_reconstruction_test.rs
│   │       ├── id_path_consistency_test.rs
│   │       ├── reassignment_test.rs
│   │       └── specification_test.rs
│   ├── metis-docs-gui/
│   │   ├── postcss.config.js
│   │   ├── src/
│   │   │   ├── composables/
│   │   │   │   ├── useProject.ts
│   │   │   │   ├── useTheme.ts
│   │   │   │   └── useToolbar.ts
│   │   │   ├── lib/
│   │   │   │   ├── board-config.ts
│   │   │   │   └── tauri-api.ts
│   │   │   ├── lib.rs
│   │   │   ├── main.ts
│   │   │   ├── test/
│   │   │   │   └── setup.ts
│   │   │   ├── themes/
│   │   │   │   └── definitions.ts
│   │   │   ├── types/
│   │   │   │   ├── board.ts
│   │   │   │   └── theme.ts
│   │   │   ├── utils/
│   │   │   │   └── drag-n-drop.ts
│   │   │   └── vite-env.d.ts
│   │   ├── src-tauri/
│   │   │   ├── build.rs
│   │   │   └── src/
│   │   │       ├── lib.rs
│   │   │       ├── main.rs
│   │   │       └── services/
│   │   │           ├── archive.rs
│   │   │           ├── cli_installer.rs
│   │   │           ├── document.rs
│   │   │           ├── mod.rs
│   │   │           ├── project.rs
│   │   │           ├── sync.rs
│   │   │           ├── transition.rs
│   │   │           └── version.rs
│   │   ├── tailwind.config.js
│   │   └── vite.config.ts
│   └── metis-docs-mcp/
│       ├── src/
│       │   ├── config.rs
│       │   ├── error.rs
│       │   ├── error_utils.rs
│       │   ├── formatting.rs
│       │   ├── lib.rs
│       │   ├── main.rs
│       │   ├── server.rs
│       │   └── tools/
│       │       ├── all_tools.rs
│       │       ├── archive_document.rs
│       │       ├── create_document.rs
│       │       ├── edit_document.rs
│       │       ├── index_code.rs
│       │       ├── initialize_project.rs
│       │       ├── list_documents.rs
│       │       ├── mod.rs
│       │       ├── read_document.rs
│       │       ├── reassign_parent.rs
│       │       ├── search_documents.rs
│       │       └── transition_phase.rs
│       └── tests/
│           ├── common/
│           │   └── mod.rs
│           ├── comprehensive_functional_test.rs
│           ├── configuration_scenarios_test.rs
│           ├── functional_test.rs
│           ├── mcp_archive_test.rs
│           └── mcp_server_integration_test.rs
└── tests/
    └── e2e/
        ├── app.spec.ts
        ├── fixtures.ts
        ├── playwright.config.ts
        ├── project.spec.ts
        └── search.spec.ts
```

## Modules

### crates/metis-code-index/src

**Role**: Extracts structured symbol information from multi-language source code using tree-sitter, producing a compressed markdown index suitable for AI agent navigation.

**Key abstractions**:
- `Symbol` — A named symbol (function, struct, trait, class, etc.) extracted from a source file, carrying its kind, visibility, file path, line range, optional signature, and optional doc comment. The `id()` method generates a stable, content-derived identifier.
- `SymbolKind` / `Visibility` — Enumerations classifying what a symbol is and who can see it. `is_type_definition()` and `is_callable()` allow quick categorization without matching on variants.
- `Parser` — Wraps a `tree-sitter::Parser` with lazily initialized per-language `LanguageConfig` instances. `parse_file()` detects the language from extension; `parse_source()` accepts explicit language. One `Parser` instance supports all languages simultaneously.
- `WalkResult` / `SourceFile` — Output of `walk_directory()`: a flat, sorted list of source files with their detected `Language`. `walk_directory()` respects `.gitignore` and hard-codes a skip list (`target/`, `node_modules/`, `__pycache__/`, etc.).
- `HashManifest` / `SymbolCache` / `IncrementalDiff` — Together enable incremental indexing. `HashManifest` stores per-file BLAKE3 hashes. `diff()` classifies files as changed/unchanged/deleted. `SymbolCache` stores previously extracted symbols keyed by file path. Both persist as JSON in `.metis/`.
- `format_index()` / `write_index_file()` — Convert a `WalkResult` + symbol map into the `code-index.md` markdown format, preserving any existing AI-authored summaries via `parse_existing_summaries()`.

**Internal flow**: `walk_directory()` discovers files → `Parser.parse_file()` produces an AST → a language-specific extractor (e.g., `RustExtractor`) queries the AST for symbols → `format_index()` assembles the markdown → `write_index_file()` writes it, re-injecting any summaries already present in the file. The incremental path short-circuits parsing by loading `SymbolCache` for unchanged files and only re-parsing those flagged by `HashManifest.diff()`.

**Dependencies**: `tree-sitter` + language grammar crates (Rust, Python, TypeScript, JavaScript, Go) for AST parsing; `blake3` for fast file hashing; `ignore` (gitignore-aware walker); `serde_json` for cache persistence.

#### crates/metis-code-index/src/formatter.rs

- pub `format_index` function L24-41 — `( walk_result: &WalkResult, symbols_by_file: &BTreeMap<PathBuf, Vec<Symbol>>, ti...` — Format a code index as markdown.
- pub `parse_existing_summaries` function L47-92 — `(content: &str) -> BTreeMap<String, String>` — Parse existing semantic summaries from a code-index.md file.
- pub `write_index_file` function L358-372 — `( output_path: &Path, walk_result: &WalkResult, symbols_by_file: &BTreeMap<PathB...` — Write the formatted index to a file, preserving existing summaries.
-  `SUMMARY_PLACEHOLDER` variable L16 — `: &str` — Placeholder text for modules that haven't been summarized yet.
-  `save_summary` function L95-100 — `(summaries: &mut BTreeMap<String, String>, module: String, raw: &str)` — Save a non-placeholder summary to the map.
-  `write_header` function L103-118 — `(output: &mut String, walk_result: &WalkResult, timestamp: &str)` — Write the document header with metadata.
-  `write_project_structure` function L121-129 — `(output: &mut String, walk_result: &WalkResult)` — Write the project structure as an ASCII tree.
-  `write_modules` function L132-258 — `( output: &mut String, walk_result: &WalkResult, symbols_by_file: &BTreeMap<Path...` — Write modules grouped by directory with co-located semantic summaries and symbols.
-  `first_sentence` function L261-279 — `(doc: &str) -> Option<&str>` — Extract the first sentence from a doc comment.
-  `TreeNode` struct L282-286 — `{ name: String, is_dir: bool, children: Vec<TreeNode> }` — A node in the file tree for rendering.
-  `build_tree` function L289-307 — `(walk_result: &WalkResult) -> TreeNode` — Build a tree structure from the walk result.
-  `insert_path` function L310-338 — `(node: &mut TreeNode, components: &[&str], depth: usize)` — Recursively insert a path into the tree.
-  `render_tree` function L341-355 — `(output: &mut String, children: &[TreeNode], prefix: &str)` — Render the tree with box-drawing characters.
-  `tests` module L375-945 — `-` — that are preserved across regenerations.
-  `make_walk_result` function L380-392 — `(files: Vec<(&str, Language)>) -> WalkResult` — that are preserved across regenerations.
-  `make_symbol` function L394-406 — `( name: &str, kind: SymbolKind, file: &str, vis: Visibility, sig: Option<&str>, ...` — that are preserved across regenerations.
-  `test_format_header` function L409-422 — `()` — that are preserved across regenerations.
-  `test_format_project_structure` function L425-446 — `()` — that are preserved across regenerations.
-  `test_modules_section_exists` function L449-470 — `()` — that are preserved across regenerations.
-  `test_module_grouping` function L473-533 — `()` — that are preserved across regenerations.
-  `test_symbols_under_file_headers` function L536-566 — `()` — that are preserved across regenerations.
-  `test_public_symbols_first` function L569-602 — `()` — that are preserved across regenerations.
-  `test_tree_rendering` function L605-626 — `()` — that are preserved across regenerations.
-  `test_empty_project` function L629-642 — `()` — that are preserved across regenerations.
-  `test_files_without_symbols_skipped` function L645-671 — `()` — that are preserved across regenerations.
-  `test_module_without_symbols_skipped` function L674-700 — `()` — that are preserved across regenerations.
-  `test_write_index_file` function L703-714 — `()` — that are preserved across regenerations.
-  `test_format_with_doc_comment` function L717-738 — `()` — that are preserved across regenerations.
-  `test_format_single_line_symbol` function L741-758 — `()` — that are preserved across regenerations.
-  `test_first_sentence_extraction` function L761-770 — `()` — that are preserved across regenerations.
-  `test_multi_language_project` function L773-819 — `()` — that are preserved across regenerations.
-  `test_summary_preservation` function L822-895 — `()` — that are preserved across regenerations.
-  `test_parse_existing_summaries` function L898-944 — `()` — that are preserved across regenerations.

#### crates/metis-code-index/src/hasher.rs

- pub `HashManifest` struct L16-19 — `{ files: BTreeMap<String, String> }` — Hash manifest stored at `.metis/code-index-hashes.json`.
- pub `IncrementalDiff` struct L23-30 — `{ changed: Vec<SourceFile>, unchanged: Vec<SourceFile>, deleted: Vec<String> }` — Result of comparing current files against a previous hash manifest.
- pub `changed_count` function L34-36 — `(&self) -> usize` — Number of files that need re-indexing.
- pub `unchanged_count` function L39-41 — `(&self) -> usize` — Number of files skipped.
- pub `deleted_count` function L44-46 — `(&self) -> usize` — Number of files removed.
- pub `load` function L51-62 — `(path: &Path) -> Result<Self, std::io::Error>` — Load a manifest from a JSON file.
- pub `save` function L65-68 — `(&self, path: &Path) -> Result<(), std::io::Error>` — Save the manifest to a JSON file.
- pub `hash_file` function L71-74 — `(path: &Path) -> Result<String, std::io::Error>` — Compute the BLAKE3 hash of a file's contents.
- pub `diff` function L80-116 — `(&self, walk_result: &WalkResult) -> IncrementalDiff` — Compare current files against this manifest to determine what changed.
- pub `from_walk_result` function L119-128 — `(walk_result: &WalkResult) -> Self` — Build a fresh manifest from a walk result by hashing all files.
- pub `update` function L131-144 — `(&mut self, diff: &IncrementalDiff)` — Update the manifest with hashes from changed files and remove deleted paths.
- pub `affected_directories` function L147-164 — `(diff: &IncrementalDiff) -> std::collections::BTreeSet<PathBuf>` — Get the set of directories that contain changed or deleted files.
- pub `SymbolCache` struct L172-175 — `{ files: BTreeMap<String, Vec<Symbol>> }` — Cached symbol data stored at `.metis/code-index-symbols.json`.
- pub `load` function L179-190 — `(path: &Path) -> Result<Self, std::io::Error>` — Load a symbol cache from a JSON file.
- pub `save` function L193-196 — `(&self, path: &Path) -> Result<(), std::io::Error>` — Save the symbol cache to a JSON file (compact format).
- pub `to_path_map` function L199-204 — `(&self) -> BTreeMap<PathBuf, Vec<Symbol>>` — Convert to a `BTreeMap<PathBuf, Vec<Symbol>>` for use with `format_index`.
- pub `from_path_map` function L207-213 — `(map: &BTreeMap<PathBuf, Vec<Symbol>>) -> Self` — Build from a `BTreeMap<PathBuf, Vec<Symbol>>`.
- pub `update` function L216-224 — `(&mut self, changed: &BTreeMap<PathBuf, Vec<Symbol>>, deleted: &[String])` — Update cache: add/replace changed file symbols and remove deleted file entries.
-  `IncrementalDiff` type L32-47 — `= IncrementalDiff` — `metis index --incremental` can skip unchanged files.
-  `HashManifest` type L49-165 — `= HashManifest` — `metis index --incremental` can skip unchanged files.
-  `SymbolCache` type L177-225 — `= SymbolCache` — `metis index --incremental` can skip unchanged files.
-  `tests` module L228-583 — `-` — `metis index --incremental` can skip unchanged files.
-  `make_source_file` function L233-239 — `(root: &Path, rel: &str, lang: Language) -> SourceFile` — `metis index --incremental` can skip unchanged files.
-  `test_hash_file` function L242-254 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_hash_changes_with_content` function L257-268 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_manifest_save_load` function L271-289 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_load_nonexistent_returns_empty` function L292-295 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_from_walk_result` function L298-315 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_diff_all_new` function L318-333 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_diff_unchanged` function L336-351 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_diff_modified` function L354-373 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_diff_deleted` function L376-399 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_diff_mixed_scenario` function L402-437 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_update_manifest` function L440-473 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_affected_directories` function L476-494 — `()` — `metis index --incremental` can skip unchanged files.
-  `make_symbol` function L498-501 — `(name: &str, file: &str) -> Symbol` — `metis index --incremental` can skip unchanged files.
-  `test_symbol_cache_save_load` function L504-520 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_symbol_cache_load_nonexistent` function L523-526 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_symbol_cache_roundtrip_path_map` function L529-547 — `()` — `metis index --incremental` can skip unchanged files.
-  `test_symbol_cache_update` function L550-582 — `()` — `metis index --incremental` can skip unchanged files.

#### crates/metis-code-index/src/lib.rs

- pub `formatter` module L10 — `-` — This crate provides symbol extraction from source code using tree-sitter
- pub `hasher` module L11 — `-` — graph building, and file watching removed.
- pub `lang` module L12 — `-` — graph building, and file watching removed.
- pub `parser` module L13 — `-` — graph building, and file watching removed.
- pub `symbols` module L14 — `-` — graph building, and file watching removed.
- pub `walker` module L15 — `-` — graph building, and file watching removed.

#### crates/metis-code-index/src/parser.rs

- pub `ParseError` enum L15-30 — `UnsupportedExtension | IoError | ParseFailed | LanguageError | QueryError` — Errors that can occur during parsing.
- pub `Language` enum L34-40 — `Rust | Python | TypeScript | JavaScript | Go` — Supported programming languages.
- pub `from_extension` function L44-53 — `(ext: &str) -> Option<Self>` — Detect language from file extension.
- pub `from_path` function L56-60 — `(path: &Path) -> Option<Self>` — Detect language from file path.
- pub `tree_sitter_language` function L63-72 — `(&self) -> tree_sitter::Language` — Get the tree-sitter language for this language.
- pub `extensions` function L75-83 — `(&self) -> &'static [&'static str]` — Get file extensions associated with this language.
- pub `name` function L86-94 — `(&self) -> &'static str` — Get the display name for this language.
- pub `all` function L97-105 — `() -> &'static [Language]` — Return all supported language variants.
- pub `LanguageConfig` struct L109-114 — `{ language: tree_sitter::Language, symbols_query: tree_sitter::Query }` — Configuration for a language including tree-sitter queries.
- pub `new` function L118-129 — `(lang: Language) -> Result<Self, ParseError>` — Create a new language configuration.
- pub `LazyLanguageConfig` struct L147-150 — `{ language: Language, config: OnceLock<Result<LanguageConfig, String>> }` — Lazily initialized language configuration.
- pub `new` function L154-159 — `(language: Language) -> Self` — Create a new lazy config for the given language.
- pub `get` function L162-167 — `(&self) -> Result<&LanguageConfig, ParseError>` — Get the configuration, initializing if needed.
- pub `ParsedFile` struct L171-180 — `{ language: Language, tree: tree_sitter::Tree, source: String, path: Option<Stri...` — A parsed source file with its AST.
- pub `root_node` function L184-186 — `(&self) -> tree_sitter::Node<'_>` — Get the root node of the syntax tree.
- pub `source_bytes` function L189-191 — `(&self) -> &[u8]` — Get the source code as bytes.
- pub `node_text` function L194-196 — `(&self, node: tree_sitter::Node) -> &str` — Get text for a node.
- pub `Parser` struct L203-208 — `{ ts_parser: tree_sitter::Parser, configs: HashMap<Language, LazyLanguageConfig>...` — Multi-language source code parser.
- pub `new` function L212-222 — `() -> Self` — Create a new parser with all supported languages.
- pub `parse_file` function L225-239 — `(&mut self, path: &Path) -> Result<ParsedFile, ParseError>` — Parse a file from the filesystem.
- pub `parse_source` function L242-268 — `( &mut self, source: &str, language: Language, ) -> Result<ParsedFile, ParseErro...` — Parse source code string with a specified language.
- pub `get_config` function L271-276 — `(&self, language: Language) -> Result<&LanguageConfig, ParseError>` — Get the language configuration for a language.
- pub `supports_extension` function L279-281 — `(ext: &str) -> bool` — Check if a file extension is supported.
- pub `supported_extensions` function L284-290 — `() -> Vec<&'static str>` — Get all supported extensions.
-  `Language` type L42-106 — `= Language` — grammars and uses lazy initialization for efficient resource usage.
-  `LanguageConfig` type L116-141 — `= LanguageConfig` — grammars and uses lazy initialization for efficient resource usage.
-  `get_symbols_query` function L132-140 — `(lang: Language) -> &'static str` — Get the symbols query source for a language.
-  `LazyLanguageConfig` type L152-168 — `= LazyLanguageConfig` — grammars and uses lazy initialization for efficient resource usage.
-  `ParsedFile` type L182-197 — `= ParsedFile` — grammars and uses lazy initialization for efficient resource usage.
-  `Parser` type L210-291 — `= Parser` — grammars and uses lazy initialization for efficient resource usage.
-  `Parser` type L293-297 — `impl Default for Parser` — grammars and uses lazy initialization for efficient resource usage.
-  `default` function L294-296 — `() -> Self` — grammars and uses lazy initialization for efficient resource usage.
-  `tests` module L300-465 — `-` — grammars and uses lazy initialization for efficient resource usage.
-  `test_language_from_extension` function L304-314 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_language_from_path` function L317-339 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parser_parse_rust_source` function L342-353 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parser_parse_python_source` function L356-370 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parsed_file_node_text` function L373-380 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parser_parse_typescript_source` function L383-399 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parser_parse_javascript_source` function L402-419 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_language_from_go_extension` function L422-424 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_parser_parse_go_source` function L427-442 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_supported_extensions` function L445-454 — `()` — grammars and uses lazy initialization for efficient resource usage.
-  `test_supports_extension` function L457-464 — `()` — grammars and uses lazy initialization for efficient resource usage.

#### crates/metis-code-index/src/symbols.rs

- pub `SymbolKind` enum L11-34 — `File | Module | Class | Struct | Interface | Enum | Function | Method | Variable...` — The kind of symbol extracted from source code.
- pub `as_str` function L38-52 — `(&self) -> &'static str` — Returns the string representation used in node IDs and queries.
- pub `is_type_definition` function L55-64 — `(&self) -> bool` — Returns true if this symbol kind represents a type definition.
- pub `is_callable` function L67-72 — `(&self) -> bool` — Returns true if this symbol kind represents a callable.
- pub `Visibility` enum L79-89 — `Public | Private | Crate | Restricted` — Visibility/accessibility of a symbol.
- pub `Symbol` struct L93-120 — `{ name: String, kind: SymbolKind, file_path: String, start_line: usize, end_line...` — A symbol extracted from source code.
- pub `new` function L124-142 — `( name: impl Into<String>, kind: SymbolKind, file_path: impl Into<String>, start...` — Create a new symbol with required fields.
- pub `with_signature` function L145-148 — `(mut self, signature: impl Into<String>) -> Self` — Set the signature.
- pub `with_qualified_name` function L151-154 — `(mut self, qualified_name: impl Into<String>) -> Self` — Set the qualified name.
- pub `with_doc_comment` function L157-160 — `(mut self, doc_comment: impl Into<String>) -> Self` — Set the doc comment.
- pub `with_visibility` function L163-166 — `(mut self, visibility: Visibility) -> Self` — Set the visibility.
- pub `id` function L172-181 — `(&self) -> String` — Generate a stable ID for this symbol.
- pub `line_count` function L184-186 — `(&self) -> usize` — Returns the number of lines this symbol spans.
- pub `location` function L189-191 — `(&self) -> String` — Returns a location string for display (file:line-line).
- pub `compact_signature` function L198-213 — `(raw: &str, max_len: usize) -> String` — Collapse whitespace and truncate a signature to at most `max_len` characters.
-  `SymbolKind` type L36-73 — `= SymbolKind` — extracted from source files.
-  `Symbol` type L122-192 — `= Symbol` — extracted from source files.
-  `tests` module L216-326 — `-` — extracted from source files.
-  `test_symbol_kind_as_str` function L220-224 — `()` — extracted from source files.
-  `test_symbol_kind_classification` function L227-235 — `()` — extracted from source files.
-  `test_symbol_creation` function L238-255 — `()` — extracted from source files.
-  `test_symbol_id_generation` function L258-261 — `()` — extracted from source files.
-  `test_symbol_location` function L264-267 — `()` — extracted from source files.
-  `test_visibility_default` function L270-272 — `()` — extracted from source files.
-  `test_symbol_serialization` function L275-285 — `()` — extracted from source files.
-  `test_symbol_kind_serialization` function L288-295 — `()` — extracted from source files.
-  `test_visibility_serialization` function L298-304 — `()` — extracted from source files.
-  `test_compact_signature_short` function L307-309 — `()` — extracted from source files.
-  `test_compact_signature_collapses_whitespace` function L312-317 — `()` — extracted from source files.
-  `test_compact_signature_truncates` function L320-325 — `()` — extracted from source files.

#### crates/metis-code-index/src/walker.rs

- pub `SourceFile` struct L15-22 — `{ path: PathBuf, relative_path: PathBuf, language: Language }` — A source file discovered during directory walking.
- pub `WalkResult` struct L26-31 — `{ root: PathBuf, files: Vec<SourceFile> }` — Result of walking a directory for source files.
- pub `by_language` function L35-41 — `(&self) -> std::collections::HashMap<Language, Vec<&SourceFile>>` — Group files by language.
- pub `file_count` function L44-46 — `(&self) -> usize` — Get file count.
- pub `walk_directory` function L71-121 — `(root: &Path) -> Result<WalkResult, WalkError>` — Walk a directory tree for source files, respecting gitignore rules.
- pub `WalkError` enum L125-133 — `IoError | WalkError` — Errors that can occur during directory walking.
-  `WalkResult` type L33-47 — `= WalkResult` — source file extensions and skips common non-source directories.
-  `SKIP_DIRS` variable L50-64 — `: &[&str]` — Directories to always skip, regardless of gitignore rules.
-  `tests` module L136-403 — `-` — source file extensions and skips common non-source directories.
-  `create_test_project` function L141-171 — `(dir: &Path)` — Helper to create a test directory structure.
-  `test_walk_finds_source_files` function L174-199 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_respects_gitignore` function L202-233 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_skips_target_directory` function L236-258 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_skips_node_modules` function L261-283 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_skips_pycache` function L286-308 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_by_language` function L311-341 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_sorted_output` function L344-361 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_empty_directory` function L364-369 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_nonexistent_directory` function L372-375 — `()` — source file extensions and skips common non-source directories.
-  `test_walk_language_detection` function L378-402 — `()` — source file extensions and skips common non-source directories.

### crates/metis-code-index/src/lang

**Role**: Provides language-specific symbol extractors that translate tree-sitter ASTs into normalized `Symbol` lists, one module per supported language.

**Key abstractions**:
- `RustExtractor` — Extracts Rust structs, enums, traits, functions, impl blocks, type aliases, consts, macros, and `use` imports. Also extracts `TraitImpl` (trait-for-type pairs) and `FFIMarker` (extern blocks). Visibility is parsed from `pub`, `pub(crate)`, `pub(super)`, etc. Doc comments are recovered by scanning source lines backward from the symbol's start line, since tree-sitter does not attach them directly.
- `PythonExtractor` — Extracts classes (with base class signatures), functions and methods (with return types), `import`/`from...import` statements, call sites, and FFI markers (`ctypes`/`cffi` usage). Visibility follows the Python convention: names starting with `_` are private.
- `GoExtractor` — Extracts functions, methods (with receiver), structs, interfaces, type definitions, and const/var declarations. Visibility follows the Go capitalization convention via `go_visibility()`. Deduplication removes struct names that appear twice due to the type-declaration vs. struct-body grammar split.
- `TypeScriptExtractor` — Handles both TypeScript and JavaScript via separate `TS_QUERIES`/`JS_QUERIES` and separate `extract_ts_symbols()`/`extract_js_symbols()` paths. Covers classes, interfaces, type aliases, enums, functions, and arrow functions. JSDoc comments are extracted from preceding `/** */` blocks.

**Internal flow**: Each extractor receives a pre-parsed `tree_sitter::Tree` and raw source bytes. Compiled tree-sitter queries are stored in per-language `OnceLock` statics so they are compiled only once per process. Extractors run the queries, iterate captures, build `Symbol` values with signatures (built by traversing child nodes), and return a deduplicated `Vec<Symbol>`.

**Mixed concerns / gotchas**: `TypeScriptExtractor::extract_symbols()` dispatches to two distinct internal functions depending on the `Language` variant passed in — TS and JS use different grammars and different query sets even though they share the same extractor struct.

**Dependencies**: Each extractor depends only on `tree-sitter` node traversal APIs and the `symbols` module's `Symbol`/`SymbolKind`/`Visibility` types. Language grammars (`tree-sitter-rust`, `tree-sitter-python`, etc.) are loaded through `parser.rs`'s `LanguageConfig`.

#### crates/metis-code-index/src/lang/go.rs

- pub `Import` struct L17-24 — `{ path: String, alias: Option<String>, line: usize }` — Import statement from Go source.
- pub `GoExtractor` struct L101 — `-` — Go symbol extractor.
- pub `extract_symbols` function L105-243 — `( tree: &Tree, source: &str, file_path: &str, ) -> Result<Vec<Symbol>, String>` — Extract symbols from a parsed Go syntax tree.
- pub `extract_imports` function L246-290 — `(tree: &Tree, source: &str) -> Result<Vec<Import>, String>` — Extract import statements from a parsed Go syntax tree.
-  `GoQueries` struct L27-30 — `{ symbols: Query, imports: Query }` — Compiled tree-sitter queries for Go.
-  `GO_QUERIES` variable L32 — `: OnceLock<Result<GoQueries, String>>` — export conventions.
-  `get_queries` function L34-46 — `() -> Result<&'static GoQueries, &'static str>` — export conventions.
-  `SYMBOLS_QUERY` variable L49-87 — `: &str` — Tree-sitter query for extracting Go symbols.
-  `IMPORTS_QUERY` variable L90-94 — `: &str` — Tree-sitter query for extracting Go import statements.
-  `GoExtractor` type L103-291 — `= GoExtractor` — export conventions.
-  `build_go_struct_signature` function L294-328 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a Go struct: `{ Name string, Age int }`.
-  `build_go_interface_signature` function L331-363 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a Go interface: `{ Read(), Write() }`.
-  `build_go_type_signature` function L366-371 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a Go type definition: `= underlying_type`.
-  `build_go_var_signature` function L374-385 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a Go const/var: `: type` if type is annotated.
-  `find_child_by_kind` function L388-404 — `( node: tree_sitter::Node<'a>, kind: &str, ) -> Option<tree_sitter::Node<'a>>` — Find a child node by kind name.
-  `go_visibility` function L408-414 — `(name: &str) -> Visibility` — Determine visibility based on Go naming convention.
-  `build_func_signature` function L417-440 — `( name: &str, params: Option<&str>, receiver: Option<&str>, return_type: Option<...` — Build a function/method signature string.
-  `extract_go_doc` function L443-465 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — Extract Go doc comments (// lines preceding a declaration).
-  `deduplicate_symbols` function L469-477 — `(symbols: &mut Vec<Symbol>)` — Deduplicate symbols by (name, start_line).
-  `kind_priority` function L480-486 — `(kind: &SymbolKind) -> u8` — Priority for deduplication: lower is better (kept first).
-  `node_text` function L488-490 — `(node: &tree_sitter::Node, source: &str) -> String` — export conventions.
-  `tests` module L493-845 — `-` — export conventions.
-  `parse_go` function L497-501 — `(source: &str) -> Tree` — export conventions.
-  `test_extract_function` function L504-533 — `()` — export conventions.
-  `test_extract_method_with_receiver` function L536-575 — `()` — export conventions.
-  `test_extract_struct` function L578-612 — `()` — export conventions.
-  `test_extract_interface` function L615-656 — `()` — export conventions.
-  `test_extract_type_definition` function L659-689 — `()` — export conventions.
-  `test_extract_constants` function L692-713 — `()` — export conventions.
-  `test_extract_imports` function L716-738 — `()` — export conventions.
-  `test_go_visibility` function L741-746 — `()` — export conventions.
-  `test_struct_not_duplicated_as_type` function L749-770 — `()` — export conventions.
-  `test_mixed_go_file` function L773-844 — `()` — export conventions.

#### crates/metis-code-index/src/lang/mod.rs

- pub `go` module L6 — `-` — Each language module provides extraction logic for symbols, imports,
- pub `python` module L7 — `-` — and call relationships from parsed syntax trees.
- pub `rust` module L8 — `-` — and call relationships from parsed syntax trees.
- pub `typescript` module L9 — `-` — and call relationships from parsed syntax trees.

#### crates/metis-code-index/src/lang/python.rs

- pub `Import` struct L14-23 — `{ module: String, name: Option<String>, alias: Option<String>, line: usize }` — Import statement from Python source.
- pub `Call` struct L27-34 — `{ callee: String, line: usize, is_method: bool }` — Function/method call.
- pub `FFIMarker` enum L38-46 — `Ctypes | Cffi` — FFI marker detected in Python code.
- pub `PythonExtractor` struct L180 — `-` — Python-specific symbol extractor.
- pub `extract_symbols` function L184-305 — `( tree: &Tree, source: &str, file_path: &str, ) -> Result<Vec<Symbol>, String>` — Extract symbols from a Python source file.
- pub `extract_imports` function L308-355 — `(tree: &Tree, source: &str) -> Result<Vec<Import>, String>` — Extract import statements.
- pub `extract_calls` function L358-400 — `(tree: &Tree, source: &str) -> Result<Vec<Call>, String>` — Extract function/method calls.
- pub `extract_ffi_markers` function L403-469 — `(tree: &Tree, source: &str) -> Result<Vec<FFIMarker>, String>` — Extract FFI markers (ctypes, cffi usage).
-  `PythonQueries` struct L49-54 — `{ symbols: Query, imports: Query, calls: Query, ffi: Query }` — Compiled tree-sitter queries for Python.
-  `PYTHON_QUERIES` variable L56 — `: OnceLock<PythonQueries>` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `get_queries` function L58-177 — `() -> &'static PythonQueries` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `PythonExtractor` type L182-582 — `= PythonExtractor` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `build_class_signature` function L472-534 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a Python class: base classes and key method names.
-  `node_text` function L537-539 — `(node: &tree_sitter::Node, source: &str) -> String` — Get text content of a node.
-  `extract_docstring` function L542-571 — `(block: &tree_sitter::Node, source: &str) -> Option<String>` — Extract docstring from a block (first string literal).
-  `visibility_from_name` function L574-581 — `(name: &str) -> Visibility` — Determine visibility from Python naming convention.
-  `tests` module L585-818 — `-` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `parse_python` function L589-595 — `(source: &str) -> Tree` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_class` function L598-625 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_class_with_bases` function L628-641 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_function` function L644-675 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_methods` function L678-696 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_method_return_type` function L699-723 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_imports` function L726-751 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_calls` function L754-778 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_visibility_convention` function L781-798 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.
-  `test_extract_ffi_markers` function L801-817 — `()` — Extracts classes, functions, imports, and FFI patterns from Python source code.

#### crates/metis-code-index/src/lang/rust.rs

- pub `Import` struct L14-21 — `{ path: String, alias: Option<String>, line: usize }` — Represents a use/import statement.
- pub `Call` struct L25-32 — `{ callee: String, line: usize, is_method: bool }` — Represents a function call site.
- pub `FFIMarker` struct L36-44 — `{ abi: Option<String>, line: usize, start_line: usize, end_line: usize }` — Represents an FFI boundary marker.
- pub `TraitImpl` struct L48-55 — `{ type_name: String, trait_name: String, line: usize }` — Represents a trait implementation.
- pub `RustExtractor` struct L215 — `-` — Rust language extractor.
- pub `extract_symbols` function L219-370 — `( tree: &tree_sitter::Tree, source: &str, file_path: &str, ) -> Result<Vec<Symbo...` — Extract symbols from a parsed Rust syntax tree.
- pub `extract_imports` function L373-409 — `(tree: &tree_sitter::Tree, source: &str) -> Result<Vec<Import>, String>` — Extract import statements from a parsed Rust syntax tree.
- pub `extract_calls` function L412-457 — `(tree: &tree_sitter::Tree, source: &str) -> Result<Vec<Call>, String>` — Extract function calls from a parsed Rust syntax tree.
- pub `extract_implementations` function L460-505 — `( tree: &tree_sitter::Tree, source: &str, ) -> Result<Vec<TraitImpl>, String>` — Extract trait implementations from a parsed Rust syntax tree.
- pub `extract_ffi_markers` function L508-552 — `( tree: &tree_sitter::Tree, source: &str, ) -> Result<Vec<FFIMarker>, String>` — Extract FFI markers (extern blocks) from a parsed Rust syntax tree.
-  `RustQueries` struct L58-64 — `{ symbols: tree_sitter::Query, imports: tree_sitter::Query, calls: tree_sitter::...` — Compiled queries for Rust symbol extraction.
-  `RustQueries` type L66-86 — `= RustQueries` — from Rust source code.
-  `new` function L67-85 — `(language: tree_sitter::Language) -> Result<Self, String>` — from Rust source code.
-  `RUST_QUERIES` variable L88 — `: OnceLock<Result<RustQueries, String>>` — from Rust source code.
-  `get_queries` function L90-98 — `() -> Result<&'static RustQueries, &'static str>` — from Rust source code.
-  `SYMBOLS_QUERY` variable L101-152 — `: &str` — Tree-sitter query for extracting Rust symbols.
-  `IMPORTS_QUERY` variable L155-175 — `: &str` — Tree-sitter query for extracting use statements.
-  `CALLS_QUERY` variable L178-192 — `: &str` — Tree-sitter query for extracting function calls.
-  `EXTERNS_QUERY` variable L195-204 — `: &str` — Tree-sitter query for extracting extern blocks.
-  `IMPLS_QUERY` variable L207-212 — `: &str` — Tree-sitter query for extracting trait implementations.
-  `RustExtractor` type L217-553 — `= RustExtractor` — from Rust source code.
-  `build_struct_signature` function L556-586 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a struct: `{ field: Type, field2: Type2 }`.
-  `build_enum_signature` function L589-611 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for an enum: `Variant1 | Variant2 | Variant3`.
-  `build_trait_signature` function L614-639 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a trait: `{ fn method1(), fn method2() }`.
-  `build_const_signature` function L642-647 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a const/static: `: Type`.
-  `build_type_alias_signature` function L650-655 — `(node: tree_sitter::Node, source: &str) -> Option<String>` — Build a signature for a type alias: `= RHS`.
-  `parse_visibility` function L658-681 — `(vis_text: &str) -> Visibility` — Parse a Rust visibility modifier into our Visibility enum.
-  `extract_doc_comment` function L684-746 — `( tree: &tree_sitter::Tree, symbol_line: usize, source: &str, ) -> Option<String...` — Extract doc comments preceding a symbol by looking at source lines.
-  `tests` module L749-1020 — `-` — from Rust source code.
-  `parse_rust` function L752-758 — `(source: &str) -> tree_sitter::Tree` — from Rust source code.
-  `test_extract_struct` function L761-788 — `()` — from Rust source code.
-  `test_struct_multi_field_signature` function L791-805 — `()` — from Rust source code.
-  `test_extract_function` function L808-822 — `()` — from Rust source code.
-  `test_extract_enum` function L825-845 — `()` — from Rust source code.
-  `test_extract_trait` function L848-870 — `()` — from Rust source code.
-  `test_extract_impl` function L873-892 — `()` — from Rust source code.
-  `test_extract_module` function L895-910 — `()` — from Rust source code.
-  `test_extract_imports` function L913-924 — `()` — from Rust source code.
-  `test_extract_calls` function L927-941 — `()` — from Rust source code.
-  `test_extract_ffi_markers` function L944-957 — `()` — from Rust source code.
-  `test_visibility_parsing` function L960-971 — `()` — from Rust source code.
-  `test_extract_constant` function L974-988 — `()` — from Rust source code.
-  `test_extract_type_alias` function L991-1004 — `()` — from Rust source code.
-  `test_extract_macro` function L1007-1019 — `()` — from Rust source code.

#### crates/metis-code-index/src/lang/typescript.rs

- pub `Import` struct L18-29 — `{ source: String, names: Vec<String>, default_name: Option<String>, namespace: O...` — Import statement from TypeScript/JavaScript source.
- pub `TypeScriptExtractor` struct L199 — `-` — TypeScript/JavaScript symbol extractor.
- pub `extract_symbols` function L203-217 — `( tree: &Tree, source: &str, file_path: &str, language: Language, ) -> Result<Ve...` — Extract symbols from a parsed TypeScript or JavaScript syntax tree.
- pub `extract_imports` function L468-512 — `( tree: &Tree, source: &str, language: Language, ) -> Result<Vec<Import>, String...` — Extract import statements from TypeScript or JavaScript.
-  `TypeScriptQueries` struct L32-35 — `{ symbols: Query, imports: Query }` — Compiled tree-sitter queries for TypeScript.
-  `JavaScriptQueries` struct L38-41 — `{ symbols: Query, imports: Query }` — Compiled tree-sitter queries for JavaScript.
-  `TS_QUERIES` variable L43 — `: OnceLock<Result<TypeScriptQueries, String>>` — and the JavaScript grammar for JS files.
-  `JS_QUERIES` variable L44 — `: OnceLock<Result<JavaScriptQueries, String>>` — and the JavaScript grammar for JS files.
-  `get_ts_queries` function L46-58 — `() -> Result<&'static TypeScriptQueries, &'static str>` — and the JavaScript grammar for JS files.
-  `get_js_queries` function L60-72 — `() -> Result<&'static JavaScriptQueries, &'static str>` — and the JavaScript grammar for JS files.
-  `TS_SYMBOLS_QUERY` variable L75-139 — `: &str` — Tree-sitter query for extracting TypeScript symbols.
-  `JS_SYMBOLS_QUERY` variable L142-179 — `: &str` — Tree-sitter query for extracting JavaScript symbols.
-  `TS_IMPORTS_QUERY` variable L182-186 — `: &str` — Tree-sitter query for extracting TypeScript imports.
-  `JS_IMPORTS_QUERY` variable L189-193 — `: &str` — Tree-sitter query for extracting JavaScript imports.
-  `TypeScriptExtractor` type L201-902 — `= TypeScriptExtractor` — and the JavaScript grammar for JS files.
-  `extract_ts_symbols` function L219-376 — `( tree: &Tree, source: &str, file_path: &str, ) -> Result<Vec<Symbol>, String>` — and the JavaScript grammar for JS files.
-  `extract_js_symbols` function L378-465 — `( tree: &Tree, source: &str, file_path: &str, ) -> Result<Vec<Symbol>, String>` — and the JavaScript grammar for JS files.
-  `extract_import_from_match` function L514-564 — `( match_: &tree_sitter::QueryMatch, query: &Query, source: &str, ) -> Option<Imp...` — and the JavaScript grammar for JS files.
-  `extract_import_details` function L566-637 — `( import_node: &tree_sitter::Node, source: &str, names: &mut Vec<String>, defaul...` — and the JavaScript grammar for JS files.
-  `extract_function_symbol` function L639-667 — `( node: &tree_sitter::Node, source: &str, file_path: &str, is_export: bool, ) ->...` — and the JavaScript grammar for JS files.
-  `extract_class_symbol` function L669-721 — `( node: &tree_sitter::Node, source: &str, file_path: &str, is_export: bool, is_t...` — and the JavaScript grammar for JS files.
-  `build_function_signature` function L723-740 — `( node: &tree_sitter::Node, source: &str, name: &str, ) -> Option<String>` — and the JavaScript grammar for JS files.
-  `build_method_signature` function L742-758 — `( node: &tree_sitter::Node, source: &str, name: &str, ) -> Option<String>` — and the JavaScript grammar for JS files.
-  `build_interface_signature` function L760-800 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — and the JavaScript grammar for JS files.
-  `build_type_alias_signature` function L802-806 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — and the JavaScript grammar for JS files.
-  `build_enum_signature` function L808-833 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — and the JavaScript grammar for JS files.
-  `build_arrow_signature` function L835-852 — `( node: &tree_sitter::Node, source: &str, name: &str, ) -> Option<String>` — and the JavaScript grammar for JS files.
-  `extract_jsdoc` function L854-879 — `(node: &tree_sitter::Node, source: &str) -> Option<String>` — and the JavaScript grammar for JS files.
-  `deduplicate_symbols` function L884-897 — `(symbols: &mut Vec<Symbol>)` — Deduplicate symbols by (name, start_line, kind).
-  `node_text` function L899-901 — `(node: &tree_sitter::Node, source: &str) -> String` — and the JavaScript grammar for JS files.
-  `tests` module L905-1325 — `-` — and the JavaScript grammar for JS files.
-  `parse_typescript` function L909-913 — `(source: &str) -> Tree` — and the JavaScript grammar for JS files.
-  `parse_javascript` function L915-919 — `(source: &str) -> Tree` — and the JavaScript grammar for JS files.
-  `test_extract_ts_function` function L924-942 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_class` function L945-979 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_interface` function L982-1014 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_type_alias` function L1017-1043 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_enum` function L1046-1079 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_arrow_function` function L1082-1113 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_ts_imports` function L1116-1135 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_tsx_component` function L1138-1169 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_js_function` function L1174-1195 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_js_class` function L1198-1237 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_js_arrow_function` function L1240-1260 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_js_imports` function L1263-1280 — `()` — and the JavaScript grammar for JS files.
-  `test_extract_js_mixed` function L1283-1324 — `()` — and the JavaScript grammar for JS files.

### crates/metis-docs-cli/src

**Role**: The `metis` command-line binary — parses user commands, locates the workspace, and delegates each subcommand to its handler in `commands/`.

**Key abstractions**:
- `Cli` / `Commands` — Clap-derived root parser. `execute()` dispatches to the concrete `*Command` struct for the chosen subcommand. Verbosity (`-v`/`-vv`/`-vvv`) controls the tracing log level.
- `has_metis_vault()` in `workspace.rs` — Walks up the directory tree from `CWD` to find a `.metis/` directory, returning `(bool, Option<PathBuf>)`. Called by nearly every command as its first action; commands fail fast with a clear message if no workspace is found.
- `utils.rs` — Shared utilities used across command implementations.

**Internal flow**: `main()` calls `Cli::parse()` then `Cli::execute()`. Each command struct is instantiated from CLI arguments by Clap and its `execute()` method is called. Most commands call `has_metis_vault()` first, then open the SQLite database via `metis_core::dal::Database`, construct the appropriate service from `metis_core::application::services`, run the operation, and print results to stdout.

**Dependencies**: `metis_core` for all business logic; `clap` for argument parsing; `tracing` / `tracing_subscriber` for logging.

#### crates/metis-docs-cli/src/cli.rs

- pub `Cli` struct L15-22 — `{ verbose: u8, command: Commands }`
- pub `Commands` enum L25-50 — `Init | Sync | Create | Search | Transition | List | Status | Archive | Validate ...`
- pub `init_logging` function L53-65 — `(&self)`
- pub `execute` function L67-82 — `(&self) -> Result<()>`
-  `Cli` type L52-83 — `= Cli`
-  `tests` module L86-274 — `-`
-  `test_comprehensive_cli_workflow` function L97-273 — `()`

#### crates/metis-docs-cli/src/lib.rs

- pub `cli` module L5 — `-` — This library exposes CLI components for testing purposes.
- pub `commands` module L6 — `-` — This library exposes CLI components for testing purposes.
- pub `workspace` module L7 — `-` — This library exposes CLI components for testing purposes.

#### crates/metis-docs-cli/src/main.rs

-  `cli` module L1 — `-`
-  `commands` module L2 — `-`
-  `utils` module L3 — `-`
-  `workspace` module L4 — `-`
-  `main` function L11-20 — `() -> Result<()>`

#### crates/metis-docs-cli/src/workspace.rs

- pub `has_metis_vault` function L10-33 — `() -> (bool, Option<PathBuf>)` — Check if we're in a Metis workspace by walking up the directory tree
-  `tests` module L36-65 — `-`
-  `test_has_metis_vault_false_when_no_directory` function L41-47 — `()`
-  `test_has_metis_vault_true_when_valid` function L50-64 — `()`

### crates/metis-docs-cli/src/commands

**Role**: Houses one file per CLI subcommand; each file owns the argument struct and the `execute()` implementation for that command.

**Key abstractions**:
- `IndexCommand` — Drives the code-index pipeline: walks the project directory, parses source files via `metis_code_index::Parser`, dispatches to language extractors, and calls `write_index_file()`. The `--incremental` flag uses `HashManifest` + `SymbolCache` to skip unchanged files. `--structure-only` skips parsing entirely and writes only the project tree.
- `SyncCommand` — Calls `Application::sync_directory()`, which performs the full recovery-then-sync sequence: checks if database recovery is needed, syncs `config.toml` → DB, then walks the workspace markdown files and imports/updates/deletes database records. Prints a per-file change summary.
- `TransitionCommand` — Finds a document by short code, parses the target phase string, delegates to `PhaseTransitionService`, and prints the new phase. The auto-transition path (no phase specified) advances to the next phase in the document type's natural sequence.
- `StatusCommand` — Queries the database for all document types and phases, sorts results by actionability priority (active > todo > blocked > other), and renders them as a table, compact list, or JSON. Includes a phase-count insight block.
- `CreateCommand` / `create/` submodule — Dispatches to per-type creation functions (`create_new_strategy`, `create_new_initiative`, etc.) that call `DocumentCreationService` and print the new document's short code and path.
- `ConfigCommand` — Reads and writes `FlightLevelConfig` via `ConfigurationRepository`. The `set` action accepts either a named preset or individual `--strategies`/`--initiatives` boolean flags. Changes are persisted to both the database and `config.toml`.
- `McpCommand` — Launches `metis_docs_mcp::run()` (the MCP stdio server) in the current process. The CLI becomes the server process.

**Mixed concerns / gotchas**: `index.rs` duplicates the `extract_symbols_for_language()` dispatch function that also exists in `metis-docs-mcp/src/tools/index_code.rs` — both contain identical logic for routing a `ParsedFile` to the right extractor.

**Dependencies**: `metis_core` for all workspace services; `metis_code_index` for `IndexCommand`; `metis_docs_mcp` for `McpCommand`.

#### crates/metis-docs-cli/src/commands/archive.rs

- pub `ArchiveCommand` struct L7-14 — `{ short_code: String, document_type: Option<String> }`
- pub `execute` function L17-55 — `(&self) -> Result<()>`
-  `ArchiveCommand` type L16-56 — `= ArchiveCommand`
-  `tests` module L59-175 — `-`
-  `test_archive_command_no_workspace` function L65-91 — `()`
-  `test_archive_document_not_found` function L94-125 — `()`
-  `test_archive_vision_document` function L128-174 — `()`

#### crates/metis-docs-cli/src/commands/config.rs

- pub `ConfigCommand` struct L7-10 — `{ action: ConfigAction }`
- pub `ConfigAction` enum L13-30 — `Show | Set | Get`
- pub `execute` function L33-61 — `(&self) -> Result<()>`
-  `ConfigCommand` type L32-151 — `= ConfigCommand`
-  `show_config` function L63-86 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `set_config` function L88-130 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `get_config` function L132-150 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `tests` module L154-287 — `-`
-  `test_config_show_default` function L160-189 — `()`
-  `test_config_set_streamlined_preset` function L192-224 — `()`
-  `test_config_set_invalid_preset` function L227-260 — `()`
-  `test_config_without_workspace` function L263-286 — `()`

#### crates/metis-docs-cli/src/commands/index.rs

- pub `IndexCommand` struct L24-32 — `{ structure_only: bool, incremental: bool }` — parse with tree-sitter, extract symbols, and write the markdown index.
- pub `execute` function L35-120 — `(&self) -> Result<()>` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `IndexCommand` type L34-209 — `= IndexCommand` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `extract_incremental` function L123-208 — `( &self, walk_result: &metis_code_index::walker::WalkResult, hash_path: &Path, s...` — Perform incremental indexing: only re-parse changed files, use cached symbols for the rest.
-  `extract_symbols_for_language` function L212-230 — `( language: Language, parsed: &ParsedFile, file_path: &str, ) -> Result<Vec<Symb...` — Dispatch symbol extraction to the appropriate language extractor.
-  `extract_all_symbols` function L234-266 — `( walk_result: &metis_code_index::walker::WalkResult, ) -> (BTreeMap<PathBuf, Ve...` — Parse and extract symbols from all files in the walk result.
-  `tests` module L269-519 — `-` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_no_workspace` function L276-295 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_generates_file` function L298-340 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_structure_only` function L343-384 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_full_index_creates_hash_files` function L387-422 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_incremental_skips_unchanged` function L425-466 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_incremental_detects_changes` function L469-518 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.

#### crates/metis-docs-cli/src/commands/init.rs

- pub `InitCommand` struct L11-24 — `{ name: Option<String>, prefix: Option<String>, preset: Option<String>, initiati...`
- pub `execute` function L27-92 — `(&self) -> Result<()>`
-  `InitCommand` type L26-145 — `= InitCommand`
-  `determine_project_prefix` function L95-118 — `(&self, project_name: &str) -> String` — Determine the project prefix from command arguments or project name
-  `determine_flight_config` function L121-144 — `(&self) -> Result<FlightLevelConfig>` — Determine the flight level configuration based on command arguments
-  `tests` module L148-429 — `-`
-  `test_init_command_creates_workspace` function L154-219 — `()`
-  `test_init_command_workspace_already_exists` function L222-255 — `()`
-  `test_init_command_default_name` function L258-286 — `()`
-  `test_init_command_with_preset` function L289-327 — `()`
-  `test_init_command_with_custom_flags` function L330-362 — `()`
-  `test_init_command_default_streamlined` function L365-401 — `()`
-  `test_init_command_invalid_preset` function L404-428 — `()`

#### crates/metis-docs-cli/src/commands/list.rs

- pub `OutputFormat` enum L9-17 — `Table | Compact | Json` — Output format for CLI commands
- pub `ListCommand` struct L20-40 — `{ document_type: Option<String>, phase: Option<String>, all: bool, include_archi...`
- pub `execute` function L53-114 — `(&self) -> Result<()>`
-  `DocumentOutput` struct L44-50 — `{ doc_type: String, code: String, title: String, phase: String }` — JSON-serializable document for output
-  `ListCommand` type L52-211 — `= ListCommand`
-  `list_all_documents` function L116-154 — `( &self, repo: &mut metis_core::dal::database::repository::DocumentRepository, )...`
-  `display_table` function L158-176 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display documents as a human-readable table
-  `display_compact` function L180-184 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display documents in compact format (one line per document)
-  `display_json` function L187-202 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display documents as JSON array
-  `truncate_string` function L204-210 — `(&self, s: &str, max_len: usize) -> String`
-  `tests` module L214-287 — `-`
-  `test_list_command_no_workspace` function L220-249 — `()`
-  `test_list_command_empty_workspace` function L252-286 — `()`

#### crates/metis-docs-cli/src/commands/mcp.rs

- pub `McpCommand` struct L5-9 — `{ log_level: String }`
- pub `execute` function L12-18 — `(&self) -> Result<()>`
-  `McpCommand` type L11-19 — `= McpCommand`

#### crates/metis-docs-cli/src/commands/mod.rs

- pub `archive` module L1 — `-`
- pub `config` module L2 — `-`
- pub `create` module L3 — `-`
- pub `index` module L4 — `-`
- pub `init` module L5 — `-`
- pub `list` module L6 — `-`
- pub `mcp` module L7 — `-`
- pub `search` module L8 — `-`
- pub `status` module L9 — `-`
- pub `sync` module L10 — `-`
- pub `transition` module L11 — `-`
- pub `validate` module L12 — `-`

#### crates/metis-docs-cli/src/commands/search.rs

- pub `SearchCommand` struct L10-21 — `{ query: String, limit: usize, format: OutputFormat }`
- pub `execute` function L33-77 — `(&self) -> Result<()>`
-  `SearchResultOutput` struct L25-30 — `{ code: String, title: String, doc_type: String }` — JSON-serializable search result for output
-  `SearchCommand` type L32-133 — `= SearchCommand`
-  `perform_search` function L79-82 — `(&self, app: &mut Application, query: &str) -> Result<Vec<Document>>`
-  `display_table` function L86-107 — `(&self, documents: &[Document])` — Display results as a human-readable table
-  `display_compact` function L111-115 — `(&self, documents: &[Document])` — Display results in compact format (one line per document)
-  `display_json` function L118-132 — `(&self, documents: &[Document])` — Display results as JSON array
-  `truncate` function L136-142 — `(s: &str, max_len: usize) -> String`
-  `tests` module L145-154 — `-`
-  `test_truncate` function L149-153 — `()`

#### crates/metis-docs-cli/src/commands/status.rs

- pub `StatusCommand` struct L9-17 — `{ include_archived: bool, format: OutputFormat }`
- pub `execute` function L105-144 — `(&self) -> Result<()>`
-  `StatusOutput` struct L21-30 — `{ code: String, title: String, doc_type: String, phase: String, blocked_by: Stri...` — JSON-serializable status row for output
-  `StatusCommand` type L32-308 — `= StatusCommand`
-  `get_document_types` function L36-38 — `() -> &'static [&'static str]` — Get all document types to query
-  `connect_to_database` function L41-53 — `( ) -> Result<metis_core::dal::database::repository::DocumentRepository>` — Initialize database connection from workspace
-  `fetch_documents` function L56-74 — `( &self, repo: &mut metis_core::dal::database::repository::DocumentRepository, )...` — Fetch and filter documents from repository
-  `sort_documents_by_priority` function L77-92 — `(&self, docs: &mut [metis_core::dal::database::models::Document])` — Sort documents by actionability and recency
-  `count_documents_by_phase` function L95-103 — `( &self, documents: &[metis_core::dal::database::models::Document], ) -> (usize,...` — Count documents by phase for insights
-  `get_action_priority` function L146-159 — `(&self, doc: &metis_core::dal::database::models::Document) -> u8`
-  `display_table` function L162-189 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display status as a human-readable table
-  `display_compact` function L192-204 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display status in compact format (one line per document)
-  `display_json` function L207-226 — `(&self, documents: &[metis_core::dal::database::models::Document])` — Display status as JSON array
-  `extract_blocked_by_info` function L228-251 — `(&self, doc: &metis_core::dal::database::models::Document) -> String`
-  `format_relative_time` function L253-282 — `(&self, dt: chrono::DateTime<chrono::Utc>) -> String`
-  `display_insights` function L284-299 — `(&self, documents: &[metis_core::dal::database::models::Document])`
-  `truncate_string` function L301-307 — `(&self, s: &str, max_len: usize) -> String`
-  `tests` module L311-419 — `-`
-  `test_status_command_no_workspace` function L317-343 — `()`
-  `test_status_command_empty_workspace` function L346-377 — `()`
-  `test_action_priority` function L380-418 — `()`

#### crates/metis-docs-cli/src/commands/sync.rs

- pub `SyncCommand` struct L7 — `-`
- pub `execute` function L10-111 — `(&self) -> Result<()>`
-  `SyncCommand` type L9-112 — `= SyncCommand`
-  `tests` module L115-174 — `-`
-  `test_sync_command_no_workspace` function L122-142 — `()`
-  `test_sync_command_with_workspace` function L145-173 — `()`

#### crates/metis-docs-cli/src/commands/transition.rs

- pub `TransitionCommand` struct L10-16 — `{ short_code: String, phase: Option<String> }`
- pub `execute` function L19-58 — `(&self) -> Result<()>`
-  `TransitionCommand` type L18-80 — `= TransitionCommand`
-  `parse_phase` function L60-79 — `(&self, phase_str: &str) -> Result<Phase>`
-  `tests` module L83-538 — `-`
-  `test_parse_phase` function L90-100 — `()`
-  `test_transition_command_no_workspace` function L103-129 — `()`
-  `test_find_document_not_found` function L132-163 — `()`
-  `test_vision_full_transition_sequence` function L166-222 — `()`
-  `test_initiative_full_transition_sequence` function L225-272 — `()`
-  `test_task_full_transition_sequence` function L275-371 — `()`
-  `test_adr_full_transition_sequence` function L374-437 — `()`
-  `test_invalid_transitions` function L440-487 — `()`
-  `test_auto_transitions` function L490-537 — `()`

#### crates/metis-docs-cli/src/commands/validate.rs

- pub `ValidateCommand` struct L7-10 — `{ file_path: PathBuf }`
- pub `execute` function L13-41 — `(&self) -> Result<()>`
-  `ValidateCommand` type L12-42 — `= ValidateCommand`
-  `tests` module L45-140 — `-`
-  `test_validate_command_missing_file` function L52-75 — `()`
-  `test_validate_command_valid_vision` function L78-109 — `()`
-  `test_validate_command_invalid_document` function L112-139 — `()`

### crates/metis-docs-cli/src/commands/create

**Role**: Implements the `metis create` subcommands, one file per document type, each providing a public function that gathers parent references and calls `DocumentCreationService`.

**Key abstractions**:
- `create_new_strategy()` — Accepts a title and optional vision slug. When a slug is supplied, `get_vision_document_id()` scans the workspace's `vision/` directory for a matching document before calling `DocumentCreationService::create_strategy()`.
- `create_new_initiative()` — Requires a strategy short code; `find_strategy()` locates it on disk and returns its `DocumentId`. Falls back to listing available strategies (via `list_available_strategies()`) if the supplied ID is not found. Interactively prompts for complexity via `prompt_for_complexity()`.
- `create_new_task()` — Requires an initiative short code; `find_initiative()` looks it up in the database. `find_strategy_for_initiative()` then determines the grandparent strategy ID needed by `DocumentCreationService::create_task()`.
- `create_new_adr()` — Simpler than other types; calls `DocumentCreationService::create_adr()` with title only, using auto-incremented ADR numbering.

**Internal flow**: Each function: validates workspace exists → looks up parent document → calls `DocumentCreationService` with `DocumentCreationConfig` → prints the resulting short code and file path.

**Mixed concerns / gotchas**: The parent lookup logic (`find_strategy`, `find_initiative`) mixes filesystem scanning with database queries depending on what's available — the initiative lookup uses the DB, while the strategy lookup for `create_new_initiative` scans files directly.

#### crates/metis-docs-cli/src/commands/create/adr.rs

- pub `create_new_adr` function L9-40 — `(title: &str) -> Result<()>` — Create a new ADR document with defaults and write to file
-  `tests` module L43-142 — `-`
-  `test_create_new_adr_no_workspace` function L51-67 — `()`
-  `test_create_new_adr_with_workspace` function L70-141 — `()`

#### crates/metis-docs-cli/src/commands/create/initiative.rs

- pub `create_new_initiative` function L11-53 — `(title: &str, vision_id: &str) -> Result<()>` — Create a new Initiative document with defaults and write to file
-  `find_vision` function L56-98 — `( workspace_dir: &std::path::Path, vision_id: &str, ) -> Result<metis_core::doma...` — Find a vision by short code and return its DocumentId
-  `prompt_for_complexity` function L101-128 — `() -> Result<Complexity>` — Prompt user to select complexity level
-  `tests` module L131-224 — `-`
-  `test_create_new_initiative_no_workspace` function L136-154 — `()`
-  `test_find_vision_not_found` function L157-192 — `()`
-  `test_create_initiative_under_vision` function L195-223 — `()`

#### crates/metis-docs-cli/src/commands/create/mod.rs

- pub `CreateCommand` struct L11-14 — `{ document_type: CreateCommands }`
- pub `CreateCommands` enum L17-47 — `Initiative | Task | Adr | Specification`
- pub `execute` function L50-72 — `(&self) -> Result<()>`
-  `adr` module L1 — `-`
-  `initiative` module L2 — `-`
-  `specification` module L3 — `-`
-  `task` module L4 — `-`
-  `CreateCommand` type L49-73 — `= CreateCommand`

#### crates/metis-docs-cli/src/commands/create/specification.rs

- pub `create_new_specification` function L9-44 — `(title: &str, parent: &str) -> Result<()>` — Create a new Specification document with defaults and write to file
-  `tests` module L47-138 — `-`
-  `test_create_new_specification_no_workspace` function L55-69 — `()`
-  `test_create_new_specification_with_workspace` function L72-137 — `()`

#### crates/metis-docs-cli/src/commands/create/task.rs

- pub `create_new_task` function L10-44 — `(title: &str, initiative_id: &str) -> Result<()>` — Create a new Task document with defaults and write to file
-  `find_initiative` function L47-77 — `( workspace_dir: &Path, initiative_id: &str, ) -> Result<(metis_core::domain::do...` — Find an initiative by short code and return its DocumentId and file path
-  `tests` module L80-148 — `-`
-  `test_create_new_task_no_workspace` function L85-103 — `()`
-  `test_find_initiative_not_found` function L106-147 — `()`

### crates/metis-docs-cli/tests

**Role**: End-to-end functional tests that simulate complete user workflows through the CLI command structs directly, without spawning a subprocess.

**Key abstractions**:
- `init_workspace()` — Helper that calls `InitCommand::execute()` with configurable name, prefix, and preset, used as the setup step for every test.
- `verify_workspace()` / `verify_config_toml()` — Assertion helpers confirming the `.metis/` directory exists and that `config.toml` contains the expected project prefix.

**Internal flow**: Tests construct command structs with `TempDir`-based paths, call `execute()` directly, then assert on the resulting filesystem state (directory presence, document counts, configuration values). This avoids binary compilation overhead while testing real code paths end-to-end.

#### crates/metis-docs-cli/tests/comprehensive_functional_test.rs

- pub `init_workspace` function L13-36 — `( path: &PathBuf, name: Option<&str>, prefix: Option<&str>, preset: Option<&str>...` — These tests simulate real user command sequences through the CLI
- pub `verify_workspace` function L38-48 — `(path: &PathBuf) -> bool` — These tests simulate real user command sequences through the CLI
- pub `verify_config_toml` function L50-59 — `(path: &PathBuf, expected_prefix: &str) -> bool` — These tests simulate real user command sequences through the CLI
-  `cli_helpers` module L9-60 — `-` — Helper to run CLI commands programmatically
-  `test_complete_streamlined_workflow` function L63-135 — `()` — These tests simulate real user command sequences through the CLI
-  `test_complete_direct_configuration_workflow` function L138-188 — `()` — These tests simulate real user command sequences through the CLI
-  `test_config_toml_persistence` function L191-260 — `()` — These tests simulate real user command sequences through the CLI
-  `test_custom_prefix_handling` function L263-305 — `()` — These tests simulate real user command sequences through the CLI

### crates/metis-docs-core/src/application

**Role**: The application layer coordinator — holds the database handle and exposes high-level, orchestrated operations that compose multiple services.

**Key abstractions**:
- `Application` — Owns a `Database` instance and provides two generic execution contexts: `with_database()` for raw CRUD via `DatabaseService`, and `with_sync()` for file-to-database synchronization via `SyncService`. Both create fresh service instances per call to ensure connection freshness.
- `sync_directory()` — The main composite operation. It first checks whether the database needs recovery (missing/corrupt) and runs `ConfigurationRecoveryService` if so; otherwise performs a lightweight `config.toml` → database sync. It then runs `SyncService::sync_directory()` to reconcile markdown files with the database. This three-step sequence ensures the database is always in a consistent state before file sync.

**Dependencies**: `Database` from `dal`; `DatabaseService`, `SyncService`, `ConfigurationRecoveryService` from `application::services`.

#### crates/metis-docs-core/src/application/mod.rs

- pub `services` module L1 — `-`
- pub `Application` struct L9-11 — `{ database: Database }` — Application layer coordinator
- pub `new` function L15-17 — `(database: Database) -> Self` — Create a new application instance
- pub `with_database` function L20-30 — `(&mut self, f: F) -> R` — Execute a database operation
- pub `with_sync` function L33-44 — `(&mut self, f: F) -> R` — Execute a sync operation
- pub `sync_directory` function L53-108 — `( self, dir_path: P, ) -> Result<Vec<services::synchronization::SyncResult>>` — Convenience method to sync a directory
- pub `database` function L111-113 — `(&mut self) -> &mut Database` — Get access to the underlying database
-  `Application` type L13-114 — `= Application`

### crates/metis-docs-core/src/application/services

**Role**: Service layer that separates business operations into focused units: database CRUD, filesystem I/O, document-level operations, workspace lifecycle, and synchronization.

**Key abstractions**:
- `DatabaseService` — Thin façade over `DocumentRepository`, adding hierarchy queries (`find_strategy_hierarchy`, `find_initiative_hierarchy`) and short code generation. This is the primary write path for all document database records.
- `SyncService` — The bidirectional bridge between the filesystem and the SQLite database. `import_from_file()` parses a markdown file and upserts a `Document` record; `export_to_file()` does the reverse. `sync_directory()` walks all markdown files and reconciles them. Handles path moves (same ID, different path), short code collision resolution (renumbers documents and updates cross-references in sibling files), and counter recovery.
- `FilesystemService` — Stateless utility for file read/write, SHA256 hashing, `mtime` retrieval, and recursive markdown file discovery. Pure I/O operations with no business logic.
- `TemplateLoader` — Resolves document content templates using a three-level fallback: project-local template file → global template file → embedded default string. The embedded defaults live in `template.rs`'s `defaults` module as `CONTENT` and `EXIT_CRITERIA` string constants per document type.
- `DocumentCreationService` — Creates new documents of any type by: generating a short code via `ConfigurationRepository`, rendering content from the `TemplateLoader`, writing the file, and registering the relationship to the parent in the database.
- `DeletionService` — Recursively deletes a document and all its children, removing both files and empty parent directories.
- `DocumentDiscoveryService` — Locates documents by short code or ID across all document type directories, using a combination of path pattern matching and database lookups. The fast path goes through the DB; the slow path scans the filesystem.
- `DocumentValidationService` — Reads a file, detects its document type from frontmatter, and runs type-specific validation rules. Returns a `ValidationResult` with specific error strings.

**Dependencies**: `dal::database` for persistence; `domain` types for document objects; `tera` for template rendering (via `TemplateLoader`).

#### crates/metis-docs-core/src/application/services/database.rs

- pub `DatabaseService` struct L6-8 — `{ repository: DocumentRepository }` — Database service - handles all database CRUD operations
- pub `new` function L11-13 — `(repository: DocumentRepository) -> Self`
- pub `create_document` function L16-18 — `(&mut self, document: NewDocument) -> Result<Document>` — Create a new document in the database
- pub `find_by_filepath` function L21-23 — `(&mut self, filepath: &str) -> Result<Option<Document>>` — Find a document by filepath
- pub `find_by_id` function L26-28 — `(&mut self, id: &str) -> Result<Option<Document>>` — Find a document by ID
- pub `find_by_short_code` function L31-33 — `(&mut self, short_code: &str) -> Result<Option<Document>>` — Find a document by short code
- pub `update_document` function L36-38 — `(&mut self, filepath: &str, document: &Document) -> Result<Document>` — Update an existing document
- pub `delete_document` function L41-43 — `(&mut self, filepath: &str) -> Result<bool>` — Delete a document from the database
- pub `search_documents` function L46-48 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search documents using full-text search
- pub `search_documents_unarchived` function L51-53 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search non-archived documents using full-text search
- pub `find_by_type` function L56-59 — `(&mut self, doc_type: DocumentType) -> Result<Vec<Document>>` — Get all documents of a specific type
- pub `find_by_tag` function L62-64 — `(&mut self, tag: &str) -> Result<Vec<Document>>` — Get documents with a specific tag
- pub `get_tags_for_document` function L67-69 — `(&mut self, doc_filepath: &str) -> Result<Vec<String>>` — Get all tags for a specific document
- pub `find_children` function L72-74 — `(&mut self, parent_id: &str) -> Result<Vec<Document>>` — Get all children of a document
- pub `find_parent` function L77-79 — `(&mut self, child_id: &str) -> Result<Option<Document>>` — Get the parent of a document
- pub `create_relationship` function L82-96 — `( &mut self, parent_id: &str, child_id: &str, parent_filepath: &str, child_filep...` — Create a parent-child relationship
- pub `document_exists` function L99-101 — `(&mut self, filepath: &str) -> Result<bool>` — Check if a document exists by filepath
- pub `count_by_type` function L104-107 — `(&mut self, doc_type: DocumentType) -> Result<usize>` — Get document count by type
- pub `get_all_id_filepath_pairs` function L110-128 — `(&mut self) -> Result<Vec<(String, String)>>` — Get all document IDs and their filepaths (useful for validation)
- pub `find_by_initiative_id` function L131-133 — `(&mut self, initiative_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to an initiative
- pub `find_initiative_hierarchy` function L136-138 — `(&mut self, initiative_id: &str) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy (initiative + its tasks)
- pub `find_initiative_hierarchy_by_short_code` function L141-147 — `( &mut self, initiative_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy by short code (initiative + its tasks)
- pub `generate_short_code` function L150-155 — `(&mut self, doc_type: &str) -> Result<String>` — Generate a short code for a document type (requires db_path)
- pub `set_counter_if_lower` function L159-163 — `(&mut self, _doc_type: &str, _min_value: u32) -> Result<bool>` — Set counter if the current value is lower than the provided value
-  `DatabaseService` type L10-164 — `= DatabaseService`
-  `tests` module L167-365 — `-`
-  `setup_service` function L171-174 — `() -> DatabaseService`
-  `create_test_document` function L176-194 — `() -> NewDocument`
-  `create_test_document_with_lineage` function L196-229 — `( id: &str, doc_type: &str, filepath: &str, initiative_id: Option<String>, ) -> ...`
-  `test_database_service_crud` function L232-265 — `()`
-  `test_database_service_relationships` function L268-313 — `()`
-  `test_lineage_queries` function L316-364 — `()`

#### crates/metis-docs-core/src/application/services/filesystem.rs

- pub `FilesystemService` struct L8 — `-` — Filesystem operations service
- pub `read_file` function L12-14 — `(path: P) -> Result<String>` — Read file contents from disk
- pub `write_file` function L17-24 — `(path: P, content: &str) -> Result<()>` — Write file contents to disk
- pub `file_exists` function L27-29 — `(path: P) -> bool` — Check if file exists
- pub `compute_file_hash` function L32-37 — `(path: P) -> Result<String>` — Compute SHA256 hash of file contents
- pub `compute_content_hash` function L40-44 — `(content: &str) -> String` — Compute SHA256 hash of string content
- pub `get_file_mtime` function L47-57 — `(path: P) -> Result<f64>` — Get file modification time as Unix timestamp
- pub `delete_file` function L60-62 — `(path: P) -> Result<()>` — Delete a file
- pub `find_markdown_files` function L65-84 — `(dir: P) -> Result<Vec<String>>` — List all markdown files in a directory recursively
-  `FilesystemService` type L10-85 — `= FilesystemService`
-  `tests` module L88-183 — `-`
-  `test_write_and_read_file` function L93-108 — `()`
-  `test_compute_hashes` function L111-131 — `()`
-  `test_file_operations` function L134-151 — `()`
-  `test_find_markdown_files` function L154-182 — `()`

#### crates/metis-docs-core/src/application/services/mod.rs

- pub `database` module L1 — `-`
- pub `document` module L2 — `-`
- pub `filesystem` module L3 — `-`
- pub `synchronization` module L4 — `-`
- pub `template` module L5 — `-`
- pub `workspace` module L6 — `-`

#### crates/metis-docs-core/src/application/services/synchronization.rs

- pub `SyncService` struct L12-16 — `{ db_service: &'a mut DatabaseService, workspace_dir: Option<&'a Path>, db_path:...` — Synchronization service - bridges filesystem and database
- pub `new` function L19-25 — `(db_service: &'a mut DatabaseService) -> Self`
- pub `with_workspace_dir` function L31-36 — `(mut self, workspace_dir: &'a Path) -> Self` — Set the workspace directory for lineage extraction
- pub `import_from_file` function L63-90 — `(&mut self, file_path: P) -> Result<Document>` — Direction 1: File → DocumentObject → Database
- pub `export_to_file` function L94-114 — `(&mut self, filepath: &str) -> Result<()>` — Direction 2: Database → DocumentObject → File
- pub `sync_file` function L550-616 — `(&mut self, file_path: P) -> Result<SyncResult>` — Synchronize a single file between filesystem and database using directional methods
- pub `sync_directory` function L619-665 — `(&mut self, dir_path: P) -> Result<Vec<SyncResult>>` — Sync all markdown files in a directory
- pub `verify_sync` function L668-708 — `(&mut self, dir_path: P) -> Result<Vec<SyncIssue>>` — Verify database and filesystem are in sync
- pub `recover_counters_from_filesystem` function L751-865 — `( &self, dir_path: P, ) -> Result<std::collections::HashMap<String, u32>>` — Recover short code counters from filesystem by scanning all documents
- pub `SyncResult` enum L895-908 — `Imported | Updated | Deleted | UpToDate | NotFound | Error | Moved | Renumbered` — Result of synchronizing a single document
- pub `filepath` function L912-923 — `(&self) -> &str` — Get the filepath for this result
- pub `is_change` function L926-935 — `(&self) -> bool` — Check if this result represents a change
- pub `is_error` function L938-940 — `(&self) -> bool` — Check if this result represents an error
- pub `SyncIssue` enum L945-949 — `MissingFromDatabase | MissingFromFilesystem | OutOfSync` — Issues found during sync verification
-  `to_relative_path` function L40-48 — `(&self, absolute_path: P) -> String` — Convert absolute path to relative path (relative to workspace directory)
-  `to_absolute_path` function L52-59 — `(&self, relative_path: &str) -> std::path::PathBuf` — Convert relative path to absolute path (prepends workspace directory)
-  `domain_to_database_model` function L117-170 — `( &self, document_obj: &dyn DocumentTrait, filepath: &str, file_hash: String, up...` — Convert domain object to database model
-  `extract_lineage_from_path` function L174-222 — `( file_path: P, workspace_dir: &Path, ) -> Option<DocumentId>` — Extract lineage information from file path
-  `is_backlog_path` function L226-243 — `(file_path: P, workspace_dir: &Path) -> bool` — Check if a file path is within the backlog directory
-  `extract_document_short_code` function L246-278 — `(file_path: P) -> Result<String>` — Extract document short code from file without keeping the document object around
-  `update_moved_document` function L281-293 — `( &mut self, existing_doc: &Document, new_file_path: P, ) -> Result<()>` — Update a document that has been moved to a new path
-  `resolve_short_code_collisions` function L297-377 — `( &mut self, dir_path: P, ) -> Result<Vec<SyncResult>>` — Detect and resolve short code collisions across all markdown files
-  `renumber_document` function L381-487 — `( &mut self, file_path: P, old_short_code: &str, ) -> Result<String>` — Renumber a single document to resolve short code collision
-  `update_sibling_references` function L490-547 — `( &mut self, file_path: P, old_short_code: &str, new_short_code: &str, ) -> Resu...` — Update cross-references in sibling documents (same directory)
-  `update_counters_from_filesystem` function L712-742 — `(&mut self, dir_path: P) -> Result<()>` — Update counters in database based on max values seen in filesystem
-  `is_valid_short_code_format` function L868-890 — `(short_code: &str) -> bool` — Validate short code format: PREFIX-TYPE-NNNN
-  `SyncResult` type L910-941 — `= SyncResult`
-  `tests` module L952-1194 — `-`
-  `setup_services` function L957-967 — `() -> (tempfile::TempDir, DatabaseService)`
-  `create_test_document_content` function L969-984 — `() -> String`
-  `test_import_from_file` function L987-1006 — `()`
-  `test_sync_file_operations` function L1009-1089 — `()`
-  `test_sync_directory` function L1092-1146 — `()`
-  `test_is_backlog_path` function L1149-1193 — `()`

#### crates/metis-docs-core/src/application/services/template.rs

- pub `vision` module L13-17 — `-` — 3.
- pub `CONTENT` variable L14 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L15-16 — `: &str` — 3.
- pub `initiative` module L19-23 — `-` — 3.
- pub `CONTENT` variable L20 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L21-22 — `: &str` — 3.
- pub `task` module L25-29 — `-` — 3.
- pub `CONTENT` variable L26 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L27-28 — `: &str` — 3.
- pub `adr` module L31-35 — `-` — 3.
- pub `CONTENT` variable L32 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L33-34 — `: &str` — 3.
- pub `specification` module L37-42 — `-` — 3.
- pub `CONTENT` variable L38-39 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L40-41 — `: &str` — 3.
- pub `TemplateError` enum L47-56 — `IoError | ParseError | ValidationError | UnknownDocumentType` — Error type for template loading operations
- pub `TemplateType` enum L73-76 — `Content | ExitCriteria` — Template types that can be loaded
- pub `TemplateLoader` struct L93-98 — `{ project_path: Option<PathBuf>, global_path: PathBuf }` — Service for loading templates with fallback chain support.
- pub `new` function L104-113 — `(project_path: Option<PathBuf>) -> Self` — Create a new TemplateLoader with the given project workspace path.
- pub `for_workspace` function L116-118 — `(workspace_dir: P) -> Self` — Create a TemplateLoader for a specific workspace directory.
- pub `load_content_template` function L124-126 — `(&self, doc_type: &str) -> Result<String, TemplateError>` — Load a content template for the given document type.
- pub `load_exit_criteria_template` function L129-131 — `(&self, doc_type: &str) -> Result<String, TemplateError>` — Load an exit criteria template for the given document type.
- pub `validate_template` function L202-215 — `(&self, template: &str, doc_type: &str) -> Result<(), TemplateError>` — Validate a template by rendering it with sample data.
- pub `sample_context_for_type` function L220-261 — `(&self, doc_type: &str) -> Context` — Generate sample context values for validating templates.
- pub `has_custom_template` function L264-283 — `(&self, doc_type: &str, template_type: TemplateType) -> bool` — Check if custom templates exist for a document type.
- pub `template_source` function L286-309 — `(&self, doc_type: &str, template_type: TemplateType) -> TemplateSource` — Get the source of a template (for debugging/info).
- pub `TemplateSource` enum L314-321 — `Project | Global | Embedded` — Indicates where a template was loaded from.
-  `defaults` module L12-43 — `-` — Embedded default templates for each document type
-  `TemplateError` type L58-67 — `= TemplateError` — 3.
-  `fmt` function L59-66 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — 3.
-  `TemplateError` type L69 — `= TemplateError` — 3.
-  `TemplateType` type L78-85 — `= TemplateType` — 3.
-  `filename` function L79-84 — `(&self) -> &'static str` — 3.
-  `TemplateLoader` type L100-310 — `= TemplateLoader` — 3.
-  `load_template` function L134-174 — `( &self, doc_type: &str, template_type: TemplateType, ) -> Result<String, Templa...` — Load a template with the fallback chain.
-  `get_embedded_template` function L177-197 — `( &self, doc_type: &str, template_type: TemplateType, ) -> Result<String, Templa...` — Get the embedded default template for a document type.
-  `TemplateSource` type L323-331 — `= TemplateSource` — 3.
-  `fmt` function L324-330 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — 3.
-  `doc_type_letter` function L334-343 — `(doc_type: &str) -> char` — Helper to get the type letter for short codes
-  `tests` module L346-471 — `-` — 3.
-  `test_load_embedded_templates` function L351-367 — `()` — 3.
-  `test_unknown_document_type` function L370-374 — `()` — 3.
-  `test_project_template_override` function L377-396 — `()` — 3.
-  `test_template_validation_error` function L399-414 — `()` — 3.
-  `test_template_validation_missing_variable` function L417-432 — `()` — 3.
-  `test_sample_context_generation` function L435-451 — `()` — 3.
-  `test_has_custom_template` function L454-470 — `()` — 3.

### crates/metis-docs-core/src/application/services/document

**Role**: Groups the four document-lifecycle services: creation, deletion, discovery, and validation — each responsible for a distinct operation phase.

**Key abstractions**:
- `DocumentCreationService` — The authoritative path for creating new documents. It generates a short code from `ConfigurationRepository`, renders frontmatter and body from the `TemplateLoader` (with fallback to embedded defaults), writes the file, and inserts the `Document` record plus a `DocumentRelationship` record in the database. Supports all document types including `create_initiative_with_config()` and `create_task_with_config()` which factor in `FlightLevelConfig` to determine which parent types are applicable.
- `DeletionService` — Removes a document file and, for types that have subdirectories (strategies, initiatives), recursively removes child documents and cleans up empty directories. Returns a `DeletionResult` listing every path removed.
- `DocumentDiscoveryService` — Resolves a short code or document ID to a file path. The short code path decodes the type letter from the code (e.g., `PROJ-V-0001` → Vision), constructs the expected path, and falls back to a database lookup if not found at the expected location. Initiative and task lookups always go through the database because their paths embed parent IDs.
- `DocumentValidationService` — Tries to parse a document file as each known type until one succeeds, then runs type-specific `validate()`. `detect_document_type()` uses this to return just the type without validating further.

**Mixed concerns / gotchas**: `DocumentCreationService::create_initiative_with_config()` and `create_task_with_config()` contain the `FlightLevelConfig` logic that determines which fields to populate (e.g., whether `strategy_id` is null because strategies are disabled). This means flight level business rules are partially encoded here rather than purely in the domain layer.

#### crates/metis-docs-core/src/application/services/document/creation.rs

- pub `DocumentCreationService` struct L14-18 — `{ workspace_dir: PathBuf, db_path: PathBuf, template_loader: TemplateLoader }` — Service for creating new documents with proper defaults and validation
- pub `DocumentCreationConfig` struct L22-29 — `{ title: String, description: Option<String>, parent_id: Option<DocumentId>, tag...` — Configuration for creating a new document
- pub `CreationResult` struct L33-38 — `{ document_id: DocumentId, document_type: DocumentType, file_path: PathBuf, shor...` — Result of document creation
- pub `new` function L42-51 — `(workspace_dir: P) -> Self` — Create a new document creation service for a workspace
- pub `create_vision` function L67-120 — `(&self, config: DocumentCreationConfig) -> Result<CreationResult>` — Create a new vision document
- pub `create_initiative` function L123-130 — `( &self, config: DocumentCreationConfig, ) -> Result<CreationResult>` — Create a new initiative document (legacy method)
- pub `create_initiative_with_config` function L133-218 — `( &self, config: DocumentCreationConfig, flight_config: &FlightLevelConfig, ) ->...` — Create a new initiative document with flight level configuration
- pub `create_task` function L221-233 — `( &self, config: DocumentCreationConfig, initiative_id: &str, ) -> Result<Creati...` — Create a new task document (legacy method)
- pub `create_task_with_config` function L236-375 — `( &self, config: DocumentCreationConfig, initiative_id: &str, flight_config: &Fl...` — Create a new task document with flight level configuration
- pub `create_backlog_item` function L378-441 — `( &self, config: DocumentCreationConfig, ) -> Result<CreationResult>` — Create a new backlog item (task without parent)
- pub `create_adr` function L464-521 — `(&self, config: DocumentCreationConfig) -> Result<CreationResult>` — Create a new ADR document
- pub `create_specification` function L524-583 — `( &self, config: DocumentCreationConfig, ) -> Result<CreationResult>` — Create a new specification document
-  `DocumentCreationService` type L40-610 — `= DocumentCreationService`
-  `generate_short_code` function L54-64 — `(&self, doc_type: &str) -> Result<String>` — Generate a short code for a document type
-  `determine_backlog_directory` function L444-461 — `(&self, tags: &[Tag]) -> PathBuf` — Determine the backlog directory based on tags
-  `get_next_adr_number` function L586-609 — `(&self) -> Result<u32>` — Get the next ADR number by examining existing ADRs
-  `tests` module L613-1044 — `-`
-  `test_create_vision_document` function L618-651 — `()`
-  `test_create_initiative_document` function L654-692 — `()`
-  `test_get_next_adr_number` function L696-713 — `()`
-  `setup_test_service_temp` function L717-734 — `() -> (DocumentCreationService, tempfile::TempDir)`
-  `test_create_initiative_full_configuration` function L737-764 — `()`
-  `test_create_initiative_streamlined_configuration` function L767-790 — `()`
-  `test_create_initiative_disabled_in_direct_configuration` function L793-816 — `()`
-  `test_create_task_direct_configuration` function L819-843 — `()`
-  `test_create_vision_with_custom_template` function L846-900 — `()`
-  `test_create_task_with_custom_template` function L903-960 — `()`
-  `test_create_document_falls_back_to_embedded_template` function L963-1001 — `()`
-  `test_invalid_custom_template_fails_gracefully` function L1004-1043 — `()`

#### crates/metis-docs-core/src/application/services/document/deletion.rs

- pub `DeletionService` struct L11 — `-` — Service for recursive document deletion
- pub `new` function L20-22 — `() -> Self`
- pub `delete_document_recursive` function L25-77 — `(&self, filepath: &str) -> Result<DeletionResult>` — Delete a document and all its children recursively
- pub `DeletionResult` struct L132-135 — `{ deleted_files: Vec<String>, cleaned_directories: Vec<String> }` — Result of a document deletion operation
-  `DeletionService` type L13-17 — `impl Default for DeletionService`
-  `default` function L14-16 — `() -> Self`
-  `DeletionService` type L19-128 — `= DeletionService`
-  `remove_directory_recursive` function L80-127 — `( dir_path: &Path, deleted_files: &mut Vec<String>, cleaned_directories: &mut Ve...` — Recursively remove a directory and all its contents
-  `tests` module L138-386 — `-`
-  `setup_test_workspace` function L151-185 — `() -> (tempfile::TempDir, PathBuf)`
-  `test_delete_single_document_no_children` function L188-205 — `()`
-  `test_delete_initiative_with_folder` function L208-286 — `()`
-  `test_delete_nonexistent_document` function L289-303 — `()`
-  `test_delete_task_file_only` function L306-364 — `()`
-  `test_delete_document_no_folder` function L367-385 — `()`

#### crates/metis-docs-core/src/application/services/document/discovery.rs

- pub `DocumentDiscoveryService` struct L11-13 — `{ workspace_dir: PathBuf }` — Service for discovering documents by ID across all document types
- pub `DocumentDiscoveryResult` struct L17-20 — `{ document_type: DocumentType, file_path: PathBuf }` — Result of document discovery
- pub `new` function L24-44 — `(workspace_dir: P) -> Self` — Create a new document discovery service for a workspace
- pub `find_document_by_short_code` function L47-67 — `( &self, short_code: &str, ) -> Result<DocumentDiscoveryResult>` — Find a document by its short code across all document types
- pub `find_document_by_id` function L70-90 — `(&self, document_id: &str) -> Result<DocumentDiscoveryResult>` — Find a document by its ID across all document types (legacy method)
- pub `find_document_of_type` function L93-265 — `( &self, document_id: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Find a document by its ID within a specific document type
- pub `find_document_by_id_and_type` function L268-274 — `( &self, document_id: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Find a document by its ID with a specific document type constraint
- pub `document_exists` function L277-279 — `(&self, document_id: &str) -> bool` — Check if a document with the given ID exists
- pub `find_all_documents_of_type` function L282-385 — `(&self, doc_type: DocumentType) -> Result<Vec<PathBuf>>` — Get all documents of a specific type
- pub `find_initiative_hierarchy_with_database` function L389-409 — `( &self, initiative_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<...` — Find all documents in an initiative hierarchy using database lineage queries
- pub `find_documents_by_initiative_with_database` function L412-432 — `( &self, initiative_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<...` — Find all documents belonging to an initiative using database lineage queries
- pub `find_document_by_id_with_database` function L436-458 — `( &self, document_id: &str, db_service: &mut DatabaseService, ) -> Result<Docume...` — Fast document lookup using database instead of filesystem scanning
-  `DocumentDiscoveryService` type L22-607 — `= DocumentDiscoveryService`
-  `document_type_from_short_code` function L461-485 — `(&self, short_code: &str) -> Result<DocumentType>` — Extract document type from short code format (e.g., PROJ-V-0001 -> Vision)
-  `construct_path_from_short_code` function L488-515 — `( &self, short_code: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Construct file path from short code and document type
-  `find_initiative_path_by_short_code` function L518-553 — `(&self, short_code: &str) -> Result<PathBuf>` — Find initiative path by short code using database lookup
-  `find_task_path_by_short_code` function L556-606 — `(&self, short_code: &str) -> Result<PathBuf>` — Find task path by short code using database lookup
-  `tests` module L610-706 — `-`
-  `test_find_vision_document` function L616-649 — `()`
-  `test_document_not_found` function L652-662 — `()`
-  `test_find_all_documents_of_type` function L665-705 — `()`

#### crates/metis-docs-core/src/application/services/document/mod.rs

- pub `creation` module L1 — `-`
- pub `deletion` module L2 — `-`
- pub `discovery` module L3 — `-`
- pub `validation` module L4 — `-`

#### crates/metis-docs-core/src/application/services/document/validation.rs

- pub `DocumentValidationService` struct L7 — `-` — Service for validating documents and detecting their types
- pub `ValidationResult` struct L11-15 — `{ document_type: DocumentType, is_valid: bool, errors: Vec<String> }` — Result of document validation
- pub `new` function L19-21 — `() -> Self` — Create a new document validation service
- pub `validate_document` function L24-134 — `( &self, file_path: P, ) -> Result<ValidationResult>` — Validate a document file and detect its type
- pub `detect_document_type` function L137-148 — `(&self, file_path: P) -> Result<DocumentType>` — Validate a document and return just the document type (simpler interface)
- pub `validate_document_as_type` function L151-180 — `( &self, file_path: P, expected_type: DocumentType, ) -> Result<bool>` — Validate a document of a specific expected type
- pub `is_valid_document` function L183-188 — `(&self, file_path: P) -> bool` — Check if a document is valid without loading the full document
-  `DocumentValidationService` type L17-189 — `= DocumentValidationService`
-  `DocumentValidationService` type L191-195 — `impl Default for DocumentValidationService`
-  `default` function L192-194 — `() -> Self`
-  `tests` module L198-337 — `-`
-  `test_validate_valid_vision_document` function L204-235 — `()`
-  `test_validate_invalid_document` function L238-254 — `()`
-  `test_detect_document_type` function L257-286 — `()`
-  `test_validate_document_as_type` function L289-327 — `()`
-  `test_validate_nonexistent_file` function L330-336 — `()`

### crates/metis-docs-core/src/application/services/workspace

**Role**: Manages the workspace as a first-class concept: detecting it on disk, initializing it, recovering it after database loss, transitioning document phases, reassigning task parents, and archiving documents.

**Key abstractions**:
- `WorkspaceDetectionService` — Walks up the directory tree from the current directory (or a given start path) to find a `.metis/` directory. `prepare_workspace()` additionally opens or recreates the SQLite database and runs migrations. `resolve_metis_dir()` handles the ambiguity of being invoked from either the project root or from inside `.metis/` itself.
- `WorkspaceInitializationService` — Creates the full workspace scaffold: `.metis/` directory, SQLite database, `config.toml` with the project prefix and flight level preset, and an initial Vision document. `initialize_workspace_with_prefix()` is the main entry point; it validates the prefix format and rejects re-initialization of existing workspaces.
- `ConfigurationRecoveryService` — Handles the "database as cache" design: if the database is missing or corrupt, `recover_configuration()` recreates it from `config.toml` and rescans all documents to rebuild counters. `sync_config_to_database()` is a lighter-weight daily sync that only propagates config changes. `needs_recovery()` checks for the database file's absence as the trigger.
- `PhaseTransitionService` — Reads the current phase from a document file, validates the requested transition against the document type's phase adjacency rules, updates the phase tag in the file, and writes it back. `transition_to_next_phase()` advances to the natural next phase without requiring the caller to know the sequence.
- `ArchiveService` — Archives a document (and its entire descendant hierarchy discovered via database lineage queries) by moving files into an `archived/` subdirectory while marking the document's frontmatter `archived: true`. Handles strategies (which have subdirectories with initiatives) and single-file documents differently via `archive_single_file()` vs `archive_directory()`.
- `ReassignmentService` — Moves a task file from one parent initiative directory to another, updating the task's `parent_id` in its frontmatter and in the database. `reassign_to_backlog()` moves the task to the appropriate `backlog/<category>/` directory instead.

**Internal flow**: All workspace services receive a `workspace_dir` path. They use `DocumentDiscoveryService` or `DatabaseService` for lookups, read/write document objects through the domain layer, and persist results via `FilesystemService` and `DatabaseService`.

**Mixed concerns / gotchas**: `ConfigurationRecoveryService` both creates config files and resets database counters — two distinct concerns bundled together because both arise from the same "starting fresh" scenario.

#### crates/metis-docs-core/src/application/services/workspace/archive.rs

- pub `ArchiveService` struct L12-15 — `{ workspace_dir: PathBuf, discovery_service: DocumentDiscoveryService }` — Service for archiving documents and managing the archived folder structure
- pub `ArchiveResult` struct L19-22 — `{ archived_documents: Vec<ArchivedDocument>, total_archived: usize }` — Result of archive operation
- pub `ArchivedDocument` struct L26-31 — `{ document_id: String, document_type: DocumentType, original_path: PathBuf, arch...` — Information about an archived document
- pub `new` function L95-118 — `(workspace_dir: P) -> Self` — Create a new archive service for a workspace
- pub `archive_document` function L121-181 — `( &self, document_id: &str, db_service: &mut DatabaseService, ) -> Result<Archiv...` — Archive a document and all its children using database lineage queries
- pub `is_document_archived` function L366-390 — `(&self, document_id: &str) -> Result<bool>` — Check if a document is already archived
- pub `get_archived_documents` function L393-403 — `(&self) -> Result<Vec<ArchivedDocument>>` — Get all archived documents
- pub `archive_document_by_short_code` function L465-529 — `( &self, short_code: &str, db_service: &mut DatabaseService, ) -> Result<Archive...` — Archive a document by its short code
- pub `is_document_archived_by_short_code` function L532-545 — `(&self, short_code: &str) -> Result<bool>` — Check if a document is archived by its short code
-  `ArchiveService` type L33-546 — `= ArchiveService`
-  `mark_as_archived_helper` function L37-92 — `( &self, file_path: &Path, doc_type: DocumentType, ) -> Result<()>` — Common helper for loading and marking a document as archived
-  `archive_single_file` function L184-224 — `( &self, file_path: &Path, doc_type: DocumentType, ) -> Result<ArchivedDocument>` — Archive a single file
-  `archive_directory` function L227-288 — `( &self, dir_path: &Path, doc_type: DocumentType, ) -> Result<ArchivedDocument>` — Archive a directory (for strategies and initiatives)
-  `merge_directory_contents` function L292-327 — `(&self, source_dir: &Path, target_dir: &Path) -> Result<()>` — Merge directory contents by moving files/subdirs from source to target
-  `get_document_id` function L330-363 — `(&self, file_path: &Path, doc_type: DocumentType) -> Result<String>` — Get document ID from a file
-  `scan_archived_directory` function L406-438 — `( &self, dir: &Path, results: &mut Vec<ArchivedDocument>, ) -> Result<()>` — Recursively scan archived directory for documents
-  `determine_document_type` function L441-462 — `(&self, file_path: &Path) -> Result<DocumentType>` — Determine document type from file path and content
-  `tests` module L549-720 — `-`
-  `test_archive_vision_document` function L558-609 — `()`
-  `test_get_archived_documents` function L612-659 — `()`
-  `test_is_document_archived` function L662-719 — `()`

#### crates/metis-docs-core/src/application/services/workspace/detection.rs

- pub `WorkspaceDetectionService` struct L8 — `-` — Service for detecting and validating Metis workspaces
- pub `new` function L11-13 — `() -> Self`
- pub `find_workspace` function L16-34 — `(&self) -> Result<Option<PathBuf>>` — Find the nearest Metis workspace by traversing up the directory tree
- pub `find_workspace_from` function L37-55 — `(&self, start_path: &Path) -> Result<Option<PathBuf>>` — Find workspace starting from a specific directory
- pub `validate_workspace` function L59-65 — `(&self, metis_dir: &Path) -> Result<Option<PathBuf>>` — Validate that a directory is a proper Metis workspace
- pub `is_in_workspace` function L68-70 — `(&self, path: &Path) -> Result<bool>` — Check if a path is within a Metis workspace
- pub `get_workspace_root` function L73-81 — `(&self, path: &Path) -> Result<Option<PathBuf>>` — Get the workspace root for a given path
- pub `resolve_metis_dir` function L88-107 — `(&self, path: &Path) -> PathBuf` — Resolve a path to the `.metis` directory.
- pub `prepare_workspace` function L116-155 — `(&self, metis_dir: &Path) -> Result<Database>` — Prepare a workspace for use by ensuring database exists and is synced
- pub `find_and_prepare_workspace` function L159-166 — `(&self) -> Result<Option<(PathBuf, Database)>>` — Find workspace from current directory and prepare it for use
-  `WorkspaceDetectionService` type L10-167 — `= WorkspaceDetectionService`
-  `WorkspaceDetectionService` type L169-173 — `impl Default for WorkspaceDetectionService`
-  `default` function L170-172 — `() -> Self`
-  `tests` module L176-287 — `-`
-  `test_validate_workspace_missing_directory` function L182-189 — `()`
-  `test_validate_workspace_with_metis_directory` function L192-202 — `()`
-  `test_find_workspace_traversal` function L205-218 — `()`
-  `test_resolve_metis_dir_already_metis` function L221-230 — `()`
-  `test_resolve_metis_dir_from_project_root` function L233-245 — `()`
-  `test_resolve_metis_dir_no_config_toml` function L248-258 — `()`
-  `test_resolve_metis_dir_no_metis_subdir` function L261-270 — `()`
-  `test_get_workspace_root` function L273-286 — `()`

#### crates/metis-docs-core/src/application/services/workspace/initialization.rs

- pub `WorkspaceInitializationService` struct L8 — `-` — Service for initializing new Metis workspaces
- pub `WorkspaceInitializationResult` struct L11-15 — `{ metis_dir: PathBuf, database_path: PathBuf, vision_path: PathBuf }` — Result of workspace initialization
- pub `initialize_workspace` function L20-25 — `( base_path: P, project_name: &str, ) -> Result<WorkspaceInitializationResult>` — Initialize a new Metis workspace at the given base path
- pub `initialize_workspace_with_prefix` function L28-120 — `( base_path: P, project_name: &str, prefix: Option<&str>, ) -> Result<WorkspaceI...` — Initialize a new Metis workspace with an optional custom prefix
- pub `is_workspace` function L154-157 — `(path: &Path) -> bool` — Check if a directory contains a valid Metis workspace
-  `WorkspaceInitializationService` type L17-158 — `= WorkspaceInitializationService`
-  `create_default_vision` function L123-150 — `(workspace_dir: &Path, title: &str) -> Result<PathBuf>` — Create a new Vision document with defaults and write to file
-  `tests` module L161-257 — `-`
-  `test_initialize_workspace` function L167-215 — `()`
-  `test_initialize_workspace_already_exists` function L218-242 — `()`
-  `test_is_workspace` function L245-256 — `()`

#### crates/metis-docs-core/src/application/services/workspace/migration.rs

- pub `MigrationReport` struct L10-17 — `{ migrated: bool, moved_items: Vec<MigrationAction>, deleted_items: Vec<PathBuf>...` — Result of a workspace migration
- pub `MigrationAction` struct L21-24 — `{ from: PathBuf, to: PathBuf }` — A single migration action (move or delete)
- pub `WorkspaceMigrationService` struct L27 — `-` — Service for migrating workspace filesystem layouts between versions.
- pub `migrate` function L34-148 — `(metis_dir: &Path) -> std::io::Result<MigrationReport>` — Run all pending migrations on a workspace.
-  `WorkspaceMigrationService` type L29-149 — `= WorkspaceMigrationService` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `tests` module L152-374 — `-` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_noop_on_v2_workspace` function L158-168 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_idempotent` function L171-183 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_null_strategy_pattern` function L186-214 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_named_strategy_pattern` function L217-243 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_multiple_strategies` function L246-292 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_empty_strategies_dir` function L295-308 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_preserves_existing_initiatives` function L311-350 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).
-  `test_migration_logs_actions` function L353-373 — `()` — Currently supports v1→v2 migration (removing the `strategies/` nesting layer).

#### crates/metis-docs-core/src/application/services/workspace/mod.rs

- pub `archive` module L1 — `-`
- pub `detection` module L2 — `-`
- pub `initialization` module L3 — `-`
- pub `migration` module L4 — `-`
- pub `reassignment` module L5 — `-`
- pub `recovery` module L6 — `-`
- pub `transition` module L7 — `-`

#### crates/metis-docs-core/src/application/services/workspace/reassignment.rs

- pub `ReassignmentService` struct L9-11 — `{ workspace_dir: PathBuf }` — Service for reassigning tasks to different parent initiatives or the backlog
- pub `ReassignmentResult` struct L15-20 — `{ short_code: String, old_path: PathBuf, new_path: PathBuf, new_parent: Option<S...` — Result of reassignment operation
- pub `BacklogCategory` enum L24-28 — `Bug | Feature | TechDebt` — Backlog category for standalone tasks
- pub `from_str` function L31-38 — `(s: &str) -> Option<Self>`
- pub `directory_name` function L40-46 — `(&self) -> &'static str`
- pub `new` function L51-64 — `(workspace_dir: P) -> Self` — Create a new reassignment service for a workspace
- pub `reassign_to_initiative` function L67-92 — `( &self, short_code: &str, new_parent_id: &str, db_service: &mut DatabaseService...` — Reassign a task to a new parent initiative
- pub `reassign_to_backlog` function L95-127 — `( &self, short_code: &str, category: BacklogCategory, db_service: &mut DatabaseS...` — Move a task to the backlog
-  `BacklogCategory` type L30-47 — `= BacklogCategory`
-  `ReassignmentService` type L49-262 — `= ReassignmentService`
-  `find_task_by_short_code` function L130-152 — `( &self, short_code: &str, db_service: &mut DatabaseService, ) -> Result<Documen...` — Find a task by short code and validate it's a task
-  `find_and_validate_parent` function L155-188 — `( &self, parent_id: &str, db_service: &mut DatabaseService, ) -> Result<Document...` — Find and validate a parent initiative
-  `compute_initiative_task_path` function L191-214 — `( &self, parent_doc: &Document, source_doc: &Document, ) -> Result<PathBuf>` — Compute the destination path for a task under an initiative
-  `move_file` function L217-261 — `(&self, source: &Path, dest: &Path) -> Result<()>` — Move a file from source to destination
-  `tests` module L265-284 — `-`
-  `test_backlog_category_parsing` function L269-276 — `()`
-  `test_backlog_category_directory` function L279-283 — `()`

#### crates/metis-docs-core/src/application/services/workspace/recovery.rs

- pub `ConfigurationRecoveryService` struct L9 — `-` — Service for recovering workspace configuration from filesystem
- pub `recover_configuration` function L29-93 — `( workspace_dir: P, db_path: P, ) -> Result<RecoveryReport>` — Recover configuration from config.toml file to database
- pub `sync_config_to_database` function L98-140 — `( workspace_dir: P, db_path: P, ) -> Result<bool>` — Sync config.toml to database (lightweight operation, safe to call frequently)
- pub `needs_recovery` function L201-218 — `(workspace_dir: &Path) -> bool` — Check if database needs recovery
- pub `RecoveryReport` struct L223-232 — `{ config_file_created: bool, prefix_synced: bool, flight_levels_synced: bool, co...` — Report of what was recovered during configuration recovery
- pub `had_recovery_actions` function L240-245 — `(&self) -> bool` — Check if any recovery actions were taken
-  `ConfigurationRecoveryService` type L11-219 — `= ConfigurationRecoveryService`
-  `create_config_from_database` function L143-166 — `( config_file_path: &Path, db_path: &Path, ) -> Result<ConfigFile>` — Create config.toml from existing database (migration path)
-  `recover_counters` function L169-194 — `( workspace_dir: &Path, config_repo: &mut ConfigurationRepository, ) -> Result<u...` — Recover counters from filesystem by scanning all documents
-  `RecoveryReport` type L234-246 — `= RecoveryReport`
-  `new` function L235-237 — `() -> Self`

#### crates/metis-docs-core/src/application/services/workspace/transition.rs

- pub `PhaseTransitionService` struct L9-11 — `{ discovery_service: DocumentDiscoveryService }` — Service for managing document phase transitions
- pub `TransitionResult` struct L15-21 — `{ document_id: String, document_type: DocumentType, from_phase: Phase, to_phase:...` — Result of a phase transition
- pub `new` function L25-30 — `(workspace_dir: P) -> Self` — Create a new phase transition service for a workspace
- pub `transition_document` function L33-67 — `( &self, short_code: &str, target_phase: Phase, ) -> Result<TransitionResult>` — Transition a document to a specific phase
- pub `transition_to_next_phase` function L70-100 — `(&self, short_code: &str) -> Result<TransitionResult>` — Transition a document to the next phase in its natural sequence
- pub `is_valid_transition` function L266-274 — `( &self, doc_type: DocumentType, from_phase: Phase, to_phase: Phase, ) -> bool` — Check if a phase transition is valid without performing it
- pub `get_valid_transitions_for` function L277-283 — `( &self, doc_type: DocumentType, from_phase: Phase, ) -> Vec<Phase>` — Get all valid transitions for a document type and phase
-  `PhaseTransitionService` type L23-284 — `= PhaseTransitionService`
-  `get_current_phase` function L103-136 — `(&self, file_path: &Path, doc_type: DocumentType) -> Result<Phase>` — Get the current phase of a document
-  `perform_transition` function L139-225 — `( &self, file_path: &Path, doc_type: DocumentType, target_phase: Phase, ) -> Res...` — Perform the actual phase transition
-  `validate_transition` function L228-245 — `( &self, doc_type: DocumentType, from_phase: Phase, to_phase: Phase, ) -> Result...` — Validate that a phase transition is allowed
-  `get_valid_transitions` function L249-251 — `(&self, doc_type: DocumentType, from_phase: Phase) -> Vec<Phase>` — Get valid transitions from a given phase for a document type.
-  `get_next_phase` function L255-263 — `(&self, doc_type: DocumentType, current_phase: Phase) -> Result<Phase>` — Get the next phase in the natural sequence for a document type.
-  `tests` module L287-456 — `-`
-  `setup_test_workspace` function L297-312 — `() -> (tempfile::TempDir, PathBuf)`
-  `test_transition_vision_to_next_phase` function L315-340 — `()`
-  `test_transition_to_specific_phase` function L343-367 — `()`
-  `test_invalid_transition` function L370-396 — `()`
-  `test_get_valid_transitions` function L399-426 — `()`
-  `test_is_valid_transition` function L429-455 — `()`

### crates/metis-docs-core/src

**Role**: The core library crate for the Metis documentation system — re-exports the public API surface and declares all top-level modules.

**Key abstractions**:
- `constants.rs` — Single source of truth for all string literals used as directory names (`vision/`, `strategies/`, etc.), file extensions, phase names, complexity levels, risk levels, and database settings. Consuming code uses these constants rather than inline strings to avoid typos and centralize renaming.
- `error.rs` / `MetisError` — The unified error enum for the crate. Variants cover database errors (via Diesel), I/O, JSON/YAML parsing, document-not-found (carrying the document ID), invalid document type/phase, filesystem errors, and configuration errors. All service `Result` types alias `std::result::Result<T, MetisError>`.
- `lib.rs` re-exports — `Application`, `Database`, all five document types (`Vision`, `Strategy`, `Initiative`, `Task`, `Adr`), and the core trait and type names (`Document`, `DocumentId`, `DocumentType`, `Phase`, `Tag`, `MetisError`) are re-exported at the crate root for ergonomic use by CLI, GUI, and MCP crates.

**Dependencies**: `diesel` + `diesel_migrations` for SQLite; `gray_matter` for YAML frontmatter parsing; `tera` for template rendering; `chrono` for timestamps; `serde` for serialization.

#### crates/metis-docs-core/src/constants.rs

- pub `METIS_DIR_NAME` variable L4 — `: &str` — Directory and file names
- pub `DATABASE_FILE_NAME` variable L5 — `: &str`
- pub `BACKUP_DATABASE_FILE_NAME` variable L6 — `: &str`
- pub `LOG_FILE_NAME` variable L7 — `: &str`
- pub `MARKDOWN_EXT` variable L10 — `: &str` — File extensions
- pub `YAML_EXT` variable L11 — `: &str`
- pub `JSON_EXT` variable L12 — `: &str`
- pub `VISION_DIR` variable L15 — `: &str` — Document directories
- pub `INITIATIVE_DIR` variable L16 — `: &str`
- pub `TASK_DIR` variable L17 — `: &str`
- pub `ADR_DIR` variable L18 — `: &str`
- pub `SPECIFICATION_DIR` variable L19 — `: &str`
- pub `ARCHIVED_DIR` variable L20 — `: &str`
- pub `VISION_TEMPLATE` variable L23 — `: &str` — Template names
- pub `INITIATIVE_TEMPLATE` variable L24 — `: &str`
- pub `TASK_TEMPLATE` variable L25 — `: &str`
- pub `ADR_TEMPLATE` variable L26 — `: &str`
- pub `SPECIFICATION_TEMPLATE` variable L27 — `: &str`
- pub `phases` module L30-56 — `-` — Document phases
- pub `VISION_DRAFT` variable L31 — `: &str`
- pub `VISION_REVIEW` variable L32 — `: &str`
- pub `VISION_PUBLISHED` variable L33 — `: &str`
- pub `INITIATIVE_DISCOVERY` variable L35 — `: &str`
- pub `INITIATIVE_DESIGN` variable L36 — `: &str`
- pub `INITIATIVE_READY` variable L37 — `: &str`
- pub `INITIATIVE_DECOMPOSE` variable L38 — `: &str`
- pub `INITIATIVE_ACTIVE` variable L39 — `: &str`
- pub `INITIATIVE_COMPLETED` variable L40 — `: &str`
- pub `TASK_TODO` variable L42 — `: &str`
- pub `TASK_ACTIVE` variable L43 — `: &str`
- pub `TASK_BLOCKED` variable L44 — `: &str`
- pub `TASK_COMPLETED` variable L45 — `: &str`
- pub `ADR_DRAFT` variable L47 — `: &str`
- pub `ADR_DISCUSSION` variable L48 — `: &str`
- pub `ADR_DECIDED` variable L49 — `: &str`
- pub `ADR_SUPERSEDED` variable L50 — `: &str`
- pub `SPECIFICATION_DISCOVERY` variable L52 — `: &str`
- pub `SPECIFICATION_DRAFTING` variable L53 — `: &str`
- pub `SPECIFICATION_REVIEW` variable L54 — `: &str`
- pub `SPECIFICATION_PUBLISHED` variable L55 — `: &str`
- pub `complexity` module L59-65 — `-` — Complexity levels for initiatives
- pub `XS` variable L60 — `: &str`
- pub `S` variable L61 — `: &str`
- pub `M` variable L62 — `: &str`
- pub `L` variable L63 — `: &str`
- pub `XL` variable L64 — `: &str`
- pub `database` module L68-71 — `-` — Database settings
- pub `CONNECTION_TIMEOUT_SECS` variable L69 — `: u64`
- pub `MAX_RETRIES` variable L70 — `: u32`
- pub `filesystem` module L74-77 — `-` — File system settings
- pub `MAX_FILE_SIZE_BYTES` variable L75 — `: u64`
- pub `BACKUP_RETENTION_DAYS` variable L76 — `: u32`

#### crates/metis-docs-core/src/error.rs

- pub `conversions` module L3 — `-` — Error types for Metis operations
- pub `Result` type L8 — `= std::result::Result<T, MetisError>` — Error types for Metis operations
- pub `MetisError` enum L11-69 — `Database | Connection | Io | Json | Yaml | DocumentNotFound | InvalidDocumentTyp...` — Error types for Metis operations

#### crates/metis-docs-core/src/lib.rs

- pub `application` module L7 — `-` — Metis implements the Flight Levels methodology for hierarchical documentation
- pub `constants` module L8 — `-` — documents through their defined phases.
- pub `dal` module L9 — `-` — documents through their defined phases.
- pub `domain` module L10 — `-` — documents through their defined phases.
- pub `error` module L11 — `-` — documents through their defined phases.
- pub `tests` module L29-31 — `-` — documents through their defined phases.
- pub `common` module L30 — `-` — documents through their defined phases.

#### crates/metis-docs-core/src/main.rs

-  `main` function L2-4 — `()`

### crates/metis-docs-core/src/dal/database

**Role**: The data access layer for SQLite — provides the connection factory, schema migrations, ORM models, and two repositories (documents and configuration).

**Key abstractions**:
- `Database` — Manages a SQLite connection string. On `new()` it establishes a connection, sets WAL mode and a 5-second busy timeout, and runs any pending diesel migrations. Each call to `repository()` or `configuration_repository()` opens a fresh connection, making it safe to share `Database` across short-lived service instances. In-memory databases (`:memory:`) re-run migrations on each connection since each connection is isolated.
- `DocumentRepository` — All CRUD for `Document` and `DocumentRelationship` records. Exposes flat lookups (`find_by_filepath`, `find_by_id`, `find_by_short_code`) and hierarchy queries (`find_strategy_hierarchy`, `find_initiative_hierarchy`) that use the `DocumentRelationship` join table. Also drives FTS search via `search_documents()` / `search_documents_unarchived()`.
- `ConfigurationRepository` — Stores arbitrary key-value pairs in a `configuration` table, with specialized accessors for the project prefix (`get_project_prefix` / `set_project_prefix`) and `FlightLevelConfig`. The short code counter logic (`get_next_short_code_number`, `generate_short_code`) increments per-type counters atomically and formats the `PREFIX-TYPE-NNNN` string. An in-process `HashMap` cache avoids redundant DB reads within a single service call.
- `models.rs` — Diesel `Queryable`/`Insertable` structs that map directly to database tables: `Document`, `NewDocument`, `DocumentRelationship`, `DocumentTag`, `DocumentSearch` (FTS virtual table), `Configuration`.
- `schema.rs` — Diesel-generated table macros; the ground truth for column names used in queries.

**Mixed concerns / gotchas**: `generate_short_code()` in `DocumentRepository` takes a `db_path: &str` argument because it needs to open a second connection to `ConfigurationRepository` — there is no shared connection between the two repositories. This means short code generation requires a path to the database file on disk, so in-memory databases must pass a real path or use the service-layer wrapper instead.

#### crates/metis-docs-core/src/dal/database/configuration_repository.rs

- pub `ConfigurationRepository` struct L10-13 — `{ connection: SqliteConnection, cache: Option<HashMap<String, String>> }` — Repository for managing configuration data
- pub `new` function L16-21 — `(connection: SqliteConnection) -> Self`
- pub `load_cache` function L24-35 — `(&mut self) -> Result<()>` — Load all configuration into cache
- pub `get` function L38-45 — `(&mut self, key: &str) -> Result<Option<String>>` — Get configuration value by key
- pub `set` function L48-73 — `(&mut self, key: &str, value: &str) -> Result<()>` — Set configuration value
- pub `get_flight_level_config` function L76-86 — `(&mut self) -> Result<FlightLevelConfig>` — Get flight level configuration
- pub `set_flight_level_config` function L89-97 — `(&mut self, config: &FlightLevelConfig) -> Result<()>` — Set flight level configuration
- pub `get_all` function L100-105 — `(&mut self) -> Result<HashMap<String, String>>` — Get all configuration as a map
- pub `delete` function L108-119 — `(&mut self, key: &str) -> Result<bool>` — Delete configuration by key
- pub `get_project_prefix` function L122-124 — `(&mut self) -> Result<Option<String>>` — Get project prefix for short codes
- pub `set_project_prefix` function L127-138 — `(&mut self, prefix: &str) -> Result<()>` — Set project prefix for short codes (validates 2-8 uppercase letters)
- pub `get_next_short_code_number` function L141-153 — `(&mut self, doc_type: &str) -> Result<u32>` — Get next short code number for a document type and increment the counter
- pub `generate_short_code` function L156-181 — `(&mut self, doc_type: &str) -> Result<String>` — Generate a short code for a document type (PREFIX-TYPE-NNNN)
- pub `get_counter` function L184-193 — `(&mut self, doc_type: &str) -> Result<u32>` — Get current counter value for a document type without incrementing
- pub `set_counter` function L196-199 — `(&mut self, doc_type: &str, value: u32) -> Result<()>` — Set counter value for a document type
- pub `set_counter_if_lower` function L204-218 — `(&mut self, doc_type: &str, min_value: u32) -> Result<bool>` — Set counter value only if the new value is higher than current value
- pub `clear_all` function L222-229 — `(&mut self) -> Result<()>` — Clear all configuration (for testing)
-  `ConfigurationRepository` type L15-230 — `= ConfigurationRepository`
-  `tests` module L233-321 — `-`
-  `setup_test_repo` function L237-241 — `() -> ConfigurationRepository`
-  `test_basic_configuration_crud` function L244-266 — `()`
-  `test_flight_level_config` function L269-291 — `()`
-  `test_cache_functionality` function L294-309 — `()`
-  `test_nonexistent_key` function L312-320 — `()`

#### crates/metis-docs-core/src/dal/database/mod.rs

- pub `configuration_repository` module L1 — `-`
- pub `models` module L2 — `-`
- pub `repository` module L3 — `-`
- pub `schema` module L4 — `-`
- pub `MIGRATIONS` variable L10 — `: EmbeddedMigrations`
- pub `Database` struct L13-15 — `{ connection_string: String }` — Database connection and migration management
- pub `new` function L40-49 — `(connection_string: &str) -> Result<Self, Box<dyn std::error::Error + Send + Syn...` — Create a new database connection and run migrations
- pub `get_connection` function L52-65 — `( &self, ) -> Result<SqliteConnection, Box<dyn std::error::Error + Send + Sync>>` — Get a new connection to the database
- pub `repository` function L68-73 — `( &self, ) -> Result<repository::DocumentRepository, Box<dyn std::error::Error +...` — Get a document repository with a new connection
- pub `into_repository` function L76-79 — `(self) -> repository::DocumentRepository` — Get a document repository (consumes the database) - kept for compatibility
- pub `configuration_repository` function L82-92 — `( &self, ) -> Result< configuration_repository::ConfigurationRepository, Box<dyn...` — Get a configuration repository with a new connection
-  `configure_connection` function L21-33 — `(connection: &mut SqliteConnection) -> Result<(), diesel::result::Error>` — Configure SQLite connection for better concurrency
-  `Database` type L35-93 — `= Database`

#### crates/metis-docs-core/src/dal/database/models.rs

- pub `Document` struct L17-33 — `{ filepath: String, id: String, title: String, document_type: String, created_at...`
- pub `DocumentRelationship` struct L38-43 — `{ child_id: String, parent_id: String, child_filepath: String, parent_filepath: ...`
- pub `DocumentTag` struct L48-51 — `{ document_filepath: String, tag: String }`
- pub `NewDocument` struct L56-72 — `{ filepath: String, id: String, title: String, document_type: String, created_at...`
- pub `DocumentSearch` struct L78-84 — `{ rowid: i32, document_filepath: String, content: Option<String>, title: Option<...`
- pub `NewDocumentSearch` struct L88-93 — `{ document_filepath: String, content: Option<String>, title: Option<String>, doc...`
- pub `Configuration` struct L98-102 — `{ key: String, value: String, updated_at: f64 }`

#### crates/metis-docs-core/src/dal/database/repository.rs

- pub `DocumentRepository` struct L9-11 — `{ connection: SqliteConnection }` — Data access repository for document operations
- pub `new` function L14-16 — `(connection: SqliteConnection) -> Self`
- pub `create_document` function L19-27 — `(&mut self, doc: NewDocument) -> Result<Document>` — Insert a new document into the database
- pub `find_by_filepath` function L30-38 — `(&mut self, file_path: &str) -> Result<Option<Document>>` — Find a document by its filepath
- pub `find_by_id` function L41-49 — `(&mut self, document_id: &str) -> Result<Option<Document>>` — Find a document by its ID
- pub `update_document` function L52-60 — `(&mut self, file_path: &str, doc: &Document) -> Result<Document>` — Update an existing document
- pub `delete_document` function L63-71 — `(&mut self, file_path: &str) -> Result<bool>` — Delete a document and all its relationships
- pub `find_children` function L74-86 — `(&mut self, parent_document_id: &str) -> Result<Vec<Document>>` — Find all children of a document
- pub `find_parent` function L89-102 — `(&mut self, child_document_id: &str) -> Result<Option<Document>>` — Find the parent of a document
- pub `create_relationship` function L105-114 — `(&mut self, relationship: DocumentRelationship) -> Result<()>` — Create a parent-child relationship
- pub `search_documents` function L117-129 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search documents using FTS
- pub `search_documents_unarchived` function L132-144 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search non-archived documents using FTS
- pub `find_by_type` function L147-155 — `(&mut self, doc_type: &str) -> Result<Vec<Document>>` — Get all documents of a specific type
- pub `find_by_type_unarchived` function L158-167 — `(&mut self, doc_type: &str) -> Result<Vec<Document>>` — Get all non-archived documents of a specific type
- pub `find_by_tag` function L170-180 — `(&mut self, tag_name: &str) -> Result<Vec<Document>>` — Get documents with specific tags
- pub `find_by_phase` function L183-191 — `(&mut self, phase_name: &str) -> Result<Vec<Document>>` — Get documents in a specific phase
- pub `find_by_type_and_phase` function L194-207 — `( &mut self, doc_type: &str, phase_name: &str, ) -> Result<Vec<Document>>` — Get documents by type and phase
- pub `find_by_initiative_id` function L210-218 — `(&mut self, initiative_document_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to an initiative
- pub `get_tags_for_document` function L221-229 — `(&mut self, doc_filepath: &str) -> Result<Vec<String>>` — Get all tags for a specific document by filepath
- pub `find_initiative_hierarchy` function L232-246 — `( &mut self, initiative_document_id: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy (initiative + its tasks)
- pub `find_initiative_hierarchy_by_short_code` function L249-264 — `( &mut self, initiative_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy by short code (initiative + its tasks)
- pub `generate_short_code` function L267-276 — `(&mut self, doc_type: &str, db_path: &str) -> Result<String>` — Generate a short code for a document type using the database configuration
- pub `find_by_short_code` function L279-287 — `(&mut self, code: &str) -> Result<Option<Document>>` — Find a document by its short code
- pub `resolve_short_code_to_document_id` function L290-298 — `(&mut self, code: &str) -> Result<String>` — Resolve short code to document ID for parent relationships
- pub `resolve_short_code_to_filepath` function L301-309 — `(&mut self, code: &str) -> Result<String>` — Resolve short code to file path for file operations
-  `DocumentRepository` type L13-310 — `= DocumentRepository`
-  `tests` module L313-566 — `-`
-  `setup_test_repository` function L317-320 — `() -> DocumentRepository`
-  `create_test_document` function L322-340 — `() -> NewDocument`
-  `test_create_and_find_document` function L343-368 — `()`
-  `test_update_document` function L371-389 — `()`
-  `test_delete_document` function L392-416 — `()`
-  `test_document_relationships` function L419-487 — `()`
-  `test_find_by_type` function L490-550 — `()`
-  `test_document_not_found` function L553-565 — `()`

### crates/metis-docs-core/src/dal

**Role**: The data access layer module root — currently exposes `database` (SQLite via Diesel) and a `filesystem` submodule placeholder; all substantive logic lives in `database/`.

**Dependencies**: `diesel` for SQLite ORM; `diesel_migrations` for embedded schema migrations.

#### crates/metis-docs-core/src/dal/mod.rs

- pub `database` module L1 — `-`
- pub `filesystem` module L2 — `-`

### crates/metis-docs-core/src/domain

**Role**: The pure business logic layer — document types, their phase transition rules, identity, content model, configuration, and the trait contract shared by all document types.

**Key abstractions**:
- `FlightLevelConfig` — Encodes which document hierarchy levels are active (`strategies_enabled`, `initiatives_enabled`). The three named presets (`full`, `streamlined`, `direct`) map to Vision→Strategy→Initiative→Task, Vision→Initiative→Task, and Vision→Task respectively. `get_parent_type()` returns the immediate parent type for a given document type under the current config. `ConfigFile` wraps `FlightLevelConfig` + `ProjectConfig` (prefix) for TOML serialization to `.metis/config.toml`.
- `Document` trait — The interface all five document types implement. Provides default implementations for `id()` (slug from title), `tags()`, `phase()` (finds the first `Tag::Phase` variant), and `update_section()` (replaces an H2 section in the markdown body). Each type must implement `can_transition_to()`, `transition_phase()`, `validate()`, and template accessors.
- `DocumentCore` — The common data bundle (title, metadata, content, parent_id, blocked_by, tags, archived) stored inside every document type. Types embed this via a `core: DocumentCore` field.
- `DocumentType` / `Phase` — Enums for the five document types and the full set of phases across all types. `DocumentType::valid_transitions_from()` encodes the per-type phase adjacency graph; `next_phase()` returns the canonical next step in the sequence.
- `DocumentId` — A newtype wrapping a slug derived from the document title (max 35 chars). `from_title()` lowercases and hyphenates the title. ADRs use `new()` directly with a custom `NUMBER-slug` format.
- `Tag` — Either a `Tag::Phase(Phase)` (the document's lifecycle status) or `Tag::Label(String)` (arbitrary user tag). Phase is stored as a tag string in the frontmatter, so `from_str()` tries to parse known phase values first.
- `ParentReference` — A three-state enum (`Some(DocumentId)`, `None`, `Null`) used when a document type is disabled by `FlightLevelConfig`; `Null` means "this level doesn't exist in this config", distinct from "no parent assigned".

**Internal flow**: Documents are constructed either with `new()` (renders a fresh template) or `from_content()` (parses existing markdown frontmatter via `gray_matter`). `FrontmatterParser` in `helpers.rs` provides typed extraction helpers (`extract_string`, `extract_tags`, etc.) used by every `from_content()` implementation. Serialization back to markdown is via `to_content()`, which renders a Tera template with the document's data.

#### crates/metis-docs-core/src/domain/configuration.rs

- pub `FlightLevelConfig` struct L7-10 — `{ initiatives_enabled: bool }` — Flight level configuration defining which levels are enabled
- pub `new` function L14-18 — `(initiatives_enabled: bool) -> Result<Self, ConfigurationError>` — Create a new configuration with the given initiative setting
- pub `streamlined` function L21-25 — `() -> Self` — Streamlined flight levels: Vision → Initiative → Task
- pub `direct` function L28-32 — `() -> Self` — Direct flight levels: Vision → Task
- pub `full` function L36-38 — `() -> Self` — Alias for streamlined() — kept for backward compatibility
- pub `is_document_type_allowed` function L41-47 — `(&self, doc_type: DocumentType) -> bool` — Check if a document type is allowed in this configuration
- pub `get_parent_type` function L50-63 — `(&self, doc_type: DocumentType) -> Option<DocumentType>` — Get the parent document type for a given document type in this configuration
- pub `preset_name` function L66-72 — `(&self) -> &'static str` — Get the configuration name/preset
- pub `enabled_document_types` function L75-87 — `(&self) -> Vec<DocumentType>` — Get enabled document types in hierarchical order
- pub `hierarchy_display` function L90-100 — `(&self) -> String` — Get the hierarchy display string
- pub `ConfigurationError` enum L117-122 — `InvalidConfiguration | SerializationError | InvalidValue | MissingConfiguration` — Configuration validation errors
- pub `ConfigFile` struct L145-148 — `{ project: ProjectConfig, flight_levels: FlightLevelConfig }` — Configuration file structure that persists to .metis/config.toml
- pub `ProjectConfig` struct L151-153 — `{ prefix: String }`
- pub `new` function L157-169 — `(prefix: String, flight_levels: FlightLevelConfig) -> Result<Self, Configuration...` — Create a new configuration with defaults
- pub `load` function L172-180 — `(path: P) -> Result<Self, ConfigurationError>` — Load configuration from a TOML file
- pub `save` function L183-193 — `(&self, path: P) -> Result<(), ConfigurationError>` — Save configuration to a TOML file
- pub `default_with_prefix` function L196-198 — `(prefix: String) -> Result<Self, ConfigurationError>` — Create default configuration with given prefix
- pub `prefix` function L201-203 — `(&self) -> &str` — Get the project prefix
- pub `flight_levels` function L206-208 — `(&self) -> &FlightLevelConfig` — Get the flight level configuration
-  `FlightLevelConfig` type L12-101 — `= FlightLevelConfig`
-  `FlightLevelConfig` type L103-107 — `impl Default for FlightLevelConfig`
-  `default` function L104-106 — `() -> Self`
-  `FlightLevelConfig` type L109-113 — `= FlightLevelConfig`
-  `fmt` function L110-112 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `ConfigurationError` type L124-139 — `= ConfigurationError`
-  `fmt` function L125-138 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `ConfigurationError` type L141 — `= ConfigurationError`
-  `ConfigFile` type L155-209 — `= ConfigFile`
-  `ConfigFile` type L211-220 — `impl Default for ConfigFile`
-  `default` function L212-219 — `() -> Self`
-  `tests` module L223-443 — `-`
-  `test_preset_configurations` function L227-235 — `()`
-  `test_configuration_validation` function L238-241 — `()`
-  `test_document_type_allowed` function L244-256 — `()`
-  `test_parent_type_resolution` function L259-277 — `()`
-  `test_enabled_document_types` function L280-305 — `()`
-  `test_hierarchy_display` function L308-317 — `()`
-  `test_serialization` function L320-325 — `()`
-  `test_backward_compat_old_config_with_strategies_enabled` function L328-338 — `()`
-  `test_backward_compat_old_toml_with_strategies_enabled` function L341-359 — `()`
-  `test_config_file_creation` function L362-366 — `()`
-  `test_config_file_validation` function L369-381 — `()`
-  `test_config_file_save_and_load` function L384-403 — `()`
-  `test_config_file_toml_format` function L406-428 — `()`
-  `test_config_file_default` function L431-435 — `()`
-  `test_config_file_default_with_prefix` function L438-442 — `()`

#### crates/metis-docs-core/src/domain/mod.rs

- pub `configuration` module L1 — `-`
- pub `documents` module L3 — `-` — Domain layer containing pure business logic and models

### crates/metis-docs-core/src/domain/documents/adr

**Role**: Implements the ADR (Architecture Decision Record) document type with its custom identity format, three-phase lifecycle, and decision metadata fields.

**Key abstractions**:
- `Adr` — Holds a `DocumentCore` plus `number: u32`, `decision_maker: String`, and `decision_date: Option<DateTime<Utc>>`. The `id()` override returns `NUMBER-slug` (e.g., `0001-use-sqlite`) rather than a plain slug, making ADR IDs stable and ordered.
- Phase sequence: Draft → Discussion → Decided (→ Superseded as a terminal escape hatch). ADRs cannot be blocked by other documents; the `blocked_by` field is always empty and `validate()` enforces this.
- `from_content()` — Parses frontmatter for the `level` field (ADR number), `decision_maker`, and `decision_date`. `to_content()` renders back to markdown using the ADR Tera template.

**Mixed concerns / gotchas**: The ADR number is stored in the frontmatter as `level` (a field shared with other document types for different purposes — initiatives use it for complexity). This creates a cross-type overloading of the `level` key in frontmatter.

#### crates/metis-docs-core/src/domain/documents/adr/mod.rs

- pub `Adr` struct L13-18 — `{ core: super::traits::DocumentCore, number: u32, decision_maker: String, decisi...` — An ADR (Architecture Decision Record) documents architectural decisions
- pub `new` function L23-46 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create a new ADR document with content rendered from template
- pub `new_with_template` function L50-96 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create a new ADR document with a custom template
- pub `from_parts` function L100-126 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create an ADR document from existing data (used when loading from file)
- pub `number` function L128-130 — `(&self) -> u32`
- pub `decision_maker` function L132-134 — `(&self) -> &str`
- pub `decision_date` function L136-138 — `(&self) -> Option<chrono::DateTime<Utc>>`
- pub `from_file` function L141-147 — `(path: P) -> Result<Self, DocumentValidationError>` — Create an ADR document by reading and parsing a file
- pub `from_content` function L150-221 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create an ADR document from raw file content string
- pub `to_file` function L246-251 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the ADR document to a file
- pub `to_content` function L254-322 — `(&self) -> Result<String, DocumentValidationError>` — Convert the ADR document to its markdown string representation using templates
-  `Adr` type L20-323 — `= Adr`
-  `next_phase_in_sequence` function L224-233 — `(current: Phase) -> Option<Phase>` — Get the next phase in the ADR sequence
-  `update_phase_tag` function L236-243 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Adr` type L325-453 — `impl Document for Adr`
-  `id` function L327-330 — `(&self) -> DocumentId` — ADRs have special ID format: number-slug
-  `document_type` function L332-334 — `(&self) -> DocumentType`
-  `title` function L336-338 — `(&self) -> &str`
-  `metadata` function L340-342 — `(&self) -> &DocumentMetadata`
-  `content` function L344-346 — `(&self) -> &DocumentContent`
-  `core` function L348-350 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L352-359 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L361-363 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L365-367 — `(&self) -> &[DocumentId]`
-  `validate` function L369-390 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L392-397 — `(&self) -> bool`
-  `template` function L399-406 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L408-410 — `(&self) -> &'static str`
-  `content_template` function L412-414 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L416-418 — `(&self) -> &'static str`
-  `transition_phase` function L420-448 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L450-452 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L456-814 — `-`
-  `test_adr_from_content` function L461-507 — `()`
-  `test_adr_special_id_format` function L510-525 — `()`
-  `test_adr_invalid_level` function L528-556 — `()`
-  `test_adr_validation` function L559-588 — `()`
-  `test_adr_cannot_be_blocked` function L591-606 — `()`
-  `test_adr_phase_transitions` function L609-640 — `()`
-  `test_adr_number_formatting` function L643-683 — `()`
-  `test_adr_transition_phase_auto` function L686-718 — `()`
-  `test_adr_transition_phase_explicit` function L721-743 — `()`
-  `test_adr_transition_phase_invalid` function L746-772 — `()`
-  `test_adr_update_section` function L775-813 — `()`

### crates/metis-docs-core/src/domain/documents

**Role**: The document model sub-namespace — contains the shared infrastructure (`content`, `metadata`, `traits`, `types`, `helpers`, `factory`) and the five concrete document type modules.

**Key abstractions**:
- `DocumentContent` — Separates the markdown body from the acceptance criteria section (everything after a `## Exit Criteria` / `## Acceptance Criteria` heading). `from_markdown()` splits on that boundary; `full_content()` reunites them for writing.
- `DocumentMetadata` — Timestamps (`created_at`, `updated_at`), the short code, and an `exit_criteria_met` flag. `update()` bumps `updated_at` to now; `mark_exit_criteria_met()` also sets the flag.
- `DocumentFactory` — Determines document type from frontmatter `type:` field and constructs the correct concrete type. Used when the caller doesn't know the type ahead of time (e.g., in `DocumentValidationService` and `SyncService`).
- `FrontmatterParser` — Typed extraction helpers that operate on `HashMap<String, gray_matter::Pod>`. Handles missing keys, wrong types, and malformed dates with consistent `DocumentValidationError` results.

**Internal flow**: Concrete document types (`Vision`, `Strategy`, etc.) in their submodules all follow the same pattern: `from_content()` calls `gray_matter` to parse frontmatter, `FrontmatterParser` to extract fields, then assembles `DocumentCore`. `to_content()` calls `tera::Tera::render()` with a context containing all document fields.

#### crates/metis-docs-core/src/domain/documents/content.rs

- pub `DocumentContent` struct L5-10 — `{ body: String, acceptance_criteria: Option<String> }` — Document content containing the main body and acceptance criteria
- pub `new` function L14-19 — `(body: &str) -> Self` — Create new content from body text
- pub `with_acceptance_criteria` function L22-27 — `(body: &str, acceptance_criteria: &str) -> Self` — Create content with both body and acceptance criteria
- pub `from_markdown` function L30-42 — `(content: &str) -> Self` — Parse content from markdown, separating main content from acceptance criteria
- pub `full_content` function L45-50 — `(&self) -> String` — Get the full content including acceptance criteria
- pub `has_acceptance_criteria` function L53-55 — `(&self) -> bool` — Check if acceptance criteria are present
-  `DocumentContent` type L12-56 — `= DocumentContent`

#### crates/metis-docs-core/src/domain/documents/factory.rs

- pub `DocumentFactory` struct L15 — `-` — Factory for creating documents from files
- pub `from_file` function L20-53 — `( path: P, ) -> Result<Box<dyn Document>, DocumentValidationError>` — Create a document from a file path
- pub `from_content` function L56-84 — `( raw_content: &str, _filepath: &str, ) -> Result<Box<dyn Document>, DocumentVal...` — Create a document from raw content string
-  `DocumentFactory` type L17-123 — `= DocumentFactory`
-  `extract_document_type` function L87-122 — `(raw_content: &str) -> Result<DocumentType, DocumentValidationError>` — Extract document type from frontmatter
-  `tests` module L126-183 — `-`
-  `test_extract_document_type` function L130-155 — `()`
-  `test_extract_document_type_missing` function L158-168 — `()`
-  `test_extract_document_type_invalid` function L171-182 — `()`

#### crates/metis-docs-core/src/domain/documents/helpers.rs

- pub `FrontmatterParser` struct L7 — `-` — Helper methods for parsing frontmatter
- pub `extract_string` function L10-24 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Res...`
- pub `extract_bool` function L26-40 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Res...`
- pub `extract_integer` function L42-56 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Res...`
- pub `extract_datetime` function L58-71 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Res...`
- pub `extract_tags` function L73-95 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, ) -> Result<Vec<Tag...`
- pub `extract_string_array` function L97-119 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Res...`
- pub `extract_optional_string` function L123-138 — `( map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str, ) -> Opt...` — Extract an optional string field from frontmatter
-  `FrontmatterParser` type L9-139 — `= FrontmatterParser`
-  `tests` module L142-294 — `-`
-  `create_test_map` function L148-176 — `() -> HashMap<String, Pod>`
-  `test_extract_string` function L179-192 — `()`
-  `test_extract_bool` function L195-205 — `()`
-  `test_extract_integer` function L208-221 — `()`
-  `test_extract_datetime` function L224-240 — `()`
-  `test_extract_tags` function L243-260 — `()`
-  `test_extract_string_array` function L263-274 — `()`
-  `test_extract_tags_with_invalid_tags` function L277-293 — `()`

#### crates/metis-docs-core/src/domain/documents/metadata.rs

- pub `DocumentMetadata` struct L6-11 — `{ created_at: DateTime<Utc>, updated_at: DateTime<Utc>, exit_criteria_met: bool,...` — Document metadata containing timestamps and other document properties
- pub `new` function L15-23 — `(short_code: String) -> Self` — Create new metadata with current timestamps and short code
- pub `from_frontmatter` function L26-38 — `( created_at: DateTime<Utc>, updated_at: DateTime<Utc>, exit_criteria_met: bool,...` — Create metadata from parsed frontmatter data
- pub `update` function L41-43 — `(&mut self)` — Update the updated_at timestamp to now
- pub `mark_exit_criteria_met` function L46-49 — `(&mut self)` — Mark exit criteria as met and update timestamp
-  `DocumentMetadata` type L13-50 — `= DocumentMetadata`

#### crates/metis-docs-core/src/domain/documents/mod.rs

- pub `content` module L1 — `-`
- pub `factory` module L2 — `-`
- pub `helpers` module L3 — `-`
- pub `metadata` module L4 — `-`
- pub `traits` module L6 — `-` — Document domain module
- pub `types` module L7 — `-`
- pub `adr` module L10 — `-`
- pub `initiative` module L11 — `-`
- pub `specification` module L12 — `-`
- pub `task` module L13 — `-`
- pub `vision` module L14 — `-`

#### crates/metis-docs-core/src/domain/documents/traits.rs

- pub `Document` interface L7-170 — `{ fn id(), fn document_type(), fn title(), fn metadata(), fn content(), fn core(...` — Core document trait that all document types must implement
- pub `DocumentTemplate` struct L173-178 — `{ frontmatter: &'static str, content: &'static str, acceptance_criteria: &'stati...` — Template information for a document
- pub `DocumentCore` struct L182-191 — `{ title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Common document data that all document types share
- pub `DocumentValidationError` enum L195-213 — `InvalidTitle | InvalidParent | InvalidPhaseTransition | MissingRequiredField | I...` — Validation errors for documents
-  `id` function L9-11 — `(&self) -> DocumentId` — Get the unique identifier for this document (derived from title)
-  `tags` function L29-31 — `(&self) -> &[Tag]` — Get the document tags
-  `phase` function L34-43 — `(&self) -> Result<Phase, DocumentValidationError>` — Get the current phase of the document (parsed from tags)
-  `update_section` function L55-128 — `( &mut self, content: &str, heading: &str, append: bool, ) -> Result<(), Documen...` — Update a specific section (H2 heading) in the document content
-  `update_content_body` function L131-137 — `(&mut self, new_body: String) -> Result<(), DocumentValidationError>` — Helper method for update_section to actually mutate the content
-  `archived` function L143-145 — `(&self) -> bool` — Check if this document is archived

#### crates/metis-docs-core/src/domain/documents/types.rs

- pub `DocumentId` struct L10 — `-` — Document identifier - always derived from title as a slug
- pub `new` function L14-22 — `(id: &str) -> Self` — Create a new DocumentId from a raw string (used for ADRs with custom format)
- pub `from_title` function L25-28 — `(title: &str) -> Self` — Create a DocumentId from a title by converting to slug
- pub `title_to_slug` function L31-57 — `(title: &str) -> String` — Convert title to URL-friendly slug
- pub `as_str` function L60-62 — `(&self) -> &str` — Get the string representation
- pub `ParentReference` enum L86-94 — `Some | None | Null` — Parent reference for documents in flexible flight levels
- pub `to_path_string` function L98-104 — `(&self) -> String` — Convert to string for path construction
- pub `has_parent` function L107-109 — `(&self) -> bool` — Check if this reference points to an actual parent
- pub `parent_id` function L112-117 — `(&self) -> Option<&DocumentId>` — Get the parent ID if it exists
- pub `from_option` function L120-125 — `(id: Option<DocumentId>) -> Self` — Create from optional document ID
- pub `null` function L128-130 — `() -> Self` — Create a null reference for disabled levels
- pub `DocumentType` enum L153-159 — `Vision | Initiative | Task | Adr | Specification` — Document type enumeration
- pub `valid_transitions_from` function L192-227 — `(&self, from_phase: Phase) -> Vec<Phase>` — Get valid transitions from a given phase for this document type.
- pub `can_transition` function L230-232 — `(&self, from: Phase, to: Phase) -> bool` — Check if a transition from one phase to another is valid for this document type.
- pub `next_phase` function L236-238 — `(&self, current: Phase) -> Option<Phase>` — Get the next phase in the natural sequence for this document type.
- pub `phase_sequence` function L241-271 — `(&self) -> Vec<Phase>` — Get the ordered phase sequence for this document type (for display purposes).
- pub `Phase` enum L276-302 — `Draft | Review | Published | Discussion | Decided | Superseded | Backlog | Todo ...` — Document phase/status
- pub `Tag` enum L329-332 — `Phase | Label` — Document tag that can be either a phase or a string
- pub `to_str` function L402-413 — `(&self) -> String` — Convert tag back to its string representation (reverse of from_str)
-  `MAX_ID_LENGTH` variable L6 — `: usize` — Maximum length for document IDs
-  `DocumentId` type L12-63 — `= DocumentId`
-  `DocumentId` type L65-69 — `= DocumentId`
-  `fmt` function L66-68 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `DocumentId` type L71-75 — `= DocumentId`
-  `from` function L72-74 — `(s: String) -> Self`
-  `DocumentId` type L77-81 — `= DocumentId`
-  `from` function L78-80 — `(s: &str) -> Self`
-  `ParentReference` type L96-131 — `= ParentReference`
-  `ParentReference` type L133-137 — `= ParentReference`
-  `from` function L134-136 — `(id: DocumentId) -> Self`
-  `ParentReference` type L139-143 — `= ParentReference`
-  `from` function L140-142 — `(opt: Option<DocumentId>) -> Self`
-  `ParentReference` type L145-149 — `= ParentReference`
-  `fmt` function L146-148 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `DocumentType` type L161-171 — `= DocumentType`
-  `fmt` function L162-170 — `(&self, f: &mut fmt::Formatter) -> fmt::Result`
-  `DocumentType` type L173-186 — `impl FromStr for DocumentType`
-  `Err` type L174 — `= String`
-  `from_str` function L176-185 — `(s: &str) -> Result<Self, Self::Err>`
-  `DocumentType` type L188-272 — `= DocumentType`
-  `Phase` type L304-325 — `= Phase`
-  `fmt` function L305-324 — `(&self, f: &mut fmt::Formatter) -> fmt::Result`
-  `Tag` type L334-347 — `= Tag`
-  `fmt` function L335-346 — `(&self, f: &mut fmt::Formatter) -> fmt::Result`
-  `Tag` type L349-353 — `= Tag`
-  `from` function L350-352 — `(phase: Phase) -> Self`
-  `Tag` type L355-359 — `= Tag`
-  `from` function L356-358 — `(label: String) -> Self`
-  `Tag` type L361-365 — `= Tag`
-  `from` function L362-364 — `(label: &str) -> Self`
-  `Tag` type L367-398 — `= Tag`
-  `Err` type L368 — `= ()`
-  `from_str` function L370-397 — `(s: &str) -> Result<Self, Self::Err>`
-  `Tag` type L400-414 — `= Tag`
-  `tests` module L417-598 — `-`
-  `test_title_to_slug` function L421-438 — `()`
-  `test_id_length_capping` function L441-446 — `()`
-  `test_adr_custom_id` function L449-452 — `()`
-  `test_tag_parsing` function L455-480 — `()`
-  `test_tag_to_str` function L483-491 — `()`
-  `test_tag_roundtrip` function L494-507 — `()`
-  `test_document_type_valid_transitions` function L510-533 — `()`
-  `test_document_type_can_transition` function L536-552 — `()`
-  `test_document_type_next_phase` function L555-574 — `()`
-  `test_document_type_phase_sequence` function L577-597 — `()`

### crates/metis-docs-core/src/domain/documents/initiative

**Role**: Implements the Initiative document type — a concrete implementation plan sitting between Strategy and Tasks in the Flight Levels hierarchy, with a six-phase lifecycle and complexity sizing.

**Key abstractions**:
- `Initiative` — Embeds `DocumentCore` plus `estimated_complexity: Complexity`. Parent is typically a Strategy but may be None in streamlined configs. `strategy_id` is a second optional parent reference storing the grandparent strategy ID for hierarchy queries.
- `Complexity` enum — XS/S/M/L/XL sizing. Stored in frontmatter as `level`. `from_str()` returns `DocumentValidationError` for unknown values.
- Phase sequence: Discovery → Design → Ready → Decompose → Active → Completed. Tasks can only be added when the initiative is in Decompose or Active phase (enforced at the task creation level, not here).

**Mixed concerns / gotchas**: `new_with_template()` accepts both `parent_id` (the Strategy's `DocumentId`) and `strategy_id` (also the Strategy's `DocumentId`). These are redundant in the normal case but both required because some callers only have one or the other available.

#### crates/metis-docs-core/src/domain/documents/initiative/mod.rs

- pub `Complexity` enum L13-19 — `XS | S | M | L | XL` — Complexity level for initiatives
- pub `Initiative` struct L53-56 — `{ core: super::traits::DocumentCore, estimated_complexity: Complexity }` — An Initiative document represents a concrete implementation project
- pub `new` function L61-82 — `( title: String, parent_id: Option<DocumentId>, blocked_by: Vec<DocumentId>, tag...` — Create a new Initiative document with content rendered from template
- pub `new_with_template` function L86-130 — `( title: String, parent_id: Option<DocumentId>, blocked_by: Vec<DocumentId>, tag...` — Create a new Initiative document with a custom template
- pub `from_parts` function L134-159 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create an Initiative document from existing data (used when loading from file)
- pub `estimated_complexity` function L161-163 — `(&self) -> Complexity`
- pub `from_file` function L190-196 — `(path: P) -> Result<Self, DocumentValidationError>` — Create an Initiative document by reading and parsing a file
- pub `from_content` function L199-274 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create an Initiative document from raw file content string
- pub `to_file` function L277-282 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Initiative document to a file
- pub `to_content` function L285-358 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Initiative document to its markdown string representation using templates
-  `Complexity` type L21-31 — `= Complexity`
-  `fmt` function L22-30 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`
-  `Complexity` type L33-49 — `= Complexity`
-  `Err` type L34 — `= DocumentValidationError`
-  `from_str` function L36-48 — `(s: &str) -> Result<Self, Self::Err>`
-  `Initiative` type L58-359 — `= Initiative`
-  `next_phase_in_sequence` function L166-177 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Initiative sequence
-  `update_phase_tag` function L180-187 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Initiative` type L361-480 — `impl Document for Initiative`
-  `document_type` function L364-366 — `(&self) -> DocumentType`
-  `title` function L368-370 — `(&self) -> &str`
-  `metadata` function L372-374 — `(&self) -> &DocumentMetadata`
-  `content` function L376-378 — `(&self) -> &DocumentContent`
-  `core` function L380-382 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L384-391 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L393-395 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L397-399 — `(&self) -> &[DocumentId]`
-  `validate` function L401-417 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L419-424 — `(&self) -> bool`
-  `template` function L426-433 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L435-437 — `(&self) -> &'static str`
-  `content_template` function L439-441 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L443-445 — `(&self) -> &'static str`
-  `transition_phase` function L447-475 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L477-479 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L483-684 — `-`
-  `test_initiative_from_content` function L488-553 — `()`
-  `test_initiative_complexity_parsing` function L556-565 — `()`
-  `test_initiative_invalid_level` function L568-594 — `()`
-  `test_initiative_validation` function L597-641 — `()`
-  `test_initiative_phase_transitions` function L644-674 — `()`
-  `test_complexity_display` function L677-683 — `()`

### crates/metis-docs-core/src/domain/documents/specification

> *Semantic summary to be generated by AI agent.*

#### crates/metis-docs-core/src/domain/documents/specification/mod.rs

- pub `Specification` struct L16-18 — `{ core: super::traits::DocumentCore }` — A Specification captures system-level or project-level design:
- pub `new` function L22-31 — `( title: String, parent_id: DocumentId, tags: Vec<Tag>, archived: bool, short_co...` — Create a new Specification document with content rendered from template
- pub `new_with_template` function L34-72 — `( title: String, parent_id: DocumentId, tags: Vec<Tag>, archived: bool, short_co...` — Create a new Specification document with a custom template
- pub `from_parts` function L75-95 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create a Specification from existing data (used when loading from file)
- pub `from_file` function L98-104 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Specification document by reading and parsing a file
- pub `from_content` function L107-163 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Specification document from raw file content string
- pub `to_file` function L185-190 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Specification document to a file
- pub `to_content` function L193-248 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Specification document to its markdown string representation
-  `Specification` type L20-249 — `= Specification`
-  `next_phase_in_sequence` function L166-175 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Specification sequence
-  `update_phase_tag` function L178-182 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Specification` type L251-358 — `impl Document for Specification`
-  `id` function L252-254 — `(&self) -> DocumentId`
-  `document_type` function L256-258 — `(&self) -> DocumentType`
-  `title` function L260-262 — `(&self) -> &str`
-  `metadata` function L264-266 — `(&self) -> &DocumentMetadata`
-  `content` function L268-270 — `(&self) -> &DocumentContent`
-  `core` function L272-274 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L276-282 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L284-286 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L288-290 — `(&self) -> &[DocumentId]`
-  `validate` function L292-300 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L302-304 — `(&self) -> bool`
-  `template` function L306-313 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L315-317 — `(&self) -> &'static str`
-  `content_template` function L319-321 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L323-325 — `(&self) -> &'static str`
-  `transition_phase` function L327-353 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L355-357 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L361-604 — `-`
-  `test_specification_from_content` function L365-408 — `()`
-  `test_specification_invalid_level` function L411-439 — `()`
-  `test_specification_validation` function L442-453 — `()`
-  `test_specification_empty_title_validation` function L456-467 — `()`
-  `test_specification_cannot_be_blocked` function L470-481 — `()`
-  `test_specification_phase_transitions` function L484-522 — `()`
-  `test_specification_transition_phase_auto` function L525-550 — `()`
-  `test_specification_transition_phase_invalid` function L553-576 — `()`
-  `test_specification_to_content_roundtrip` function L579-603 — `()`

### crates/metis-docs-core/src/domain/documents/task

**Role**: Implements the Task document type — the leaf-level unit of work, with a four-phase lifecycle that includes a blocking state.

**Key abstractions**:
- `Task` — The simplest document type; `DocumentCore` with no additional fields. Parent is an Initiative (or Vision in `direct` config). Tasks support `blocked_by: Vec<DocumentId>` — a non-empty blocked list forces the task into the `Blocked` phase; transitioning out of `Blocked` requires clearing all blockers.
- Phase sequence: Todo → Active → Blocked (from Active) ↔ Active → Completed. `Blocked` is a lateral state from `Active`, not a forward step; `can_transition_to()` enforces this asymmetry.
- `validate()` — Checks that blocked tasks actually list blockers and that the blocker IDs are non-empty strings.

**Mixed concerns / gotchas**: The `Blocked` phase is both a valid phase and a validation constraint — `validate()` rejects a Task tagged `Blocked` with no `blocked_by` entries. This means the domain type validates logical consistency beyond mere structural validity.

#### crates/metis-docs-core/src/domain/documents/task/mod.rs

- pub `Task` struct L13-15 — `{ core: super::traits::DocumentCore }` — A Task document represents a concrete, actionable piece of work
- pub `new` function L20-43 — `( title: String, parent_id: Option<DocumentId>, // Usually an Initiative parent_...` — Create a new Task document with content rendered from template
- pub `new_with_template` function L47-93 — `( title: String, parent_id: Option<DocumentId>, parent_title: Option<String>, in...` — Create a new Task document with a custom template
- pub `from_parts` function L97-119 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create a Task document from existing data (used when loading from file)
- pub `from_file` function L122-128 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Task document by reading and parsing a file
- pub `from_content` function L131-206 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Task document from raw file content string
- pub `to_file` function L232-237 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Task document to a file
- pub `to_content` function L240-309 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Task document to its markdown string representation using templates
-  `Task` type L17-310 — `= Task`
-  `next_phase_in_sequence` function L209-219 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Task sequence
-  `update_phase_tag` function L222-229 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Task` type L312-449 — `impl Document for Task`
-  `document_type` function L315-317 — `(&self) -> DocumentType`
-  `title` function L319-321 — `(&self) -> &str`
-  `metadata` function L323-325 — `(&self) -> &DocumentMetadata`
-  `content` function L327-329 — `(&self) -> &DocumentContent`
-  `core` function L331-333 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L335-342 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L344-346 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L348-350 — `(&self) -> &[DocumentId]`
-  `validate` function L352-386 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L388-393 — `(&self) -> bool`
-  `template` function L395-402 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L404-406 — `(&self) -> &'static str`
-  `content_template` function L408-410 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L412-414 — `(&self) -> &'static str`
-  `transition_phase` function L416-444 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L446-448 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L452-801 — `-`
-  `test_task_from_content` function L458-514 — `()`
-  `test_task_invalid_level` function L517-542 — `()`
-  `test_task_validation` function L545-574 — `()`
-  `test_task_blocked_validation` function L577-607 — `()`
-  `test_task_phase_transitions` function L610-627 — `()`
-  `test_task_active_phase_transitions` function L630-646 — `()`
-  `test_task_blocked_phase_transitions` function L649-665 — `()`
-  `test_task_transition_phase_auto` function L668-695 — `()`
-  `test_task_transition_phase_blocking` function L698-726 — `()`
-  `test_task_transition_phase_invalid` function L729-755 — `()`
-  `test_task_update_section` function L758-800 — `()`

### crates/metis-docs-core/src/domain/documents/vision

**Role**: Implements the Vision document type — the root of the document hierarchy, representing the project's long-term direction, with a three-phase lifecycle.

**Key abstractions**:
- `Vision` — Embeds `DocumentCore` with no type-specific fields beyond what `DocumentCore` provides. Visions have no parent. A workspace can have multiple Visions (though typically one).
- Phase sequence: Draft → Review → Published. Once Published, no further transitions are possible (no Completed or Archived phase in the type's sequence — archiving is a separate operation).
- `validate()` — Requires a non-empty title and that exactly one `Tag::Phase` tag is present.

**Internal flow**: `from_content()` is simpler than other types because Vision has no extra frontmatter fields. `to_content()` renders the embedded Vision template with title, short code, timestamps, tags, and content.

#### crates/metis-docs-core/src/domain/documents/vision/mod.rs

- pub `Vision` struct L12-14 — `{ core: super::traits::DocumentCore }` — A Vision document represents the high-level direction and goals
- pub `new` function L18-27 — `( title: String, tags: Vec<Tag>, archived: bool, short_code: String, ) -> Result...` — Create a new Vision document with content rendered from template
- pub `new_with_template` function L30-68 — `( title: String, tags: Vec<Tag>, archived: bool, short_code: String, template_co...` — Create a new Vision document with a custom template
- pub `from_parts` function L71-90 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, tags: Vec...` — Create a Vision document from existing data (used when loading from file)
- pub `from_file` function L93-99 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Vision document by reading and parsing a file
- pub `from_content` function L102-154 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Vision document from raw file content string
- pub `to_file` function L178-183 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Vision document to a file
- pub `to_content` function L186-237 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Vision document to its markdown string representation using templates
-  `Vision` type L16-238 — `= Vision`
-  `next_phase_in_sequence` function L157-165 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Vision sequence
-  `update_phase_tag` function L168-175 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Vision` type L240-364 — `impl Document for Vision`
-  `document_type` function L243-245 — `(&self) -> DocumentType`
-  `title` function L247-249 — `(&self) -> &str`
-  `metadata` function L251-253 — `(&self) -> &DocumentMetadata`
-  `content` function L255-257 — `(&self) -> &DocumentContent`
-  `core` function L259-261 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L263-270 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L272-274 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L276-278 — `(&self) -> &[DocumentId]`
-  `validate` function L280-301 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L303-308 — `(&self) -> bool`
-  `template` function L310-317 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L319-321 — `(&self) -> &'static str`
-  `content_template` function L323-325 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L327-329 — `(&self) -> &'static str`
-  `transition_phase` function L331-359 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L361-363 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L367-760 — `-`
-  `test_vision_from_content` function L373-431 — `()`
-  `test_vision_invalid_level` function L434-459 — `()`
-  `test_vision_missing_title` function L462-486 — `()`
-  `test_vision_tag_parsing` function L489-537 — `()`
-  `test_vision_validation` function L540-569 — `()`
-  `test_vision_phase_transitions` function L572-597 — `()`
-  `test_vision_transition_phase_auto` function L600-640 — `()`
-  `test_vision_transition_phase_explicit` function L643-675 — `()`
-  `test_vision_transition_phase_invalid` function L678-707 — `()`
-  `test_vision_update_section` function L710-759 — `()`

### crates/metis-docs-core/src/error

**Role**: Provides error enrichment utilities and user-facing error message generation on top of the base `MetisError` type.

**Key abstractions**:
- `ErrorContext` trait — Adds `.with_context(f)` and `.with_static_context(ctx)` to `Result<T, E>` for any `E` that converts to `MetisError`. Lets call sites annotate errors with additional context without custom error variants.
- `UserFriendlyError` trait — Implemented for `MetisError`; translates each error variant into a human-readable string suitable for CLI/GUI display, grouped by `ErrorCategory`. The `to_user_message()` method generates actionable messages (e.g., "Document not found: {id}. Use 'metis list' to see available documents.").
- `ErrorCategory` — An enum (Workspace, Document, Database, FileSystem, Validation, Network, Configuration) used by the GUI and CLI to decide how to present errors (e.g., whether to show a recovery suggestion).

#### crates/metis-docs-core/src/error/conversions.rs

- pub `ErrorContext` interface L6-14 — `{ fn with_context(), fn with_static_context() }` — Trait for converting errors with additional context
- pub `UserFriendlyError` interface L43-49 — `{ fn to_user_message(), fn error_category() }` — Trait for creating user-friendly error messages from MetisError
- pub `ErrorCategory` enum L52-60 — `Workspace | Document | Database | FileSystem | Validation | Network | Configurat...` — Error conversion traits and utilities for consistent error handling across crates
-  `with_context` function L20-30 — `(self, f: F) -> Result<T, MetisError>` — Error conversion traits and utilities for consistent error handling across crates
-  `with_static_context` function L32-39 — `(self, context: &'static str) -> Result<T, MetisError>` — Error conversion traits and utilities for consistent error handling across crates
-  `MetisError` type L62-151 — `impl UserFriendlyError for MetisError` — Error conversion traits and utilities for consistent error handling across crates
-  `to_user_message` function L63-129 — `(&self) -> String` — Error conversion traits and utilities for consistent error handling across crates
-  `error_category` function L131-150 — `(&self) -> ErrorCategory` — Error conversion traits and utilities for consistent error handling across crates
-  `tests` module L154-191 — `-` — Error conversion traits and utilities for consistent error handling across crates
-  `test_error_context` function L158-170 — `()` — Error conversion traits and utilities for consistent error handling across crates
-  `test_user_friendly_error_document_not_found` function L173-180 — `()` — Error conversion traits and utilities for consistent error handling across crates
-  `test_user_friendly_error_validation` function L183-190 — `()` — Error conversion traits and utilities for consistent error handling across crates

### crates/metis-docs-core/src/tests

**Role**: Shared test infrastructure for the core crate and other crates via the `test-utils` Cargo feature.

**Key abstractions**:
- `MetisTestHelper` — Sets up a fully initialized Metis workspace in a `tempfile::TempDir` in one call. Provides helpers to get a `Database` connection, write test files, create subdirectories, and validate the workspace structure. Used as the single setup pattern across all integration and unit tests that need a real workspace on disk.

#### crates/metis-docs-core/src/tests/common.rs

- pub `MetisTestHelper` struct L9-14 — `{ temp_dir: tempfile::TempDir, project_path: PathBuf, metis_dir: PathBuf, db_pat...` — Shared test helper for workspace setup
- pub `new` function L18-34 — `() -> Result<Self>` — Create a new test helper with initialized workspace
- pub `with_project_name` function L37-53 — `(project_name: &str) -> Result<Self>` — Create a new test helper with custom project name
- pub `get_database` function L56-59 — `(&self) -> Result<Database>` — Get a database connection for testing
- pub `project_path_string` function L62-64 — `(&self) -> String` — Get the project path as a string
- pub `metis_dir_string` function L67-69 — `(&self) -> String` — Get the metis directory path as a string
- pub `validate_workspace` function L72-82 — `(&self) -> Result<()>` — Ensure the workspace exists and is valid
- pub `create_test_subdirs` function L85-95 — `(&self, subdirs: &[&str]) -> Result<Vec<PathBuf>>` — Create additional directories for testing
- pub `write_test_file` function L98-112 — `( &self, relative_path: P, content: &str, ) -> Result<PathBuf>` — Write a test file to the workspace
-  `MetisTestHelper` type L16-113 — `= MetisTestHelper` — Common test utilities for Metis core and other crates

#### crates/metis-docs-core/src/tests/mod.rs

- pub `common` module L1 — `-`

### crates/metis-docs-core/tests

**Role**: Integration test suite for complex multi-step scenarios that exercise the full stack from filesystem to database, covering edge cases in correctness guarantees.

**Key abstractions**:
- `collision_resolution_test.rs` — Tests the short code collision resolution path in `SyncService`: creates duplicate short codes across files (simulating concurrent developer commits), runs `sync_directory()`, and verifies that colliding documents get renumbered and cross-references in sibling files are updated consistently.
- `configuration_recovery_test.rs` — Tests the "database as cache" property: deletes or corrupts the SQLite database file, calls sync/recovery, and verifies that the workspace is fully reconstructed from `config.toml` and the markdown files without data loss.
- `database_reconstruction_test.rs` — Tests `WorkspaceDetectionService::prepare_workspace()` when the database file is absent; confirms auto-reconstruction without user intervention.
- `id_path_consistency_test.rs` — Regression tests for the ID/path contract: creates documents, renames them, and verifies that the short code in frontmatter matches the file path component and the database record — catching any slug-generation or path-construction mismatches.
- `reassignment_test.rs` — End-to-end tests for `ReassignmentService`: reassigning tasks between initiatives, moving to backlog, and validating that non-task and wrong-phase reassignments are rejected.

#### crates/metis-docs-core/tests/collision_resolution_test.rs

-  `test_short_code_collision_resolution` function L12-178 — `()` — Integration test for METIS-T-0001: Multi-developer short code collision resolution
-  `test_sibling_cross_reference_update` function L182-273 — `()` — Test cross-reference updating in sibling documents
-  `test_collision_resolution_depth_ordering` function L277-372 — `()` — Test that collision resolution preserves document order by path depth

#### crates/metis-docs-core/tests/configuration_recovery_test.rs

-  `setup_test_workspace` function L16-31 — `() -> (TempDir, String, String)` — Helper to create a test workspace
-  `test_recovery_from_complete_database_loss` function L34-121 — `()` — - Config.toml synchronization
-  `test_counter_recovery_prevents_duplicates` function L124-168 — `()` — - Config.toml synchronization
-  `test_config_sync_on_normal_operations` function L171-215 — `()` — - Config.toml synchronization
-  `test_recovery_validates_short_code_format` function L218-268 — `()` — - Config.toml synchronization
-  `test_migration_from_old_workspace_without_config_toml` function L271-308 — `()` — - Config.toml synchronization
-  `test_recovery_from_corrupted_database_file` function L311-413 — `()` — - Config.toml synchronization

#### crates/metis-docs-core/tests/database_reconstruction_test.rs

-  `test_database_auto_reconstruction` function L16-101 — `()` — Integration test for METIS-T-0012: Database as cache only
-  `test_workspace_detection_without_database` function L105-122 — `()` — Test that workspace detection works without database present
-  `test_is_workspace_without_database` function L126-144 — `()` — Test that is_workspace only checks for .metis directory

#### crates/metis-docs-core/tests/id_path_consistency_test.rs

-  `setup_test_workspace` function L12-28 — `(project_name: &str) -> (tempfile::TempDir, PathBuf)`
-  `test_initiative_short_code_matches_path` function L31-73 — `()`
-  `test_initiative_id_path_consistency` function L76-108 — `()`
-  `test_task_id_path_consistency` function L111-170 — `()`
-  `test_adr_id_consistency` function L173-201 — `()`
-  `test_long_title_id_path_consistency` function L204-239 — `()`
-  `test_unicode_title_id_path_consistency` function L242-289 — `()`
-  `test_regression_id_path_mismatch_bug` function L295-393 — `()` — Regression test for the ID/path mismatch bug

#### crates/metis-docs-core/tests/reassignment_test.rs

-  `setup_test_workspace` function L10-86 — `() -> (tempfile::TempDir, std::path::PathBuf)` — Helper to create a test workspace with vision, initiative, and task
-  `test_reassign_backlog_to_initiative` function L90-120 — `()` — Test reassigning a backlog task to an initiative
-  `test_reassign_task_to_backlog` function L124-157 — `()` — Test reassigning a task from initiative to backlog
-  `test_reassign_between_initiatives` function L161-221 — `()` — Test reassigning a task between initiatives
-  `test_reassign_across_initiatives` function L225-285 — `()` — Test reassigning a task between initiatives under different parents
-  `test_reassign_non_task_fails` function L289-314 — `()` — Test that reassignment fails for non-task documents
-  `test_reassign_to_non_initiative_fails` function L318-347 — `()` — Test that reassignment to non-initiative parent fails
-  `test_reassign_to_wrong_phase_initiative_fails` function L351-383 — `()` — Test that reassignment to initiative in wrong phase fails
-  `test_reassign_missing_document_fails` function L387-412 — `()` — Test reassignment with missing source document

#### crates/metis-docs-core/tests/specification_test.rs

-  `setup_workspace` function L12-19 — `() -> (tempfile::TempDir, std::path::PathBuf)` — Helper to set up workspace and return (temp_dir, metis_dir)
-  `test_create_specification_with_parent` function L22-51 — `()` — Integration tests for the Specification document type
-  `test_create_specification_without_parent_fails` function L54-75 — `()` — Integration tests for the Specification document type
-  `test_specification_phase_transitions` function L78-112 — `()` — Integration tests for the Specification document type
-  `test_specification_sync_and_discovery` function L115-141 — `()` — Integration tests for the Specification document type
-  `test_specification_archive_no_cascade` function L144-182 — `()` — Integration tests for the Specification document type

### crates/metis-docs-gui/src/composables

**Role**: Vue 3 composables that encapsulate stateful UI concerns shared across components: project loading, theme management, and rich-text editor toolbar actions.

**Key abstractions**:
- `useProject()` — Manages the active project state (`ProjectInfo | null`) and a recent-projects list. `loadProject()` calls `MetisAPI.loadProject()` and stores the result reactively. Recent projects are persisted to `localStorage` via `saveRecentProject()`. Returns reactive refs and all project management functions to consuming components.
- `useTheme()` — Maintains the current `ThemeName` in a shared reactive ref. `setTheme()` applies the theme by updating a CSS custom property on `document.documentElement`; `toggleTheme()` cycles through available themes. `useThemedStyles()` returns computed style objects derived from the current theme's `ThemeColors`.
- `useToolbar()` — Wraps a TipTap `Editor` instance with all rich-text commands (bold, italic, headings, lists, tables, undo/redo). `isActive()` returns a `ComputedRef<boolean>` for a given mark/node name and attribute set, allowing toolbar buttons to show active state reactively.

**Dependencies**: Vue 3 Composition API (`ref`, `computed`); TipTap editor (for `useToolbar`); `MetisAPI` from `lib/tauri-api.ts` (for `useProject`); `ThemeName`/`ThemeColors` types from `types/theme.ts`.

#### crates/metis-docs-gui/src/composables/useProject.ts

- pub `useProject` function L18-110 — `function useProject()`
-  `ProjectState` interface L4-9 — `{ currentProject: : ProjectInfo | null, recentProjects: : ProjectInfo[], isLoadi...`
-  `loadProject` function L32-58 — `const loadProject = (path: string): Promise<void>`
-  `clearProject` function L60-62 — `const clearProject = ()`
-  `setCurrentProject` function L64-66 — `const setCurrentProject = (project: ProjectInfo | null)`
-  `addRecentProject` function L68-71 — `const addRecentProject = (project: ProjectInfo)`
-  `getRecentProjects` function L73-75 — `const getRecentProjects = (): ProjectInfo[]`
-  `saveRecentProject` function L77-81 — `const saveRecentProject = (project: ProjectInfo)`
-  `removeProject` function L83-92 — `const removeProject = (path: string)`

#### crates/metis-docs-gui/src/composables/useTheme.ts

- pub `useTheme` function L7-56 — `function useTheme()`
- pub `useThemedStyles` function L59-76 — `function useThemedStyles()`
-  `setTheme` function L39-41 — `const setTheme = (newThemeName: ThemeName)`
-  `toggleTheme` function L43-48 — `const toggleTheme = ()`

#### crates/metis-docs-gui/src/composables/useToolbar.ts

- pub `useToolbar` function L5-84 — `function useToolbar(editor: Ref<Editor | null>)`
-  `toggleBold` function L7 — `const toggleBold = ()`
-  `toggleItalic` function L8 — `const toggleItalic = ()`
-  `toggleStrike` function L9 — `const toggleStrike = ()`
-  `toggleHeading` function L12 — `const toggleHeading = (level: Level)`
-  `toggleBulletList` function L15 — `const toggleBulletList = ()`
-  `toggleOrderedList` function L16 — `const toggleOrderedList = ()`
-  `toggleBlockquote` function L17 — `const toggleBlockquote = ()`
-  `setHorizontalRule` function L20 — `const setHorizontalRule = ()`
-  `undo` function L21 — `const undo = ()`
-  `redo` function L22 — `const redo = ()`
-  `insertTable` function L25 — `const insertTable = ()`
-  `deleteTable` function L26 — `const deleteTable = ()`
-  `addColumnBefore` function L27 — `const addColumnBefore = ()`
-  `addColumnAfter` function L28 — `const addColumnAfter = ()`
-  `deleteColumn` function L29 — `const deleteColumn = ()`
-  `addRowBefore` function L30 — `const addRowBefore = ()`
-  `addRowAfter` function L31 — `const addRowAfter = ()`
-  `deleteRow` function L32 — `const deleteRow = ()`
-  `toggleHeaderColumn` function L33 — `const toggleHeaderColumn = ()`
-  `toggleHeaderRow` function L34 — `const toggleHeaderRow = ()`
-  `toggleHeaderCell` function L35 — `const toggleHeaderCell = ()`
-  `mergeCells` function L36 — `const mergeCells = ()`
-  `splitCell` function L37 — `const splitCell = ()`
-  `mergeOrSplit` function L38 — `const mergeOrSplit = ()`
-  `setCellAttribute` function L39 — `const setCellAttribute = (name: string, value: any)`
-  `isActive` function L42-43 — `const isActive = (name: string, attrs: Record<string, any> = {}): ComputedRef<bo...`

### crates/metis-docs-gui/src/lib

**Role**: Frontend library utilities — the Tauri command bridge (`tauri-api.ts`) and the kanban board configuration (`board-config.ts`).

**Key abstractions**:
- `MetisAPI` / `DocumentAPI` — Thin TypeScript wrappers around `@tauri-apps/api/core::invoke()`. Each method maps directly to a Tauri command registered in the Rust backend. `MetisAPI` covers project and navigation operations; `DocumentAPI` covers document CRUD and phase transitions. Type-safe interfaces (`ProjectInfo`, `DocumentInfo`, `DocumentContent`, etc.) mirror the Rust structs serialized across the bridge.
- `getBoardConfig()` — Returns a `BoardConfig` for a given `BoardType` (vision/strategy/initiative/task/adr/backlog), defining the ordered phase columns, their display titles, and empty-state messages. This configuration drives which kanban columns appear and in what order.
- `getDocumentsByPhase()` — Filters a flat `DocumentInfo[]` list by board type and phase, optionally further filtering by a selected initiative. This is the primary data transformation between the raw document list and the kanban view.
- `formatDate()`, `getDocumentTypeIcon()`, `getPhaseColor()` — Pure display utilities used in templates.

**Mixed concerns / gotchas**: `MetisAPI` and `DocumentAPI` are two separate classes rather than one, creating an inconsistent surface — some document operations live on `MetisAPI` (search, read, list) while others are on `DocumentAPI` (create, update, archive). Callers must know which class to import.

#### crates/metis-docs-gui/src/lib/board-config.ts

- pub `PhaseConfig` interface L4-9 — `{ key: : string, title: : string, description: : string, emptyMessage: : string ...`
- pub `BoardConfig` interface L11-17 — `{ id: : BoardType, title: : string, description: : string, phases: : PhaseConfig...`
- pub `getBoardConfig` function L242-244 — `function getBoardConfig(boardType: BoardType): BoardConfig | undefined`
- pub `InitiativeFilterOption` interface L246-249 — `{ short_code: : string, title: : string }`
- pub `getActiveInitiatives` function L254-259 — `function getActiveInitiatives(documents: DocumentInfo[]): InitiativeFilterOption...`
- pub `getAllInitiatives` function L264-269 — `function getAllInitiatives(documents: DocumentInfo[]): InitiativeFilterOption[]`
- pub `getDocumentsByPhase` function L277-335 — `function getDocumentsByPhase( documents: DocumentInfo[], boardType: BoardType, i...`

#### crates/metis-docs-gui/src/lib/tauri-api.ts

- pub `ProjectInfo` interface L4-8 — `{ path: : string, is_valid: : boolean, vision_exists: : boolean }`
- pub `DocumentInfo` interface L10-23 — `{ id: : string, title: : string, document_type: : string, short_code: : string, ...`
- pub `DocumentContent` interface L25-30 — `{ id: : string, title: : string, content: : string, frontmatter_json: : string }`
- pub `InitializationResult` interface L32-36 — `{ metis_dir: : string, database_path: : string, vision_path: : string }`
- pub `MetisAPI` class L39-127 — `-`
- pub `initializeProject` method L43-49 — `initializeProject( path: string, prefix?: string, preset?: string ): Promise<Ini...`
- pub `loadProject` method L54-56 — `loadProject(path: string): Promise<ProjectInfo>`
- pub `listDocuments` method L61-63 — `listDocuments(): Promise<DocumentInfo[]>`
- pub `readDocument` method L68-70 — `readDocument(shortCode: string): Promise<DocumentContent>`
- pub `searchDocuments` method L75-77 — `searchDocuments(query: string): Promise<DocumentInfo[]>`
- pub `getProjectConfig` method L82-84 — `getProjectConfig(): Promise<ProjectConfig>`
- pub `syncProject` method L89-91 — `syncProject(): Promise<SyncResult>`
- pub `getAvailableParents` method L96-98 — `getAvailableParents(childDocumentType: string): Promise<ParentOption[]>`
- pub `transitionPhase` method L103-105 — `transitionPhase(shortCode: string, newPhase?: string): Promise<string>`
- pub `getAppVersion` method L110-112 — `getAppVersion(): Promise<string>`
- pub `installCli` method L117-119 — `installCli(): Promise<void>`
- pub `installCliElevated` method L124-126 — `installCliElevated(): Promise<void>`
- pub `DocumentType` enum L130-137 — `Vision | Strategy | Initiative | Task | ADR | Specification`
- pub `Phase` enum L139-156 — `Draft | Review | Published | Shaping | Design | Ready | Active | Completed | Dis...`
- pub `formatDate` function L159-161 — `function formatDate(timestamp: number): string`
- pub `getDocumentTypeIcon` function L163-180 — `function getDocumentTypeIcon(type: string): string`
- pub `getPhaseColor` function L182-207 — `function getPhaseColor(phase: string): string`
- pub `CreateDocumentRequest` interface L209-216 — `{ document_type: : string, title: : string, parent_id: : string, complexity: : s...`
- pub `CreateDocumentResult` interface L218-222 — `{ id: : string, short_code: : string, filepath: : string }`
- pub `ParentOption` interface L224-229 — `{ short_code: : string, title: : string, document_type: : string, phase: : strin...`
- pub `ProjectConfig` interface L231-235 — `{ strategies_enabled: : boolean, initiatives_enabled: : boolean, preset_name: : ...`
- pub `ArchiveResult` interface L237-240 — `{ total_archived: : number, archived_documents: : ArchivedDocument[] }`
- pub `ArchivedDocument` interface L242-247 — `{ document_id: : string, document_type: : string, original_path: : string, archi...`
- pub `SyncResult` interface L249-256 — `{ imported: : number, updated: : number, deleted: : number, up_to_date: : number...`
- pub `DocumentAPI` class L259-287 — `-`
- pub `createDocument` method L263-265 — `createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResult>`
- pub `updateDocument` method L270-272 — `updateDocument(shortCode: string, content: string): Promise<void>`
- pub `transitionPhase` method L277-279 — `transitionPhase(shortCode: string, newPhase?: string): Promise<string>`
- pub `archiveDocument` method L284-286 — `archiveDocument(shortCode: string): Promise<ArchiveResult>`

### crates/metis-docs-gui/src/types

**Role**: Shared TypeScript type definitions for the GUI's two primary cross-cutting concerns: board view types and theme definitions.

**Key abstractions**:
- `BoardType` — A string union literal identifying which kanban board is active. Used throughout `board-config.ts` and the board components to select the right phase columns.
- `ThemeColors` / `Theme` / `ThemeName` — A deeply nested `ThemeColors` interface defining every semantic color token used by the UI (background layers, text, border, accent, status colors). `Theme` pairs a name with its `ThemeColors`. `ThemeName` constrains which themes exist (`'light' | 'dark' | 'hyper'`). The actual color values live in `themes/definitions.ts`.

#### crates/metis-docs-gui/src/types/board.ts

- pub `BoardType` type L1 — `= 'vision' | 'strategy' | 'initiative' | 'task' | 'adr' | 'backlog' | 'specifica...`

#### crates/metis-docs-gui/src/types/theme.ts

- pub `ThemeColors` interface L1-58 — `{ background: : { primary: string; secondary: string; tertiary: string; elevated...`
- pub `Theme` interface L60-63 — `{ name: : string, colors: : ThemeColors }`
- pub `ThemeName` type L65 — `= 'light' | 'dark' | 'hyper'`

### crates/metis-docs-gui/src/utils

**Role**: Frontend utility functions for drag-and-drop list reordering.

**Key abstractions**:
- `applyDrag()` — Takes an array and a `dragResult` object (from a drag-and-drop library) and returns a new array with the dragged item removed from its source index and inserted at the destination index. This is the sole utility in this module; it is consumed by kanban board components to reorder cards.

#### crates/metis-docs-gui/src/utils/drag-n-drop.ts

- pub `applyDrag` function L1-24 — `function applyDrag(arr: any[], dragResult: any)`

### crates/metis-docs-gui/src-tauri

**Role**: Build script for the Tauri application — synchronizes the Rust crate version into `tauri.conf.json` at build time.

**Key abstractions**:
- `sync_version_to_tauri_config()` — Reads `CARGO_PKG_VERSION` and writes it into the `tauri.conf.json` `"version"` field, ensuring the GUI app version always matches the workspace crate version without manual coordination.

#### crates/metis-docs-gui/src-tauri/build.rs

-  `main` function L4-9 — `()`
-  `sync_version_to_tauri_config` function L11-60 — `()`

### crates/metis-docs-gui/src-tauri/src

**Role**: The Tauri application entry point and state management — registers all Tauri commands and initializes the CLI auto-installer on startup.

**Key abstractions**:
- `AppState` — Holds `current_project: Option<PathBuf>`, wrapped in a `Mutex` managed by Tauri. All service commands receive this via `State<'_, Mutex<AppState>>` and lock it to read or update the active project path.
- `run()` — Configures the Tauri builder: installs plugins (dialog, shell, logger in debug mode), spawns the CLI `auto_install_cli` task on startup, registers every Tauri command handler, and starts the event loop. This is the single location where all service functions are wired up to their Tauri command names.

**Dependencies**: All service modules in `services/`; Tauri plugins `tauri_plugin_dialog`, `tauri_plugin_shell`, `tauri_plugin_log`.

#### crates/metis-docs-gui/src-tauri/src/lib.rs

- pub `AppState` struct L13-15 — `{ current_project: Option<PathBuf> }`
- pub `run` function L17-63 — `()`
-  `services` module L3 — `-`

#### crates/metis-docs-gui/src-tauri/src/main.rs

-  `main` function L4-6 — `()`

### crates/metis-docs-gui/src-tauri/src/services

**Role**: Tauri command implementations — the bridge between the Vue frontend and `metis_core` services, one file per domain concern.

**Key abstractions**:
- `project.rs` — `initialize_project()` creates a new workspace at a path with optional prefix and preset; `load_project()` validates an existing workspace and updates `AppState.current_project`; `get_project_config()` reads `FlightLevelConfig` from the database and returns it as the serializable `ProjectConfig` struct.
- `document.rs` — The most complex service file. `create_document()` dispatches to the appropriate `DocumentCreationService` method based on `document_type`, handling parent lookup for initiatives and tasks. `list_documents()` queries all five document types and assembles `DocumentInfo` structs. `read_document()` resolves a short code to a file path, reads the file, and returns both raw content and parsed frontmatter JSON. `get_available_parents()` returns the set of valid parent documents for a given child type under the current `FlightLevelConfig`.
- `sync.rs` — `sync_project()` runs the full `Application::sync_directory()` pipeline and counts `SyncResult` variants to produce a summary `SyncResult` struct.
- `transition.rs` — `transition_phase()` parses the optional phase string (or auto-advances) and delegates to `PhaseTransitionService`.
- `archive.rs` — `archive_document()` calls `ArchiveService::archive_document_by_short_code()` and serializes the result.
- `cli_installer.rs` — Manages the bundled `metis` CLI binary: copies the Tauri sidecar binary to an app-data directory, optionally creates a symlink in `/usr/local/bin` (elevated on macOS/Linux, registry PATH on Windows), and writes a `version-info.json` for update detection. `auto_install_cli()` runs on app startup and re-installs if the bundled version differs from the installed version.
- `version.rs` — Returns `env!("CARGO_PKG_VERSION")` as a string.

**Mixed concerns / gotchas**: `document.rs` contains `find_strategy_short_code_for_initiative()` as a private helper — the same logic also exists in `metis-docs-mcp/src/tools/create_document.rs`, meaning parent-resolution logic is duplicated across the GUI and MCP service layers.

#### crates/metis-docs-gui/src-tauri/src/services/archive.rs

- pub `ArchiveResult` struct L10-13 — `{ total_archived: usize, archived_documents: Vec<ArchivedDocument> }`
- pub `ArchivedDocument` struct L16-21 — `{ document_id: String, document_type: String, original_path: String, archived_pa...`
- pub `archive_document` function L24-93 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, ) -> Result<...`
-  `tests` module L96-121 — `-`
-  `test_archive_service_creation` function L103-120 — `()`

#### crates/metis-docs-gui/src-tauri/src/services/cli_installer.rs

- pub `CliInstallStatus` struct L17-23 — `{ installed: bool, version: Option<String>, binary_path: Option<String>, symlink...` — Status of CLI installation
- pub `CliInstallResult` struct L27-31 — `{ success: bool, message: String, needs_elevation: bool }` — Result of CLI installation attempt
- pub `get_cli_install_status` function L106-134 — `() -> Result<CliInstallStatus, String>` — Check CLI installation status
- pub `install_cli_internal` function L190-342 — `(app: &AppHandle) -> Result<CliInstallResult, String>` — Internal installation function - copies binary and attempts symlink
- pub `install_cli` function L346-348 — `(app: AppHandle) -> Result<CliInstallResult, String>` — Install CLI with user-level permissions
- pub `install_cli_elevated` function L352-398 — `(app: AppHandle) -> Result<CliInstallResult, String>` — Install CLI with elevated permissions (creates symlink in /usr/local/bin)
- pub `uninstall_cli` function L402-425 — `() -> Result<(), String>` — Uninstall CLI - remove binary and symlink
- pub `auto_install_cli` function L457-521 — `(app: AppHandle)` — Run auto-installation on app startup
-  `CliVersionInfo` struct L35-39 — `{ version: String, installed_at: String, binary_path: String }` — Version tracking info stored in app data
-  `get_cli_data_dir` function L42-44 — `() -> Option<PathBuf>` — Get the app data directory for CLI storage
-  `get_cli_binary_path` function L47-58 — `() -> Option<PathBuf>` — Get the CLI binary destination path within app data
-  `get_symlink_path` function L62-64 — `() -> Option<PathBuf>` — Get the symlink location for PATH integration
-  `get_symlink_path` function L67-69 — `() -> Option<PathBuf>` — 3.
-  `get_version_info_path` function L72-74 — `() -> Option<PathBuf>` — Get the version info file path
-  `read_version_info` function L77-81 — `() -> Option<CliVersionInfo>` — Read current installed CLI version info
-  `write_version_info` function L84-102 — `(version: &str, binary_path: &PathBuf) -> Result<(), String>` — Write CLI version info after installation
-  `get_sidecar_path` function L137-187 — `(app: &AppHandle) -> Result<PathBuf, String>` — Get the path to the bundled sidecar binary
-  `add_to_windows_path` function L429-454 — `(bin_dir: &std::path::Path) -> Result<(), String>` — Add a directory to Windows user PATH via registry

#### crates/metis-docs-gui/src-tauri/src/services/document.rs

- pub `DocumentInfo` struct L13-26 — `{ id: String, title: String, document_type: String, short_code: String, filepath...`
- pub `DocumentContent` struct L29-34 — `{ id: String, title: String, content: String, frontmatter_json: String }`
- pub `CreateDocumentRequest` struct L37-43 — `{ document_type: String, title: String, parent_id: Option<String>, complexity: O...`
- pub `CreateDocumentResult` struct L46-50 — `{ id: String, short_code: String, filepath: String }`
- pub `create_document` function L53-181 — `( state: State<'_, std::sync::Mutex<AppState>>, request: CreateDocumentRequest, ...`
- pub `list_documents` function L184-263 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<Vec<DocumentInfo>, S...`
- pub `read_document` function L266-310 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, ) -> Result<...`
- pub `search_documents` function L313-369 — `( state: State<'_, std::sync::Mutex<AppState>>, query: String, ) -> Result<Vec<D...`
- pub `update_document` function L372-424 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, content: Str...`
- pub `ParentOption` struct L427-432 — `{ short_code: String, title: String, document_type: String, phase: String }`
- pub `get_available_parents` function L435-546 — `( state: State<'_, std::sync::Mutex<AppState>>, child_document_type: String, ) -...`
-  `tests` module L549-644 — `-`
-  `test_create_adr_document` function L557-581 — `()`
-  `test_create_task_with_initiative_parent` function L584-643 — `()`
-  `add_tag_to_frontmatter` function L647-700 — `(file_path: &std::path::Path, tag: &str) -> Result<(), String>` — Add a tag to the frontmatter of a document file
-  `extract_tags_from_task_file` function L703-742 — `(filepath: &str) -> Result<Vec<String>, String>` — Extract tags from a task file by parsing it like the TUI does

#### crates/metis-docs-gui/src-tauri/src/services/mod.rs

- pub `archive` module L1 — `-`
- pub `cli_installer` module L2 — `-`
- pub `document` module L3 — `-`
- pub `project` module L4 — `-`
- pub `sync` module L5 — `-`
- pub `transition` module L6 — `-`
- pub `version` module L7 — `-`

#### crates/metis-docs-gui/src-tauri/src/services/project.rs

- pub `ProjectInfo` struct L13-17 — `{ path: String, is_valid: bool, vision_exists: bool }`
- pub `InitializationResult` struct L20-24 — `{ metis_dir: String, database_path: String, vision_path: String }`
- pub `ProjectConfig` struct L27-30 — `{ initiatives_enabled: bool, preset_name: String }`
- pub `initialize_project` function L33-74 — `( path: String, prefix: Option<String>, preset: Option<String>, ) -> Result<Init...`
- pub `load_project` function L77-99 — `( state: State<'_, std::sync::Mutex<AppState>>, path: String, ) -> Result<Projec...`
- pub `get_project_config` function L102-132 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<ProjectConfig, Strin...`
-  `determine_flight_config` function L135-145 — `(preset_name: Option<&str>) -> Result<FlightLevelConfig, String>` — Determine the flight level configuration based on preset name
-  `tests` module L148-185 — `-`
-  `test_initialize_project_success` function L153-171 — `()`
-  `test_initialize_project_with_default_prefix` function L174-184 — `()`

#### crates/metis-docs-gui/src-tauri/src/services/sync.rs

- pub `SyncResult` struct L7-14 — `{ imported: u32, updated: u32, deleted: u32, up_to_date: u32, errors: u32, messa...`
- pub `sync_project` function L17-120 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<SyncResult, String>`

#### crates/metis-docs-gui/src-tauri/src/services/transition.rs

- pub `transition_phase` function L31-108 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, new_phase: O...`
-  `parse_phase` function L8-28 — `(phase_str: &str) -> Result<Phase, String>`
-  `tests` module L111-128 — `-`
-  `test_parse_phase_valid` function L116-120 — `()`
-  `test_parse_phase_invalid` function L123-127 — `()`

#### crates/metis-docs-gui/src-tauri/src/services/version.rs

- pub `get_app_version` function L4-6 — `() -> String`

### crates/metis-docs-mcp/src

**Role**: The MCP (Model Context Protocol) server — exposes Metis workspace operations as tools callable by AI agents over a stdio JSON-RPC transport.

**Key abstractions**:
- `MetisServerHandler` — Implements `rust_mcp_sdk::ServerHandler`. `handle_list_tools_request()` returns the tool schema for all registered tools. `handle_call_tool_request()` deserializes the tool name and parameters and dispatches to the `MetisTools` enum (generated by `tool_box!` macro in `all_tools.rs`).
- `run()` in `lib.rs` — The async entry point. Detects the current workspace to configure log file location, generates dynamic server instructions (injecting current `FlightLevelConfig` into the static `instructions.md`), and starts the `rust_mcp_sdk` server on stdio transport.
- `generate_dynamic_instructions()` — Reads the current workspace's `FlightLevelConfig` from the database and prepends a "Current Project Configuration" section to the static `instructions.md` embedded in the binary. This means the AI agent sees context-sensitive usage guidance that reflects the project's actual hierarchy.
- `ToolOutput` in `formatting.rs` — A fluent builder for constructing markdown-formatted tool responses. Methods like `header()`, `field()`, `table()`, `diff()`, `phase_progress()`, and `build_result()` produce consistent, well-structured output optimized for rendering in Claude Code and similar terminals.
- `MetisServerConfig` — Minimal server configuration loaded from environment variables.

**Internal flow**: AI agent sends a JSON-RPC `tools/call` request → `MetisServerHandler::handle_call_tool_request()` matches the tool name → deserializes parameters into the specific tool struct → calls `tool.call_tool()` → returns a `CallToolResult` containing a `ToolOutput`-built markdown string.

**Dependencies**: `rust_mcp_sdk` for the MCP protocol implementation; `metis_core` for all workspace operations; `metis_code_index` for `IndexCodeTool`.

#### crates/metis-docs-mcp/src/config.rs

- pub `MetisServerConfig` struct L4-7 — `-`
- pub `from_env` function L10-13 — `() -> Result<Self>`
- pub `new` function L15-17 — `() -> Self`
-  `MetisServerConfig` type L9-18 — `= MetisServerConfig`

#### crates/metis-docs-mcp/src/error.rs

- pub `McpServerError` enum L4-22 — `DocumentNotFound | InvalidParameter | ProjectNotInitialized | CoreLibrary | Conf...`
- pub `Result` type L24 — `= std::result::Result<T, McpServerError>`

#### crates/metis-docs-mcp/src/error_utils.rs

- pub `tool_error` function L4-6 — `(msg: &str) -> CallToolError` — Helper function to create CallToolError from string messages
-  `tool_error` macro L10-14 — `-` — Helper function to create CallToolError from formatted string

#### crates/metis-docs-mcp/src/formatting.rs

- pub `Icons` struct L10 — `-` — Status icons for visual indicators
- pub `SUCCESS` variable L13 — `: &'static str` — that renders well in terminal contexts.
- pub `ERROR` variable L14 — `: &'static str` — that renders well in terminal contexts.
- pub `WARNING` variable L15 — `: &'static str` — that renders well in terminal contexts.
- pub `INFO` variable L16 — `: &'static str` — that renders well in terminal contexts.
- pub `PENDING` variable L17 — `: &'static str` — that renders well in terminal contexts.
- pub `ACTIVE` variable L18 — `: &'static str` — that renders well in terminal contexts.
- pub `ToolOutput` struct L23-25 — `{ sections: Vec<String> }` — Builder for constructing formatted tool output
- pub `new` function L28-30 — `() -> Self` — that renders well in terminal contexts.
- pub `header` function L33-36 — `(mut self, text: &str) -> Self` — Add an H2 header (primary section header)
- pub `subheader` function L39-42 — `(mut self, text: &str) -> Self` — Add an H3 header (subsection header)
- pub `text` function L45-48 — `(mut self, text: &str) -> Self` — Add plain text
- pub `blank` function L51-54 — `(mut self) -> Self` — Add an empty line for spacing
- pub `rule` function L57-60 — `(mut self) -> Self` — Add a horizontal rule
- pub `success` function L63-66 — `(mut self, msg: &str) -> Self` — Add a success message with checkmark
- pub `error` function L69-72 — `(mut self, msg: &str) -> Self` — Add an error message with X
- pub `warning` function L75-78 — `(mut self, msg: &str) -> Self` — Add a warning message
- pub `info` function L81-84 — `(mut self, msg: &str) -> Self` — Add an info message
- pub `field` function L87-90 — `(mut self, key: &str, value: &str) -> Self` — Add a key-value pair
- pub `code_inline` function L93-96 — `(mut self, code: &str) -> Self` — Add inline code
- pub `code_block` function L99-103 — `(mut self, code: &str, lang: Option<&str>) -> Self` — Add a fenced code block
- pub `diff` function L106-120 — `(mut self, old: &str, new: &str) -> Self` — Add a diff block showing before/after
- pub `table` function L123-175 — `(mut self, headers: &[&str], rows: Vec<Vec<String>>) -> Self` — Add a simple key-value table with proper column padding
- pub `status_list` function L178-189 — `(mut self, items: Vec<(&str, bool)>) -> Self` — Add a list of items with status indicators
- pub `bullet_list` function L192-196 — `(mut self, items: &[&str]) -> Self` — Add a bulleted list
- pub `indented` function L199-210 — `(mut self, items: Vec<(bool, &str)>) -> Self` — Add indented content (for nested items)
- pub `phase_progress` function L213-228 — `(mut self, phases: &[&str], current_index: usize) -> Self` — Add a phase progression indicator
- pub `hint` function L231-234 — `(mut self, msg: &str) -> Self` — Add a hint/tip message
- pub `build` function L237-239 — `(self) -> String` — Build the final output string
- pub `build_result` function L242-250 — `(self) -> CallToolResult` — Build a CallToolResult with TextContent (no structuredContent for proper Claude Code rendering)
- pub `format_error` function L254-266 — `(title: &str, message: &str, hint: Option<&str>) -> String` — Helper for formatting error responses consistently
- pub `error_result` function L269-287 — `(title: &str, message: &str, hint: Option<&str>) -> CallToolResult` — Helper for formatting error responses as CallToolResult
- pub `format_not_found` function L290-300 — `(resource_type: &str, identifier: &str, hint: Option<&str>) -> String` — Helper for formatting not-found errors
-  `Icons` type L12-19 — `= Icons` — that renders well in terminal contexts.
-  `ToolOutput` type L27-251 — `= ToolOutput` — that renders well in terminal contexts.
-  `tests` module L303-375 — `-` — that renders well in terminal contexts.
-  `test_basic_output` function L307-316 — `()` — that renders well in terminal contexts.
-  `test_table_output` function L319-336 — `()` — that renders well in terminal contexts.
-  `test_diff_output` function L339-348 — `()` — that renders well in terminal contexts.
-  `test_phase_progress` function L351-361 — `()` — that renders well in terminal contexts.
-  `test_error_formatting` function L364-374 — `()` — that renders well in terminal contexts.

#### crates/metis-docs-mcp/src/lib.rs

- pub `config` module L4 — `-`
- pub `error` module L5 — `-`
- pub `error_utils` module L6 — `-`
- pub `formatting` module L7 — `-`
- pub `server` module L8 — `-`
- pub `tools` module L9 — `-`
- pub `run` function L127-197 — `() -> AnyhowResult<()>` — Run the MCP server
-  `find_metis_log_path` function L30-44 — `() -> Option<String>`
-  `get_current_configuration` function L46-62 — `() -> Option<FlightLevelConfig>`
-  `generate_dynamic_instructions` function L64-108 — `() -> String`
-  `generate_operation_notes` function L110-124 — `(config: &FlightLevelConfig) -> String`

#### crates/metis-docs-mcp/src/main.rs

-  `main` function L4-6 — `() -> Result<()>`

#### crates/metis-docs-mcp/src/server.rs

- pub `MetisServerHandler` struct L18-21 — `{ config: Arc<MetisServerConfig> }`
- pub `new` function L24-29 — `(config: MetisServerConfig) -> Self`
-  `MetisServerHandler` type L23-30 — `= MetisServerHandler`
-  `MetisServerHandler` type L33-102 — `impl ServerHandler for MetisServerHandler`
-  `handle_list_tools_request` function L34-44 — `( &self, _params: Option<PaginatedRequestParams>, _runtime: Arc<dyn McpServer>, ...`
-  `handle_call_tool_request` function L46-101 — `( &self, params: CallToolRequestParams, _runtime: Arc<dyn McpServer>, ) -> Resul...`

### crates/metis-docs-mcp/src/tools

**Role**: One file per MCP tool; each file defines a parameter struct and implements `call_tool()` that performs the operation and returns a `CallToolResult`.

**Key abstractions**:
- `CreateDocumentTool` — The most complex tool. Dispatches to the appropriate `DocumentCreationService` method based on `document_type`. For initiatives and tasks it resolves parent IDs: `find_strategy_short_code_for_initiative()` queries the database to find the grandparent strategy. Returns a formatted summary with the new document's short code, type, and file path.
- `ReadDocumentTool` — Resolves a short code to a file path (via database lookup), reads the raw content, and renders a rich output including frontmatter fields (`extract_metadata()`), content sections (`extract_sections()`), and exit criteria with completion checkboxes (`extract_exit_criteria()`).
- `EditDocumentTool` — Performs a targeted find-and-replace inside a document's content. `perform_edit()` supports literal replacement or `replace_all`. Returns a diff showing the change.
- `TransitionPhaseTool` — Parses the optional `phase` parameter and calls `PhaseTransitionService`. Includes a `force` flag for bypassing validation (used in recovery scenarios). `get_phase_sequence()` is used to render the progress indicator in the response.
- `SearchDocumentsTool` — Sanitizes the query (`sanitize_search_query()` removes FTS special characters that would cause SQLite FTS syntax errors), then calls `DatabaseService::search_documents()`. Optionally filters by document type.
- `IndexCodeTool` — Re-implements the `metis index` command as an MCP tool, allowing AI agents to trigger code index generation directly.
- `ReassignParentTool` — Moves a task to a new parent initiative or to the backlog, with explicit `backlog_category` support.
- `all_tools.rs` — Uses the `rust_mcp_sdk::tool_box!` macro to generate a `MetisTools` enum that dispatches `call_tool()` to the right struct based on the tool name string.

**Mixed concerns / gotchas**: `IndexCodeTool::call_tool()` and `CreateDocumentTool::find_strategy_short_code_for_initiative()` contain logic duplicated from the CLI crate (`index.rs` and `create/initiative.rs` respectively). These should eventually be unified in service layer functions.

#### crates/metis-docs-mcp/src/tools/archive_document.rs

- pub `ArchiveDocumentTool` struct L19-24 — `{ project_path: String, short_code: String }`
- pub `call_tool` function L27-92 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ArchiveDocumentTool` type L26-93 — `= ArchiveDocumentTool`

#### crates/metis-docs-mcp/src/tools/create_document.rs

- pub `CreateDocumentTool` struct L26-43 — `{ project_path: String, document_type: String, title: String, parent_id: Option<...`
- pub `call_tool` function L46-242 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `CreateDocumentTool` type L45-243 — `= CreateDocumentTool`

#### crates/metis-docs-mcp/src/tools/edit_document.rs

- pub `EditDocumentTool` struct L20-31 — `{ project_path: String, short_code: String, search: String, replace: String, rep...`
- pub `call_tool` function L34-114 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `EditDocumentTool` type L33-137 — `= EditDocumentTool`
-  `perform_edit` function L116-136 — `(&self, content: &str) -> Result<(String, usize), CallToolError>`

#### crates/metis-docs-mcp/src/tools/index_code.rs

- pub `IndexCodeTool` struct L32-41 — `{ project_path: String, structure_only: Option<bool>, incremental: Option<bool> ...` — trigger index generation programmatically.
- pub `call_tool` function L44-275 — `(&self) -> std::result::Result<CallToolResult, CallToolError>` — trigger index generation programmatically.
-  `IndexCodeTool` type L43-276 — `= IndexCodeTool` — trigger index generation programmatically.
-  `extract_symbols_for_language` function L279-297 — `( language: Language, parsed: &ParsedFile, file_path: &str, ) -> Result<Vec<Symb...` — Dispatch symbol extraction to the appropriate language extractor.

#### crates/metis-docs-mcp/src/tools/initialize_project.rs

- pub `InitializeProjectTool` struct L19-24 — `{ project_path: String, prefix: Option<String> }`
- pub `call_tool` function L27-82 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `InitializeProjectTool` type L26-83 — `= InitializeProjectTool`

#### crates/metis-docs-mcp/src/tools/list_documents.rs

- pub `ListDocumentsTool` struct L20-26 — `{ project_path: String, include_archived: Option<bool> }`
- pub `call_tool` function L29-93 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ListDocumentsTool` type L28-127 — `= ListDocumentsTool`
-  `list_all_documents` function L95-126 — `( &self, repo: &mut metis_core::dal::database::repository::DocumentRepository, i...`

#### crates/metis-docs-mcp/src/tools/mod.rs

- pub `all_tools` module L1 — `-`
- pub `archive_document` module L2 — `-`
- pub `create_document` module L3 — `-`
- pub `edit_document` module L4 — `-`
- pub `index_code` module L5 — `-`
- pub `initialize_project` module L6 — `-`
- pub `list_documents` module L7 — `-`
- pub `read_document` module L8 — `-`
- pub `reassign_parent` module L9 — `-`
- pub `search_documents` module L10 — `-`
- pub `transition_phase` module L11 — `-`

#### crates/metis-docs-mcp/src/tools/read_document.rs

- pub `ReadDocumentTool` struct L20-25 — `{ project_path: String, short_code: String }`
- pub `call_tool` function L55-99 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ReadDocumentTool` type L27-211 — `= ReadDocumentTool`
-  `resolve_short_code_to_path` function L29-53 — `(&self, metis_dir: &Path) -> Result<String, CallToolError>` — Resolve short code to file path
-  `extract_metadata` function L101-160 — `(&self, content: &str) -> (String, String, String, String, String)`
-  `extract_sections` function L163-175 — `(&self, content: &str) -> Vec<String>`
-  `extract_exit_criteria` function L177-210 — `(&self, content: &str) -> Vec<ExitCriterion>`
-  `ExitCriterion` struct L214-217 — `{ text: String, completed: bool }`

#### crates/metis-docs-mcp/src/tools/reassign_parent.rs

- pub `ReassignParentTool` struct L21-30 — `{ project_path: String, short_code: String, new_parent_id: Option<String>, backl...`
- pub `call_tool` function L33-156 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ReassignParentTool` type L32-157 — `= ReassignParentTool`

#### crates/metis-docs-mcp/src/tools/search_documents.rs

- pub `SearchDocumentsTool` struct L19-31 — `{ project_path: String, query: String, document_type: Option<String>, limit: Opt...`
- pub `call_tool` function L49-126 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `SearchDocumentsTool` type L33-127 — `= SearchDocumentsTool`
-  `sanitize_search_query` function L35-47 — `(&self, query: &str) -> String` — Sanitize search query to prevent FTS syntax errors

#### crates/metis-docs-mcp/src/tools/transition_phase.rs

- pub `TransitionPhaseTool` struct L22-31 — `{ project_path: String, short_code: String, phase: Option<String>, force: Option...`
- pub `call_tool` function L34-91 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `TransitionPhaseTool` type L33-134 — `= TransitionPhaseTool`
-  `parse_phase` function L93-116 — `(&self, phase_str: &str) -> Result<Phase, CallToolError>`
-  `get_phase_sequence` function L118-133 — `(&self, document_type: &str) -> Vec<String>`

### crates/metis-docs-mcp/tests/common

**Role**: MCP-specific test helper that wraps `MetisTestHelper` from `metis_core` and adds MCP-layer setup conveniences.

**Key abstractions**:
- `McpTestHelper` — Delegates workspace creation to `MetisTestHelper` and adds `initialize_project()` (runs the `InitializeProjectTool` to set up the MCP-facing workspace), `set_flight_level_config()` (updates both `config.toml` and the database for flight level tests), and `get_project_name()` (extracts the project name from the initialized workspace directory).

#### crates/metis-docs-mcp/tests/common/mod.rs

- pub `McpTestHelper` struct L12-14 — `{ core_helper: MetisTestHelper }` — MCP-specific test helper that wraps the core helper
- pub `new` function L17-20 — `() -> Result<Self>` — Common utilities and helper functions for integration tests
- pub `metis_dir` function L24-26 — `(&self) -> String` — Get metis directory as string (for backward compatibility)
- pub `initialize_project` function L28-42 — `(&self) -> Result<()>` — Common utilities and helper functions for integration tests
- pub `get_database` function L44-46 — `(&self) -> Result<Database>` — Common utilities and helper functions for integration tests
- pub `get_project_name` function L48-57 — `(&self) -> String` — Common utilities and helper functions for integration tests
- pub `set_flight_level_config` function L61-77 — `(&self, flight_config: FlightLevelConfig) -> Result<()>` — Update flight level configuration in config.toml
-  `McpTestHelper` type L16-78 — `= McpTestHelper` — Common utilities and helper functions for integration tests

### crates/metis-docs-mcp/tests

**Role**: Integration and functional tests for the MCP server, ranging from in-process tool invocations to real subprocess communication over the MCP protocol.

**Key abstractions**:
- `functional_test.rs` — Tests complete workflows by calling tool structs directly (no subprocess): initialize project, create documents of all types, read, search, archive, and handle backlog items. Uses short codes throughout rather than raw IDs.
- `comprehensive_functional_test.rs` — Tests all three `FlightLevelConfig` presets (full, streamlined, direct) end-to-end, verifying that the correct document types are available, that invalid cross-preset operations are rejected, and that the hierarchy constraints are enforced.
- `configuration_scenarios_test.rs` — Focuses on configuration edge cases: error messages when creating disallowed document types, behavior after config switching, and the guidance text in error responses.
- `mcp_archive_test.rs` — Specifically tests cascading archive behavior: archiving a strategy also archives all its initiatives and tasks, and the archived state is reflected correctly in list/search results.
- `mcp_server_integration_test.rs` — Spawns the actual MCP server binary as a subprocess and communicates via the MCP JSON-RPC protocol over stdin/stdout, testing real protocol handshake, tool listing, and archive functionality. This is the highest-fidelity test of the deployed server.

**Mixed concerns / gotchas**: `extract_text_from_result()` and `extract_short_code()` helpers are duplicated across four test files rather than consolidated in `common/mod.rs`. Only `mcp_archive_test.rs` and `configuration_scenarios_test.rs` import from `common`.

#### crates/metis-docs-mcp/tests/comprehensive_functional_test.rs

-  `setup_project_with_config` function L10-41 — `(config: FlightLevelConfig) -> (TempDir, String, String)` — Helper to setup project with specific flight configuration
-  `extract_text_from_result` function L44-59 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `get_vision_short_code` function L62-79 — `(metis_path: &str) -> String` — Helper to get vision short code from list results (parses markdown table format)
-  `extract_short_code` function L82-93 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `test_streamlined_configuration_workflow` function L96-208 — `()` — These tests mirror real user workflows through MCP tool calls
-  `test_direct_configuration_workflow` function L211-323 — `()` — These tests mirror real user workflows through MCP tool calls
-  `test_specification_workflow` function L326-436 — `()` — These tests mirror real user workflows through MCP tool calls

#### crates/metis-docs-mcp/tests/configuration_scenarios_test.rs

-  `common` module L10 — `-` — These tests validate real-world usage scenarios for each configuration preset
-  `extract_text_from_result` function L13-28 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L31-42 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `test_streamlined_configuration_workflows` function L47-130 — `() -> Result<()>` — Test MCP server behavior with default streamlined configuration
-  `test_direct_configuration_workflows` function L135-202 — `() -> Result<()>` — Test MCP server behavior with direct configuration
-  `test_configuration_error_messages` function L206-259 — `() -> Result<()>` — Test configuration error messages provide actionable guidance
-  `test_configuration_switching_compatibility` function L263-328 — `() -> Result<()>` — Test configuration switching doesn't break existing documents

#### crates/metis-docs-mcp/tests/functional_test.rs

-  `extract_text_from_result` function L8-23 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L26-37 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `get_vision_short_code` function L40-57 — `(metis_path: &str) -> String` — Helper to get vision short code from list results (parses markdown table format)
-  `test_initialize_and_create_documents` function L60-133 — `()` — Clean functional tests for MCP tools using short codes
-  `test_archive_with_short_codes` function L136-171 — `()` — Clean functional tests for MCP tools using short codes
-  `test_search_with_short_code_hyphen` function L174-229 — `()` — Clean functional tests for MCP tools using short codes
-  `test_list_and_search_include_archived` function L232-331 — `()` — Clean functional tests for MCP tools using short codes
-  `test_create_backlog_items` function L334-450 — `()` — Clean functional tests for MCP tools using short codes

#### crates/metis-docs-mcp/tests/mcp_archive_test.rs

-  `common` module L3 — `-` — Archive behavior tests for MCP server to verify the fixed cascading functionality
-  `extract_text_from_result` function L11-26 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L29-40 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `test_mcp_archive_cascading_behavior` function L45-262 — `() -> Result<()>` — Test MCP server archive cascading behavior
-  `test_mcp_archive_error_handling` function L266-318 — `() -> Result<()>` — Test MCP server archive error handling

#### crates/metis-docs-mcp/tests/mcp_server_integration_test.rs

-  `McpServerProcess` struct L12-16 — `{ temp_dir: TempDir, project_path: String, metis_dir: String }` — Helper to build and spawn the MCP server binary
-  `McpServerProcess` type L18-230 — `= McpServerProcess` — This tests the real MCP protocol communication including the archive_document fix
-  `new` function L19-29 — `() -> Result<Self>` — This tests the real MCP protocol communication including the archive_document fix
-  `build_server` function L32-45 — `() -> Result<()>` — Build the MCP server binary
-  `spawn_server` function L48-56 — `(&self) -> Result<std::process::Child>` — Spawn the MCP server process and return handles for communication
-  `send_mcp_request` function L59-83 — `(child: &mut std::process::Child, request: Value) -> Result<Value>` — Send an MCP request and get response
-  `mcp_initialize` function L86-108 — `(child: &mut std::process::Child) -> Result<()>` — Send MCP protocol initialization handshake
-  `initialize_project` function L111-132 — `(&self, child: &mut std::process::Child) -> Result<()>` — Initialize the project using MCP protocol
-  `create_test_document` function L135-190 — `(&self, child: &mut std::process::Child) -> Result<String>` — Create a test document to archive
-  `test_archive_document` function L193-229 — `( &self, child: &mut std::process::Child, short_code: &str, ) -> Result<()>` — Test the archive_document functionality
-  `test_mcp_server_archive_document_integration` function L233-271 — `() -> Result<()>` — This tests the real MCP protocol communication including the archive_document fix
-  `test_mcp_server_list_tools` function L274-317 — `() -> Result<()>` — This tests the real MCP protocol communication including the archive_document fix

### tests/e2e

**Role**: Playwright end-to-end tests for the Tauri GUI application, testing the full user journey through a real browser/WebView instance.

**Key abstractions**:
- `MetisPage` in `fixtures.ts` — Page object model wrapping a Playwright `Page`. Provides typed accessors for all major UI elements (search input, kanban board, columns, theme button) and workflow methods: `setupTestProject()` (initializes a fresh workspace via the GUI) and `loadTestProject()` (reopens an existing one). Centralizes all selector strings.
- `app.spec.ts` — Basic application launch tests: window opens, title is correct, initial state is valid.
- `project.spec.ts` — Tests workspace initialization, project loading, document creation via the GUI, and kanban board rendering with the correct phase columns.
- `search.spec.ts` — Tests the search functionality: typing a query, dropdown results appearing, selecting a result, and the document detail view loading.

**Dependencies**: Playwright test framework; the compiled Tauri application binary (must be built before tests run).

#### tests/e2e/fixtures.ts

- pub `MetisPage` class L229-371 — `-`
- pub `constructor` method L230 — `constructor(private page: Page)`
- pub `goto` method L233-237 — `goto()`
- pub `setupTestProject` method L240-247 — `setupTestProject()`
- pub `loadTestProject` method L250-257 — `loadTestProject()`
- pub `searchInput` method L260-262 — `searchInput()`
- pub `searchDropdown` method L264-266 — `searchDropdown()`
- pub `searchResults` method L268-270 — `searchResults()`
- pub `themeButton` method L272-274 — `themeButton()`
- pub `kanbanBoard` method L276-278 — `kanbanBoard()`
- pub `kanbanColumns` method L280-282 — `kanbanColumns()`
- pub `kanbanCards` method L284-286 — `kanbanCards()`
- pub `boardTabs` method L288-290 — `boardTabs()`
- pub `projectSidebar` method L292-294 — `projectSidebar()`
- pub `mascotImage` method L296-298 — `mascotImage()`
- pub `homeIcon` method L300-302 — `homeIcon()`
- pub `search` method L305-309 — `search(query: string)`
- pub `clearSearch` method L311-313 — `clearSearch()`
- pub `selectSearchResult` method L315-317 — `selectSearchResult(index: number)`
- pub `navigateSearchResults` method L319-321 — `navigateSearchResults(direction: 'up' | 'down')`
- pub `selectTheme` method L323-326 — `selectTheme(theme: 'Light' | 'Dark' | 'Hyper')`
- pub `goHome` method L328-330 — `goHome()`
- pub `expectHomeScreen` method L333-335 — `expectHomeScreen()`
- pub `expectProjectLoaded` method L337-339 — `expectProjectLoaded()`
- pub `expectSearchDropdownVisible` method L341-343 — `expectSearchDropdownVisible()`
- pub `expectSearchDropdownHidden` method L345-347 — `expectSearchDropdownHidden()`
- pub `expectSearchResultsCount` method L349-351 — `expectSearchResultsCount(count: number)`
- pub `expectNoSearchResults` method L353-355 — `expectNoSearchResults()`
- pub `expectKanbanColumnsVisible` method L357-359 — `expectKanbanColumnsVisible()`
- pub `expectKanbanBoardVisible` method L361-363 — `expectKanbanBoardVisible()`
- pub `switchToTasksBoard` method L365-370 — `switchToTasksBoard()`
-  `setupTauriMocks` function L108-226 — `function setupTauriMocks(page: Page)`

