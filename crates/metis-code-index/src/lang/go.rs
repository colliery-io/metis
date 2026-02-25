//! Go symbol extraction using tree-sitter.
//!
//! Extracts functions, methods, structs, interfaces, type definitions,
//! constants, variables, and imports from Go source code.
//! Handles Go-specific patterns like method receivers and capitalized-name
//! export conventions.

use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, Tree};

use crate::symbols::{Symbol, SymbolKind, Visibility};

/// Import statement from Go source.
#[derive(Debug, Clone)]
pub struct Import {
    /// The import path (e.g., "fmt", "github.com/foo/bar")
    pub path: String,
    /// Optional alias (e.g., `myalias "pkg/path"`)
    pub alias: Option<String>,
    /// Line number
    pub line: usize,
}

/// Compiled tree-sitter queries for Go.
struct GoQueries {
    symbols: Query,
    imports: Query,
}

static GO_QUERIES: OnceLock<Result<GoQueries, String>> = OnceLock::new();

fn get_queries() -> Result<&'static GoQueries, &'static str> {
    GO_QUERIES
        .get_or_init(|| {
            let language: tree_sitter::Language = tree_sitter_go::LANGUAGE.into();
            let symbols = Query::new(&language, SYMBOLS_QUERY)
                .map_err(|e| format!("Failed to compile Go symbols query: {e}"))?;
            let imports = Query::new(&language, IMPORTS_QUERY)
                .map_err(|e| format!("Failed to compile Go imports query: {e}"))?;
            Ok(GoQueries { symbols, imports })
        })
        .as_ref()
        .map_err(|e| e.as_str())
}

/// Tree-sitter query for extracting Go symbols.
const SYMBOLS_QUERY: &str = r#"
; Functions
(function_declaration
  name: (identifier) @name
  parameters: (parameter_list) @params) @function

; Methods with receiver
(method_declaration
  receiver: (parameter_list) @receiver
  name: (field_identifier) @name
  parameters: (parameter_list) @params) @method

; Struct types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (struct_type))) @struct

; Interface types
(type_declaration
  (type_spec
    name: (type_identifier) @name
    type: (interface_type))) @interface

; Generic type definitions (also matches struct/interface, dedup in code)
(type_declaration
  (type_spec
    name: (type_identifier) @type_name)) @type_def

; Constants
(const_declaration
  (const_spec
    name: (identifier) @const_name)) @constant

; Variables
(var_declaration
  (var_spec
    name: (identifier) @var_name)) @variable
"#;

/// Tree-sitter query for extracting Go import statements.
const IMPORTS_QUERY: &str = r#"
; Import specs (matches both single imports and items in import blocks)
(import_spec
  path: (interpreted_string_literal) @path) @import
"#;

/// Go symbol extractor.
///
/// Handles Go source files (.go), extracting functions, methods with receivers,
/// struct/interface definitions, type definitions, constants, and variables.
/// Uses Go's naming convention (capitalized = exported) for visibility.
pub struct GoExtractor;

impl GoExtractor {
    /// Extract symbols from a parsed Go syntax tree.
    pub fn extract_symbols(
        tree: &Tree,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, String> {
        let queries = get_queries()?;
        let source_bytes = source.as_bytes();
        let mut symbols = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.symbols, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut name = None;
            let mut kind = None;
            let mut start_line = 0;
            let mut end_line = 0;
            let mut params = None;
            let mut receiver = None;
            let mut outer_node = None;

            for capture in match_.captures {
                let capture_name = queries.symbols.capture_names()[capture.index as usize];
                let node = capture.node;
                let text = node.utf8_text(source_bytes).unwrap_or("");

                match capture_name {
                    "name" | "type_name" | "const_name" | "var_name" => {
                        name = Some(text.to_string());
                    }
                    "params" => {
                        params = Some(text.to_string());
                    }
                    "receiver" => {
                        receiver = Some(text.to_string());
                    }
                    "function" => {
                        kind = Some(SymbolKind::Function);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "method" => {
                        kind = Some(SymbolKind::Method);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "struct" => {
                        kind = Some(SymbolKind::Struct);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "interface" => {
                        kind = Some(SymbolKind::Interface);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "type_def" => {
                        kind = Some(SymbolKind::Type);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "constant" => {
                        kind = Some(SymbolKind::Variable);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    "variable" => {
                        kind = Some(SymbolKind::Variable);
                        start_line = node.start_position().row + 1;
                        end_line = node.end_position().row + 1;
                        outer_node = Some(node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(kind)) = (name, kind) {
                let visibility = go_visibility(&name);

                // Build signature for functions and methods
                let signature = match kind {
                    SymbolKind::Function => {
                        let return_type = outer_node
                            .and_then(|n| n.child_by_field_name("result"))
                            .map(|n| node_text(&n, source));
                        build_func_signature(&name, params.as_deref(), None, return_type.as_deref())
                    }
                    SymbolKind::Method => {
                        let return_type = outer_node
                            .and_then(|n| n.child_by_field_name("result"))
                            .map(|n| node_text(&n, source));
                        build_func_signature(
                            &name,
                            params.as_deref(),
                            receiver.as_deref(),
                            return_type.as_deref(),
                        )
                    }
                    _ => None,
                };

                // Extract doc comment
                let doc_comment = outer_node.and_then(|n| extract_go_doc(&n, source));

                let mut symbol = Symbol::new(name, kind, file_path, start_line, end_line)
                    .with_visibility(visibility);

                if let Some(sig) = signature {
                    symbol = symbol.with_signature(sig);
                }
                if let Some(doc) = doc_comment {
                    symbol = symbol.with_doc_comment(doc);
                }

                symbols.push(symbol);
            }
        }

        deduplicate_symbols(&mut symbols);
        Ok(symbols)
    }

    /// Extract import statements from a parsed Go syntax tree.
    pub fn extract_imports(tree: &Tree, source: &str) -> Result<Vec<Import>, String> {
        let queries = get_queries()?;
        let source_bytes = source.as_bytes();
        let mut imports = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut path = None;
            let mut line = 0;
            let mut import_node = None;

            for capture in match_.captures {
                let capture_name = queries.imports.capture_names()[capture.index as usize];
                let node = capture.node;

                match capture_name {
                    "path" => {
                        let text = node.utf8_text(source_bytes).unwrap_or("");
                        // Remove surrounding quotes
                        path = Some(text.trim_matches('"').to_string());
                        line = node.start_position().row + 1;
                    }
                    "import" => {
                        import_node = Some(node);
                    }
                    _ => {}
                }
            }

            if let Some(path) = path {
                // Check for alias by looking at the import_spec node
                let alias = import_node.and_then(|n| {
                    n.child_by_field_name("name")
                        .map(|name_node| node_text(&name_node, source))
                        .filter(|a| a != "." && a != "_")
                });

                imports.push(Import { path, alias, line });
            }
        }

        Ok(imports)
    }
}

/// Determine visibility based on Go naming convention.
/// Capitalized names are exported (public), lowercase are unexported (private).
fn go_visibility(name: &str) -> Visibility {
    if name.starts_with(|c: char| c.is_uppercase()) {
        Visibility::Public
    } else {
        Visibility::Private
    }
}

/// Build a function/method signature string.
fn build_func_signature(
    name: &str,
    params: Option<&str>,
    receiver: Option<&str>,
    return_type: Option<&str>,
) -> Option<String> {
    let params_str = params.unwrap_or("()");
    let mut sig = String::from("func ");

    if let Some(recv) = receiver {
        sig.push_str(recv);
        sig.push(' ');
    }

    sig.push_str(name);
    sig.push_str(params_str);

    if let Some(ret) = return_type {
        sig.push(' ');
        sig.push_str(ret);
    }

    Some(sig)
}

/// Extract Go doc comments (// lines preceding a declaration).
fn extract_go_doc(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut sibling = node.prev_sibling();

    while let Some(sib) = sibling {
        if sib.kind() == "comment" {
            let text = sib.utf8_text(source.as_bytes()).unwrap_or("").trim();
            if let Some(content) = text.strip_prefix("//") {
                doc_lines.push(content.trim().to_string());
                sibling = sib.prev_sibling();
                continue;
            }
        }
        break;
    }

    if doc_lines.is_empty() {
        return None;
    }

    doc_lines.reverse();
    Some(doc_lines.join("\n"))
}

/// Deduplicate symbols by (name, start_line).
/// Struct/Interface are more specific than Type, so keep them when both exist.
fn deduplicate_symbols(symbols: &mut Vec<Symbol>) {
    symbols.sort_by(|a, b| {
        a.start_line
            .cmp(&b.start_line)
            .then(a.name.cmp(&b.name))
            .then(kind_priority(&a.kind).cmp(&kind_priority(&b.kind)))
    });
    symbols.dedup_by(|a, b| a.name == b.name && a.start_line == b.start_line);
}

/// Priority for deduplication: lower is better (kept first).
fn kind_priority(kind: &SymbolKind) -> u8 {
    match kind {
        SymbolKind::Struct | SymbolKind::Interface => 0,
        SymbolKind::Type => 1,
        _ => 0,
    }
}

fn node_text(node: &tree_sitter::Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Language, Parser};

    fn parse_go(source: &str) -> Tree {
        let mut parser = Parser::new();
        let parsed = parser.parse_source(source, Language::Go).unwrap();
        parsed.tree
    }

    #[test]
    fn test_extract_function() {
        let source = r#"package main

// Add returns the sum of two integers.
func Add(a int, b int) int {
    return a + b
}

func helper(x string) {
    println(x)
}
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "main.go").unwrap();

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert_eq!(funcs.len(), 2);

        let add = funcs.iter().find(|f| f.name == "Add").unwrap();
        assert_eq!(add.visibility, Visibility::Public);
        assert!(add.signature.is_some());
        assert!(add.signature.as_ref().unwrap().contains("func Add"));
        assert!(add.doc_comment.is_some());

        let helper = funcs.iter().find(|f| f.name == "helper").unwrap();
        assert_eq!(helper.visibility, Visibility::Private);
    }

    #[test]
    fn test_extract_method_with_receiver() {
        let source = r#"package main

type Server struct {
    port int
}

// Start begins listening on the configured port.
func (s *Server) Start() error {
    return nil
}

func (s *Server) stop() {
}
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "server.go").unwrap();

        let methods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Method)
            .collect();
        assert_eq!(methods.len(), 2);

        let start = methods.iter().find(|m| m.name == "Start").unwrap();
        assert_eq!(start.visibility, Visibility::Public);
        assert!(start.signature.is_some());
        let sig = start.signature.as_ref().unwrap();
        assert!(
            sig.contains("*Server"),
            "signature should contain receiver: {sig}"
        );
        assert!(
            sig.contains("Start"),
            "signature should contain name: {sig}"
        );

        let stop = methods.iter().find(|m| m.name == "stop").unwrap();
        assert_eq!(stop.visibility, Visibility::Private);
    }

    #[test]
    fn test_extract_struct() {
        let source = r#"package main

// Config holds application configuration.
type Config struct {
    Host string
    Port int
    Debug bool
}

type internalState struct {
    count int
}
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "config.go").unwrap();

        let structs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Struct)
            .collect();
        assert_eq!(structs.len(), 2);

        let config = structs.iter().find(|s| s.name == "Config").unwrap();
        assert_eq!(config.visibility, Visibility::Public);
        assert!(config.doc_comment.is_some());

        let internal = structs.iter().find(|s| s.name == "internalState").unwrap();
        assert_eq!(internal.visibility, Visibility::Private);
    }

    #[test]
    fn test_extract_interface() {
        let source = r#"package main

// Reader is the interface for reading data.
type Reader interface {
    Read(p []byte) (n int, err error)
}

type Writer interface {
    Write(p []byte) (n int, err error)
}

type ReadWriter interface {
    Reader
    Writer
}
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "io.go").unwrap();

        let interfaces: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Interface)
            .collect();
        assert_eq!(interfaces.len(), 3);
        assert!(interfaces.iter().any(|i| i.name == "Reader"));
        assert!(interfaces.iter().any(|i| i.name == "Writer"));
        assert!(interfaces.iter().any(|i| i.name == "ReadWriter"));
    }

    #[test]
    fn test_extract_type_definition() {
        let source = r#"package main

type UserID string

type Callback func(int) error

type StringSlice []string
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "types.go").unwrap();

        let types: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Type)
            .collect();
        assert_eq!(types.len(), 3);
        assert!(types.iter().any(|t| t.name == "UserID"));
        assert!(types.iter().any(|t| t.name == "Callback"));
        assert!(types.iter().any(|t| t.name == "StringSlice"));
    }

    #[test]
    fn test_extract_constants() {
        let source = r#"package main

const MaxRetries = 3

const (
    StatusActive = "active"
    StatusInactive = "inactive"
)
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "const.go").unwrap();

        let constants: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Variable && s.name != "_")
            .collect();
        assert!(constants.len() >= 3, "found {} constants", constants.len());
        assert!(constants.iter().any(|c| c.name == "MaxRetries"));
        assert!(constants.iter().any(|c| c.name == "StatusActive"));
        assert!(constants.iter().any(|c| c.name == "StatusInactive"));
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"package main

import "fmt"

import (
    "os"
    "strings"
    myio "io"
    _ "net/http/pprof"
)
"#;
        let tree = parse_go(source);
        let imports = GoExtractor::extract_imports(&tree, source).unwrap();

        assert!(imports.len() >= 4, "found {} imports", imports.len());
        assert!(imports.iter().any(|i| i.path == "fmt"));
        assert!(imports.iter().any(|i| i.path == "os"));
        assert!(imports.iter().any(|i| i.path == "strings"));
        assert!(imports
            .iter()
            .any(|i| i.path == "io" && i.alias.as_deref() == Some("myio")));
    }

    #[test]
    fn test_go_visibility() {
        assert_eq!(go_visibility("Exported"), Visibility::Public);
        assert_eq!(go_visibility("unexported"), Visibility::Private);
        assert_eq!(go_visibility("_blank"), Visibility::Private);
        assert_eq!(go_visibility("URL"), Visibility::Public);
    }

    #[test]
    fn test_struct_not_duplicated_as_type() {
        let source = r#"package main

type Config struct {
    Host string
}

type UserID string
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "test.go").unwrap();

        // Config should appear as Struct, not also as Type
        let config_symbols: Vec<_> = symbols.iter().filter(|s| s.name == "Config").collect();
        assert_eq!(config_symbols.len(), 1);
        assert_eq!(config_symbols[0].kind, SymbolKind::Struct);

        // UserID should appear as Type
        let userid_symbols: Vec<_> = symbols.iter().filter(|s| s.name == "UserID").collect();
        assert_eq!(userid_symbols.len(), 1);
        assert_eq!(userid_symbols[0].kind, SymbolKind::Type);
    }

    #[test]
    fn test_mixed_go_file() {
        let source = r#"package main

import (
    "fmt"
    "net/http"
)

// Handler handles HTTP requests.
type Handler struct {
    mux *http.ServeMux
}

// ServeHTTP implements the http.Handler interface.
func (h *Handler) ServeHTTP(w http.ResponseWriter, r *http.Request) {
    h.mux.ServeHTTP(w, r)
}

type Middleware func(http.Handler) http.Handler

func NewHandler() *Handler {
    return &Handler{mux: http.NewServeMux()}
}

const DefaultPort = 8080

var globalHandler *Handler
"#;
        let tree = parse_go(source);
        let symbols = GoExtractor::extract_symbols(&tree, source, "handler.go").unwrap();

        assert!(
            symbols
                .iter()
                .any(|s| s.name == "Handler" && s.kind == SymbolKind::Struct),
            "should find Handler struct"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "ServeHTTP" && s.kind == SymbolKind::Method),
            "should find ServeHTTP method"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "Middleware" && s.kind == SymbolKind::Type),
            "should find Middleware type"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "NewHandler" && s.kind == SymbolKind::Function),
            "should find NewHandler function"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "DefaultPort" && s.kind == SymbolKind::Variable),
            "should find DefaultPort constant"
        );
        assert!(
            symbols
                .iter()
                .any(|s| s.name == "globalHandler" && s.kind == SymbolKind::Variable),
            "should find globalHandler variable"
        );

        let imports = GoExtractor::extract_imports(&tree, source).unwrap();
        assert!(imports.iter().any(|i| i.path == "fmt"));
        assert!(imports.iter().any(|i| i.path == "net/http"));
    }
}
