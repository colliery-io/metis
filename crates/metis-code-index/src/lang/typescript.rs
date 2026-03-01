//! TypeScript/JavaScript symbol extraction using tree-sitter.
//!
//! Extracts functions, classes, interfaces, type aliases, enums, imports,
//! and method definitions from TypeScript and JavaScript source code.
//! Uses the TSX grammar for TypeScript files (superset that handles both .ts and .tsx)
//! and the JavaScript grammar for JS files.

use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, Tree};

use crate::parser::Language;
use crate::symbols::{compact_signature, Symbol, SymbolKind, Visibility};

/// Import statement from TypeScript/JavaScript source.
#[derive(Debug, Clone)]
pub struct Import {
    /// The module specifier (e.g., "react", "./utils", "@scope/pkg")
    pub source: String,
    /// Imported names (for `import { a, b } from ...`)
    pub names: Vec<String>,
    /// Default import name (for `import Foo from ...`)
    pub default_name: Option<String>,
    /// Namespace import (for `import * as ns from ...`)
    pub namespace: Option<String>,
    /// Line number
    pub line: usize,
}

/// Compiled tree-sitter queries for TypeScript.
struct TypeScriptQueries {
    symbols: Query,
    imports: Query,
}

/// Compiled tree-sitter queries for JavaScript.
struct JavaScriptQueries {
    symbols: Query,
    imports: Query,
}

static TS_QUERIES: OnceLock<Result<TypeScriptQueries, String>> = OnceLock::new();
static JS_QUERIES: OnceLock<Result<JavaScriptQueries, String>> = OnceLock::new();

fn get_ts_queries() -> Result<&'static TypeScriptQueries, &'static str> {
    TS_QUERIES
        .get_or_init(|| {
            let language: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TSX.into();
            let symbols = Query::new(&language, TS_SYMBOLS_QUERY)
                .map_err(|e| format!("Failed to compile TS symbols query: {e}"))?;
            let imports = Query::new(&language, TS_IMPORTS_QUERY)
                .map_err(|e| format!("Failed to compile TS imports query: {e}"))?;
            Ok(TypeScriptQueries { symbols, imports })
        })
        .as_ref()
        .map_err(|e| e.as_str())
}

fn get_js_queries() -> Result<&'static JavaScriptQueries, &'static str> {
    JS_QUERIES
        .get_or_init(|| {
            let language: tree_sitter::Language = tree_sitter_javascript::LANGUAGE.into();
            let symbols = Query::new(&language, JS_SYMBOLS_QUERY)
                .map_err(|e| format!("Failed to compile JS symbols query: {e}"))?;
            let imports = Query::new(&language, JS_IMPORTS_QUERY)
                .map_err(|e| format!("Failed to compile JS imports query: {e}"))?;
            Ok(JavaScriptQueries { symbols, imports })
        })
        .as_ref()
        .map_err(|e| e.as_str())
}

/// Tree-sitter query for extracting TypeScript symbols.
const TS_SYMBOLS_QUERY: &str = r#"
; Top-level function declarations
(function_declaration
  name: (identifier) @name) @function

; Class declarations
(class_declaration
  name: (type_identifier) @name) @class

; Interface declarations (TS-specific)
(interface_declaration
  name: (type_identifier) @name) @interface

; Type alias declarations (TS-specific)
(type_alias_declaration
  name: (type_identifier) @name) @type_alias

; Enum declarations (TS-specific)
(enum_declaration
  name: (identifier) @name) @enum

; Method definitions in classes
(class_declaration
  body: (class_body
    (method_definition
      name: (property_identifier) @method_name) @method))

; Exported function declarations
(export_statement
  declaration: (function_declaration
    name: (identifier) @export_func_name) @export_function)

; Exported class declarations
(export_statement
  declaration: (class_declaration
    name: (type_identifier) @export_class_name) @export_class)

; Exported interface declarations
(export_statement
  declaration: (interface_declaration
    name: (type_identifier) @export_interface_name) @export_interface)

; Exported type alias declarations
(export_statement
  declaration: (type_alias_declaration
    name: (type_identifier) @export_type_name) @export_type)

; Exported enum declarations
(export_statement
  declaration: (enum_declaration
    name: (identifier) @export_enum_name) @export_enum)

; Variable declarations with arrow functions (const foo = () => {})
(lexical_declaration
  (variable_declarator
    name: (identifier) @arrow_name
    value: (arrow_function)) @arrow_var)

; Exported variable declarations with arrow functions
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @export_arrow_name
      value: (arrow_function)) @export_arrow_var))
"#;

/// Tree-sitter query for extracting JavaScript symbols.
const JS_SYMBOLS_QUERY: &str = r#"
; Top-level function declarations
(function_declaration
  name: (identifier) @name) @function

; Class declarations
(class_declaration
  name: (identifier) @name) @class

; Method definitions in classes
(class_declaration
  body: (class_body
    (method_definition
      name: (property_identifier) @method_name) @method))

; Exported function declarations
(export_statement
  declaration: (function_declaration
    name: (identifier) @export_func_name) @export_function)

; Exported class declarations
(export_statement
  declaration: (class_declaration
    name: (identifier) @export_class_name) @export_class)

; Variable declarations with arrow functions (const foo = () => {})
(lexical_declaration
  (variable_declarator
    name: (identifier) @arrow_name
    value: (arrow_function)) @arrow_var)

; Exported variable declarations with arrow functions
(export_statement
  declaration: (lexical_declaration
    (variable_declarator
      name: (identifier) @export_arrow_name
      value: (arrow_function)) @export_arrow_var))
"#;

/// Tree-sitter query for extracting TypeScript imports.
const TS_IMPORTS_QUERY: &str = r#"
; import ... from "source"
(import_statement
  source: (string) @source) @import
"#;

/// Tree-sitter query for extracting JavaScript imports.
const JS_IMPORTS_QUERY: &str = r#"
; import ... from "source"
(import_statement
  source: (string) @source) @import
"#;

/// TypeScript/JavaScript symbol extractor.
///
/// Handles both TypeScript (.ts, .tsx) and JavaScript (.js, .jsx, .mjs, .cjs)
/// using language-specific compiled queries.
pub struct TypeScriptExtractor;

impl TypeScriptExtractor {
    /// Extract symbols from a parsed TypeScript or JavaScript syntax tree.
    pub fn extract_symbols(
        tree: &Tree,
        source: &str,
        file_path: &str,
        language: Language,
    ) -> Result<Vec<Symbol>, String> {
        match language {
            Language::TypeScript => Self::extract_ts_symbols(tree, source, file_path),
            Language::JavaScript => Self::extract_js_symbols(tree, source, file_path),
            _ => Err(format!(
                "Unsupported language for TypeScriptExtractor: {:?}",
                language
            )),
        }
    }

    fn extract_ts_symbols(
        tree: &Tree,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, String> {
        let queries = get_ts_queries()?;
        let source_bytes = source.as_bytes();
        let mut symbols = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.symbols, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.symbols.capture_names()[capture.index as usize];

                match capture_name {
                    "function" | "export_function" => {
                        if let Some(sym) = Self::extract_function_symbol(
                            &node,
                            source,
                            file_path,
                            capture_name.starts_with("export"),
                        ) {
                            symbols.push(sym);
                        }
                    }
                    "class" | "export_class" => {
                        if let Some(sym) = Self::extract_class_symbol(
                            &node,
                            source,
                            file_path,
                            capture_name.starts_with("export"),
                            true,
                        ) {
                            symbols.push(sym);
                        }
                    }
                    "interface" | "export_interface" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let is_export = capture_name.starts_with("export");
                            let sig = Self::build_interface_signature(&node, source);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Interface,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(if is_export {
                                Visibility::Public
                            } else {
                                Visibility::Private
                            });
                            if let Some(s) = sig {
                                symbol = symbol.with_signature(s);
                            }
                            symbols.push(symbol);
                        }
                    }
                    "type_alias" | "export_type" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let is_export = capture_name.starts_with("export");
                            let sig = Self::build_type_alias_signature(&node, source);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Type,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(if is_export {
                                Visibility::Public
                            } else {
                                Visibility::Private
                            });
                            if let Some(s) = sig {
                                symbol = symbol.with_signature(s);
                            }
                            symbols.push(symbol);
                        }
                    }
                    "enum" | "export_enum" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let is_export = capture_name.starts_with("export");
                            let sig = Self::build_enum_signature(&node, source);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Enum,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(if is_export {
                                Visibility::Public
                            } else {
                                Visibility::Private
                            });
                            if let Some(s) = sig {
                                symbol = symbol.with_signature(s);
                            }
                            symbols.push(symbol);
                        }
                    }
                    "method" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let signature = Self::build_method_signature(&node, source, &name);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Method,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(Visibility::Public);
                            if let Some(sig) = signature {
                                symbol = symbol.with_signature(sig);
                            }
                            symbols.push(symbol);
                        }
                    }
                    "arrow_var" | "export_arrow_var" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let is_export = capture_name.starts_with("export");
                            let sig = Self::build_arrow_signature(&node, source, &name);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Function,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(if is_export {
                                Visibility::Public
                            } else {
                                Visibility::Private
                            });
                            if let Some(s) = sig {
                                symbol = symbol.with_signature(s);
                            }
                            symbols.push(symbol);
                        }
                    }
                    _ => {}
                }
            }
        }

        // Deduplicate: exported declarations match both export and non-export patterns
        Self::deduplicate_symbols(&mut symbols);
        Ok(symbols)
    }

    fn extract_js_symbols(
        tree: &Tree,
        source: &str,
        file_path: &str,
    ) -> Result<Vec<Symbol>, String> {
        let queries = get_js_queries()?;
        let source_bytes = source.as_bytes();
        let mut symbols = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.symbols, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;
                let capture_name = queries.symbols.capture_names()[capture.index as usize];

                match capture_name {
                    "function" | "export_function" => {
                        if let Some(sym) = Self::extract_function_symbol(
                            &node,
                            source,
                            file_path,
                            capture_name.starts_with("export"),
                        ) {
                            symbols.push(sym);
                        }
                    }
                    "class" | "export_class" => {
                        if let Some(sym) = Self::extract_class_symbol(
                            &node,
                            source,
                            file_path,
                            capture_name.starts_with("export"),
                            false,
                        ) {
                            symbols.push(sym);
                        }
                    }
                    "method" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let signature = Self::build_method_signature(&node, source, &name);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Method,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(Visibility::Public);
                            if let Some(sig) = signature {
                                symbol = symbol.with_signature(sig);
                            }
                            symbols.push(symbol);
                        }
                    }
                    "arrow_var" | "export_arrow_var" => {
                        if let Some(name_node) = node.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let is_export = capture_name.starts_with("export");
                            let sig = Self::build_arrow_signature(&node, source, &name);
                            let mut symbol = Symbol::new(
                                name,
                                SymbolKind::Function,
                                file_path,
                                node.start_position().row + 1,
                                node.end_position().row + 1,
                            )
                            .with_visibility(if is_export {
                                Visibility::Public
                            } else {
                                Visibility::Private
                            });
                            if let Some(s) = sig {
                                symbol = symbol.with_signature(s);
                            }
                            symbols.push(symbol);
                        }
                    }
                    _ => {}
                }
            }
        }

        Self::deduplicate_symbols(&mut symbols);
        Ok(symbols)
    }

    /// Extract import statements from TypeScript or JavaScript.
    pub fn extract_imports(
        tree: &Tree,
        source: &str,
        language: Language,
    ) -> Result<Vec<Import>, String> {
        let source_bytes = source.as_bytes();
        let mut imports = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();

        match language {
            Language::TypeScript => {
                let queries = get_ts_queries()?;
                let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

                while let Some(match_) = matches.next() {
                    if let Some(import) =
                        Self::extract_import_from_match(match_, &queries.imports, source)
                    {
                        imports.push(import);
                    }
                }
            }
            Language::JavaScript => {
                let queries = get_js_queries()?;
                let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

                while let Some(match_) = matches.next() {
                    if let Some(import) =
                        Self::extract_import_from_match(match_, &queries.imports, source)
                    {
                        imports.push(import);
                    }
                }
            }
            _ => {
                return Err(format!(
                    "Unsupported language for TypeScriptExtractor: {:?}",
                    language
                ))
            }
        }

        Ok(imports)
    }

    fn extract_import_from_match(
        match_: &tree_sitter::QueryMatch,
        query: &Query,
        source: &str,
    ) -> Option<Import> {
        let mut module_source = None;
        let mut line = 0;
        let mut import_node = None;

        for capture in match_.captures {
            let capture_name = query.capture_names()[capture.index as usize];
            let node = capture.node;

            match capture_name {
                "source" => {
                    // Remove quotes from string literal
                    let text = Self::node_text(&node, source);
                    module_source = Some(text.trim_matches('"').trim_matches('\'').to_string());
                    line = node.start_position().row + 1;
                }
                "import" => {
                    import_node = Some(node);
                }
                _ => {}
            }
        }

        let source_str = module_source?;
        let mut names = Vec::new();
        let mut default_name = None;
        let mut namespace = None;

        // Walk the import node to extract named/default/namespace imports
        if let Some(import) = import_node {
            Self::extract_import_details(
                &import,
                source,
                &mut names,
                &mut default_name,
                &mut namespace,
            );
        }

        Some(Import {
            source: source_str,
            names,
            default_name,
            namespace,
            line,
        })
    }

    fn extract_import_details(
        import_node: &tree_sitter::Node,
        source: &str,
        names: &mut Vec<String>,
        default_name: &mut Option<String>,
        namespace: &mut Option<String>,
    ) {
        let mut cursor = import_node.walk();
        if !cursor.goto_first_child() {
            return;
        }

        loop {
            let node = cursor.node();
            if node.kind() == "import_clause" {
                // Walk children of import_clause
                let mut clause_cursor = node.walk();
                if clause_cursor.goto_first_child() {
                    loop {
                        let child = clause_cursor.node();
                        match child.kind() {
                            "identifier" => {
                                // Default import
                                *default_name = Some(Self::node_text(&child, source));
                            }
                            "named_imports" => {
                                // { a, b, c }
                                let mut import_cursor = child.walk();
                                if import_cursor.goto_first_child() {
                                    loop {
                                        let import_child = import_cursor.node();
                                        if import_child.kind() == "import_specifier" {
                                            if let Some(name_node) =
                                                import_child.child_by_field_name("name")
                                            {
                                                names.push(Self::node_text(&name_node, source));
                                            }
                                        }
                                        if !import_cursor.goto_next_sibling() {
                                            break;
                                        }
                                    }
                                }
                            }
                            "namespace_import" => {
                                // * as ns
                                let mut ns_cursor = child.walk();
                                if ns_cursor.goto_first_child() {
                                    loop {
                                        let ns_child = ns_cursor.node();
                                        if ns_child.kind() == "identifier" {
                                            *namespace = Some(Self::node_text(&ns_child, source));
                                        }
                                        if !ns_cursor.goto_next_sibling() {
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                        if !clause_cursor.goto_next_sibling() {
                            break;
                        }
                    }
                }
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    fn extract_function_symbol(
        node: &tree_sitter::Node,
        source: &str,
        file_path: &str,
        is_export: bool,
    ) -> Option<Symbol> {
        let name_node = node.child_by_field_name("name")?;
        let name = Self::node_text(&name_node, source);
        let signature = Self::build_function_signature(node, source, &name);

        let mut symbol = Symbol::new(
            name,
            SymbolKind::Function,
            file_path,
            node.start_position().row + 1,
            node.end_position().row + 1,
        )
        .with_visibility(if is_export {
            Visibility::Public
        } else {
            Visibility::Private
        });

        if let Some(sig) = signature {
            symbol = symbol.with_signature(sig);
        }

        Some(symbol)
    }

    fn extract_class_symbol(
        node: &tree_sitter::Node,
        source: &str,
        file_path: &str,
        is_export: bool,
        is_typescript: bool,
    ) -> Option<Symbol> {
        let name_node = node.child_by_field_name("name")?;
        let name = Self::node_text(&name_node, source);

        // Check for JSDoc comment above
        let doc_comment = Self::extract_jsdoc(node, source);

        let mut symbol = Symbol::new(
            name,
            SymbolKind::Class,
            file_path,
            node.start_position().row + 1,
            node.end_position().row + 1,
        )
        .with_visibility(if is_export {
            Visibility::Public
        } else {
            Visibility::Private
        });

        if let Some(doc) = doc_comment {
            symbol = symbol.with_doc_comment(doc);
        }

        // For TS, check if class has `implements` or `extends`
        if is_typescript {
            let mut heritage = Vec::new();
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    // Look for heritage clauses like "extends Base" or "implements IFoo"
                    if child.kind() == "class_heritage" {
                        heritage.push(Self::node_text(&child, source));
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            if !heritage.is_empty() {
                symbol = symbol.with_signature(heritage.join(" "));
            }
        }

        Some(symbol)
    }

    fn build_function_signature(
        node: &tree_sitter::Node,
        source: &str,
        name: &str,
    ) -> Option<String> {
        let params_node = node.child_by_field_name("parameters")?;
        let params = Self::node_text(&params_node, source);

        // Try to get return type (TypeScript only)
        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| Self::node_text(&n, source));

        match return_type {
            Some(ret) => Some(format!("function {}{}{}", name, params, ret)),
            None => Some(format!("function {}{}", name, params)),
        }
    }

    fn build_method_signature(
        node: &tree_sitter::Node,
        source: &str,
        name: &str,
    ) -> Option<String> {
        let params_node = node.child_by_field_name("parameters")?;
        let params = Self::node_text(&params_node, source);

        let return_type = node
            .child_by_field_name("return_type")
            .map(|n| Self::node_text(&n, source));

        match return_type {
            Some(ret) => Some(format!("{}{}{}", name, params, ret)),
            None => Some(format!("{}{}", name, params)),
        }
    }

    fn build_interface_signature(node: &tree_sitter::Node, source: &str) -> Option<String> {
        let body = node.child_by_field_name("body")?;
        let mut members = Vec::new();
        let mut cursor = body.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "property_signature" | "public_field_definition" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            let type_ann = child
                                .child_by_field_name("type")
                                .map(|n| Self::node_text(&n, source));
                            match type_ann {
                                Some(ty) => members.push(format!("{}: {}", name, ty)),
                                None => members.push(name),
                            }
                        }
                    }
                    "method_signature" => {
                        if let Some(name_node) = child.child_by_field_name("name") {
                            let name = Self::node_text(&name_node, source);
                            members.push(format!("{}()", name));
                        }
                    }
                    _ => {}
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        if members.is_empty() {
            return None;
        }
        Some(compact_signature(
            &format!("{{ {} }}", members.join(", ")),
            120,
        ))
    }

    fn build_type_alias_signature(node: &tree_sitter::Node, source: &str) -> Option<String> {
        let value = node.child_by_field_name("value")?;
        let text = Self::node_text(&value, source);
        Some(compact_signature(&format!("= {}", text), 120))
    }

    fn build_enum_signature(node: &tree_sitter::Node, source: &str) -> Option<String> {
        let body = node.child_by_field_name("body")?;
        let mut members = Vec::new();
        let mut cursor = body.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "enum_assignment" || child.kind() == "property_identifier" {
                    let text = Self::node_text(&child, source);
                    // enum_assignment has name = value, just take the name
                    if let Some(name_node) = child.child_by_field_name("name") {
                        members.push(Self::node_text(&name_node, source));
                    } else if child.kind() == "property_identifier" {
                        members.push(text);
                    }
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        if members.is_empty() {
            return None;
        }
        Some(compact_signature(&members.join(" | "), 120))
    }

    fn build_arrow_signature(
        node: &tree_sitter::Node,
        source: &str,
        name: &str,
    ) -> Option<String> {
        // variable_declarator > value: arrow_function
        let arrow_node = node.child_by_field_name("value")?;
        let params = arrow_node
            .child_by_field_name("parameters")
            .map(|n| Self::node_text(&n, source))?;
        let return_type = arrow_node
            .child_by_field_name("return_type")
            .map(|n| Self::node_text(&n, source));
        match return_type {
            Some(ret) => Some(format!("const {} = {}{}", name, params, ret)),
            None => Some(format!("const {} = {}", name, params)),
        }
    }

    fn extract_jsdoc(node: &tree_sitter::Node, source: &str) -> Option<String> {
        // Look for comment node immediately preceding the symbol
        let sibling = node.prev_sibling()?;
        if sibling.kind() != "comment" {
            return None;
        }
        let text = Self::node_text(&sibling, source);
        if !text.starts_with("/**") {
            return None;
        }
        // JSDoc comment - strip markers
        let cleaned = text
            .strip_prefix("/**")
            .and_then(|s| s.strip_suffix("*/"))
            .unwrap_or(&text)
            .lines()
            .map(|line| line.trim().trim_start_matches('*').trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        if cleaned.is_empty() {
            None
        } else {
            Some(cleaned)
        }
    }

    /// Deduplicate symbols by (name, start_line, kind).
    /// Exported declarations can match both the export pattern and the inner declaration.
    /// Keep the one with Public visibility when duplicates exist.
    fn deduplicate_symbols(symbols: &mut Vec<Symbol>) {
        symbols.sort_by(|a, b| a.start_line.cmp(&b.start_line).then(a.name.cmp(&b.name)));
        symbols.dedup_by(|a, b| {
            if a.name == b.name && a.start_line == b.start_line && a.kind == b.kind {
                // Keep the one with Public visibility
                if a.visibility == Visibility::Public {
                    b.visibility = Visibility::Public;
                }
                true
            } else {
                false
            }
        });
    }

    fn node_text(node: &tree_sitter::Node, source: &str) -> String {
        source[node.byte_range()].to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    fn parse_typescript(source: &str) -> Tree {
        let mut parser = Parser::new();
        let parsed = parser.parse_source(source, Language::TypeScript).unwrap();
        parsed.tree
    }

    fn parse_javascript(source: &str) -> Tree {
        let mut parser = Parser::new();
        let parsed = parser.parse_source(source, Language::JavaScript).unwrap();
        parsed.tree
    }

    // --- TypeScript tests ---

    #[test]
    fn test_extract_ts_function() {
        let source = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert_eq!(funcs.len(), 1);
        assert_eq!(funcs[0].name, "greet");
        assert!(funcs[0].signature.is_some());
    }

    #[test]
    fn test_extract_ts_class() {
        let source = r#"
export class UserService {
    private users: User[] = [];

    getUser(id: string): User | undefined {
        return this.users.find(u => u.id === id);
    }

    addUser(user: User): void {
        this.users.push(user);
    }
}
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let classes: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "UserService");
        assert_eq!(classes[0].visibility, Visibility::Public);

        let methods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Method)
            .collect();
        assert!(methods.len() >= 2, "Should find at least 2 methods");
        assert!(methods.iter().any(|m| m.name == "getUser"));
        assert!(methods.iter().any(|m| m.name == "addUser"));
    }

    #[test]
    fn test_extract_ts_interface() {
        let source = r#"
export interface User {
    id: string;
    name: string;
    email: string;
}

interface InternalConfig {
    debug: boolean;
}
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let interfaces: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Interface)
            .collect();
        assert_eq!(interfaces.len(), 2);
        let user = interfaces.iter().find(|i| i.name == "User").unwrap();
        assert_eq!(user.visibility, Visibility::Public);
        assert!(user.signature.is_some(), "Interface should have signature");
        let sig = user.signature.as_ref().unwrap();
        assert!(sig.contains("id"), "sig = {sig}");
        assert!(sig.contains("name"), "sig = {sig}");

        assert!(interfaces
            .iter()
            .any(|i| i.name == "InternalConfig" && i.visibility == Visibility::Private));
    }

    #[test]
    fn test_extract_ts_type_alias() {
        let source = r#"
export type Result<T> = { ok: true; value: T } | { ok: false; error: Error };

type UserId = string;
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let types: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Type)
            .collect();
        assert_eq!(types.len(), 2);

        let result_type = types.iter().find(|t| t.name == "Result").unwrap();
        assert!(result_type.signature.is_some(), "Type alias should have signature");
        let sig = result_type.signature.as_ref().unwrap();
        assert!(sig.starts_with("= "), "Type alias sig should start with '= ': {sig}");

        let userid = types.iter().find(|t| t.name == "UserId").unwrap();
        assert!(userid.signature.is_some(), "Type alias should have signature");
        let sig = userid.signature.as_ref().unwrap();
        assert!(sig.contains("= string"), "sig = {sig}");
    }

    #[test]
    fn test_extract_ts_enum() {
        let source = r#"
export enum Status {
    Active = "active",
    Inactive = "inactive",
}

enum Direction {
    Up,
    Down,
    Left,
    Right,
}
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let enums: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Enum)
            .collect();
        assert_eq!(enums.len(), 2);
        let status = enums.iter().find(|e| e.name == "Status").unwrap();
        assert_eq!(status.visibility, Visibility::Public);

        let direction = enums.iter().find(|e| e.name == "Direction").unwrap();
        // Enum should have member signature if extracted
        // Note: enum body structure varies by grammar, so we check if present
        if let Some(sig) = &direction.signature {
            assert!(sig.contains("Up") || sig.contains("Down"), "sig = {sig}");
        }
    }

    #[test]
    fn test_extract_ts_arrow_function() {
        let source = r#"
export const fetchUser = async (id: string): Promise<User> => {
    const response = await fetch(`/api/users/${id}`);
    return response.json();
};

const helper = (x: number) => x * 2;
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.ts", Language::TypeScript)
                .unwrap();

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        let fetch_user = funcs.iter().find(|f| f.name == "fetchUser").unwrap();
        assert!(
            fetch_user.signature.is_some(),
            "Arrow function should have signature"
        );
        let sig = fetch_user.signature.as_ref().unwrap();
        assert!(sig.contains("id: string"), "sig = {sig}");

        let helper = funcs.iter().find(|f| f.name == "helper").unwrap();
        assert!(
            helper.signature.is_some(),
            "Arrow function should have signature"
        );
    }

    #[test]
    fn test_extract_ts_imports() {
        let source = r#"
import React from 'react';
import { useState, useEffect } from 'react';
import * as path from 'path';
import type { User } from './types';
"#;
        let tree = parse_typescript(source);
        let imports =
            TypeScriptExtractor::extract_imports(&tree, source, Language::TypeScript).unwrap();

        assert!(
            imports.len() >= 3,
            "Should find at least 3 imports, found {}",
            imports.len()
        );
        assert!(imports.iter().any(|i| i.source == "react"));
        assert!(imports.iter().any(|i| i.source == "path"));
        assert!(imports.iter().any(|i| i.source == "./types"));
    }

    #[test]
    fn test_extract_tsx_component() {
        let source = r#"
interface Props {
    name: string;
    count: number;
}

export function Greeting({ name, count }: Props): JSX.Element {
    return <div>Hello {name}, count: {count}</div>;
}

export const Counter = ({ initial }: { initial: number }) => {
    const [count, setCount] = useState(initial);
    return <button onClick={() => setCount(c => c + 1)}>{count}</button>;
};
"#;
        let tree = parse_typescript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.tsx", Language::TypeScript)
                .unwrap();

        // Should find interface, function component, arrow component
        assert!(symbols
            .iter()
            .any(|s| s.name == "Props" && s.kind == SymbolKind::Interface));
        assert!(symbols
            .iter()
            .any(|s| s.name == "Greeting" && s.kind == SymbolKind::Function));
        assert!(symbols
            .iter()
            .any(|s| s.name == "Counter" && s.kind == SymbolKind::Function));
    }

    // --- JavaScript tests ---

    #[test]
    fn test_extract_js_function() {
        let source = r#"
function processData(data) {
    return data.map(item => item.value);
}

export function formatOutput(result) {
    return JSON.stringify(result, null, 2);
}
"#;
        let tree = parse_javascript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.js", Language::JavaScript)
                .unwrap();

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert!(funcs.iter().any(|f| f.name == "processData"));
        assert!(funcs.iter().any(|f| f.name == "formatOutput"));
    }

    #[test]
    fn test_extract_js_class() {
        let source = r#"
class EventEmitter {
    constructor() {
        this.listeners = {};
    }

    on(event, callback) {
        if (!this.listeners[event]) {
            this.listeners[event] = [];
        }
        this.listeners[event].push(callback);
    }

    emit(event, ...args) {
        const handlers = this.listeners[event] || [];
        handlers.forEach(fn => fn(...args));
    }
}
"#;
        let tree = parse_javascript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.js", Language::JavaScript)
                .unwrap();

        let classes: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Class)
            .collect();
        assert_eq!(classes.len(), 1);
        assert_eq!(classes[0].name, "EventEmitter");

        let methods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Method)
            .collect();
        assert!(methods.iter().any(|m| m.name == "constructor"));
        assert!(methods.iter().any(|m| m.name == "on"));
        assert!(methods.iter().any(|m| m.name == "emit"));
    }

    #[test]
    fn test_extract_js_arrow_function() {
        let source = r#"
const double = (x) => x * 2;

export const fetchData = async (url) => {
    const response = await fetch(url);
    return response.json();
};
"#;
        let tree = parse_javascript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.js", Language::JavaScript)
                .unwrap();

        let funcs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert!(funcs.iter().any(|f| f.name == "double"));
        assert!(funcs.iter().any(|f| f.name == "fetchData"));
    }

    #[test]
    fn test_extract_js_imports() {
        let source = r#"
import express from 'express';
import { Router, Request } from 'express';
import * as fs from 'fs';
"#;
        let tree = parse_javascript(source);
        let imports =
            TypeScriptExtractor::extract_imports(&tree, source, Language::JavaScript).unwrap();

        assert!(
            imports.len() >= 2,
            "Should find at least 2 imports, found {}",
            imports.len()
        );
        assert!(imports.iter().any(|i| i.source == "express"));
        assert!(imports.iter().any(|i| i.source == "fs"));
    }

    #[test]
    fn test_extract_js_mixed() {
        let source = r#"
import { readFile } from 'fs/promises';

function parseConfig(path) {
    return readFile(path, 'utf-8');
}

class ConfigManager {
    constructor(configPath) {
        this.path = configPath;
    }

    async load() {
        const content = await parseConfig(this.path);
        return JSON.parse(content);
    }
}

const DEFAULT_PATH = './config.json';

export { ConfigManager, DEFAULT_PATH };
"#;
        let tree = parse_javascript(source);
        let symbols =
            TypeScriptExtractor::extract_symbols(&tree, source, "test.js", Language::JavaScript)
                .unwrap();

        assert!(symbols
            .iter()
            .any(|s| s.name == "parseConfig" && s.kind == SymbolKind::Function));
        assert!(symbols
            .iter()
            .any(|s| s.name == "ConfigManager" && s.kind == SymbolKind::Class));

        let methods: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Method)
            .collect();
        assert!(methods.iter().any(|m| m.name == "constructor"));
        assert!(methods.iter().any(|m| m.name == "load"));
    }
}
