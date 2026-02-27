# Code Index

> Generated: 2026-02-25T14:29:41Z | 142 files | JavaScript, Rust, TypeScript

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
│   │   │   │   │   ├── strategy.rs
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
│   │   │   │   │   ├── strategy/
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
│   │       └── reassignment_test.rs
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

## Module Summaries

*Module summaries will be generated by the AI agent skill.*

## Key Symbols

### crates/metis-code-index/src/formatter.rs

- pub `format_index` function L18-31 — `( walk_result: &WalkResult, symbols_by_file: &BTreeMap<PathBuf, Vec<Symbol>>, ti...` — Format a code index as markdown.
- pub `write_index_file` function L244-252 — `( output_path: &Path, walk_result: &WalkResult, symbols_by_file: &BTreeMap<PathB...` — Write the formatted index to a file.
-  `write_header` function L34-49 — `(output: &mut String, walk_result: &WalkResult, timestamp: &str)` — Write the document header with metadata.
-  `write_project_structure` function L52-60 — `(output: &mut String, walk_result: &WalkResult)` — Write the project structure as an ASCII tree.
-  `write_module_summaries` function L63-66 — `(output: &mut String)` — Write the module summaries placeholder section.
-  `write_key_symbols` function L69-144 — `( output: &mut String, walk_result: &WalkResult, symbols_by_file: &BTreeMap<Path...` — Write key symbols grouped by file path.
-  `first_sentence` function L147-165 — `(doc: &str) -> Option<&str>` — Extract the first sentence from a doc comment.
-  `TreeNode` struct L168-172 — `{ name: String, is_dir: bool, children: Vec<TreeNode> }` — A node in the file tree for rendering.
-  `build_tree` function L175-193 — `(walk_result: &WalkResult) -> TreeNode` — Build a tree structure from the walk result.
-  `insert_path` function L196-224 — `(node: &mut TreeNode, components: &[&str], depth: usize)` — Recursively insert a path into the tree.
-  `render_tree` function L227-241 — `(output: &mut String, children: &[TreeNode], prefix: &str)` — Render the tree with box-drawing characters.
-  `tests` module L255-590 — `-` — and Key Symbols.
-  `make_walk_result` function L260-272 — `(files: Vec<(&str, Language)>) -> WalkResult` — and Key Symbols.
-  `make_symbol` function L274-286 — `( name: &str, kind: SymbolKind, file: &str, vis: Visibility, sig: Option<&str>, ...` — and Key Symbols.
-  `test_format_header` function L289-302 — `()` — and Key Symbols.
-  `test_format_project_structure` function L305-326 — `()` — and Key Symbols.
-  `test_format_module_summaries_placeholder` function L329-337 — `()` — and Key Symbols.
-  `test_format_key_symbols` function L340-372 — `()` — and Key Symbols.
-  `test_public_symbols_first` function L375-408 — `()` — and Key Symbols.
-  `test_tree_rendering` function L411-432 — `()` — and Key Symbols.
-  `test_empty_project` function L435-448 — `()` — and Key Symbols.
-  `test_files_without_symbols_skipped` function L451-477 — `()` — and Key Symbols.
-  `test_write_index_file` function L480-491 — `()` — and Key Symbols.
-  `test_format_with_doc_comment` function L494-515 — `()` — and Key Symbols.
-  `test_format_single_line_symbol` function L518-532 — `()` — and Key Symbols.
-  `test_first_sentence_extraction` function L535-544 — `()` — and Key Symbols.
-  `test_multi_language_project` function L547-589 — `()` — and Key Symbols.

### crates/metis-code-index/src/hasher.rs

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

### crates/metis-code-index/src/lang/go.rs

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

### crates/metis-code-index/src/lang/mod.rs

- pub `go` module L6 — `-` — Each language module provides extraction logic for symbols, imports,
- pub `python` module L7 — `-` — and call relationships from parsed syntax trees.
- pub `rust` module L8 — `-` — and call relationships from parsed syntax trees.
- pub `typescript` module L9 — `-` — and call relationships from parsed syntax trees.

### crates/metis-code-index/src/lang/python.rs

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

### crates/metis-code-index/src/lang/rust.rs

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

### crates/metis-code-index/src/lang/typescript.rs

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

### crates/metis-code-index/src/lib.rs

- pub `formatter` module L10 — `-` — This crate provides symbol extraction from source code using tree-sitter
- pub `hasher` module L11 — `-` — graph building, and file watching removed.
- pub `lang` module L12 — `-` — graph building, and file watching removed.
- pub `parser` module L13 — `-` — graph building, and file watching removed.
- pub `symbols` module L14 — `-` — graph building, and file watching removed.
- pub `walker` module L15 — `-` — graph building, and file watching removed.

### crates/metis-code-index/src/parser.rs

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

### crates/metis-code-index/src/symbols.rs

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

### crates/metis-code-index/src/walker.rs

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

### crates/metis-docs-cli/src/cli.rs

- pub `Cli` struct L15-22 — `{ verbose: u8, command: Commands }`
- pub `Commands` enum L25-50 — `Init | Sync | Create | Search | Transition | List | Status | Archive | Validate ...`
- pub `init_logging` function L53-65 — `(&self)`
- pub `execute` function L67-82 — `(&self) -> Result<()>`
-  `Cli` type L52-83 — `= Cli`
-  `tests` module L86-305 — `-`
-  `test_comprehensive_cli_workflow` function L97-304 — `()`

### crates/metis-docs-cli/src/commands/archive.rs

- pub `ArchiveCommand` struct L7-14 — `{ short_code: String, document_type: Option<String> }`
- pub `execute` function L17-55 — `(&self) -> Result<()>`
-  `ArchiveCommand` type L16-56 — `= ArchiveCommand`
-  `tests` module L59-175 — `-`
-  `test_archive_command_no_workspace` function L65-91 — `()`
-  `test_archive_document_not_found` function L94-125 — `()`
-  `test_archive_vision_document` function L128-174 — `()`

### crates/metis-docs-cli/src/commands/config.rs

- pub `ConfigCommand` struct L7-10 — `{ action: ConfigAction }`
- pub `ConfigAction` enum L13-33 — `Show | Set | Get`
- pub `execute` function L36-65 — `(&self) -> Result<()>`
-  `ConfigCommand` type L35-164 — `= ConfigCommand`
-  `show_config` function L67-91 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `set_config` function L93-143 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `get_config` function L145-163 — `( &self, config_repo: &mut metis_core::dal::database::configuration_repository::...`
-  `tests` module L167-302 — `-`
-  `test_config_show_default` function L173-202 — `()`
-  `test_config_set_streamlined_preset` function L205-238 — `()`
-  `test_config_set_invalid_preset` function L241-275 — `()`
-  `test_config_without_workspace` function L278-301 — `()`

### crates/metis-docs-cli/src/commands/create/adr.rs

- pub `create_new_adr` function L9-41 — `(title: &str) -> Result<()>` — Create a new ADR document with defaults and write to file
-  `tests` module L44-143 — `-`
-  `test_create_new_adr_no_workspace` function L52-68 — `()`
-  `test_create_new_adr_with_workspace` function L71-142 — `()`

### crates/metis-docs-cli/src/commands/create/initiative.rs

- pub `create_new_initiative` function L12-55 — `(title: &str, strategy_id: &str) -> Result<()>` — Create a new Initiative document with defaults and write to file
-  `find_strategy` function L58-113 — `( workspace_dir: &Path, strategy_id: &str, ) -> Result<(DocumentId, std::path::P...` — Find a strategy by ID and return its DocumentId and file path
-  `list_available_strategies` function L116-131 — `(strategies_dir: &Path) -> Result<Vec<String>>` — List available strategy IDs
-  `prompt_for_complexity` function L134-161 — `() -> Result<Complexity>` — Prompt user to select complexity level
-  `tests` module L164-298 — `-`
-  `test_create_new_initiative_no_workspace` function L171-187 — `()`
-  `test_find_strategy_not_found` function L190-224 — `()`
-  `test_list_available_strategies` function L227-242 — `()`
-  `test_create_initiative_flow_without_prompt` function L247-297 — `()`

### crates/metis-docs-cli/src/commands/create/mod.rs

- pub `CreateCommand` struct L11-14 — `{ document_type: CreateCommands }`
- pub `CreateCommands` enum L17-47 — `Strategy | Initiative | Task | Adr`
- pub `execute` function L50-72 — `(&self) -> Result<()>`
-  `adr` module L1 — `-`
-  `initiative` module L2 — `-`
-  `strategy` module L3 — `-`
-  `task` module L4 — `-`
-  `CreateCommand` type L49-73 — `= CreateCommand`

### crates/metis-docs-cli/src/commands/create/strategy.rs

- pub `create_new_strategy` function L11-56 — `(title: &str, vision_slug: Option<&str>) -> Result<()>` — Create a new Strategy document with defaults and write to file
-  `get_vision_document_id` function L59-87 — `(workspace_dir: &Path, vision_slug: &str) -> Result<DocumentId>` — Get the actual DocumentId by parsing the vision document
-  `tests` module L90-198 — `-`
-  `test_create_new_strategy_no_workspace` function L98-114 — `()`
-  `test_create_new_strategy_with_workspace` function L117-197 — `()`

### crates/metis-docs-cli/src/commands/create/task.rs

- pub `create_new_task` function L11-50 — `(title: &str, initiative_id: &str) -> Result<()>` — Create a new Task document with defaults and write to file
-  `find_initiative` function L53-90 — `( workspace_dir: &Path, initiative_id: &str, ) -> Result<(DocumentId, std::path:...` — Find an initiative by short code and return its DocumentId and file path
-  `find_strategy_for_initiative` function L93-113 — `(workspace_dir: &Path, initiative_id: &str) -> Result<String>` — Find the strategy ID that contains the given initiative
-  `tests` module L116-189 — `-`
-  `test_create_new_task_no_workspace` function L123-141 — `()`
-  `test_find_initiative_not_found` function L144-188 — `()`

### crates/metis-docs-cli/src/commands/index.rs

- pub `IndexCommand` struct L24-32 — `{ structure_only: bool, incremental: bool }` — parse with tree-sitter, extract symbols, and write the markdown index.
- pub `execute` function L35-115 — `(&self) -> Result<()>` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `IndexCommand` type L34-204 — `= IndexCommand` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `extract_incremental` function L118-203 — `( &self, walk_result: &metis_code_index::walker::WalkResult, hash_path: &Path, s...` — Perform incremental indexing: only re-parse changed files, use cached symbols for the rest.
-  `extract_symbols_for_language` function L207-225 — `( language: Language, parsed: &ParsedFile, file_path: &str, ) -> Result<Vec<Symb...` — Dispatch symbol extraction to the appropriate language extractor.
-  `extract_all_symbols` function L229-261 — `( walk_result: &metis_code_index::walker::WalkResult, ) -> (BTreeMap<PathBuf, Ve...` — Parse and extract symbols from all files in the walk result.
-  `tests` module L264-514 — `-` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_no_workspace` function L271-290 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_generates_file` function L293-335 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_index_structure_only` function L338-379 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_full_index_creates_hash_files` function L382-417 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_incremental_skips_unchanged` function L420-461 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.
-  `test_incremental_detects_changes` function L464-513 — `()` — parse with tree-sitter, extract symbols, and write the markdown index.

### crates/metis-docs-cli/src/commands/init.rs

- pub `InitCommand` struct L10-26 — `{ name: Option<String>, prefix: Option<String>, preset: Option<String>, strategi...`
- pub `execute` function L29-100 — `(&self) -> Result<()>`
-  `InitCommand` type L28-158 — `= InitCommand`
-  `determine_project_prefix` function L103-126 — `(&self, project_name: &str) -> String` — Determine the project prefix from command arguments or project name
-  `determine_flight_config` function L129-157 — `(&self) -> Result<FlightLevelConfig>` — Determine the flight level configuration based on command arguments
-  `tests` module L161-450 — `-`
-  `test_init_command_creates_workspace` function L167-237 — `()`
-  `test_init_command_workspace_already_exists` function L240-273 — `()`
-  `test_init_command_default_name` function L276-304 — `()`
-  `test_init_command_with_preset` function L307-346 — `()`
-  `test_init_command_with_custom_flags` function L349-383 — `()`
-  `test_init_command_default_streamlined` function L386-422 — `()`
-  `test_init_command_invalid_preset` function L425-449 — `()`

### crates/metis-docs-cli/src/commands/list.rs

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

### crates/metis-docs-cli/src/commands/mcp.rs

- pub `McpCommand` struct L5-9 — `{ log_level: String }`
- pub `execute` function L12-18 — `(&self) -> Result<()>`
-  `McpCommand` type L11-19 — `= McpCommand`

### crates/metis-docs-cli/src/commands/mod.rs

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

### crates/metis-docs-cli/src/commands/search.rs

- pub `SearchCommand` struct L10-21 — `{ query: String, limit: usize, format: OutputFormat }`
- pub `execute` function L33-77 — `(&self) -> Result<()>`
-  `SearchResultOutput` struct L25-30 — `{ code: String, title: String, doc_type: String }` — JSON-serializable search result for output
-  `SearchCommand` type L32-130 — `= SearchCommand`
-  `perform_search` function L79-82 — `(&self, app: &mut Application, query: &str) -> Result<Vec<Document>>`
-  `display_table` function L86-104 — `(&self, documents: &[Document])` — Display results as a human-readable table
-  `display_compact` function L108-112 — `(&self, documents: &[Document])` — Display results in compact format (one line per document)
-  `display_json` function L115-129 — `(&self, documents: &[Document])` — Display results as JSON array
-  `truncate` function L133-139 — `(s: &str, max_len: usize) -> String`
-  `tests` module L142-151 — `-`
-  `test_truncate` function L146-150 — `()`

### crates/metis-docs-cli/src/commands/status.rs

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

### crates/metis-docs-cli/src/commands/sync.rs

- pub `SyncCommand` struct L7 — `-`
- pub `execute` function L10-111 — `(&self) -> Result<()>`
-  `SyncCommand` type L9-112 — `= SyncCommand`
-  `tests` module L115-184 — `-`
-  `test_sync_command_no_workspace` function L122-142 — `()`
-  `test_sync_command_with_workspace` function L145-183 — `()`

### crates/metis-docs-cli/src/commands/transition.rs

- pub `TransitionCommand` struct L10-16 — `{ short_code: String, phase: Option<String> }`
- pub `execute` function L19-58 — `(&self) -> Result<()>`
-  `TransitionCommand` type L18-81 — `= TransitionCommand`
-  `parse_phase` function L60-80 — `(&self, phase_str: &str) -> Result<Phase>`
-  `tests` module L84-654 — `-`
-  `test_parse_phase` function L91-101 — `()`
-  `test_transition_command_no_workspace` function L104-130 — `()`
-  `test_find_document_not_found` function L133-164 — `()`
-  `test_vision_full_transition_sequence` function L167-223 — `()`
-  `test_strategy_full_transition_sequence` function L226-311 — `()`
-  `test_initiative_full_transition_sequence` function L314-389 — `()`
-  `test_task_full_transition_sequence` function L392-487 — `()`
-  `test_adr_full_transition_sequence` function L490-553 — `()`
-  `test_invalid_transitions` function L556-603 — `()`
-  `test_auto_transitions` function L606-653 — `()`

### crates/metis-docs-cli/src/commands/validate.rs

- pub `ValidateCommand` struct L7-10 — `{ file_path: PathBuf }`
- pub `execute` function L13-41 — `(&self) -> Result<()>`
-  `ValidateCommand` type L12-42 — `= ValidateCommand`
-  `tests` module L45-140 — `-`
-  `test_validate_command_missing_file` function L52-75 — `()`
-  `test_validate_command_valid_vision` function L78-109 — `()`
-  `test_validate_command_invalid_document` function L112-139 — `()`

### crates/metis-docs-cli/src/lib.rs

- pub `cli` module L5 — `-` — This library exposes CLI components for testing purposes.
- pub `commands` module L6 — `-` — This library exposes CLI components for testing purposes.
- pub `workspace` module L7 — `-` — This library exposes CLI components for testing purposes.

### crates/metis-docs-cli/src/main.rs

-  `cli` module L1 — `-`
-  `commands` module L2 — `-`
-  `utils` module L3 — `-`
-  `workspace` module L4 — `-`
-  `main` function L11-20 — `() -> Result<()>`

### crates/metis-docs-cli/src/workspace.rs

- pub `has_metis_vault` function L10-33 — `() -> (bool, Option<PathBuf>)` — Check if we're in a Metis workspace by walking up the directory tree
-  `tests` module L36-65 — `-`
-  `test_has_metis_vault_false_when_no_directory` function L41-47 — `()`
-  `test_has_metis_vault_true_when_valid` function L50-64 — `()`

### crates/metis-docs-cli/tests/comprehensive_functional_test.rs

- pub `init_workspace` function L13-37 — `( path: &PathBuf, name: Option<&str>, prefix: Option<&str>, preset: Option<&str>...` — These tests simulate real user command sequences through the CLI
- pub `verify_workspace` function L39-46 — `(path: &PathBuf) -> bool` — These tests simulate real user command sequences through the CLI
- pub `verify_config_toml` function L48-57 — `(path: &PathBuf, expected_prefix: &str) -> bool` — These tests simulate real user command sequences through the CLI
-  `cli_helpers` module L9-58 — `-` — Helper to run CLI commands programmatically
-  `test_complete_streamlined_workflow` function L61-137 — `()` — These tests simulate real user command sequences through the CLI
-  `test_complete_full_configuration_workflow` function L140-194 — `()` — These tests simulate real user command sequences through the CLI
-  `test_config_toml_persistence` function L197-266 — `()` — These tests simulate real user command sequences through the CLI
-  `test_custom_prefix_handling` function L269-311 — `()` — These tests simulate real user command sequences through the CLI

### crates/metis-docs-core/src/application/mod.rs

- pub `services` module L1 — `-`
- pub `Application` struct L9-11 — `{ database: Database }` — Application layer coordinator
- pub `new` function L15-17 — `(database: Database) -> Self` — Create a new application instance
- pub `with_database` function L20-30 — `(&mut self, f: F) -> R` — Execute a database operation
- pub `with_sync` function L33-44 — `(&mut self, f: F) -> R` — Execute a sync operation
- pub `sync_directory` function L53-111 — `( self, dir_path: P, ) -> Result<Vec<services::synchronization::SyncResult>>` — Convenience method to sync a directory
- pub `database` function L114-116 — `(&mut self) -> &mut Database` — Get access to the underlying database
-  `Application` type L13-117 — `= Application`

### crates/metis-docs-core/src/application/services/database.rs

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
- pub `get_all_id_filepath_pairs` function L110-129 — `(&mut self) -> Result<Vec<(String, String)>>` — Get all document IDs and their filepaths (useful for validation)
- pub `find_by_strategy_id` function L132-134 — `(&mut self, strategy_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to a strategy
- pub `find_by_initiative_id` function L137-139 — `(&mut self, initiative_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to an initiative
- pub `find_strategy_hierarchy` function L142-144 — `(&mut self, strategy_id: &str) -> Result<Vec<Document>>` — Get all documents in a strategy hierarchy (strategy + its initiatives + their tasks)
- pub `find_strategy_hierarchy_by_short_code` function L147-153 — `( &mut self, strategy_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in a strategy hierarchy by short code (strategy + its initiatives + their tasks)
- pub `find_initiative_hierarchy` function L156-158 — `(&mut self, initiative_id: &str) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy (initiative + its tasks)
- pub `find_initiative_hierarchy_by_short_code` function L161-167 — `( &mut self, initiative_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy by short code (initiative + its tasks)
- pub `generate_short_code` function L170-175 — `(&mut self, doc_type: &str) -> Result<String>` — Generate a short code for a document type (requires db_path)
- pub `set_counter_if_lower` function L179-183 — `(&mut self, _doc_type: &str, _min_value: u32) -> Result<bool>` — Set counter if the current value is lower than the provided value
-  `DatabaseService` type L10-184 — `= DatabaseService`
-  `tests` module L187-423 — `-`
-  `setup_service` function L191-194 — `() -> DatabaseService`
-  `create_test_document` function L196-214 — `() -> NewDocument`
-  `create_test_document_with_lineage` function L216-251 — `( id: &str, doc_type: &str, filepath: &str, strategy_id: Option<String>, initiat...`
-  `test_database_service_crud` function L254-287 — `()`
-  `test_database_service_relationships` function L290-335 — `()`
-  `test_lineage_queries` function L338-422 — `()`

### crates/metis-docs-core/src/application/services/document/creation.rs

- pub `DocumentCreationService` struct L15-19 — `{ workspace_dir: PathBuf, db_path: PathBuf, template_loader: TemplateLoader }` — Service for creating new documents with proper defaults and validation
- pub `DocumentCreationConfig` struct L23-31 — `{ title: String, description: Option<String>, parent_id: Option<DocumentId>, tag...` — Configuration for creating a new document
- pub `CreationResult` struct L35-40 — `{ document_id: DocumentId, document_type: DocumentType, file_path: PathBuf, shor...` — Result of document creation
- pub `new` function L44-53 — `(workspace_dir: P) -> Self` — Create a new document creation service for a workspace
- pub `create_vision` function L69-122 — `(&self, config: DocumentCreationConfig) -> Result<CreationResult>` — Create a new vision document
- pub `create_strategy` function L125-179 — `(&self, config: DocumentCreationConfig) -> Result<CreationResult>` — Create a new strategy document
- pub `create_initiative` function L182-190 — `( &self, config: DocumentCreationConfig, strategy_id: &str, ) -> Result<Creation...` — Create a new initiative document (legacy method)
- pub `create_initiative_with_config` function L193-339 — `( &self, config: DocumentCreationConfig, strategy_id: &str, flight_config: &Flig...` — Create a new initiative document with flight level configuration
- pub `create_task` function L342-356 — `( &self, config: DocumentCreationConfig, strategy_id: &str, initiative_id: &str,...` — Create a new task document (legacy method)
- pub `create_task_with_config` function L359-532 — `( &self, config: DocumentCreationConfig, strategy_id: &str, initiative_id: &str,...` — Create a new task document with flight level configuration
- pub `create_backlog_item` function L535-599 — `( &self, config: DocumentCreationConfig, ) -> Result<CreationResult>` — Create a new backlog item (task without parent)
- pub `create_adr` function L622-679 — `(&self, config: DocumentCreationConfig) -> Result<CreationResult>` — Create a new ADR document
-  `DocumentCreationService` type L42-706 — `= DocumentCreationService`
-  `generate_short_code` function L56-66 — `(&self, doc_type: &str) -> Result<String>` — Generate a short code for a document type
-  `determine_backlog_directory` function L602-619 — `(&self, tags: &[Tag]) -> PathBuf` — Determine the backlog directory based on tags
-  `get_next_adr_number` function L682-705 — `(&self) -> Result<u32>` — Get the next ADR number by examining existing ADRs
-  `tests` module L709-1244 — `-`
-  `test_create_vision_document` function L714-748 — `()`
-  `test_create_strategy_document` function L751-785 — `()`
-  `test_create_initiative_document` function L788-850 — `()`
-  `test_get_next_adr_number` function L853-870 — `()`
-  `setup_test_service_temp` function L874-891 — `() -> (DocumentCreationService, tempfile::TempDir)`
-  `test_create_initiative_full_configuration` function L894-948 — `()`
-  `test_create_initiative_streamlined_configuration` function L951-980 — `()`
-  `test_create_initiative_disabled_in_direct_configuration` function L983-1007 — `()`
-  `test_create_task_direct_configuration` function L1010-1039 — `()`
-  `test_create_vision_with_custom_template` function L1042-1097 — `()`
-  `test_create_task_with_custom_template` function L1100-1158 — `()`
-  `test_create_document_falls_back_to_embedded_template` function L1161-1200 — `()`
-  `test_invalid_custom_template_fails_gracefully` function L1203-1243 — `()`

### crates/metis-docs-core/src/application/services/document/deletion.rs

- pub `DeletionService` struct L11 — `-` — Service for recursive document deletion
- pub `new` function L20-22 — `() -> Self`
- pub `delete_document_recursive` function L25-78 — `(&self, filepath: &str) -> Result<DeletionResult>` — Delete a document and all its children recursively
- pub `DeletionResult` struct L133-136 — `{ deleted_files: Vec<String>, cleaned_directories: Vec<String> }` — Result of a document deletion operation
-  `DeletionService` type L13-17 — `impl Default for DeletionService`
-  `default` function L14-16 — `() -> Self`
-  `DeletionService` type L19-129 — `= DeletionService`
-  `remove_directory_recursive` function L81-128 — `( dir_path: &Path, deleted_files: &mut Vec<String>, cleaned_directories: &mut Ve...` — Recursively remove a directory and all its contents
-  `tests` module L139-540 — `-`
-  `setup_test_workspace` function L152-187 — `() -> (tempfile::TempDir, PathBuf)`
-  `test_delete_single_document_no_children` function L190-207 — `()`
-  `test_delete_strategy_with_folder` function L210-302 — `()`
-  `test_delete_initiative_with_folder` function L305-413 — `()`
-  `test_delete_nonexistent_document` function L416-430 — `()`
-  `test_delete_task_file_only` function L433-518 — `()`
-  `test_delete_document_no_folder` function L521-539 — `()`

### crates/metis-docs-core/src/application/services/document/discovery.rs

- pub `DocumentDiscoveryService` struct L11-13 — `{ workspace_dir: PathBuf }` — Service for discovering documents by ID across all document types
- pub `DocumentDiscoveryResult` struct L17-20 — `{ document_type: DocumentType, file_path: PathBuf }` — Result of document discovery
- pub `new` function L24-40 — `(workspace_dir: P) -> Self` — Create a new document discovery service for a workspace
- pub `find_document_by_short_code` function L43-63 — `( &self, short_code: &str, ) -> Result<DocumentDiscoveryResult>` — Find a document by its short code across all document types
- pub `find_document_by_id` function L66-87 — `(&self, document_id: &str) -> Result<DocumentDiscoveryResult>` — Find a document by its ID across all document types (legacy method)
- pub `find_document_of_type` function L90-324 — `( &self, document_id: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Find a document by its ID within a specific document type
- pub `find_document_by_id_and_type` function L327-333 — `( &self, document_id: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Find a document by its ID with a specific document type constraint
- pub `document_exists` function L336-338 — `(&self, document_id: &str) -> bool` — Check if a document with the given ID exists
- pub `find_all_documents_of_type` function L341-472 — `(&self, doc_type: DocumentType) -> Result<Vec<PathBuf>>` — Get all documents of a specific type
- pub `find_strategy_hierarchy_with_database` function L476-496 — `( &self, strategy_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<Do...` — Find all documents in a strategy hierarchy using database lineage queries
- pub `find_initiative_hierarchy_with_database` function L500-520 — `( &self, initiative_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<...` — Find all documents in an initiative hierarchy using database lineage queries
- pub `find_documents_by_strategy_with_database` function L523-543 — `( &self, strategy_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<Do...` — Find all documents belonging to a strategy using database lineage queries
- pub `find_documents_by_initiative_with_database` function L546-566 — `( &self, initiative_id: &str, db_service: &mut DatabaseService, ) -> Result<Vec<...` — Find all documents belonging to an initiative using database lineage queries
- pub `find_document_by_id_with_database` function L570-592 — `( &self, document_id: &str, db_service: &mut DatabaseService, ) -> Result<Docume...` — Fast document lookup using database instead of filesystem scanning
-  `DocumentDiscoveryService` type L22-779 — `= DocumentDiscoveryService`
-  `document_type_from_short_code` function L595-619 — `(&self, short_code: &str) -> Result<DocumentType>` — Extract document type from short code format (e.g., PROJ-V-0001 -> Vision)
-  `construct_path_from_short_code` function L622-649 — `( &self, short_code: &str, doc_type: DocumentType, ) -> Result<PathBuf>` — Construct file path from short code and document type
-  `find_initiative_path_by_short_code` function L652-709 — `(&self, short_code: &str) -> Result<PathBuf>` — Find initiative path by short code using database lookup
-  `find_task_path_by_short_code` function L712-778 — `(&self, short_code: &str) -> Result<PathBuf>` — Find task path by short code using database lookup
-  `tests` module L782-878 — `-`
-  `test_find_vision_document` function L788-821 — `()`
-  `test_document_not_found` function L824-834 — `()`
-  `test_find_all_documents_of_type` function L837-877 — `()`

### crates/metis-docs-core/src/application/services/document/mod.rs

- pub `creation` module L1 — `-`
- pub `deletion` module L2 — `-`
- pub `discovery` module L3 — `-`
- pub `validation` module L4 — `-`

### crates/metis-docs-core/src/application/services/document/validation.rs

- pub `DocumentValidationService` struct L7 — `-` — Service for validating documents and detecting their types
- pub `ValidationResult` struct L11-15 — `{ document_type: DocumentType, is_valid: bool, errors: Vec<String> }` — Result of document validation
- pub `new` function L19-21 — `() -> Self` — Create a new document validation service
- pub `validate_document` function L24-152 — `( &self, file_path: P, ) -> Result<ValidationResult>` — Validate a document file and detect its type
- pub `detect_document_type` function L155-166 — `(&self, file_path: P) -> Result<DocumentType>` — Validate a document and return just the document type (simpler interface)
- pub `validate_document_as_type` function L169-198 — `( &self, file_path: P, expected_type: DocumentType, ) -> Result<bool>` — Validate a document of a specific expected type
- pub `is_valid_document` function L201-206 — `(&self, file_path: P) -> bool` — Check if a document is valid without loading the full document
-  `DocumentValidationService` type L17-207 — `= DocumentValidationService`
-  `DocumentValidationService` type L209-213 — `impl Default for DocumentValidationService`
-  `default` function L210-212 — `() -> Self`
-  `tests` module L216-355 — `-`
-  `test_validate_valid_vision_document` function L222-253 — `()`
-  `test_validate_invalid_document` function L256-272 — `()`
-  `test_detect_document_type` function L275-304 — `()`
-  `test_validate_document_as_type` function L307-345 — `()`
-  `test_validate_nonexistent_file` function L348-354 — `()`

### crates/metis-docs-core/src/application/services/filesystem.rs

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

### crates/metis-docs-core/src/application/services/mod.rs

- pub `database` module L1 — `-`
- pub `document` module L2 — `-`
- pub `filesystem` module L3 — `-`
- pub `synchronization` module L4 — `-`
- pub `template` module L5 — `-`
- pub `workspace` module L6 — `-`

### crates/metis-docs-core/src/application/services/synchronization.rs

- pub `SyncService` struct L12-16 — `{ db_service: &'a mut DatabaseService, workspace_dir: Option<&'a Path>, db_path:...` — Synchronization service - bridges filesystem and database
- pub `new` function L19-25 — `(db_service: &'a mut DatabaseService) -> Self`
- pub `with_workspace_dir` function L31-36 — `(mut self, workspace_dir: &'a Path) -> Self` — Set the workspace directory for lineage extraction
- pub `import_from_file` function L63-90 — `(&mut self, file_path: P) -> Result<Document>` — Direction 1: File → DocumentObject → Database
- pub `export_to_file` function L94-114 — `(&mut self, filepath: &str) -> Result<()>` — Direction 2: Database → DocumentObject → File
- pub `sync_file` function L574-640 — `(&mut self, file_path: P) -> Result<SyncResult>` — Synchronize a single file between filesystem and database using directional methods
- pub `sync_directory` function L643-689 — `(&mut self, dir_path: P) -> Result<Vec<SyncResult>>` — Sync all markdown files in a directory
- pub `verify_sync` function L692-732 — `(&mut self, dir_path: P) -> Result<Vec<SyncIssue>>` — Verify database and filesystem are in sync
- pub `recover_counters_from_filesystem` function L775-896 — `( &self, dir_path: P, ) -> Result<std::collections::HashMap<String, u32>>` — Recover short code counters from filesystem by scanning all documents
- pub `SyncResult` enum L926-955 — `Imported | Updated | Deleted | UpToDate | NotFound | Error | Moved | Renumbered` — Result of synchronizing a single document
- pub `filepath` function L959-970 — `(&self) -> &str` — Get the filepath for this result
- pub `is_change` function L973-982 — `(&self) -> bool` — Check if this result represents a change
- pub `is_error` function L985-987 — `(&self) -> bool` — Check if this result represents an error
- pub `SyncIssue` enum L992-996 — `MissingFromDatabase | MissingFromFilesystem | OutOfSync` — Issues found during sync verification
-  `to_relative_path` function L40-48 — `(&self, absolute_path: P) -> String` — Convert absolute path to relative path (relative to workspace directory)
-  `to_absolute_path` function L52-59 — `(&self, relative_path: &str) -> std::path::PathBuf` — Convert relative path to absolute path (prepends workspace directory)
-  `domain_to_database_model` function L117-173 — `( &self, document_obj: &dyn DocumentTrait, filepath: &str, file_hash: String, up...` — Convert domain object to database model
-  `extract_lineage_from_path` function L177-243 — `( file_path: P, workspace_dir: &Path, ) -> (Option<DocumentId>, Option<DocumentI...` — Extract lineage information from file path
-  `is_backlog_path` function L247-264 — `(file_path: P, workspace_dir: &Path) -> bool` — Check if a file path is within the backlog directory
-  `extract_document_short_code` function L267-299 — `(file_path: P) -> Result<String>` — Extract document short code from file without keeping the document object around
-  `update_moved_document` function L302-314 — `( &mut self, existing_doc: &Document, new_file_path: P, ) -> Result<()>` — Update a document that has been moved to a new path
-  `resolve_short_code_collisions` function L318-398 — `( &mut self, dir_path: P, ) -> Result<Vec<SyncResult>>` — Detect and resolve short code collisions across all markdown files
-  `renumber_document` function L402-507 — `( &mut self, file_path: P, old_short_code: &str, ) -> Result<String>` — Renumber a single document to resolve short code collision
-  `update_sibling_references` function L510-571 — `( &mut self, file_path: P, old_short_code: &str, new_short_code: &str, ) -> Resu...` — Update cross-references in sibling documents (same directory)
-  `update_counters_from_filesystem` function L736-766 — `(&mut self, dir_path: P) -> Result<()>` — Update counters in database based on max values seen in filesystem
-  `is_valid_short_code_format` function L899-921 — `(short_code: &str) -> bool` — Validate short code format: PREFIX-TYPE-NNNN
-  `SyncResult` type L957-988 — `= SyncResult`
-  `tests` module L999-1246 — `-`
-  `setup_services` function L1004-1018 — `() -> (tempfile::TempDir, DatabaseService)`
-  `create_test_document_content` function L1020-1035 — `() -> String`
-  `test_import_from_file` function L1038-1057 — `()`
-  `test_sync_file_operations` function L1060-1140 — `()`
-  `test_sync_directory` function L1143-1198 — `()`
-  `test_is_backlog_path` function L1201-1245 — `()`

### crates/metis-docs-core/src/application/services/template.rs

- pub `vision` module L13-17 — `-` — 3.
- pub `CONTENT` variable L14 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L15-16 — `: &str` — 3.
- pub `strategy` module L19-23 — `-` — 3.
- pub `CONTENT` variable L20 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L21-22 — `: &str` — 3.
- pub `initiative` module L25-29 — `-` — 3.
- pub `CONTENT` variable L26 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L27-28 — `: &str` — 3.
- pub `task` module L31-35 — `-` — 3.
- pub `CONTENT` variable L32 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L33-34 — `: &str` — 3.
- pub `adr` module L37-41 — `-` — 3.
- pub `CONTENT` variable L38 — `: &str` — 3.
- pub `EXIT_CRITERIA` variable L39-40 — `: &str` — 3.
- pub `TemplateError` enum L46-55 — `IoError | ParseError | ValidationError | UnknownDocumentType` — Error type for template loading operations
- pub `TemplateType` enum L72-75 — `Content | ExitCriteria` — Template types that can be loaded
- pub `TemplateLoader` struct L92-97 — `{ project_path: Option<PathBuf>, global_path: PathBuf }` — Service for loading templates with fallback chain support.
- pub `new` function L103-112 — `(project_path: Option<PathBuf>) -> Self` — Create a new TemplateLoader with the given project workspace path.
- pub `for_workspace` function L115-117 — `(workspace_dir: P) -> Self` — Create a TemplateLoader for a specific workspace directory.
- pub `load_content_template` function L123-125 — `(&self, doc_type: &str) -> Result<String, TemplateError>` — Load a content template for the given document type.
- pub `load_exit_criteria_template` function L128-130 — `(&self, doc_type: &str) -> Result<String, TemplateError>` — Load an exit criteria template for the given document type.
- pub `validate_template` function L201-214 — `(&self, template: &str, doc_type: &str) -> Result<(), TemplateError>` — Validate a template by rendering it with sample data.
- pub `sample_context_for_type` function L219-266 — `(&self, doc_type: &str) -> Context` — Generate sample context values for validating templates.
- pub `has_custom_template` function L269-288 — `(&self, doc_type: &str, template_type: TemplateType) -> bool` — Check if custom templates exist for a document type.
- pub `template_source` function L291-314 — `(&self, doc_type: &str, template_type: TemplateType) -> TemplateSource` — Get the source of a template (for debugging/info).
- pub `TemplateSource` enum L319-326 — `Project | Global | Embedded` — Indicates where a template was loaded from.
-  `defaults` module L12-42 — `-` — Embedded default templates for each document type
-  `TemplateError` type L57-66 — `= TemplateError` — 3.
-  `fmt` function L58-65 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — 3.
-  `TemplateError` type L68 — `= TemplateError` — 3.
-  `TemplateType` type L77-84 — `= TemplateType` — 3.
-  `filename` function L78-83 — `(&self) -> &'static str` — 3.
-  `TemplateLoader` type L99-315 — `= TemplateLoader` — 3.
-  `load_template` function L133-173 — `( &self, doc_type: &str, template_type: TemplateType, ) -> Result<String, Templa...` — Load a template with the fallback chain.
-  `get_embedded_template` function L176-196 — `( &self, doc_type: &str, template_type: TemplateType, ) -> Result<String, Templa...` — Get the embedded default template for a document type.
-  `TemplateSource` type L328-336 — `= TemplateSource` — 3.
-  `fmt` function L329-335 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result` — 3.
-  `doc_type_letter` function L339-348 — `(doc_type: &str) -> char` — Helper to get the type letter for short codes
-  `tests` module L351-478 — `-` — 3.
-  `test_load_embedded_templates` function L356-372 — `()` — 3.
-  `test_unknown_document_type` function L375-379 — `()` — 3.
-  `test_project_template_override` function L382-401 — `()` — 3.
-  `test_template_validation_error` function L404-419 — `()` — 3.
-  `test_template_validation_missing_variable` function L422-437 — `()` — 3.
-  `test_sample_context_generation` function L440-458 — `()` — 3.
-  `test_has_custom_template` function L461-477 — `()` — 3.

### crates/metis-docs-core/src/application/services/workspace/archive.rs

- pub `ArchiveService` struct L12-15 — `{ workspace_dir: PathBuf, discovery_service: DocumentDiscoveryService }` — Service for archiving documents and managing the archived folder structure
- pub `ArchiveResult` struct L19-22 — `{ archived_documents: Vec<ArchivedDocument>, total_archived: usize }` — Result of archive operation
- pub `ArchivedDocument` struct L26-31 — `{ document_id: String, document_type: DocumentType, original_path: PathBuf, arch...` — Information about an archived document
- pub `new` function L96-117 — `(workspace_dir: P) -> Self` — Create a new archive service for a workspace
- pub `archive_document` function L120-202 — `( &self, document_id: &str, db_service: &mut DatabaseService, ) -> Result<Archiv...` — Archive a document and all its children using database lineage queries
- pub `is_document_archived` function L400-424 — `(&self, document_id: &str) -> Result<bool>` — Check if a document is already archived
- pub `get_archived_documents` function L427-437 — `(&self) -> Result<Vec<ArchivedDocument>>` — Get all archived documents
- pub `archive_document_by_short_code` function L499-586 — `( &self, short_code: &str, db_service: &mut DatabaseService, ) -> Result<Archive...` — Archive a document by its short code
- pub `is_document_archived_by_short_code` function L589-602 — `(&self, short_code: &str) -> Result<bool>` — Check if a document is archived by its short code
-  `ArchiveService` type L33-603 — `= ArchiveService`
-  `mark_as_archived_helper` function L37-93 — `( &self, file_path: &Path, doc_type: DocumentType, ) -> Result<()>` — Common helper for loading and marking a document as archived
-  `archive_single_file` function L205-251 — `( &self, file_path: &Path, doc_type: DocumentType, ) -> Result<ArchivedDocument>` — Archive a single file
-  `archive_directory` function L254-322 — `( &self, dir_path: &Path, doc_type: DocumentType, ) -> Result<ArchivedDocument>` — Archive a directory (for strategies and initiatives)
-  `merge_directory_contents` function L326-361 — `(&self, source_dir: &Path, target_dir: &Path) -> Result<()>` — Merge directory contents by moving files/subdirs from source to target
-  `get_document_id` function L364-397 — `(&self, file_path: &Path, doc_type: DocumentType) -> Result<String>` — Get document ID from a file
-  `scan_archived_directory` function L440-472 — `( &self, dir: &Path, results: &mut Vec<ArchivedDocument>, ) -> Result<()>` — Recursively scan archived directory for documents
-  `determine_document_type` function L475-496 — `(&self, file_path: &Path) -> Result<DocumentType>` — Determine document type from file path and content
-  `tests` module L606-860 — `-`
-  `test_archive_vision_document` function L615-667 — `()`
-  `test_archive_strategy_with_initiatives` function L670-747 — `()`
-  `test_get_archived_documents` function L750-798 — `()`
-  `test_is_document_archived` function L801-859 — `()`

### crates/metis-docs-core/src/application/services/workspace/detection.rs

- pub `WorkspaceDetectionService` struct L7 — `-` — Service for detecting and validating Metis workspaces
- pub `new` function L10-12 — `() -> Self`
- pub `find_workspace` function L15-33 — `(&self) -> Result<Option<PathBuf>>` — Find the nearest Metis workspace by traversing up the directory tree
- pub `find_workspace_from` function L36-54 — `(&self, start_path: &Path) -> Result<Option<PathBuf>>` — Find workspace starting from a specific directory
- pub `validate_workspace` function L58-64 — `(&self, metis_dir: &Path) -> Result<Option<PathBuf>>` — Validate that a directory is a proper Metis workspace
- pub `is_in_workspace` function L67-69 — `(&self, path: &Path) -> Result<bool>` — Check if a path is within a Metis workspace
- pub `get_workspace_root` function L72-80 — `(&self, path: &Path) -> Result<Option<PathBuf>>` — Get the workspace root for a given path
- pub `resolve_metis_dir` function L87-110 — `(&self, path: &Path) -> PathBuf` — Resolve a path to the `.metis` directory.
- pub `prepare_workspace` function L119-143 — `(&self, metis_dir: &Path) -> Result<Database>` — Prepare a workspace for use by ensuring database exists and is synced
- pub `find_and_prepare_workspace` function L147-154 — `(&self) -> Result<Option<(PathBuf, Database)>>` — Find workspace from current directory and prepare it for use
-  `WorkspaceDetectionService` type L9-155 — `= WorkspaceDetectionService`
-  `WorkspaceDetectionService` type L157-161 — `impl Default for WorkspaceDetectionService`
-  `default` function L158-160 — `() -> Self`
-  `tests` module L164-279 — `-`
-  `test_validate_workspace_missing_directory` function L170-177 — `()`
-  `test_validate_workspace_with_metis_directory` function L180-190 — `()`
-  `test_find_workspace_traversal` function L193-206 — `()`
-  `test_resolve_metis_dir_already_metis` function L209-218 — `()`
-  `test_resolve_metis_dir_from_project_root` function L221-237 — `()`
-  `test_resolve_metis_dir_no_config_toml` function L240-250 — `()`
-  `test_resolve_metis_dir_no_metis_subdir` function L253-262 — `()`
-  `test_get_workspace_root` function L265-278 — `()`

### crates/metis-docs-core/src/application/services/workspace/initialization.rs

- pub `WorkspaceInitializationService` struct L8 — `-` — Service for initializing new Metis workspaces
- pub `WorkspaceInitializationResult` struct L11-15 — `{ metis_dir: PathBuf, database_path: PathBuf, vision_path: PathBuf }` — Result of workspace initialization
- pub `initialize_workspace` function L20-25 — `( base_path: P, project_name: &str, ) -> Result<WorkspaceInitializationResult>` — Initialize a new Metis workspace at the given base path
- pub `initialize_workspace_with_prefix` function L28-128 — `( base_path: P, project_name: &str, prefix: Option<&str>, ) -> Result<WorkspaceI...` — Initialize a new Metis workspace with an optional custom prefix
- pub `is_workspace` function L162-165 — `(path: &Path) -> bool` — Check if a directory contains a valid Metis workspace
-  `WorkspaceInitializationService` type L17-166 — `= WorkspaceInitializationService`
-  `create_default_vision` function L131-158 — `(workspace_dir: &Path, title: &str) -> Result<PathBuf>` — Create a new Vision document with defaults and write to file
-  `tests` module L169-269 — `-`
-  `test_initialize_workspace` function L175-227 — `()`
-  `test_initialize_workspace_already_exists` function L230-254 — `()`
-  `test_is_workspace` function L257-268 — `()`

### crates/metis-docs-core/src/application/services/workspace/mod.rs

- pub `archive` module L1 — `-`
- pub `detection` module L2 — `-`
- pub `initialization` module L3 — `-`
- pub `reassignment` module L4 — `-`
- pub `recovery` module L5 — `-`
- pub `transition` module L6 — `-`

### crates/metis-docs-core/src/application/services/workspace/reassignment.rs

- pub `ReassignmentService` struct L9-11 — `{ workspace_dir: PathBuf }` — Service for reassigning tasks to different parent initiatives or the backlog
- pub `ReassignmentResult` struct L15-20 — `{ short_code: String, old_path: PathBuf, new_path: PathBuf, new_parent: Option<S...` — Result of reassignment operation
- pub `BacklogCategory` enum L24-28 — `Bug | Feature | TechDebt` — Backlog category for standalone tasks
- pub `from_str` function L31-38 — `(s: &str) -> Option<Self>`
- pub `directory_name` function L40-46 — `(&self) -> &'static str`
- pub `new` function L51-62 — `(workspace_dir: P) -> Self` — Create a new reassignment service for a workspace
- pub `reassign_to_initiative` function L65-90 — `( &self, short_code: &str, new_parent_id: &str, db_service: &mut DatabaseService...` — Reassign a task to a new parent initiative
- pub `reassign_to_backlog` function L93-126 — `( &self, short_code: &str, category: BacklogCategory, db_service: &mut DatabaseS...` — Move a task to the backlog
-  `BacklogCategory` type L30-47 — `= BacklogCategory`
-  `ReassignmentService` type L49-252 — `= ReassignmentService`
-  `find_task_by_short_code` function L129-148 — `( &self, short_code: &str, db_service: &mut DatabaseService, ) -> Result<Documen...` — Find a task by short code and validate it's a task
-  `find_and_validate_parent` function L151-181 — `( &self, parent_id: &str, db_service: &mut DatabaseService, ) -> Result<Document...` — Find and validate a parent initiative
-  `compute_initiative_task_path` function L184-211 — `( &self, parent_doc: &Document, source_doc: &Document, ) -> Result<PathBuf>` — Compute the destination path for a task under an initiative
-  `move_file` function L214-251 — `(&self, source: &Path, dest: &Path) -> Result<()>` — Move a file from source to destination
-  `tests` module L255-286 — `-`
-  `test_backlog_category_parsing` function L259-278 — `()`
-  `test_backlog_category_directory` function L281-285 — `()`

### crates/metis-docs-core/src/application/services/workspace/recovery.rs

- pub `ConfigurationRecoveryService` struct L9 — `-` — Service for recovering workspace configuration from filesystem
- pub `recover_configuration` function L29-95 — `( workspace_dir: P, db_path: P, ) -> Result<RecoveryReport>` — Recover configuration from config.toml file to database
- pub `sync_config_to_database` function L100-142 — `(workspace_dir: P, db_path: P) -> Result<bool>` — Sync config.toml to database (lightweight operation, safe to call frequently)
- pub `needs_recovery` function L202-222 — `(workspace_dir: &Path) -> bool` — Check if database needs recovery
- pub `RecoveryReport` struct L227-236 — `{ config_file_created: bool, prefix_synced: bool, flight_levels_synced: bool, co...` — Report of what was recovered during configuration recovery
- pub `had_recovery_actions` function L244-249 — `(&self) -> bool` — Check if any recovery actions were taken
-  `ConfigurationRecoveryService` type L11-223 — `= ConfigurationRecoveryService`
-  `create_config_from_database` function L145-167 — `(config_file_path: &Path, db_path: &Path) -> Result<ConfigFile>` — Create config.toml from existing database (migration path)
-  `recover_counters` function L170-195 — `( workspace_dir: &Path, config_repo: &mut ConfigurationRepository, ) -> Result<u...` — Recover counters from filesystem by scanning all documents
-  `RecoveryReport` type L238-250 — `= RecoveryReport`
-  `new` function L239-241 — `() -> Self`

### crates/metis-docs-core/src/application/services/workspace/transition.rs

- pub `PhaseTransitionService` struct L9-11 — `{ discovery_service: DocumentDiscoveryService }` — Service for managing document phase transitions
- pub `TransitionResult` struct L15-21 — `{ document_id: String, document_type: DocumentType, from_phase: Phase, to_phase:...` — Result of a phase transition
- pub `new` function L25-30 — `(workspace_dir: P) -> Self` — Create a new phase transition service for a workspace
- pub `transition_document` function L33-67 — `( &self, short_code: &str, target_phase: Phase, ) -> Result<TransitionResult>` — Transition a document to a specific phase
- pub `transition_to_next_phase` function L70-100 — `(&self, short_code: &str) -> Result<TransitionResult>` — Transition a document to the next phase in its natural sequence
- pub `is_valid_transition` function L267-275 — `( &self, doc_type: DocumentType, from_phase: Phase, to_phase: Phase, ) -> bool` — Check if a phase transition is valid without performing it
- pub `get_valid_transitions_for` function L278-284 — `( &self, doc_type: DocumentType, from_phase: Phase, ) -> Vec<Phase>` — Get all valid transitions for a document type and phase
-  `PhaseTransitionService` type L23-285 — `= PhaseTransitionService`
-  `get_current_phase` function L103-136 — `(&self, file_path: &Path, doc_type: DocumentType) -> Result<Phase>` — Get the current phase of a document
-  `perform_transition` function L139-226 — `( &self, file_path: &Path, doc_type: DocumentType, target_phase: Phase, ) -> Res...` — Perform the actual phase transition
-  `validate_transition` function L229-246 — `( &self, doc_type: DocumentType, from_phase: Phase, to_phase: Phase, ) -> Result...` — Validate that a phase transition is allowed
-  `get_valid_transitions` function L250-252 — `(&self, doc_type: DocumentType, from_phase: Phase) -> Vec<Phase>` — Get valid transitions from a given phase for a document type.
-  `get_next_phase` function L256-264 — `(&self, doc_type: DocumentType, current_phase: Phase) -> Result<Phase>` — Get the next phase in the natural sequence for a document type.
-  `tests` module L288-527 — `-`
-  `setup_test_workspace` function L298-313 — `() -> (tempfile::TempDir, PathBuf)`
-  `test_transition_vision_to_next_phase` function L316-342 — `()`
-  `test_transition_strategy_through_phases` function L345-392 — `()`
-  `test_transition_to_specific_phase` function L395-420 — `()`
-  `test_invalid_transition` function L423-450 — `()`
-  `test_get_valid_transitions` function L453-482 — `()`
-  `test_is_valid_transition` function L485-526 — `()`

### crates/metis-docs-core/src/constants.rs

- pub `METIS_DIR_NAME` variable L4 — `: &str` — Directory and file names
- pub `DATABASE_FILE_NAME` variable L5 — `: &str`
- pub `BACKUP_DATABASE_FILE_NAME` variable L6 — `: &str`
- pub `LOG_FILE_NAME` variable L7 — `: &str`
- pub `MARKDOWN_EXT` variable L10 — `: &str` — File extensions
- pub `YAML_EXT` variable L11 — `: &str`
- pub `JSON_EXT` variable L12 — `: &str`
- pub `VISION_DIR` variable L15 — `: &str` — Document directories
- pub `STRATEGY_DIR` variable L16 — `: &str`
- pub `INITIATIVE_DIR` variable L17 — `: &str`
- pub `TASK_DIR` variable L18 — `: &str`
- pub `ADR_DIR` variable L19 — `: &str`
- pub `ARCHIVED_DIR` variable L20 — `: &str`
- pub `VISION_TEMPLATE` variable L23 — `: &str` — Template names
- pub `STRATEGY_TEMPLATE` variable L24 — `: &str`
- pub `INITIATIVE_TEMPLATE` variable L25 — `: &str`
- pub `TASK_TEMPLATE` variable L26 — `: &str`
- pub `ADR_TEMPLATE` variable L27 — `: &str`
- pub `phases` module L30-57 — `-` — Document phases
- pub `VISION_DRAFT` variable L31 — `: &str`
- pub `VISION_REVIEW` variable L32 — `: &str`
- pub `VISION_PUBLISHED` variable L33 — `: &str`
- pub `STRATEGY_SHAPING` variable L35 — `: &str`
- pub `STRATEGY_DESIGN` variable L36 — `: &str`
- pub `STRATEGY_READY` variable L37 — `: &str`
- pub `STRATEGY_ACTIVE` variable L38 — `: &str`
- pub `STRATEGY_COMPLETED` variable L39 — `: &str`
- pub `INITIATIVE_DISCOVERY` variable L41 — `: &str`
- pub `INITIATIVE_DESIGN` variable L42 — `: &str`
- pub `INITIATIVE_READY` variable L43 — `: &str`
- pub `INITIATIVE_DECOMPOSE` variable L44 — `: &str`
- pub `INITIATIVE_ACTIVE` variable L45 — `: &str`
- pub `INITIATIVE_COMPLETED` variable L46 — `: &str`
- pub `TASK_TODO` variable L48 — `: &str`
- pub `TASK_ACTIVE` variable L49 — `: &str`
- pub `TASK_BLOCKED` variable L50 — `: &str`
- pub `TASK_COMPLETED` variable L51 — `: &str`
- pub `ADR_DRAFT` variable L53 — `: &str`
- pub `ADR_DISCUSSION` variable L54 — `: &str`
- pub `ADR_DECIDED` variable L55 — `: &str`
- pub `ADR_SUPERSEDED` variable L56 — `: &str`
- pub `complexity` module L60-66 — `-` — Complexity levels for initiatives
- pub `XS` variable L61 — `: &str`
- pub `S` variable L62 — `: &str`
- pub `M` variable L63 — `: &str`
- pub `L` variable L64 — `: &str`
- pub `XL` variable L65 — `: &str`
- pub `risk` module L69-73 — `-` — Risk levels for strategies
- pub `LOW` variable L70 — `: &str`
- pub `MEDIUM` variable L71 — `: &str`
- pub `HIGH` variable L72 — `: &str`
- pub `database` module L76-79 — `-` — Database settings
- pub `CONNECTION_TIMEOUT_SECS` variable L77 — `: u64`
- pub `MAX_RETRIES` variable L78 — `: u32`
- pub `filesystem` module L82-85 — `-` — File system settings
- pub `MAX_FILE_SIZE_BYTES` variable L83 — `: u64`
- pub `BACKUP_RETENTION_DAYS` variable L84 — `: u32`

### crates/metis-docs-core/src/dal/database/configuration_repository.rs

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

### crates/metis-docs-core/src/dal/database/mod.rs

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

### crates/metis-docs-core/src/dal/database/models.rs

- pub `Document` struct L17-33 — `{ filepath: String, id: String, title: String, document_type: String, created_at...`
- pub `DocumentRelationship` struct L38-43 — `{ child_id: String, parent_id: String, child_filepath: String, parent_filepath: ...`
- pub `DocumentTag` struct L48-51 — `{ document_filepath: String, tag: String }`
- pub `NewDocument` struct L56-72 — `{ filepath: String, id: String, title: String, document_type: String, created_at...`
- pub `DocumentSearch` struct L78-84 — `{ rowid: i32, document_filepath: String, content: Option<String>, title: Option<...`
- pub `NewDocumentSearch` struct L88-93 — `{ document_filepath: String, content: Option<String>, title: Option<String>, doc...`
- pub `Configuration` struct L98-102 — `{ key: String, value: String, updated_at: f64 }`

### crates/metis-docs-core/src/dal/database/repository.rs

- pub `DocumentRepository` struct L9-11 — `{ connection: SqliteConnection }` — Data access repository for document operations
- pub `new` function L14-16 — `(connection: SqliteConnection) -> Self`
- pub `create_document` function L19-27 — `(&mut self, doc: NewDocument) -> Result<Document>` — Insert a new document into the database
- pub `find_by_filepath` function L30-38 — `(&mut self, file_path: &str) -> Result<Option<Document>>` — Find a document by its filepath
- pub `find_by_id` function L41-49 — `(&mut self, document_id: &str) -> Result<Option<Document>>` — Find a document by its ID
- pub `update_document` function L52-60 — `(&mut self, file_path: &str, doc: &Document) -> Result<Document>` — Update an existing document
- pub `delete_document` function L63-71 — `(&mut self, file_path: &str) -> Result<bool>` — Delete a document and all its relationships
- pub `find_children` function L74-84 — `(&mut self, parent_document_id: &str) -> Result<Vec<Document>>` — Find all children of a document
- pub `find_parent` function L87-98 — `(&mut self, child_document_id: &str) -> Result<Option<Document>>` — Find the parent of a document
- pub `create_relationship` function L101-110 — `(&mut self, relationship: DocumentRelationship) -> Result<()>` — Create a parent-child relationship
- pub `search_documents` function L113-125 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search documents using FTS
- pub `search_documents_unarchived` function L128-140 — `(&mut self, query: &str) -> Result<Vec<Document>>` — Search non-archived documents using FTS
- pub `find_by_type` function L143-151 — `(&mut self, doc_type: &str) -> Result<Vec<Document>>` — Get all documents of a specific type
- pub `find_by_type_unarchived` function L154-163 — `(&mut self, doc_type: &str) -> Result<Vec<Document>>` — Get all non-archived documents of a specific type
- pub `find_by_tag` function L166-176 — `(&mut self, tag_name: &str) -> Result<Vec<Document>>` — Get documents with specific tags
- pub `find_by_phase` function L179-187 — `(&mut self, phase_name: &str) -> Result<Vec<Document>>` — Get documents in a specific phase
- pub `find_by_type_and_phase` function L190-203 — `( &mut self, doc_type: &str, phase_name: &str, ) -> Result<Vec<Document>>` — Get documents by type and phase
- pub `find_by_strategy_id` function L206-214 — `(&mut self, strategy_document_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to a strategy
- pub `find_by_initiative_id` function L217-225 — `(&mut self, initiative_document_id: &str) -> Result<Vec<Document>>` — Get all documents belonging to an initiative
- pub `get_tags_for_document` function L228-236 — `(&mut self, doc_filepath: &str) -> Result<Vec<String>>` — Get all tags for a specific document by filepath
- pub `find_strategy_hierarchy` function L239-250 — `(&mut self, strategy_document_id: &str) -> Result<Vec<Document>>` — Get all documents in a strategy hierarchy (strategy + its initiatives + their tasks)
- pub `find_strategy_hierarchy_by_short_code` function L253-268 — `( &mut self, strategy_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in a strategy hierarchy by short code (strategy + its initiatives + their tasks)
- pub `find_initiative_hierarchy` function L271-285 — `( &mut self, initiative_document_id: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy (initiative + its tasks)
- pub `find_initiative_hierarchy_by_short_code` function L288-303 — `( &mut self, initiative_short_code: &str, ) -> Result<Vec<Document>>` — Get all documents in an initiative hierarchy by short code (initiative + its tasks)
- pub `generate_short_code` function L306-315 — `(&mut self, doc_type: &str, db_path: &str) -> Result<String>` — Generate a short code for a document type using the database configuration
- pub `find_by_short_code` function L318-326 — `(&mut self, code: &str) -> Result<Option<Document>>` — Find a document by its short code
- pub `resolve_short_code_to_document_id` function L329-337 — `(&mut self, code: &str) -> Result<String>` — Resolve short code to document ID for parent relationships
- pub `resolve_short_code_to_filepath` function L340-348 — `(&mut self, code: &str) -> Result<String>` — Resolve short code to file path for file operations
-  `DocumentRepository` type L13-349 — `= DocumentRepository`
-  `tests` module L352-605 — `-`
-  `setup_test_repository` function L356-359 — `() -> DocumentRepository`
-  `create_test_document` function L361-379 — `() -> NewDocument`
-  `test_create_and_find_document` function L382-407 — `()`
-  `test_update_document` function L410-428 — `()`
-  `test_delete_document` function L431-455 — `()`
-  `test_document_relationships` function L458-526 — `()`
-  `test_find_by_type` function L529-589 — `()`
-  `test_document_not_found` function L592-604 — `()`

### crates/metis-docs-core/src/dal/mod.rs

- pub `database` module L1 — `-`
- pub `filesystem` module L2 — `-`

### crates/metis-docs-core/src/domain/configuration.rs

- pub `FlightLevelConfig` struct L7-12 — `{ strategies_enabled: bool, initiatives_enabled: bool }` — Flight level configuration defining which levels are enabled
- pub `new` function L16-31 — `( strategies_enabled: bool, initiatives_enabled: bool, ) -> Result<Self, Configu...` — Create a new configuration
- pub `full` function L34-39 — `() -> Self` — Full flight levels: Vision → Strategy → Initiative → Task
- pub `streamlined` function L42-47 — `() -> Self` — Streamlined flight levels: Vision → Initiative → Task
- pub `direct` function L50-55 — `() -> Self` — Direct flight levels: Vision → Task
- pub `is_document_type_allowed` function L58-65 — `(&self, doc_type: DocumentType) -> bool` — Check if a document type is allowed in this configuration
- pub `get_parent_type` function L68-87 — `(&self, doc_type: DocumentType) -> Option<DocumentType>` — Get the parent document type for a given document type in this configuration
- pub `preset_name` function L90-97 — `(&self) -> &'static str` — Get the configuration name/preset
- pub `enabled_document_types` function L100-115 — `(&self) -> Vec<DocumentType>` — Get enabled document types in hierarchical order
- pub `hierarchy_display` function L118-132 — `(&self) -> String` — Get the hierarchy display string
- pub `ConfigurationError` enum L149-154 — `InvalidConfiguration | SerializationError | InvalidValue | MissingConfiguration` — Configuration validation errors
- pub `ConfigFile` struct L177-180 — `{ project: ProjectConfig, flight_levels: FlightLevelConfig }` — Configuration file structure that persists to .metis/config.toml
- pub `ProjectConfig` struct L183-185 — `{ prefix: String }`
- pub `new` function L189-204 — `( prefix: String, flight_levels: FlightLevelConfig, ) -> Result<Self, Configurat...` — Create a new configuration with defaults
- pub `load` function L207-215 — `(path: P) -> Result<Self, ConfigurationError>` — Load configuration from a TOML file
- pub `save` function L218-228 — `(&self, path: P) -> Result<(), ConfigurationError>` — Save configuration to a TOML file
- pub `default_with_prefix` function L231-233 — `(prefix: String) -> Result<Self, ConfigurationError>` — Create default configuration with given prefix
- pub `prefix` function L236-238 — `(&self) -> &str` — Get the project prefix
- pub `flight_levels` function L241-243 — `(&self) -> &FlightLevelConfig` — Get the flight level configuration
-  `FlightLevelConfig` type L14-133 — `= FlightLevelConfig`
-  `FlightLevelConfig` type L135-139 — `impl Default for FlightLevelConfig`
-  `default` function L136-138 — `() -> Self`
-  `FlightLevelConfig` type L141-145 — `= FlightLevelConfig`
-  `fmt` function L142-144 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `ConfigurationError` type L156-171 — `= ConfigurationError`
-  `fmt` function L157-170 — `(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result`
-  `ConfigurationError` type L173 — `= ConfigurationError`
-  `ConfigFile` type L187-244 — `= ConfigFile`
-  `ConfigFile` type L246-255 — `impl Default for ConfigFile`
-  `default` function L247-254 — `() -> Self`
-  `tests` module L258-494 — `-`
-  `test_preset_configurations` function L262-277 — `()`
-  `test_configuration_validation` function L280-288 — `()`
-  `test_document_type_allowed` function L291-312 — `()`
-  `test_parent_type_resolution` function L315-347 — `()`
-  `test_enabled_document_types` function L350-382 — `()`
-  `test_hierarchy_display` function L385-398 — `()`
-  `test_serialization` function L401-406 — `()`
-  `test_config_file_creation` function L409-413 — `()`
-  `test_config_file_validation` function L416-431 — `()`
-  `test_config_file_save_and_load` function L434-454 — `()`
-  `test_config_file_toml_format` function L457-479 — `()`
-  `test_config_file_default` function L482-486 — `()`
-  `test_config_file_default_with_prefix` function L489-493 — `()`

### crates/metis-docs-core/src/domain/documents/adr/mod.rs

- pub `Adr` struct L13-18 — `{ core: super::traits::DocumentCore, number: u32, decision_maker: String, decisi...` — An ADR (Architecture Decision Record) documents architectural decisions
- pub `new` function L23-46 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create a new ADR document with content rendered from template
- pub `new_with_template` function L50-97 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create a new ADR document with a custom template
- pub `from_parts` function L101-128 — `( number: u32, title: String, decision_maker: String, decision_date: Option<chro...` — Create an ADR document from existing data (used when loading from file)
- pub `number` function L130-132 — `(&self) -> u32`
- pub `decision_maker` function L134-136 — `(&self) -> &str`
- pub `decision_date` function L138-140 — `(&self) -> Option<chrono::DateTime<Utc>>`
- pub `from_file` function L143-149 — `(path: P) -> Result<Self, DocumentValidationError>` — Create an ADR document by reading and parsing a file
- pub `from_content` function L152-223 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create an ADR document from raw file content string
- pub `to_file` function L248-253 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the ADR document to a file
- pub `to_content` function L256-325 — `(&self) -> Result<String, DocumentValidationError>` — Convert the ADR document to its markdown string representation using templates
-  `Adr` type L20-326 — `= Adr`
-  `next_phase_in_sequence` function L226-235 — `(current: Phase) -> Option<Phase>` — Get the next phase in the ADR sequence
-  `update_phase_tag` function L238-245 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Adr` type L328-456 — `impl Document for Adr`
-  `id` function L330-333 — `(&self) -> DocumentId` — ADRs have special ID format: number-slug
-  `document_type` function L335-337 — `(&self) -> DocumentType`
-  `title` function L339-341 — `(&self) -> &str`
-  `metadata` function L343-345 — `(&self) -> &DocumentMetadata`
-  `content` function L347-349 — `(&self) -> &DocumentContent`
-  `core` function L351-353 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L355-362 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L364-366 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L368-370 — `(&self) -> &[DocumentId]`
-  `validate` function L372-393 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L395-400 — `(&self) -> bool`
-  `template` function L402-409 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L411-413 — `(&self) -> &'static str`
-  `content_template` function L415-417 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L419-421 — `(&self) -> &'static str`
-  `transition_phase` function L423-451 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L453-455 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L459-817 — `-`
-  `test_adr_from_content` function L464-510 — `()`
-  `test_adr_special_id_format` function L513-528 — `()`
-  `test_adr_invalid_level` function L531-559 — `()`
-  `test_adr_validation` function L562-591 — `()`
-  `test_adr_cannot_be_blocked` function L594-609 — `()`
-  `test_adr_phase_transitions` function L612-643 — `()`
-  `test_adr_number_formatting` function L646-686 — `()`
-  `test_adr_transition_phase_auto` function L689-721 — `()`
-  `test_adr_transition_phase_explicit` function L724-746 — `()`
-  `test_adr_transition_phase_invalid` function L749-775 — `()`
-  `test_adr_update_section` function L778-816 — `()`

### crates/metis-docs-core/src/domain/documents/content.rs

- pub `DocumentContent` struct L5-10 — `{ body: String, acceptance_criteria: Option<String> }` — Document content containing the main body and acceptance criteria
- pub `new` function L14-19 — `(body: &str) -> Self` — Create new content from body text
- pub `with_acceptance_criteria` function L22-27 — `(body: &str, acceptance_criteria: &str) -> Self` — Create content with both body and acceptance criteria
- pub `from_markdown` function L30-42 — `(content: &str) -> Self` — Parse content from markdown, separating main content from acceptance criteria
- pub `full_content` function L45-50 — `(&self) -> String` — Get the full content including acceptance criteria
- pub `has_acceptance_criteria` function L53-55 — `(&self) -> bool` — Check if acceptance criteria are present
-  `DocumentContent` type L12-56 — `= DocumentContent`

### crates/metis-docs-core/src/domain/documents/factory.rs

- pub `DocumentFactory` struct L15 — `-` — Factory for creating documents from files
- pub `from_file` function L20-53 — `( path: P, ) -> Result<Box<dyn Document>, DocumentValidationError>` — Create a document from a file path
- pub `from_content` function L56-84 — `( raw_content: &str, _filepath: &str, ) -> Result<Box<dyn Document>, DocumentVal...` — Create a document from raw content string
-  `DocumentFactory` type L17-123 — `= DocumentFactory`
-  `extract_document_type` function L87-122 — `(raw_content: &str) -> Result<DocumentType, DocumentValidationError>` — Extract document type from frontmatter
-  `tests` module L126-183 — `-`
-  `test_extract_document_type` function L130-155 — `()`
-  `test_extract_document_type_missing` function L158-168 — `()`
-  `test_extract_document_type_invalid` function L171-182 — `()`

### crates/metis-docs-core/src/domain/documents/helpers.rs

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

### crates/metis-docs-core/src/domain/documents/initiative/mod.rs

- pub `Complexity` enum L13-19 — `XS | S | M | L | XL` — Complexity level for initiatives
- pub `Initiative` struct L53-56 — `{ core: super::traits::DocumentCore, estimated_complexity: Complexity }` — An Initiative document represents a concrete implementation approach for a strategy
- pub `new` function L61-84 — `( title: String, parent_id: Option<DocumentId>, // Usually a Strategy strategy_i...` — Create a new Initiative document with content rendered from template
- pub `new_with_template` function L88-135 — `( title: String, parent_id: Option<DocumentId>, strategy_id: Option<DocumentId>,...` — Create a new Initiative document with a custom template
- pub `from_parts` function L139-167 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create an Initiative document from existing data (used when loading from file)
- pub `estimated_complexity` function L169-171 — `(&self) -> Complexity`
- pub `from_file` function L198-204 — `(path: P) -> Result<Self, DocumentValidationError>` — Create an Initiative document by reading and parsing a file
- pub `from_content` function L207-287 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create an Initiative document from raw file content string
- pub `to_file` function L290-295 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Initiative document to a file
- pub `to_content` function L298-380 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Initiative document to its markdown string representation using templates
-  `Complexity` type L21-31 — `= Complexity`
-  `fmt` function L22-30 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`
-  `Complexity` type L33-49 — `= Complexity`
-  `Err` type L34 — `= DocumentValidationError`
-  `from_str` function L36-48 — `(s: &str) -> Result<Self, Self::Err>`
-  `Initiative` type L58-381 — `= Initiative`
-  `next_phase_in_sequence` function L174-185 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Initiative sequence
-  `update_phase_tag` function L188-195 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Initiative` type L383-502 — `impl Document for Initiative`
-  `document_type` function L386-388 — `(&self) -> DocumentType`
-  `title` function L390-392 — `(&self) -> &str`
-  `metadata` function L394-396 — `(&self) -> &DocumentMetadata`
-  `content` function L398-400 — `(&self) -> &DocumentContent`
-  `core` function L402-404 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L406-413 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L415-417 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L419-421 — `(&self) -> &[DocumentId]`
-  `validate` function L423-439 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L441-446 — `(&self) -> bool`
-  `template` function L448-455 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L457-459 — `(&self) -> &'static str`
-  `content_template` function L461-463 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L465-467 — `(&self) -> &'static str`
-  `transition_phase` function L469-497 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L499-501 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L505-709 — `-`
-  `test_initiative_from_content` function L510-575 — `()`
-  `test_initiative_complexity_parsing` function L578-587 — `()`
-  `test_initiative_invalid_level` function L590-616 — `()`
-  `test_initiative_validation` function L619-665 — `()`
-  `test_initiative_phase_transitions` function L668-699 — `()`
-  `test_complexity_display` function L702-708 — `()`

### crates/metis-docs-core/src/domain/documents/metadata.rs

- pub `DocumentMetadata` struct L6-11 — `{ created_at: DateTime<Utc>, updated_at: DateTime<Utc>, exit_criteria_met: bool,...` — Document metadata containing timestamps and other document properties
- pub `new` function L15-23 — `(short_code: String) -> Self` — Create new metadata with current timestamps and short code
- pub `from_frontmatter` function L26-38 — `( created_at: DateTime<Utc>, updated_at: DateTime<Utc>, exit_criteria_met: bool,...` — Create metadata from parsed frontmatter data
- pub `update` function L41-43 — `(&mut self)` — Update the updated_at timestamp to now
- pub `mark_exit_criteria_met` function L46-49 — `(&mut self)` — Mark exit criteria as met and update timestamp
-  `DocumentMetadata` type L13-50 — `= DocumentMetadata`

### crates/metis-docs-core/src/domain/documents/mod.rs

- pub `content` module L1 — `-`
- pub `factory` module L2 — `-`
- pub `helpers` module L3 — `-`
- pub `metadata` module L4 — `-`
- pub `traits` module L6 — `-` — Document domain module
- pub `types` module L7 — `-`
- pub `adr` module L10 — `-`
- pub `initiative` module L11 — `-`
- pub `strategy` module L12 — `-`
- pub `task` module L13 — `-`
- pub `vision` module L14 — `-`

### crates/metis-docs-core/src/domain/documents/strategy/mod.rs

- pub `RiskLevel` enum L13-18 — `Low | Medium | High | Critical` — Risk level for strategies
- pub `Strategy` struct L50-54 — `{ core: super::traits::DocumentCore, risk_level: RiskLevel, stakeholders: Vec<St...` — A Strategy document defines high-level approaches to achieve vision goals
- pub `new` function L59-82 — `( title: String, parent_id: Option<DocumentId>, // Usually a Vision blocked_by: ...` — Create a new Strategy document with content rendered from template
- pub `new_with_template` function L86-134 — `( title: String, parent_id: Option<DocumentId>, blocked_by: Vec<DocumentId>, tag...` — Create a new Strategy document with a custom template
- pub `from_parts` function L138-167 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create a Strategy document from existing data (used when loading from file)
- pub `risk_level` function L169-171 — `(&self) -> RiskLevel`
- pub `stakeholders` function L173-175 — `(&self) -> &[String]`
- pub `from_file` function L201-207 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Strategy document by reading and parsing a file
- pub `from_content` function L210-287 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Strategy document from raw file content string
- pub `to_file` function L290-295 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Strategy document to a file
- pub `to_content` function L298-380 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Strategy document to its markdown string representation using templates
-  `RiskLevel` type L20-29 — `= RiskLevel`
-  `fmt` function L21-28 — `(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result`
-  `RiskLevel` type L31-46 — `= RiskLevel`
-  `Err` type L32 — `= DocumentValidationError`
-  `from_str` function L34-45 — `(s: &str) -> Result<Self, Self::Err>`
-  `Strategy` type L56-383 — `= Strategy`
-  `next_phase_in_sequence` function L178-188 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Strategy sequence
-  `update_phase_tag` function L191-198 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Strategy` type L385-503 — `impl Document for Strategy`
-  `document_type` function L388-390 — `(&self) -> DocumentType`
-  `title` function L392-394 — `(&self) -> &str`
-  `metadata` function L396-398 — `(&self) -> &DocumentMetadata`
-  `content` function L400-402 — `(&self) -> &DocumentContent`
-  `core` function L404-406 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L408-415 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L417-419 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L421-423 — `(&self) -> &[DocumentId]`
-  `validate` function L425-440 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L442-447 — `(&self) -> bool`
-  `template` function L449-456 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L458-460 — `(&self) -> &'static str`
-  `content_template` function L462-464 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L466-468 — `(&self) -> &'static str`
-  `transition_phase` function L470-498 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L500-502 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L506-672 — `-`
-  `test_strategy_new` function L512-547 — `()`
-  `test_strategy_from_content` function L550-608 — `()`
-  `test_strategy_phase_transitions` function L611-638 — `()`
-  `test_strategy_validation` function L641-671 — `()`

### crates/metis-docs-core/src/domain/documents/task/mod.rs

- pub `Task` struct L13-15 — `{ core: super::traits::DocumentCore }` — A Task document represents a concrete, actionable piece of work
- pub `new` function L20-45 — `( title: String, parent_id: Option<DocumentId>, // Usually an Initiative parent_...` — Create a new Task document with content rendered from template
- pub `new_with_template` function L49-97 — `( title: String, parent_id: Option<DocumentId>, parent_title: Option<String>, st...` — Create a new Task document with a custom template
- pub `from_parts` function L101-125 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Create a Task document from existing data (used when loading from file)
- pub `from_file` function L128-134 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Task document by reading and parsing a file
- pub `from_content` function L137-215 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Task document from raw file content string
- pub `to_file` function L241-246 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Task document to a file
- pub `to_content` function L249-327 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Task document to its markdown string representation using templates
-  `Task` type L17-328 — `= Task`
-  `next_phase_in_sequence` function L218-228 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Task sequence
-  `update_phase_tag` function L231-238 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Task` type L330-467 — `impl Document for Task`
-  `document_type` function L333-335 — `(&self) -> DocumentType`
-  `title` function L337-339 — `(&self) -> &str`
-  `metadata` function L341-343 — `(&self) -> &DocumentMetadata`
-  `content` function L345-347 — `(&self) -> &DocumentContent`
-  `core` function L349-351 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L353-360 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L362-364 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L366-368 — `(&self) -> &[DocumentId]`
-  `validate` function L370-404 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L406-411 — `(&self) -> bool`
-  `template` function L413-420 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L422-424 — `(&self) -> &'static str`
-  `content_template` function L426-428 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L430-432 — `(&self) -> &'static str`
-  `transition_phase` function L434-462 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L464-466 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L470-830 — `-`
-  `test_task_from_content` function L476-532 — `()`
-  `test_task_invalid_level` function L535-560 — `()`
-  `test_task_validation` function L563-594 — `()`
-  `test_task_blocked_validation` function L597-629 — `()`
-  `test_task_phase_transitions` function L632-650 — `()`
-  `test_task_active_phase_transitions` function L653-670 — `()`
-  `test_task_blocked_phase_transitions` function L673-690 — `()`
-  `test_task_transition_phase_auto` function L693-721 — `()`
-  `test_task_transition_phase_blocking` function L724-753 — `()`
-  `test_task_transition_phase_invalid` function L756-783 — `()`
-  `test_task_update_section` function L786-829 — `()`

### crates/metis-docs-core/src/domain/documents/traits.rs

- pub `Document` interface L7-170 — `{ fn id(), fn document_type(), fn title(), fn metadata(), fn content(), fn core(...` — Core document trait that all document types must implement
- pub `DocumentTemplate` struct L173-178 — `{ frontmatter: &'static str, content: &'static str, acceptance_criteria: &'stati...` — Template information for a document
- pub `DocumentCore` struct L182-192 — `{ title: String, metadata: DocumentMetadata, content: DocumentContent, parent_id...` — Common document data that all document types share
- pub `DocumentValidationError` enum L196-214 — `InvalidTitle | InvalidParent | InvalidPhaseTransition | MissingRequiredField | I...` — Validation errors for documents
-  `id` function L9-11 — `(&self) -> DocumentId` — Get the unique identifier for this document (derived from title)
-  `tags` function L29-31 — `(&self) -> &[Tag]` — Get the document tags
-  `phase` function L34-43 — `(&self) -> Result<Phase, DocumentValidationError>` — Get the current phase of the document (parsed from tags)
-  `update_section` function L55-128 — `( &mut self, content: &str, heading: &str, append: bool, ) -> Result<(), Documen...` — Update a specific section (H2 heading) in the document content
-  `update_content_body` function L131-137 — `(&mut self, new_body: String) -> Result<(), DocumentValidationError>` — Helper method for update_section to actually mutate the content
-  `archived` function L143-145 — `(&self) -> bool` — Check if this document is archived

### crates/metis-docs-core/src/domain/documents/types.rs

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
- pub `DocumentType` enum L153-159 — `Vision | Strategy | Initiative | Task | Adr` — Document type enumeration
- pub `valid_transitions_from` function L192-228 — `(&self, from_phase: Phase) -> Vec<Phase>` — Get valid transitions from a given phase for this document type.
- pub `can_transition` function L231-233 — `(&self, from: Phase, to: Phase) -> bool` — Check if a transition from one phase to another is valid for this document type.
- pub `next_phase` function L237-239 — `(&self, current: Phase) -> Option<Phase>` — Get the next phase in the natural sequence for this document type.
- pub `phase_sequence` function L242-270 — `(&self) -> Vec<Phase>` — Get the ordered phase sequence for this document type (for display purposes).
- pub `Phase` enum L275-299 — `Draft | Review | Published | Discussion | Decided | Superseded | Backlog | Todo ...` — Document phase/status
- pub `Tag` enum L326-329 — `Phase | Label` — Document tag that can be either a phase or a string
- pub `to_str` function L399-410 — `(&self) -> String` — Convert tag back to its string representation (reverse of from_str)
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
-  `DocumentType` type L188-271 — `= DocumentType`
-  `Phase` type L301-322 — `= Phase`
-  `fmt` function L302-321 — `(&self, f: &mut fmt::Formatter) -> fmt::Result`
-  `Tag` type L331-344 — `= Tag`
-  `fmt` function L332-343 — `(&self, f: &mut fmt::Formatter) -> fmt::Result`
-  `Tag` type L346-350 — `= Tag`
-  `from` function L347-349 — `(phase: Phase) -> Self`
-  `Tag` type L352-356 — `= Tag`
-  `from` function L353-355 — `(label: String) -> Self`
-  `Tag` type L358-362 — `= Tag`
-  `from` function L359-361 — `(label: &str) -> Self`
-  `Tag` type L364-395 — `= Tag`
-  `Err` type L365 — `= ()`
-  `from_str` function L367-394 — `(s: &str) -> Result<Self, Self::Err>`
-  `Tag` type L397-411 — `= Tag`
-  `tests` module L414-596 — `-`
-  `test_title_to_slug` function L418-435 — `()`
-  `test_id_length_capping` function L438-443 — `()`
-  `test_adr_custom_id` function L446-449 — `()`
-  `test_tag_parsing` function L452-477 — `()`
-  `test_tag_to_str` function L480-488 — `()`
-  `test_tag_roundtrip` function L491-504 — `()`
-  `test_document_type_valid_transitions` function L507-530 — `()`
-  `test_document_type_can_transition` function L533-550 — `()`
-  `test_document_type_next_phase` function L553-572 — `()`
-  `test_document_type_phase_sequence` function L575-595 — `()`

### crates/metis-docs-core/src/domain/documents/vision/mod.rs

- pub `Vision` struct L12-14 — `{ core: super::traits::DocumentCore }` — A Vision document represents the high-level direction and goals
- pub `new` function L18-27 — `( title: String, tags: Vec<Tag>, archived: bool, short_code: String, ) -> Result...` — Create a new Vision document with content rendered from template
- pub `new_with_template` function L30-69 — `( title: String, tags: Vec<Tag>, archived: bool, short_code: String, template_co...` — Create a new Vision document with a custom template
- pub `from_parts` function L72-92 — `( title: String, metadata: DocumentMetadata, content: DocumentContent, tags: Vec...` — Create a Vision document from existing data (used when loading from file)
- pub `from_file` function L95-101 — `(path: P) -> Result<Self, DocumentValidationError>` — Create a Vision document by reading and parsing a file
- pub `from_content` function L104-156 — `(raw_content: &str) -> Result<Self, DocumentValidationError>` — Create a Vision document from raw file content string
- pub `to_file` function L180-185 — `(&self, path: P) -> Result<(), DocumentValidationError>` — Write the Vision document to a file
- pub `to_content` function L188-240 — `(&self) -> Result<String, DocumentValidationError>` — Convert the Vision document to its markdown string representation using templates
-  `Vision` type L16-241 — `= Vision`
-  `next_phase_in_sequence` function L159-167 — `(current: Phase) -> Option<Phase>` — Get the next phase in the Vision sequence
-  `update_phase_tag` function L170-177 — `(&mut self, new_phase: Phase)` — Update the phase tag in the document's tags
-  `Vision` type L243-367 — `impl Document for Vision`
-  `document_type` function L246-248 — `(&self) -> DocumentType`
-  `title` function L250-252 — `(&self) -> &str`
-  `metadata` function L254-256 — `(&self) -> &DocumentMetadata`
-  `content` function L258-260 — `(&self) -> &DocumentContent`
-  `core` function L262-264 — `(&self) -> &super::traits::DocumentCore`
-  `can_transition_to` function L266-273 — `(&self, phase: Phase) -> bool`
-  `parent_id` function L275-277 — `(&self) -> Option<&DocumentId>`
-  `blocked_by` function L279-281 — `(&self) -> &[DocumentId]`
-  `validate` function L283-304 — `(&self) -> Result<(), DocumentValidationError>`
-  `exit_criteria_met` function L306-311 — `(&self) -> bool`
-  `template` function L313-320 — `(&self) -> DocumentTemplate`
-  `frontmatter_template` function L322-324 — `(&self) -> &'static str`
-  `content_template` function L326-328 — `(&self) -> &'static str`
-  `acceptance_criteria_template` function L330-332 — `(&self) -> &'static str`
-  `transition_phase` function L334-362 — `( &mut self, target_phase: Option<Phase>, ) -> Result<Phase, DocumentValidationE...`
-  `core_mut` function L364-366 — `(&mut self) -> &mut super::traits::DocumentCore`
-  `tests` module L370-763 — `-`
-  `test_vision_from_content` function L376-434 — `()`
-  `test_vision_invalid_level` function L437-462 — `()`
-  `test_vision_missing_title` function L465-489 — `()`
-  `test_vision_tag_parsing` function L492-540 — `()`
-  `test_vision_validation` function L543-572 — `()`
-  `test_vision_phase_transitions` function L575-600 — `()`
-  `test_vision_transition_phase_auto` function L603-643 — `()`
-  `test_vision_transition_phase_explicit` function L646-678 — `()`
-  `test_vision_transition_phase_invalid` function L681-710 — `()`
-  `test_vision_update_section` function L713-762 — `()`

### crates/metis-docs-core/src/domain/mod.rs

- pub `configuration` module L1 — `-`
- pub `documents` module L3 — `-` — Domain layer containing pure business logic and models

### crates/metis-docs-core/src/error/conversions.rs

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

### crates/metis-docs-core/src/error.rs

- pub `conversions` module L3 — `-` — Error types for Metis operations
- pub `Result` type L8 — `= std::result::Result<T, MetisError>` — Error types for Metis operations
- pub `MetisError` enum L11-69 — `Database | Connection | Io | Json | Yaml | DocumentNotFound | InvalidDocumentTyp...` — Error types for Metis operations

### crates/metis-docs-core/src/lib.rs

- pub `application` module L7 — `-` — Metis implements the Flight Levels methodology for hierarchical documentation
- pub `constants` module L8 — `-` — documents through their defined phases.
- pub `dal` module L9 — `-` — documents through their defined phases.
- pub `domain` module L10 — `-` — documents through their defined phases.
- pub `error` module L11 — `-` — documents through their defined phases.
- pub `tests` module L29-31 — `-` — documents through their defined phases.
- pub `common` module L30 — `-` — documents through their defined phases.

### crates/metis-docs-core/src/main.rs

-  `main` function L2-4 — `()`

### crates/metis-docs-core/src/tests/common.rs

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

### crates/metis-docs-core/src/tests/mod.rs

- pub `common` module L1 — `-`

### crates/metis-docs-core/tests/collision_resolution_test.rs

-  `test_short_code_collision_resolution` function L12-188 — `()` — Integration test for METIS-T-0001: Multi-developer short code collision resolution
-  `test_sibling_cross_reference_update` function L192-281 — `()` — Test cross-reference updating in sibling documents
-  `test_collision_resolution_depth_ordering` function L285-383 — `()` — Test that collision resolution preserves document order by path depth

### crates/metis-docs-core/tests/configuration_recovery_test.rs

-  `setup_test_workspace` function L16-31 — `() -> (TempDir, String, String)` — Helper to create a test workspace
-  `test_recovery_from_complete_database_loss` function L34-121 — `()` — - Config.toml synchronization
-  `test_counter_recovery_prevents_duplicates` function L124-168 — `()` — - Config.toml synchronization
-  `test_config_sync_on_normal_operations` function L171-215 — `()` — - Config.toml synchronization
-  `test_recovery_validates_short_code_format` function L218-268 — `()` — - Config.toml synchronization
-  `test_migration_from_old_workspace_without_config_toml` function L271-311 — `()` — - Config.toml synchronization
-  `test_recovery_from_corrupted_database_file` function L314-420 — `()` — - Config.toml synchronization

### crates/metis-docs-core/tests/database_reconstruction_test.rs

-  `test_database_auto_reconstruction` function L16-102 — `()` — Integration test for METIS-T-0012: Database as cache only
-  `test_workspace_detection_without_database` function L106-123 — `()` — Test that workspace detection works without database present
-  `test_is_workspace_without_database` function L127-142 — `()` — Test that is_workspace only checks for .metis directory

### crates/metis-docs-core/tests/id_path_consistency_test.rs

-  `setup_test_workspace` function L12-28 — `(project_name: &str) -> (tempfile::TempDir, PathBuf)`
-  `test_document_short_code_matches_path` function L31-74 — `()`
-  `test_initiative_id_path_consistency` function L77-138 — `()`
-  `test_task_id_path_consistency` function L141-224 — `()`
-  `test_adr_id_consistency` function L227-256 — `()`
-  `test_long_title_id_path_consistency` function L259-295 — `()`
-  `test_unicode_title_id_path_consistency` function L298-346 — `()`
-  `test_regression_id_path_mismatch_bug` function L352-455 — `()` — Regression test for the ID/path mismatch bug

### crates/metis-docs-core/tests/reassignment_test.rs

-  `setup_test_workspace` function L10-87 — `() -> (tempfile::TempDir, std::path::PathBuf)` — Helper to create a test workspace with vision, initiative, and task
-  `test_reassign_backlog_to_initiative` function L91-125 — `()` — Test reassigning a backlog task to an initiative
-  `test_reassign_task_to_backlog` function L129-166 — `()` — Test reassigning a task from initiative to backlog
-  `test_reassign_between_initiatives` function L170-239 — `()` — Test reassigning a task between initiatives
-  `test_reassign_across_strategies` function L243-341 — `()` — Test reassigning a task between initiatives under different strategies
-  `test_reassign_non_task_fails` function L345-370 — `()` — Test that reassignment fails for non-task documents
-  `test_reassign_to_non_initiative_fails` function L374-405 — `()` — Test that reassignment to non-initiative parent fails
-  `test_reassign_to_wrong_phase_initiative_fails` function L409-444 — `()` — Test that reassignment to initiative in wrong phase fails
-  `test_reassign_missing_document_fails` function L448-473 — `()` — Test reassignment with missing source document

### crates/metis-docs-gui/src/composables/useProject.ts

- pub `useProject` function L18-110 — `function useProject()`
-  `ProjectState` interface L4-9 — `{ currentProject: : ProjectInfo | null, recentProjects: : ProjectInfo[], isLoadi...`
-  `loadProject` function L32-58 — `const loadProject = (path: string): Promise<void>`
-  `clearProject` function L60-62 — `const clearProject = ()`
-  `setCurrentProject` function L64-66 — `const setCurrentProject = (project: ProjectInfo | null)`
-  `addRecentProject` function L68-71 — `const addRecentProject = (project: ProjectInfo)`
-  `getRecentProjects` function L73-75 — `const getRecentProjects = (): ProjectInfo[]`
-  `saveRecentProject` function L77-81 — `const saveRecentProject = (project: ProjectInfo)`
-  `removeProject` function L83-92 — `const removeProject = (path: string)`

### crates/metis-docs-gui/src/composables/useTheme.ts

- pub `useTheme` function L7-56 — `function useTheme()`
- pub `useThemedStyles` function L59-76 — `function useThemedStyles()`
-  `setTheme` function L39-41 — `const setTheme = (newThemeName: ThemeName)`
-  `toggleTheme` function L43-48 — `const toggleTheme = ()`

### crates/metis-docs-gui/src/composables/useToolbar.ts

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

### crates/metis-docs-gui/src/lib/board-config.ts

- pub `PhaseConfig` interface L4-9 — `{ key: : string, title: : string, description: : string, emptyMessage: : string ...`
- pub `BoardConfig` interface L11-17 — `{ id: : BoardType, title: : string, description: : string, phases: : PhaseConfig...`
- pub `getBoardConfig` function L208-210 — `function getBoardConfig(boardType: BoardType): BoardConfig | undefined`
- pub `InitiativeFilterOption` interface L212-215 — `{ short_code: : string, title: : string }`
- pub `getActiveInitiatives` function L220-225 — `function getActiveInitiatives(documents: DocumentInfo[]): InitiativeFilterOption...`
- pub `getAllInitiatives` function L230-235 — `function getAllInitiatives(documents: DocumentInfo[]): InitiativeFilterOption[]`
- pub `getDocumentsByPhase` function L243-301 — `function getDocumentsByPhase( documents: DocumentInfo[], boardType: BoardType, i...`

### crates/metis-docs-gui/src/lib/tauri-api.ts

- pub `ProjectInfo` interface L4-8 — `{ path: : string, is_valid: : boolean, vision_exists: : boolean }`
- pub `DocumentInfo` interface L10-22 — `{ id: : string, title: : string, document_type: : string, short_code: : string, ...`
- pub `DocumentContent` interface L24-29 — `{ id: : string, title: : string, content: : string, frontmatter_json: : string }`
- pub `InitializationResult` interface L31-35 — `{ metis_dir: : string, database_path: : string, vision_path: : string }`
- pub `MetisAPI` class L38-126 — `-`
- pub `initializeProject` method L42-48 — `initializeProject( path: string, prefix?: string, preset?: string ): Promise<Ini...`
- pub `loadProject` method L53-55 — `loadProject(path: string): Promise<ProjectInfo>`
- pub `listDocuments` method L60-62 — `listDocuments(): Promise<DocumentInfo[]>`
- pub `readDocument` method L67-69 — `readDocument(shortCode: string): Promise<DocumentContent>`
- pub `searchDocuments` method L74-76 — `searchDocuments(query: string): Promise<DocumentInfo[]>`
- pub `getProjectConfig` method L81-83 — `getProjectConfig(): Promise<ProjectConfig>`
- pub `syncProject` method L88-90 — `syncProject(): Promise<SyncResult>`
- pub `getAvailableParents` method L95-97 — `getAvailableParents(childDocumentType: string): Promise<ParentOption[]>`
- pub `transitionPhase` method L102-104 — `transitionPhase(shortCode: string, newPhase?: string): Promise<string>`
- pub `getAppVersion` method L109-111 — `getAppVersion(): Promise<string>`
- pub `installCli` method L116-118 — `installCli(): Promise<void>`
- pub `installCliElevated` method L123-125 — `installCliElevated(): Promise<void>`
- pub `DocumentType` enum L129-135 — `Vision | Strategy | Initiative | Task | ADR`
- pub `Phase` enum L137-153 — `Draft | Review | Published | Shaping | Design | Ready | Active | Completed | Dis...`
- pub `formatDate` function L156-158 — `function formatDate(timestamp: number): string`
- pub `getDocumentTypeIcon` function L160-175 — `function getDocumentTypeIcon(type: string): string`
- pub `getPhaseColor` function L177-201 — `function getPhaseColor(phase: string): string`
- pub `CreateDocumentRequest` interface L203-210 — `{ document_type: : string, title: : string, parent_id: : string, complexity: : s...`
- pub `CreateDocumentResult` interface L212-216 — `{ id: : string, short_code: : string, filepath: : string }`
- pub `ParentOption` interface L218-223 — `{ short_code: : string, title: : string, document_type: : string, phase: : strin...`
- pub `ProjectConfig` interface L225-229 — `{ strategies_enabled: : boolean, initiatives_enabled: : boolean, preset_name: : ...`
- pub `ArchiveResult` interface L231-234 — `{ total_archived: : number, archived_documents: : ArchivedDocument[] }`
- pub `ArchivedDocument` interface L236-241 — `{ document_id: : string, document_type: : string, original_path: : string, archi...`
- pub `SyncResult` interface L243-250 — `{ imported: : number, updated: : number, deleted: : number, up_to_date: : number...`
- pub `DocumentAPI` class L253-281 — `-`
- pub `createDocument` method L257-259 — `createDocument(request: CreateDocumentRequest): Promise<CreateDocumentResult>`
- pub `updateDocument` method L264-266 — `updateDocument(shortCode: string, content: string): Promise<void>`
- pub `transitionPhase` method L271-273 — `transitionPhase(shortCode: string, newPhase?: string): Promise<string>`
- pub `archiveDocument` method L278-280 — `archiveDocument(shortCode: string): Promise<ArchiveResult>`

### crates/metis-docs-gui/src/types/board.ts

- pub `BoardType` type L1 — `= 'vision' | 'strategy' | 'initiative' | 'task' | 'adr' | 'backlog'`

### crates/metis-docs-gui/src/types/theme.ts

- pub `ThemeColors` interface L1-57 — `{ background: : { primary: string; secondary: string; tertiary: string; elevated...`
- pub `Theme` interface L59-62 — `{ name: : string, colors: : ThemeColors }`
- pub `ThemeName` type L64 — `= 'light' | 'dark' | 'hyper'`

### crates/metis-docs-gui/src/utils/drag-n-drop.ts

- pub `applyDrag` function L1-24 — `function applyDrag(arr: any[], dragResult: any)`

### crates/metis-docs-gui/src-tauri/build.rs

-  `main` function L4-9 — `()`
-  `sync_version_to_tauri_config` function L11-63 — `()`

### crates/metis-docs-gui/src-tauri/src/lib.rs

- pub `AppState` struct L13-15 — `{ current_project: Option<PathBuf> }`
- pub `run` function L17-63 — `()`
-  `services` module L3 — `-`

### crates/metis-docs-gui/src-tauri/src/main.rs

-  `main` function L4-6 — `()`

### crates/metis-docs-gui/src-tauri/src/services/archive.rs

- pub `ArchiveResult` struct L10-13 — `{ total_archived: usize, archived_documents: Vec<ArchivedDocument> }`
- pub `ArchivedDocument` struct L16-21 — `{ document_id: String, document_type: String, original_path: String, archived_pa...`
- pub `archive_document` function L24-93 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, ) -> Result<...`
-  `tests` module L96-120 — `-`
-  `test_archive_service_creation` function L102-119 — `()`

### crates/metis-docs-gui/src-tauri/src/services/cli_installer.rs

- pub `CliInstallStatus` struct L17-23 — `{ installed: bool, version: Option<String>, binary_path: Option<String>, symlink...` — Status of CLI installation
- pub `CliInstallResult` struct L27-31 — `{ success: bool, message: String, needs_elevation: bool }` — Result of CLI installation attempt
- pub `get_cli_install_status` function L106-131 — `() -> Result<CliInstallStatus, String>` — Check CLI installation status
- pub `install_cli_internal` function L187-337 — `(app: &AppHandle) -> Result<CliInstallResult, String>` — Internal installation function - copies binary and attempts symlink
- pub `install_cli` function L341-343 — `(app: AppHandle) -> Result<CliInstallResult, String>` — Install CLI with user-level permissions
- pub `install_cli_elevated` function L347-392 — `(app: AppHandle) -> Result<CliInstallResult, String>` — Install CLI with elevated permissions (creates symlink in /usr/local/bin)
- pub `uninstall_cli` function L396-419 — `() -> Result<(), String>` — Uninstall CLI - remove binary and symlink
- pub `auto_install_cli` function L451-515 — `(app: AppHandle)` — Run auto-installation on app startup
-  `CliVersionInfo` struct L35-39 — `{ version: String, installed_at: String, binary_path: String }` — Version tracking info stored in app data
-  `get_cli_data_dir` function L42-44 — `() -> Option<PathBuf>` — Get the app data directory for CLI storage
-  `get_cli_binary_path` function L47-58 — `() -> Option<PathBuf>` — Get the CLI binary destination path within app data
-  `get_symlink_path` function L62-64 — `() -> Option<PathBuf>` — Get the symlink location for PATH integration
-  `get_symlink_path` function L67-69 — `() -> Option<PathBuf>` — 3.
-  `get_version_info_path` function L72-74 — `() -> Option<PathBuf>` — Get the version info file path
-  `read_version_info` function L77-81 — `() -> Option<CliVersionInfo>` — Read current installed CLI version info
-  `write_version_info` function L84-102 — `(version: &str, binary_path: &PathBuf) -> Result<(), String>` — Write CLI version info after installation
-  `get_sidecar_path` function L134-184 — `(app: &AppHandle) -> Result<PathBuf, String>` — Get the path to the bundled sidecar binary
-  `add_to_windows_path` function L423-448 — `(bin_dir: &std::path::Path) -> Result<(), String>` — Add a directory to Windows user PATH via registry

### crates/metis-docs-gui/src-tauri/src/services/document.rs

- pub `DocumentInfo` struct L13-25 — `{ id: String, title: String, document_type: String, short_code: String, filepath...`
- pub `DocumentContent` struct L28-33 — `{ id: String, title: String, content: String, frontmatter_json: String }`
- pub `CreateDocumentRequest` struct L36-43 — `{ document_type: String, title: String, parent_id: Option<String>, complexity: O...`
- pub `CreateDocumentResult` struct L46-50 — `{ id: String, short_code: String, filepath: String }`
- pub `create_document` function L85-234 — `( state: State<'_, std::sync::Mutex<AppState>>, request: CreateDocumentRequest, ...`
- pub `list_documents` function L237-315 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<Vec<DocumentInfo>, S...`
- pub `read_document` function L318-362 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, ) -> Result<...`
- pub `search_documents` function L365-420 — `( state: State<'_, std::sync::Mutex<AppState>>, query: String, ) -> Result<Vec<D...`
- pub `update_document` function L423-475 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, content: Str...`
- pub `ParentOption` struct L478-483 — `{ short_code: String, title: String, document_type: String, phase: String }`
- pub `get_available_parents` function L486-572 — `( state: State<'_, std::sync::Mutex<AppState>>, child_document_type: String, ) -...`
-  `find_strategy_short_code_for_initiative` function L52-82 — `( metis_dir: &Path, initiative_id: &str, ) -> Result<String, String>`
-  `tests` module L575-672 — `-`
-  `test_create_adr_document` function L581-606 — `()`
-  `test_create_task_with_initiative_parent` function L609-671 — `()`
-  `add_tag_to_frontmatter` function L675-728 — `(file_path: &std::path::Path, tag: &str) -> Result<(), String>` — Add a tag to the frontmatter of a document file
-  `extract_tags_from_task_file` function L731-770 — `(filepath: &str) -> Result<Vec<String>, String>` — Extract tags from a task file by parsing it like the TUI does

### crates/metis-docs-gui/src-tauri/src/services/mod.rs

- pub `archive` module L1 — `-`
- pub `cli_installer` module L2 — `-`
- pub `document` module L3 — `-`
- pub `project` module L4 — `-`
- pub `sync` module L5 — `-`
- pub `transition` module L6 — `-`
- pub `version` module L7 — `-`

### crates/metis-docs-gui/src-tauri/src/services/project.rs

- pub `ProjectInfo` struct L11-15 — `{ path: String, is_valid: bool, vision_exists: bool }`
- pub `InitializationResult` struct L18-22 — `{ metis_dir: String, database_path: String, vision_path: String }`
- pub `ProjectConfig` struct L25-29 — `{ strategies_enabled: bool, initiatives_enabled: bool, preset_name: String }`
- pub `initialize_project` function L32-73 — `( path: String, prefix: Option<String>, preset: Option<String>, ) -> Result<Init...`
- pub `load_project` function L76-98 — `( state: State<'_, std::sync::Mutex<AppState>>, path: String, ) -> Result<Projec...`
- pub `get_project_config` function L101-132 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<ProjectConfig, Strin...`
-  `determine_flight_config` function L135-146 — `(preset_name: Option<&str>) -> Result<FlightLevelConfig, String>` — Determine the flight level configuration based on preset name
-  `tests` module L149-186 — `-`
-  `test_initialize_project_success` function L154-172 — `()`
-  `test_initialize_project_with_default_prefix` function L175-185 — `()`

### crates/metis-docs-gui/src-tauri/src/services/sync.rs

- pub `SyncResult` struct L7-14 — `{ imported: u32, updated: u32, deleted: u32, up_to_date: u32, errors: u32, messa...`
- pub `sync_project` function L17-119 — `( state: State<'_, std::sync::Mutex<AppState>>, ) -> Result<SyncResult, String>`

### crates/metis-docs-gui/src-tauri/src/services/transition.rs

- pub `transition_phase` function L32-109 — `( state: State<'_, std::sync::Mutex<AppState>>, short_code: String, new_phase: O...`
-  `parse_phase` function L8-29 — `(phase_str: &str) -> Result<Phase, String>`
-  `tests` module L112-128 — `-`
-  `test_parse_phase_valid` function L116-120 — `()`
-  `test_parse_phase_invalid` function L123-127 — `()`

### crates/metis-docs-gui/src-tauri/src/services/version.rs

- pub `get_app_version` function L4-6 — `() -> String`

### crates/metis-docs-mcp/src/config.rs

- pub `MetisServerConfig` struct L4-7 — `-`
- pub `from_env` function L10-13 — `() -> Result<Self>`
- pub `new` function L15-17 — `() -> Self`
-  `MetisServerConfig` type L9-18 — `= MetisServerConfig`

### crates/metis-docs-mcp/src/error.rs

- pub `McpServerError` enum L4-22 — `DocumentNotFound | InvalidParameter | ProjectNotInitialized | CoreLibrary | Conf...`
- pub `Result` type L24 — `= std::result::Result<T, McpServerError>`

### crates/metis-docs-mcp/src/error_utils.rs

- pub `tool_error` function L4-6 — `(msg: &str) -> CallToolError` — Helper function to create CallToolError from string messages
-  `tool_error` macro L10-14 — `-` — Helper function to create CallToolError from formatted string

### crates/metis-docs-mcp/src/formatting.rs

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
- pub `diff` function L106-121 — `(mut self, old: &str, new: &str) -> Self` — Add a diff block showing before/after
- pub `table` function L124-173 — `(mut self, headers: &[&str], rows: Vec<Vec<String>>) -> Self` — Add a simple key-value table with proper column padding
- pub `status_list` function L176-191 — `(mut self, items: Vec<(&str, bool)>) -> Self` — Add a list of items with status indicators
- pub `bullet_list` function L194-198 — `(mut self, items: &[&str]) -> Self` — Add a bulleted list
- pub `indented` function L201-216 — `(mut self, items: Vec<(bool, &str)>) -> Self` — Add indented content (for nested items)
- pub `phase_progress` function L219-234 — `(mut self, phases: &[&str], current_index: usize) -> Self` — Add a phase progression indicator
- pub `hint` function L237-240 — `(mut self, msg: &str) -> Self` — Add a hint/tip message
- pub `build` function L243-245 — `(self) -> String` — Build the final output string
- pub `build_result` function L248-256 — `(self) -> CallToolResult` — Build a CallToolResult with TextContent (no structuredContent for proper Claude Code rendering)
- pub `format_error` function L260-272 — `(title: &str, message: &str, hint: Option<&str>) -> String` — Helper for formatting error responses consistently
- pub `error_result` function L275-293 — `(title: &str, message: &str, hint: Option<&str>) -> CallToolResult` — Helper for formatting error responses as CallToolResult
- pub `format_not_found` function L296-306 — `(resource_type: &str, identifier: &str, hint: Option<&str>) -> String` — Helper for formatting not-found errors
-  `Icons` type L12-19 — `= Icons` — that renders well in terminal contexts.
-  `ToolOutput` type L27-257 — `= ToolOutput` — that renders well in terminal contexts.
-  `tests` module L309-379 — `-` — that renders well in terminal contexts.
-  `test_basic_output` function L313-322 — `()` — that renders well in terminal contexts.
-  `test_table_output` function L325-340 — `()` — that renders well in terminal contexts.
-  `test_diff_output` function L343-352 — `()` — that renders well in terminal contexts.
-  `test_phase_progress` function L355-365 — `()` — that renders well in terminal contexts.
-  `test_error_formatting` function L368-378 — `()` — that renders well in terminal contexts.

### crates/metis-docs-mcp/src/lib.rs

- pub `config` module L4 — `-`
- pub `error` module L5 — `-`
- pub `error_utils` module L6 — `-`
- pub `formatting` module L7 — `-`
- pub `server` module L8 — `-`
- pub `tools` module L9 — `-`
- pub `run` function L135-208 — `() -> AnyhowResult<()>` — Run the MCP server
-  `find_metis_log_path` function L30-44 — `() -> Option<String>`
-  `get_current_configuration` function L46-62 — `() -> Option<FlightLevelConfig>`
-  `generate_dynamic_instructions` function L64-108 — `() -> String`
-  `generate_operation_notes` function L110-132 — `(config: &FlightLevelConfig) -> String`

### crates/metis-docs-mcp/src/main.rs

-  `main` function L4-6 — `() -> Result<()>`

### crates/metis-docs-mcp/src/server.rs

- pub `MetisServerHandler` struct L18-21 — `{ config: Arc<MetisServerConfig> }`
- pub `new` function L24-29 — `(config: MetisServerConfig) -> Self`
-  `MetisServerHandler` type L23-30 — `= MetisServerHandler`
-  `MetisServerHandler` type L33-102 — `impl ServerHandler for MetisServerHandler`
-  `handle_list_tools_request` function L34-44 — `( &self, _params: Option<PaginatedRequestParams>, _runtime: Arc<dyn McpServer>, ...`
-  `handle_call_tool_request` function L46-101 — `( &self, params: CallToolRequestParams, _runtime: Arc<dyn McpServer>, ) -> Resul...`

### crates/metis-docs-mcp/src/tools/archive_document.rs

- pub `ArchiveDocumentTool` struct L19-24 — `{ project_path: String, short_code: String }`
- pub `call_tool` function L27-92 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ArchiveDocumentTool` type L26-93 — `= ArchiveDocumentTool`

### crates/metis-docs-mcp/src/tools/create_document.rs

- pub `CreateDocumentTool` struct L27-46 — `{ project_path: String, document_type: String, title: String, parent_id: Option<...`
- pub `call_tool` function L49-280 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `CreateDocumentTool` type L48-336 — `= CreateDocumentTool`
-  `find_strategy_short_code_for_initiative` function L282-335 — `( &self, database: &Database, initiative_id: &str, ) -> Result<String, CallToolE...`

### crates/metis-docs-mcp/src/tools/edit_document.rs

- pub `EditDocumentTool` struct L20-31 — `{ project_path: String, short_code: String, search: String, replace: String, rep...`
- pub `call_tool` function L34-114 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `EditDocumentTool` type L33-137 — `= EditDocumentTool`
-  `perform_edit` function L116-136 — `(&self, content: &str) -> Result<(String, usize), CallToolError>`

### crates/metis-docs-mcp/src/tools/index_code.rs

- pub `IndexCodeTool` struct L32-41 — `{ project_path: String, structure_only: Option<bool>, incremental: Option<bool> ...` — trigger index generation programmatically.
- pub `call_tool` function L44-270 — `(&self) -> std::result::Result<CallToolResult, CallToolError>` — trigger index generation programmatically.
-  `IndexCodeTool` type L43-271 — `= IndexCodeTool` — trigger index generation programmatically.
-  `extract_symbols_for_language` function L274-292 — `( language: Language, parsed: &ParsedFile, file_path: &str, ) -> Result<Vec<Symb...` — Dispatch symbol extraction to the appropriate language extractor.

### crates/metis-docs-mcp/src/tools/initialize_project.rs

- pub `InitializeProjectTool` struct L19-24 — `{ project_path: String, prefix: Option<String> }`
- pub `call_tool` function L27-82 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `InitializeProjectTool` type L26-83 — `= InitializeProjectTool`

### crates/metis-docs-mcp/src/tools/list_documents.rs

- pub `ListDocumentsTool` struct L20-26 — `{ project_path: String, include_archived: Option<bool> }`
- pub `call_tool` function L29-93 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ListDocumentsTool` type L28-127 — `= ListDocumentsTool`
-  `list_all_documents` function L95-126 — `( &self, repo: &mut metis_core::dal::database::repository::DocumentRepository, i...`

### crates/metis-docs-mcp/src/tools/mod.rs

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

### crates/metis-docs-mcp/src/tools/read_document.rs

- pub `ReadDocumentTool` struct L20-25 — `{ project_path: String, short_code: String }`
- pub `call_tool` function L55-102 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ReadDocumentTool` type L27-216 — `= ReadDocumentTool`
-  `resolve_short_code_to_path` function L29-53 — `(&self, metis_dir: &Path) -> Result<String, CallToolError>` — Resolve short code to file path
-  `extract_metadata` function L104-165 — `(&self, content: &str) -> (String, String, String, String, String)`
-  `extract_sections` function L168-180 — `(&self, content: &str) -> Vec<String>`
-  `extract_exit_criteria` function L182-215 — `(&self, content: &str) -> Vec<ExitCriterion>`
-  `ExitCriterion` struct L219-222 — `{ text: String, completed: bool }`

### crates/metis-docs-mcp/src/tools/reassign_parent.rs

- pub `ReassignParentTool` struct L21-30 — `{ project_path: String, short_code: String, new_parent_id: Option<String>, backl...`
- pub `call_tool` function L33-152 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `ReassignParentTool` type L32-153 — `= ReassignParentTool`

### crates/metis-docs-mcp/src/tools/search_documents.rs

- pub `SearchDocumentsTool` struct L19-31 — `{ project_path: String, query: String, document_type: Option<String>, limit: Opt...`
- pub `call_tool` function L51-132 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `SearchDocumentsTool` type L33-133 — `= SearchDocumentsTool`
-  `sanitize_search_query` function L35-49 — `(&self, query: &str) -> String` — Sanitize search query to prevent FTS syntax errors

### crates/metis-docs-mcp/src/tools/transition_phase.rs

- pub `TransitionPhaseTool` struct L22-31 — `{ project_path: String, short_code: String, phase: Option<String>, force: Option...`
- pub `call_tool` function L34-89 — `(&self) -> std::result::Result<CallToolResult, CallToolError>`
-  `TransitionPhaseTool` type L33-132 — `= TransitionPhaseTool`
-  `parse_phase` function L91-114 — `(&self, phase_str: &str) -> Result<Phase, CallToolError>`
-  `get_phase_sequence` function L116-131 — `(&self, document_type: &str) -> Vec<String>`

### crates/metis-docs-mcp/tests/common/mod.rs

- pub `McpTestHelper` struct L12-14 — `{ core_helper: MetisTestHelper }` — MCP-specific test helper that wraps the core helper
- pub `new` function L17-20 — `() -> Result<Self>` — Common utilities and helper functions for integration tests
- pub `metis_dir` function L23-25 — `(&self) -> String` — Get metis directory as string (for backward compatibility)
- pub `initialize_project` function L27-41 — `(&self) -> Result<()>` — Common utilities and helper functions for integration tests
- pub `get_database` function L43-45 — `(&self) -> Result<Database>` — Common utilities and helper functions for integration tests
- pub `get_project_name` function L47-56 — `(&self) -> String` — Common utilities and helper functions for integration tests
- pub `set_flight_level_config` function L60-77 — `(&self, flight_config: FlightLevelConfig) -> Result<()>` — Update flight level configuration in config.toml
-  `McpTestHelper` type L16-78 — `= McpTestHelper` — Common utilities and helper functions for integration tests

### crates/metis-docs-mcp/tests/comprehensive_functional_test.rs

-  `setup_project_with_config` function L10-44 — `(config: FlightLevelConfig) -> (TempDir, String, String)` — Helper to setup project with specific flight configuration
-  `extract_text_from_result` function L47-62 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `get_vision_short_code` function L65-82 — `(metis_path: &str) -> String` — Helper to get vision short code from list results (parses markdown table format)
-  `extract_short_code` function L85-96 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `test_full_configuration_workflow` function L99-266 — `()` — These tests mirror real user workflows through MCP tool calls
-  `test_streamlined_configuration_workflow` function L269-384 — `()` — These tests mirror real user workflows through MCP tool calls
-  `test_direct_configuration_workflow` function L387-502 — `()` — These tests mirror real user workflows through MCP tool calls

### crates/metis-docs-mcp/tests/configuration_scenarios_test.rs

-  `common` module L10 — `-` — These tests validate real-world usage scenarios for each configuration preset
-  `extract_text_from_result` function L13-28 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L31-42 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `test_streamlined_configuration_workflows` function L47-169 — `() -> Result<()>` — Test MCP server behavior with default streamlined configuration
-  `test_direct_configuration_workflows` function L174-263 — `() -> Result<()>` — Test MCP server behavior with direct configuration
-  `test_full_configuration_workflows` function L268-410 — `() -> Result<()>` — Test MCP server behavior with full configuration
-  `test_configuration_error_messages` function L414-477 — `() -> Result<()>` — Test configuration error messages provide actionable guidance
-  `test_configuration_switching_compatibility` function L481-551 — `() -> Result<()>` — Test configuration switching doesn't break existing documents

### crates/metis-docs-mcp/tests/functional_test.rs

-  `extract_text_from_result` function L8-23 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L26-37 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `get_vision_short_code` function L40-57 — `(metis_path: &str) -> String` — Helper to get vision short code from list results (parses markdown table format)
-  `test_initialize_and_create_documents` function L60-181 — `()` — Clean functional tests for MCP tools using short codes
-  `test_archive_with_short_codes` function L184-235 — `()` — Clean functional tests for MCP tools using short codes
-  `test_search_with_short_code_hyphen` function L238-313 — `()` — Clean functional tests for MCP tools using short codes
-  `test_list_and_search_include_archived` function L316-449 — `()` — Clean functional tests for MCP tools using short codes
-  `test_create_backlog_items` function L452-610 — `()` — Clean functional tests for MCP tools using short codes

### crates/metis-docs-mcp/tests/mcp_archive_test.rs

-  `common` module L3 — `-` — Archive behavior tests for MCP server to verify the fixed cascading functionality
-  `extract_text_from_result` function L11-26 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> Option<String>` — Helper to extract text content from MCP response (handles EmbeddedResource)
-  `extract_short_code` function L29-40 — `(result: &rust_mcp_sdk::schema::CallToolResult) -> String` — Helper to extract short code from MCP response (parses markdown format)
-  `get_vision_short_code` function L43-60 — `(metis_path: &str) -> String` — Helper to get vision short code from list results (parses markdown table format)
-  `test_mcp_archive_cascading_behavior` function L65-434 — `() -> Result<()>` — Test MCP server archive cascading behavior that mirrors TUI test behavior
-  `test_mcp_archive_error_handling` function L438-520 — `() -> Result<()>` — Test MCP server archive error handling

### crates/metis-docs-mcp/tests/mcp_server_integration_test.rs

-  `McpServerProcess` struct L12-16 — `{ temp_dir: TempDir, project_path: String, metis_dir: String }` — Helper to build and spawn the MCP server binary
-  `McpServerProcess` type L18-233 — `= McpServerProcess` — This tests the real MCP protocol communication including the archive_document fix
-  `new` function L19-29 — `() -> Result<Self>` — This tests the real MCP protocol communication including the archive_document fix
-  `build_server` function L32-45 — `() -> Result<()>` — Build the MCP server binary
-  `spawn_server` function L48-56 — `(&self) -> Result<std::process::Child>` — Spawn the MCP server process and return handles for communication
-  `send_mcp_request` function L59-83 — `(child: &mut std::process::Child, request: Value) -> Result<Value>` — Send an MCP request and get response
-  `mcp_initialize` function L86-108 — `(child: &mut std::process::Child) -> Result<()>` — Send MCP protocol initialization handshake
-  `initialize_project` function L111-132 — `(&self, child: &mut std::process::Child) -> Result<()>` — Initialize the project using MCP protocol
-  `create_test_document` function L135-193 — `(&self, child: &mut std::process::Child) -> Result<String>` — Create a test document to archive
-  `test_archive_document` function L196-232 — `( &self, child: &mut std::process::Child, short_code: &str, ) -> Result<()>` — Test the archive_document functionality
-  `test_mcp_server_archive_document_integration` function L236-274 — `() -> Result<()>` — This tests the real MCP protocol communication including the archive_document fix
-  `test_mcp_server_list_tools` function L277-320 — `() -> Result<()>` — This tests the real MCP protocol communication including the archive_document fix

### tests/e2e/fixtures.ts

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

