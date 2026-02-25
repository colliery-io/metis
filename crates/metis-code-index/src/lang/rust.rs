//! Rust-specific symbol extraction.
//!
//! Uses tree-sitter queries to extract symbols, imports, and call relationships
//! from Rust source code.

use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;

use crate::symbols::{Symbol, SymbolKind, Visibility};

/// Represents a use/import statement.
#[derive(Debug, Clone)]
pub struct Import {
    /// The import path (e.g., "std::collections::HashMap")
    pub path: String,
    /// Optional alias (e.g., "use std::io::Result as IoResult")
    pub alias: Option<String>,
    /// Line number of the import
    pub line: usize,
}

/// Represents a function call site.
#[derive(Debug, Clone)]
pub struct Call {
    /// The callee expression (function being called)
    pub callee: String,
    /// Line number of the call
    pub line: usize,
    /// Whether this is a method call (uses `.` syntax)
    pub is_method: bool,
}

/// Represents an FFI boundary marker.
#[derive(Debug, Clone)]
pub struct FFIMarker {
    /// The type of FFI (e.g., "C", "system")
    pub abi: Option<String>,
    /// Line number
    pub line: usize,
    /// Span of the extern block
    pub start_line: usize,
    pub end_line: usize,
}

/// Represents a trait implementation.
#[derive(Debug, Clone)]
pub struct TraitImpl {
    /// The type implementing the trait (e.g., "GroqBackend")
    pub type_name: String,
    /// The trait being implemented (e.g., "LLMBackend")
    pub trait_name: String,
    /// Line number of the impl block
    pub line: usize,
}

/// Compiled queries for Rust symbol extraction.
struct RustQueries {
    symbols: tree_sitter::Query,
    imports: tree_sitter::Query,
    calls: tree_sitter::Query,
    externs: tree_sitter::Query,
    impls: tree_sitter::Query,
}

impl RustQueries {
    fn new(language: tree_sitter::Language) -> Result<Self, String> {
        let symbols = tree_sitter::Query::new(&language, SYMBOLS_QUERY)
            .map_err(|e| format!("Failed to compile symbols query: {e}"))?;
        let imports = tree_sitter::Query::new(&language, IMPORTS_QUERY)
            .map_err(|e| format!("Failed to compile imports query: {e}"))?;
        let calls = tree_sitter::Query::new(&language, CALLS_QUERY)
            .map_err(|e| format!("Failed to compile calls query: {e}"))?;
        let externs = tree_sitter::Query::new(&language, EXTERNS_QUERY)
            .map_err(|e| format!("Failed to compile externs query: {e}"))?;
        let impls = tree_sitter::Query::new(&language, IMPLS_QUERY)
            .map_err(|e| format!("Failed to compile impls query: {e}"))?;
        Ok(Self {
            symbols,
            imports,
            calls,
            externs,
            impls,
        })
    }
}

static RUST_QUERIES: OnceLock<Result<RustQueries, String>> = OnceLock::new();

fn get_queries() -> Result<&'static RustQueries, &'static str> {
    RUST_QUERIES
        .get_or_init(|| {
            let language = tree_sitter_rust::LANGUAGE.into();
            RustQueries::new(language)
        })
        .as_ref()
        .map_err(|e| e.as_str())
}

/// Tree-sitter query for extracting Rust symbols.
const SYMBOLS_QUERY: &str = r#"
; Structs
(struct_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name) @struct

; Enums
(enum_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name) @enum

; Functions
(function_item
  (visibility_modifier)? @visibility
  name: (identifier) @name
  parameters: (parameters) @params
  return_type: (_)? @return) @function

; Traits
(trait_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name) @trait

; Impl blocks
(impl_item
  trait: (type_identifier)? @trait_name
  type: (type_identifier) @type_name) @impl

; Modules
(mod_item
  (visibility_modifier)? @visibility
  name: (identifier) @name) @module

; Constants
(const_item
  (visibility_modifier)? @visibility
  name: (identifier) @name) @constant

; Statics
(static_item
  (visibility_modifier)? @visibility
  name: (identifier) @name) @static

; Type aliases
(type_item
  (visibility_modifier)? @visibility
  name: (type_identifier) @name) @type_alias

; Macro definitions
(macro_definition
  name: (identifier) @name) @macro
"#;

/// Tree-sitter query for extracting use statements.
const IMPORTS_QUERY: &str = r#"
; Simple use
(use_declaration
  argument: (scoped_identifier) @path) @use

; Use with alias
(use_declaration
  argument: (use_as_clause
    path: (scoped_identifier) @path
    alias: (identifier) @alias)) @use_alias

; Use with list
(use_declaration
  argument: (scoped_use_list
    path: (scoped_identifier)? @base_path
    list: (use_list) @list)) @use_list

; Use self
(use_declaration
  argument: (identifier) @path) @use_simple
"#;

/// Tree-sitter query for extracting function calls.
const CALLS_QUERY: &str = r#"
; Direct function calls
(call_expression
  function: (identifier) @callee) @call

; Scoped function calls (e.g., module::function())
(call_expression
  function: (scoped_identifier) @callee) @scoped_call

; Method calls
(call_expression
  function: (field_expression
    value: (_) @receiver
    field: (field_identifier) @method)) @method_call
"#;

/// Tree-sitter query for extracting extern blocks.
const EXTERNS_QUERY: &str = r#"
; Extern blocks (FFI)
(foreign_mod_item
  (extern_modifier
    (string_literal)? @abi)?) @extern

; Extern crate
(extern_crate_declaration
  name: (identifier) @crate_name) @extern_crate
"#;

/// Tree-sitter query for extracting trait implementations.
const IMPLS_QUERY: &str = r#"
; Trait implementations (impl Trait for Type)
(impl_item
  trait: (type_identifier) @trait_name
  type: (type_identifier) @type_name) @impl
"#;

/// Rust language extractor.
pub struct RustExtractor;

impl RustExtractor {
    /// Extract symbols from a parsed Rust syntax tree.
    pub fn extract_symbols(
        tree: &tree_sitter::Tree,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, String> {
        let queries = get_queries()?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();

        let mut symbols = Vec::new();
        let mut matches = cursor.matches(&queries.symbols, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut name = None;
            let mut visibility = None;
            let mut signature_parts = Vec::new();
            let mut kind = None;
            let mut start_line = 0;
            let mut end_line = 0;

            for capture in match_.captures {
                let capture_name = queries.symbols.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "name" | "type_name" => {
                        name = Some(text.to_string());
                    }
                    "visibility" => {
                        visibility = Some(parse_visibility(text));
                    }
                    "params" => {
                        signature_parts.push(text.to_string());
                    }
                    "return" => {
                        signature_parts.push(format!("-> {text}"));
                    }
                    "struct" => {
                        kind = Some(SymbolKind::Struct);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "enum" => {
                        kind = Some(SymbolKind::Enum);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "function" => {
                        kind = Some(SymbolKind::Function);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "trait" => {
                        kind = Some(SymbolKind::Interface);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "impl" => {
                        kind = Some(SymbolKind::Type);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "module" => {
                        kind = Some(SymbolKind::Module);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "constant" | "static" => {
                        kind = Some(SymbolKind::Variable);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "type_alias" => {
                        kind = Some(SymbolKind::Type);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    "macro" => {
                        kind = Some(SymbolKind::Macro);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(kind)) = (name, kind) {
                let doc_comment = extract_doc_comment(tree, start_line, source);

                let signature = if signature_parts.is_empty() {
                    None
                } else {
                    Some(signature_parts.join(" "))
                };

                symbols.push(Symbol {
                    name,
                    kind,
                    file_path: file_path.to_string(),
                    start_line,
                    end_line,
                    signature,
                    qualified_name: None,
                    doc_comment,
                    visibility: visibility.unwrap_or(Visibility::Private),
                });
            }
        }

        Ok(symbols)
    }

    /// Extract import statements from a parsed Rust syntax tree.
    pub fn extract_imports(tree: &tree_sitter::Tree, source: &str) -> Result<Vec<Import>, String> {
        let queries = get_queries()?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();

        let mut imports = Vec::new();
        let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut path = None;
            let mut alias = None;
            let mut line = 0;

            for capture in match_.captures {
                let capture_name = queries.imports.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "path" | "base_path" => {
                        path = Some(text.to_string());
                        line = node.start_position().row + 1;
                    }
                    "alias" => {
                        alias = Some(text.to_string());
                    }
                    _ => {}
                }
            }

            if let Some(path) = path {
                imports.push(Import { path, alias, line });
            }
        }

        Ok(imports)
    }

    /// Extract function calls from a parsed Rust syntax tree.
    pub fn extract_calls(tree: &tree_sitter::Tree, source: &str) -> Result<Vec<Call>, String> {
        let queries = get_queries()?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();

        let mut calls = Vec::new();
        let mut matches = cursor.matches(&queries.calls, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut callee = None;
            let mut line = 0;
            let mut is_method = false;

            for capture in match_.captures {
                let capture_name = queries.calls.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "callee" => {
                        callee = Some(text.to_string());
                        line = node.start_position().row + 1;
                    }
                    "method" => {
                        callee = Some(text.to_string());
                        line = node.start_position().row + 1;
                        is_method = true;
                    }
                    "method_call" => {
                        is_method = true;
                    }
                    _ => {}
                }
            }

            if let Some(callee) = callee {
                calls.push(Call {
                    callee,
                    line,
                    is_method,
                });
            }
        }

        Ok(calls)
    }

    /// Extract trait implementations from a parsed Rust syntax tree.
    pub fn extract_implementations(
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Result<Vec<TraitImpl>, String> {
        let queries = get_queries()?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();

        let mut impls = Vec::new();
        let mut matches = cursor.matches(&queries.impls, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut type_name = None;
            let mut trait_name = None;
            let mut line = 0;

            for capture in match_.captures {
                let capture_name = queries.impls.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "type_name" => {
                        type_name = Some(text.to_string());
                    }
                    "trait_name" => {
                        trait_name = Some(text.to_string());
                    }
                    "impl" => {
                        line = node.start_position().row + 1;
                    }
                    _ => {}
                }
            }

            if let (Some(type_name), Some(trait_name)) = (type_name, trait_name) {
                impls.push(TraitImpl {
                    type_name,
                    trait_name,
                    line,
                });
            }
        }

        Ok(impls)
    }

    /// Extract FFI markers (extern blocks) from a parsed Rust syntax tree.
    pub fn extract_ffi_markers(
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Result<Vec<FFIMarker>, String> {
        let queries = get_queries()?;
        let mut cursor = tree_sitter::QueryCursor::new();
        let source_bytes = source.as_bytes();

        let mut markers = Vec::new();
        let mut matches = cursor.matches(&queries.externs, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut abi = None;
            let mut start_line = 0;
            let mut end_line = 0;

            for capture in match_.captures {
                let capture_name = queries.externs.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "abi" => {
                        abi = Some(text.trim_matches('"').to_string());
                    }
                    "extern" | "extern_crate" => {
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                    }
                    _ => {}
                }
            }

            if start_line > 0 {
                markers.push(FFIMarker {
                    abi,
                    line: start_line,
                    start_line,
                    end_line,
                });
            }
        }

        Ok(markers)
    }
}

/// Parse a Rust visibility modifier into our Visibility enum.
fn parse_visibility(vis_text: &str) -> Visibility {
    let vis = vis_text.trim();
    if vis == "pub" {
        Visibility::Public
    } else if vis.starts_with("pub(crate)") {
        Visibility::Crate
    } else if vis.starts_with("pub(super)") {
        Visibility::Restricted("super".to_string())
    } else if vis.starts_with("pub(in") {
        let path = vis
            .strip_prefix("pub(in ")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or("unknown");
        Visibility::Restricted(path.to_string())
    } else if vis.starts_with("pub(") {
        let inner = vis
            .strip_prefix("pub(")
            .and_then(|s| s.strip_suffix(')'))
            .unwrap_or("self");
        Visibility::Restricted(inner.to_string())
    } else {
        Visibility::Private
    }
}

/// Extract doc comments preceding a symbol by looking at source lines.
fn extract_doc_comment(
    tree: &tree_sitter::Tree,
    symbol_line: usize,
    source: &str,
) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let mut doc_lines = Vec::new();

    let mut line_idx = symbol_line.saturating_sub(2); // 0-indexed, start from line before
    while line_idx > 0 {
        if line_idx >= lines.len() {
            break;
        }
        let line = lines[line_idx].trim();

        if line.starts_with("///") {
            let content = line.strip_prefix("///").unwrap_or("").trim();
            doc_lines.push(content.to_string());
        } else if line.starts_with("//!") {
            let content = line.strip_prefix("//!").unwrap_or("").trim();
            doc_lines.push(content.to_string());
        } else if line.is_empty() || line.starts_with("#[") {
            if !doc_lines.is_empty() && line.is_empty() {
                break;
            }
        } else {
            break;
        }

        if line_idx == 0 {
            break;
        }
        line_idx -= 1;
    }

    if doc_lines.is_empty() {
        let root = tree.root_node();
        let mut cursor = root.walk();

        if cursor.goto_first_child() {
            loop {
                let node = cursor.node();
                if node.kind() == "line_comment" {
                    let text = node.utf8_text(source.as_bytes()).unwrap_or("").trim();
                    if text.starts_with("//!") {
                        let content = text.strip_prefix("//!").unwrap_or("").trim();
                        doc_lines.push(content.to_string());
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    if doc_lines.is_empty() {
        None
    } else {
        doc_lines.reverse();
        Some(doc_lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_rust(source: &str) -> tree_sitter::Tree {
        let mut parser = tree_sitter::Parser::new();
        parser
            .set_language(&tree_sitter_rust::LANGUAGE.into())
            .unwrap();
        parser.parse(source, None).unwrap()
    }

    #[test]
    fn test_extract_struct() {
        let source = r#"
/// A documented struct.
pub struct Foo {
    bar: i32,
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Foo");
        assert_eq!(symbols[0].kind, SymbolKind::Struct);
        assert_eq!(symbols[0].visibility, Visibility::Public);
        assert!(symbols[0].doc_comment.is_some());
        assert!(symbols[0]
            .doc_comment
            .as_ref()
            .unwrap()
            .contains("documented"));
    }

    #[test]
    fn test_extract_function() {
        let source = r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "add");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
        assert_eq!(symbols[0].visibility, Visibility::Public);
        assert!(symbols[0].signature.is_some());
    }

    #[test]
    fn test_extract_enum() {
        let source = r#"
pub(crate) enum Status {
    Active,
    Inactive,
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "Status");
        assert_eq!(symbols[0].kind, SymbolKind::Enum);
        assert_eq!(symbols[0].visibility, Visibility::Crate);
    }

    #[test]
    fn test_extract_trait() {
        let source = r#"
pub trait Greet {
    fn greet(&self) -> String;
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        let trait_symbols: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Interface)
            .collect();
        assert_eq!(trait_symbols.len(), 1);
        assert_eq!(trait_symbols[0].name, "Greet");
    }

    #[test]
    fn test_extract_impl() {
        let source = r#"
struct Foo;

impl Foo {
    fn new() -> Self {
        Foo
    }
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert!(symbols
            .iter()
            .any(|s| s.name == "Foo" && s.kind == SymbolKind::Struct));
        assert!(symbols
            .iter()
            .any(|s| s.name == "Foo" && s.kind == SymbolKind::Type));
    }

    #[test]
    fn test_extract_module() {
        let source = r#"
pub mod utils {
    pub fn helper() {}
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        let modules: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Module)
            .collect();
        assert_eq!(modules.len(), 1);
        assert_eq!(modules[0].name, "utils");
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"
use std::collections::HashMap;
use std::io::Result as IoResult;
use crate::utils;
"#;
        let tree = parse_rust(source);
        let imports = RustExtractor::extract_imports(&tree, source).unwrap();

        assert!(!imports.is_empty());
        assert!(imports.iter().any(|i| i.path.contains("HashMap")));
    }

    #[test]
    fn test_extract_calls() {
        let source = r#"
fn main() {
    println!("Hello");
    let x = foo();
    bar.baz();
    module::func();
}
"#;
        let tree = parse_rust(source);
        let calls = RustExtractor::extract_calls(&tree, source).unwrap();

        assert!(calls.iter().any(|c| c.callee == "foo"));
        assert!(calls.iter().any(|c| c.callee == "baz" && c.is_method));
    }

    #[test]
    fn test_extract_ffi_markers() {
        let source = r#"
extern "C" {
    fn external_function();
}

extern crate libc;
"#;
        let tree = parse_rust(source);
        let markers = RustExtractor::extract_ffi_markers(&tree, source).unwrap();

        assert!(!markers.is_empty());
        assert!(markers.iter().any(|m| m.abi.as_deref() == Some("C")));
    }

    #[test]
    fn test_visibility_parsing() {
        assert_eq!(parse_visibility("pub"), Visibility::Public);
        assert_eq!(parse_visibility("pub(crate)"), Visibility::Crate);
        assert_eq!(
            parse_visibility("pub(super)"),
            Visibility::Restricted("super".to_string())
        );
        assert_eq!(
            parse_visibility("pub(self)"),
            Visibility::Restricted("self".to_string())
        );
    }

    #[test]
    fn test_extract_constant() {
        let source = r#"
pub const MAX_SIZE: usize = 100;
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "MAX_SIZE");
        assert_eq!(symbols[0].kind, SymbolKind::Variable);
    }

    #[test]
    fn test_extract_macro() {
        let source = r#"
macro_rules! my_macro {
    () => {};
}
"#;
        let tree = parse_rust(source);
        let symbols = RustExtractor::extract_symbols(&tree, source, "test.rs").unwrap();

        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "my_macro");
        assert_eq!(symbols[0].kind, SymbolKind::Macro);
    }
}
