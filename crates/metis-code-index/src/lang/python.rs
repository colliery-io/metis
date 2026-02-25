//! Python-specific symbol extraction using tree-sitter.
//!
//! Extracts classes, functions, imports, and FFI patterns from Python source code.

use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, Tree};

use crate::symbols::{Symbol, SymbolKind, Visibility};

/// Import statement from Python source.
#[derive(Debug, Clone)]
pub struct Import {
    /// The module path (e.g., "os.path" or "typing")
    pub module: String,
    /// Imported name (for `from x import y`)
    pub name: Option<String>,
    /// Alias if present (`import x as y`)
    pub alias: Option<String>,
    /// Line number
    pub line: usize,
}

/// Function/method call.
#[derive(Debug, Clone)]
pub struct Call {
    /// The callee expression
    pub callee: String,
    /// Line number
    pub line: usize,
    /// Whether this is a method call (uses `.` syntax)
    pub is_method: bool,
}

/// FFI marker detected in Python code.
#[derive(Debug, Clone)]
pub enum FFIMarker {
    /// ctypes library loading (cdll.LoadLibrary, CDLL, etc.)
    Ctypes {
        library: Option<String>,
        line: usize,
    },
    /// cffi usage (ffi.cdef, ffi.dlopen, etc.)
    Cffi { method: String, line: usize },
}

/// Compiled tree-sitter queries for Python.
struct PythonQueries {
    symbols: Query,
    imports: Query,
    calls: Query,
    ffi: Query,
}

static PYTHON_QUERIES: OnceLock<PythonQueries> = OnceLock::new();

fn get_queries() -> &'static PythonQueries {
    PYTHON_QUERIES.get_or_init(|| {
        let language = tree_sitter_python::LANGUAGE.into();

        let symbols_query = Query::new(
            &language,
            r#"
            ;; Classes
            (class_definition
              name: (identifier) @class_name
              body: (block) @class_body) @class

            ;; Top-level functions
            (module
              (function_definition
                name: (identifier) @func_name
                parameters: (parameters) @func_params
                body: (block) @func_body) @function)

            ;; Methods (functions inside class body)
            (class_definition
              body: (block
                (function_definition
                  name: (identifier) @method_name
                  parameters: (parameters) @method_params
                  body: (block) @method_body) @method))

            ;; Decorated definitions
            (decorated_definition
              definition: (function_definition
                name: (identifier) @decorated_func_name)) @decorated_func

            (decorated_definition
              definition: (class_definition
                name: (identifier) @decorated_class_name)) @decorated_class
            "#,
        )
        .expect("Invalid Python symbols query");

        let imports_query = Query::new(
            &language,
            r#"
            ;; import x
            (import_statement
              name: (dotted_name) @module) @import

            ;; import x as y
            (import_statement
              name: (aliased_import
                name: (dotted_name) @module
                alias: (identifier) @alias)) @import_alias

            ;; from x import y
            (import_from_statement
              module_name: (dotted_name) @from_module
              name: (dotted_name) @import_name) @from_import

            ;; from x import y as z
            (import_from_statement
              module_name: (dotted_name) @from_module_alias
              name: (aliased_import
                name: (dotted_name) @import_name_alias
                alias: (identifier) @import_alias)) @from_import_alias

            ;; from x import *
            (import_from_statement
              module_name: (dotted_name) @from_module_star
              (wildcard_import) @star) @from_import_star
            "#,
        )
        .expect("Invalid Python imports query");

        let calls_query = Query::new(
            &language,
            r#"
            ;; Simple function calls
            (call
              function: (identifier) @func_callee) @func_call

            ;; Method calls
            (call
              function: (attribute
                object: (_) @receiver
                attribute: (identifier) @method_callee)) @method_call

            ;; Chained calls
            (call
              function: (attribute
                attribute: (identifier) @chain_callee)) @chain_call
            "#,
        )
        .expect("Invalid Python calls query");

        let ffi_query = Query::new(
            &language,
            r#"
            ;; ctypes.CDLL / ctypes.cdll.LoadLibrary
            (call
              function: (attribute
                attribute: (identifier) @ctypes_method)
              arguments: (argument_list
                (string) @ctypes_lib)?) @ctypes_call

            ;; cffi ffi.cdef / ffi.dlopen
            (call
              function: (attribute
                object: (identifier) @ffi_obj
                attribute: (identifier) @ffi_method)) @ffi_call
            "#,
        )
        .expect("Invalid Python FFI query");

        PythonQueries {
            symbols: symbols_query,
            imports: imports_query,
            calls: calls_query,
            ffi: ffi_query,
        }
    })
}

/// Python-specific symbol extractor.
pub struct PythonExtractor;

impl PythonExtractor {
    /// Extract symbols from a Python source file.
    pub fn extract_symbols(
        tree: &Tree,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, String> {
        let queries = get_queries();
        let source_bytes = source.as_bytes();
        let mut symbols = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.symbols, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.symbols.capture_names()[capture.index as usize];

                match capture_name {
                    "class" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let mut symbol = Symbol::new(
                                name.clone(),
                                SymbolKind::Class,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            );

                            if let Some(body) = node.child_by_field_name("body") {
                                if let Some(docstring) = Self::extract_docstring(&body, source) {
                                    symbol = symbol.with_doc_comment(docstring);
                                }
                            }

                            symbol = symbol.with_visibility(Self::visibility_from_name(&name));

                            symbols.push(symbol);
                        }
                    }
                    "function" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let mut symbol = Symbol::new(
                                name.clone(),
                                SymbolKind::Function,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            );

                            if let Some(params) = node.child_by_field_name("parameters") {
                                let params_text = Self::node_text(&params, source);
                                let return_type = node
                                    .child_by_field_name("return_type")
                                    .map(|n| Self::node_text(&n, source));

                                let sig = match return_type {
                                    Some(ret) => format!("def {}{} -> {}", name, params_text, ret),
                                    None => format!("def {}{}", name, params_text),
                                };
                                symbol = symbol.with_signature(sig);
                            }

                            if let Some(body) = node.child_by_field_name("body") {
                                if let Some(docstring) = Self::extract_docstring(&body, source) {
                                    symbol = symbol.with_doc_comment(docstring);
                                }
                            }

                            symbol = symbol.with_visibility(Self::visibility_from_name(&name));
                            symbols.push(symbol);
                        }
                    }
                    "method" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let mut symbol = Symbol::new(
                                name.clone(),
                                SymbolKind::Method,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            );

                            if let Some(params) = node.child_by_field_name("parameters") {
                                let params_text = Self::node_text(&params, source);
                                let sig = format!("def {}{}", name, params_text);
                                symbol = symbol.with_signature(sig);
                            }

                            if let Some(body) = node.child_by_field_name("body") {
                                if let Some(docstring) = Self::extract_docstring(&body, source) {
                                    symbol = symbol.with_doc_comment(docstring);
                                }
                            }

                            symbol = symbol.with_visibility(Self::visibility_from_name(&name));
                            symbols.push(symbol);
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(symbols)
    }

    /// Extract import statements.
    pub fn extract_imports(tree: &Tree, source: &str) -> Result<Vec<Import>, String> {
        let queries = get_queries();
        let source_bytes = source.as_bytes();
        let mut imports = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut module = String::new();
            let mut name = None;
            let mut alias = None;
            let mut line = 0;

            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.imports.capture_names()[capture.index as usize];

                match capture_name {
                    "module" | "from_module" | "from_module_alias" | "from_module_star" => {
                        module = Self::node_text(&node, source);
                        line = node.start_position().row + 1;
                    }
                    "import_name" | "import_name_alias" => {
                        name = Some(Self::node_text(&node, source));
                    }
                    "alias" | "import_alias" => {
                        alias = Some(Self::node_text(&node, source));
                    }
                    "star" => {
                        name = Some("*".to_string());
                    }
                    _ => {}
                }
            }

            if !module.is_empty() {
                imports.push(Import {
                    module,
                    name,
                    alias,
                    line,
                });
            }
        }

        Ok(imports)
    }

    /// Extract function/method calls.
    pub fn extract_calls(tree: &Tree, source: &str) -> Result<Vec<Call>, String> {
        let queries = get_queries();
        let source_bytes = source.as_bytes();
        let mut calls = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.calls, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut callee = String::new();
            let mut line = 0;
            let mut is_method = false;

            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.calls.capture_names()[capture.index as usize];

                match capture_name {
                    "func_callee" => {
                        callee = Self::node_text(&node, source);
                        line = node.start_position().row + 1;
                        is_method = false;
                    }
                    "method_callee" | "chain_callee" => {
                        callee = Self::node_text(&node, source);
                        line = node.start_position().row + 1;
                        is_method = true;
                    }
                    _ => {}
                }
            }

            if !callee.is_empty() {
                calls.push(Call {
                    callee,
                    line,
                    is_method,
                });
            }
        }

        Ok(calls)
    }

    /// Extract FFI markers (ctypes, cffi usage).
    pub fn extract_ffi_markers(tree: &Tree, source: &str) -> Result<Vec<FFIMarker>, String> {
        let queries = get_queries();
        let source_bytes = source.as_bytes();
        let mut markers = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.ffi, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            let mut ctypes_method = None;
            let mut ctypes_lib = None;
            let mut ffi_method = None;
            let mut ffi_obj = None;
            let mut line = 0;

            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.ffi.capture_names()[capture.index as usize];

                match capture_name {
                    "ctypes_method" => {
                        ctypes_method = Some(Self::node_text(&node, source));
                        line = node.start_position().row + 1;
                    }
                    "ctypes_lib" => {
                        let text = Self::node_text(&node, source);
                        ctypes_lib = Some(text.trim_matches(|c| c == '"' || c == '\'').to_string());
                    }
                    "ffi_method" => {
                        ffi_method = Some(Self::node_text(&node, source));
                        line = node.start_position().row + 1;
                    }
                    "ffi_obj" => {
                        ffi_obj = Some(Self::node_text(&node, source));
                    }
                    _ => {}
                }
            }

            if let Some(method) = ctypes_method {
                if matches!(
                    method.as_str(),
                    "LoadLibrary" | "CDLL" | "cdll" | "WinDLL" | "windll" | "OleDLL" | "PyDLL"
                ) {
                    markers.push(FFIMarker::Ctypes {
                        library: ctypes_lib,
                        line,
                    });
                }
            }

            if let Some(method) = ffi_method {
                if let Some(obj) = ffi_obj {
                    if obj == "ffi"
                        && matches!(
                            method.as_str(),
                            "cdef" | "dlopen" | "verify" | "new" | "cast"
                        )
                    {
                        markers.push(FFIMarker::Cffi { method, line });
                    }
                }
            }
        }

        Ok(markers)
    }

    /// Get text content of a node.
    fn node_text(node: &tree_sitter::Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }

    /// Extract docstring from a block (first string literal).
    fn extract_docstring(block: &tree_sitter::Node, source: &str) -> Option<String> {
        let mut cursor = block.walk();
        cursor.goto_first_child();

        loop {
            let node = cursor.node();
            if node.kind() == "expression_statement" {
                if let Some(child) = node.child(0) {
                    if child.kind() == "string" {
                        let text = Self::node_text(&child, source);
                        let trimmed = text
                            .trim_start_matches("\"\"\"")
                            .trim_end_matches("\"\"\"")
                            .trim_start_matches("'''")
                            .trim_end_matches("'''")
                            .trim_start_matches('"')
                            .trim_end_matches('"')
                            .trim_start_matches('\'')
                            .trim_end_matches('\'')
                            .trim();
                        return Some(trimmed.to_string());
                    }
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        None
    }

    /// Determine visibility from Python naming convention.
    fn visibility_from_name(name: &str) -> Visibility {
        if name.starts_with('_') && !name.ends_with("__") {
            // Single underscore (conventionally private) or double underscore (name mangled)
            Visibility::Private
        } else {
            Visibility::Public
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_python(source: &str) -> Tree {
        let mut parser = Parser::new();
        let parsed = parser
            .parse_source(source, crate::parser::Language::Python)
            .unwrap();
        parsed.tree
    }

    #[test]
    fn test_extract_class() {
        let source = r#"
class MyClass:
    """A simple class."""

    def __init__(self, value):
        self.value = value

    def get_value(self):
        return self.value
"#;
        let tree = parse_python(source);
        let symbols = PythonExtractor::extract_symbols(&tree, source, "test.py").unwrap();

        let class = symbols.iter().find(|s| s.kind == SymbolKind::Class);
        assert!(class.is_some(), "Should find class");
        let class = class.unwrap();
        assert_eq!(class.name, "MyClass");
        assert!(class
            .doc_comment
            .as_ref()
            .is_some_and(|d| d.contains("simple class")));
    }

    #[test]
    fn test_extract_function() {
        let source = r#"
def greet(name: str) -> str:
    """Return a greeting message."""
    return f"Hello, {name}!"

def _private_func():
    pass
"#;
        let tree = parse_python(source);
        let symbols = PythonExtractor::extract_symbols(&tree, source, "test.py").unwrap();

        let functions: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();

        assert_eq!(functions.len(), 2, "Should find 2 functions");

        let greet = functions.iter().find(|f| f.name == "greet").unwrap();
        assert!(greet
            .signature
            .as_ref()
            .is_some_and(|s| s.contains("name: str")));
        assert_eq!(greet.visibility, Visibility::Public);

        let private = functions
            .iter()
            .find(|f| f.name == "_private_func")
            .unwrap();
        assert_eq!(private.visibility, Visibility::Private);
    }

    #[test]
    fn test_extract_methods() {
        let source = r#"
class Calculator:
    def add(self, a, b):
        return a + b

    def __private_method(self):
        pass
"#;
        let tree = parse_python(source);
        let symbols = PythonExtractor::extract_symbols(&tree, source, "test.py").unwrap();

        let methods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Method)
            .collect();

        assert!(!methods.is_empty(), "Should find at least 1 method");
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"
import os
import sys as system
from typing import List, Optional
from collections import defaultdict as dd
from pathlib import *
"#;
        let tree = parse_python(source);
        let imports = PythonExtractor::extract_imports(&tree, source).unwrap();

        assert!(
            imports.iter().any(|i| i.module == "os"),
            "Should find 'os' import"
        );
        assert!(
            imports
                .iter()
                .any(|i| i.module == "sys" && i.alias.as_deref() == Some("system")),
            "Should find 'sys as system' import"
        );
        assert!(
            imports.iter().any(|i| i.module == "typing"),
            "Should find 'from typing' import"
        );
    }

    #[test]
    fn test_extract_calls() {
        let source = r#"
def main():
    print("hello")
    result = calculate(1, 2)
    obj.method()
"#;
        let tree = parse_python(source);
        let calls = PythonExtractor::extract_calls(&tree, source).unwrap();

        assert!(
            calls.iter().any(|c| c.callee == "print" && !c.is_method),
            "Should find 'print' call"
        );
        assert!(
            calls
                .iter()
                .any(|c| c.callee == "calculate" && !c.is_method),
            "Should find 'calculate' call"
        );
        assert!(
            calls.iter().any(|c| c.callee == "method" && c.is_method),
            "Should find 'method' call"
        );
    }

    #[test]
    fn test_visibility_convention() {
        assert_eq!(
            PythonExtractor::visibility_from_name("public"),
            Visibility::Public
        );
        assert_eq!(
            PythonExtractor::visibility_from_name("_private"),
            Visibility::Private
        );
        assert_eq!(
            PythonExtractor::visibility_from_name("__mangled"),
            Visibility::Private
        );
        assert_eq!(
            PythonExtractor::visibility_from_name("__dunder__"),
            Visibility::Public
        );
    }

    #[test]
    fn test_extract_ffi_markers() {
        let source = r#"
import ctypes

lib = ctypes.CDLL("./mylib.so")
lib2 = ctypes.cdll.LoadLibrary("./other.so")
"#;
        let tree = parse_python(source);
        let markers = PythonExtractor::extract_ffi_markers(&tree, source).unwrap();

        let ctypes_count = markers
            .iter()
            .filter(|m| matches!(m, FFIMarker::Ctypes { .. }))
            .count();

        assert!(ctypes_count >= 1, "Should find at least 1 ctypes marker");
    }
}
