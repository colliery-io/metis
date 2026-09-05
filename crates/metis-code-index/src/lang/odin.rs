//! Odin symbol extraction using tree-sitter.
//!
//! Extracts procedures, procedure groups, structs, bit fields, enums, unions,
//! constants, variables, and imports from Odin source code.
//!
//! Odin-specific handling:
//! - Declarations are `name :: value` forms, so a declaration node can carry
//!   several names (`A, B :: 1, 2`). Names are the identifiers preceding the
//!   binding token (`::`, `:`, or `:=`); anything after is the value. The
//!   grammar uses a distinct node per binding form, so all five are captured:
//!   `const_declaration`, `const_type_declaration` (`MAX: u64 : 32`),
//!   `var_declaration` (`x: int = 0`), `variable_declaration` (`x := 0`), and
//!   `bit_field_declaration`.
//! - Visibility comes from `@(private)` attributes rather than naming
//!   convention. Package scope is the default.
//! - Declarations nested inside a procedure body are locals and are skipped,
//!   while those inside `when` and `foreign` blocks are kept — they are
//!   package-level despite being nested.
//!
//! Known limitation: `Foo :: int` (a type alias whose value is a bare
//! identifier) is indistinguishable from a constant like `MAX :: SOME_CONST`
//! without type resolution, so it is reported as a constant. Aliases written
//! with an explicit type form (`distinct int`, `bit_set[..]`, `[4]f32`, ...)
//! are correctly reported as types.

use std::sync::OnceLock;

use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, Tree};

use crate::symbols::{compact_signature, Symbol, SymbolKind, Visibility};

/// Import statement from Odin source.
#[derive(Debug, Clone)]
pub struct Import {
    /// The import path (e.g., "core:fmt", "system:c")
    pub path: String,
    /// Optional alias (e.g., `import fm "core:fmt"`, `foreign import lib "system:c"`)
    pub alias: Option<String>,
    /// Line number
    pub line: usize,
}

/// Compiled tree-sitter queries for Odin.
struct OdinQueries {
    symbols: Query,
    imports: Query,
}

static ODIN_QUERIES: OnceLock<Result<OdinQueries, String>> = OnceLock::new();

fn get_queries() -> Result<&'static OdinQueries, &'static str> {
    ODIN_QUERIES
        .get_or_init(|| {
            let language: tree_sitter::Language = tree_sitter_odin::LANGUAGE.into();
            let symbols = Query::new(&language, SYMBOLS_QUERY)
                .map_err(|e| format!("Failed to compile Odin symbols query: {e}"))?;
            let imports = Query::new(&language, IMPORTS_QUERY)
                .map_err(|e| format!("Failed to compile Odin imports query: {e}"))?;
            Ok(OdinQueries { symbols, imports })
        })
        .as_ref()
        .map_err(|e| e.as_str())
}

/// Tree-sitter query for extracting Odin symbols.
///
/// Only the outer declaration node is captured; names are resolved in code
/// because a declaration may bind several names and its value may itself be a
/// bare identifier, which a query cannot tell apart from a name.
const SYMBOLS_QUERY: &str = r#"
; Procedures
(procedure_declaration) @function

; Procedure groups: `Grouped :: proc{a, b}`
(overloaded_procedure_declaration) @proc_group

; Structs
(struct_declaration) @struct

; Enums
(enum_declaration) @enum

; Unions
(union_declaration) @union

; Constants and type aliases
(const_declaration) @constant

; Typed constants: `MAX: u64 : 32`
(const_type_declaration) @typed_constant

; Bit fields
(bit_field_declaration) @bit_field

; Variables with an explicit type: `count: int = 0`
(var_declaration) @variable

; Variables with an inferred type: `count := 0`
(variable_declaration) @inferred_variable
"#;

/// Tree-sitter query for extracting Odin import statements.
///
/// Covers both `import "core:fmt"` and `foreign import lib "system:c"`.
const IMPORTS_QUERY: &str = r#"
(import_declaration) @import
"#;

/// Odin symbol extractor.
///
/// Handles Odin source files (.odin), extracting procedures, procedure groups,
/// structs, enums, unions, constants, type aliases, and variables. Visibility
/// is derived from `@(private)` attributes.
pub struct OdinExtractor;

impl OdinExtractor {
    /// Extract symbols from a parsed Odin syntax tree.
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
            for capture in match_.captures {
                let capture_name = queries.symbols.capture_names()[capture.index as usize];
                let node = capture.node;

                // Locals inside a procedure body are not package-level symbols.
                if is_inside_procedure_body(node) {
                    continue;
                }

                let kind = match capture_name {
                    "function" | "proc_group" => SymbolKind::Function,
                    "struct" => SymbolKind::Struct,
                    "enum" => SymbolKind::Enum,
                    "union" => SymbolKind::Type,
                    "bit_field" => SymbolKind::Struct,
                    "constant" => const_kind(node),
                    "typed_constant" | "variable" | "inferred_variable" => SymbolKind::Variable,
                    _ => continue,
                };

                let visibility = visibility_of(node, source);
                let doc_comment = extract_odin_doc(&node, source);
                let start_line = node.start_position().row + 1;
                let end_line = node.end_position().row + 1;

                let signature = match capture_name {
                    "function" => build_proc_signature(node, source),
                    "proc_group" => build_proc_group_signature(node, source),
                    "struct" => build_struct_signature(node, source),
                    "enum" => build_enum_signature(node, source),
                    "union" => build_union_signature(node, source),
                    "bit_field" => build_bit_field_signature(node, source),
                    "constant" | "inferred_variable" => build_const_signature(node, source),
                    "typed_constant" => build_typed_const_signature(node, source),
                    "variable" => build_var_signature(node, source),
                    _ => None,
                };

                // A single declaration may bind several names: `A, B :: 1, 2`.
                for name in declared_names(node, source) {
                    let mut symbol = Symbol::new(name, kind, file_path, start_line, end_line)
                        .with_visibility(visibility.clone());

                    if let Some(sig) = signature.clone() {
                        symbol = symbol.with_signature(sig);
                    }
                    if let Some(doc) = doc_comment.clone() {
                        symbol = symbol.with_doc_comment(doc);
                    }

                    symbols.push(symbol);
                }
            }
        }

        symbols.sort_by(|a, b| a.start_line.cmp(&b.start_line).then(a.name.cmp(&b.name)));
        Ok(symbols)
    }

    /// Extract import statements from a parsed Odin syntax tree.
    pub fn extract_imports(tree: &Tree, source: &str) -> Result<Vec<Import>, String> {
        let queries = get_queries()?;
        let source_bytes = source.as_bytes();
        let mut imports = Vec::new();

        let mut cursor = tree_sitter::QueryCursor::new();
        let mut matches = cursor.matches(&queries.imports, tree.root_node(), source_bytes);

        while let Some(match_) = matches.next() {
            for capture in match_.captures {
                let node = capture.node;

                // The path lives in a string_content node, so quotes are already excluded.
                let Some(path) =
                    find_descendant_by_kind(node, "string_content").map(|n| node_text(&n, source))
                else {
                    continue;
                };

                let alias = node
                    .child_by_field_name("alias")
                    .map(|n| node_text(&n, source));

                imports.push(Import {
                    path,
                    alias,
                    line: node.start_position().row + 1,
                });
            }
        }

        Ok(imports)
    }
}

/// Names bound by a declaration: the identifiers preceding the `::` or `:` token.
///
/// For declarations that introduce a single named entity (procedures, structs,
/// enums, unions), this is the leading identifier; later identifiers belong to
/// the value (enum variants, procedure group members).
fn declared_names(node: tree_sitter::Node, source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut cursor = node.walk();
    if !cursor.goto_first_child() {
        return names;
    }
    loop {
        let child = cursor.node();
        match child.kind() {
            // Everything from the binding token onwards is the value.
            "::" | ":" | ":=" => break,
            "identifier" => names.push(node_text(&child, source)),
            _ => {}
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    names
}

/// Whether a node sits inside a procedure body (making it a local declaration).
///
/// `when` and `foreign` blocks are not procedure bodies, so declarations inside
/// them remain package-level.
fn is_inside_procedure_body(node: tree_sitter::Node) -> bool {
    let mut current = node.parent();
    while let Some(n) = current {
        if n.kind() == "procedure" {
            return true;
        }
        current = n.parent();
    }
    false
}

/// Classify a `const_declaration` as a type alias or a constant.
fn const_kind(node: tree_sitter::Node) -> SymbolKind {
    let value_kinds = [
        "distinct_type",
        "bit_set_type",
        "array_type",
        "pointer_type",
        "multi_pointer_type",
        "slice_type",
        "dynamic_array_type",
        "map_type",
        "matrix_type",
        "tuple_type",
        "procedure_type",
        "variadic_type",
    ];

    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if value_kinds.contains(&cursor.node().kind()) {
                return SymbolKind::Type;
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    SymbolKind::Variable
}

/// Determine visibility from `@(private)` attributes.
///
/// Odin declarations are visible to the whole package by default;
/// `@(private)` narrows to the package, `@(private="file")` to the file.
fn visibility_of(node: tree_sitter::Node, source: &str) -> Visibility {
    let Some(attributes) = find_child_by_kind(node, "attributes") else {
        return Visibility::Public;
    };

    let mut stack = vec![attributes];
    let mut has_private = false;
    let mut scope: Option<String> = None;

    // Attributes nest as attributes > attribute > identifier/string.
    while let Some(current) = stack.pop() {
        let mut cursor = current.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                match child.kind() {
                    "identifier" if node_text(&child, source) == "private" => has_private = true,
                    "string_content" => scope = Some(node_text(&child, source)),
                    _ => {}
                }
                stack.push(child);
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }

    if !has_private {
        return Visibility::Public;
    }
    match scope.as_deref() {
        Some("file") => Visibility::Private,
        _ => Visibility::Crate,
    }
}

/// Build a procedure signature: `proc(a: int, b: int) -> int`.
///
/// The body (and the `---` of foreign declarations) is excluded.
fn build_proc_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let procedure = find_child_by_kind(node, "procedure")?;

    // Take everything up to the body so the signature stays one line.
    let end = find_child_by_kind(procedure, "block")
        .or_else(|| find_child_by_kind(procedure, "uninitialized"))
        .map(|n| n.start_byte())
        .unwrap_or(procedure.end_byte());

    let text = source.get(procedure.start_byte()..end)?.trim();
    if text.is_empty() {
        return None;
    }
    Some(compact_signature(text, 120))
}

/// Build a procedure group signature: `proc{add_int, add_f32}`.
fn build_proc_group_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let members: Vec<String> = named_children_of_kind(node, "identifier", source)
        .into_iter()
        .skip(1) // first identifier is the group's own name
        .collect();
    if members.is_empty() {
        return None;
    }
    Some(compact_signature(
        &format!("proc{{{}}}", members.join(", ")),
        120,
    ))
}

/// Build a struct signature: `{ x: f32, y: f32 }`, prefixed with any
/// polymorphic parameters.
fn build_struct_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut fields = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            if child.kind() == "field" {
                fields.push(node_text(&child, source));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }

    let poly = find_child_by_kind(node, "polymorphic_parameters").map(|n| node_text(&n, source));

    if fields.is_empty() && poly.is_none() {
        return None;
    }

    let body = format!("{{ {} }}", fields.join(", "));
    let sig = match poly {
        Some(p) => format!("{} {}", p, body),
        None => body,
    };
    Some(compact_signature(&sig, 120))
}

/// Build an enum signature: `{ Red, Green }`.
fn build_enum_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let variants: Vec<String> = named_children_of_kind(node, "identifier", source)
        .into_iter()
        .skip(1) // first identifier is the enum's own name
        .collect();
    if variants.is_empty() {
        return None;
    }
    Some(compact_signature(
        &format!("{{ {} }}", variants.join(", ")),
        120,
    ))
}

/// Build a union signature: `union { int, string }`.
fn build_union_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let variants = named_children_of_kind(node, "type", source);
    if variants.is_empty() {
        return None;
    }
    Some(compact_signature(
        &format!("union {{ {} }}", variants.join(", ")),
        120,
    ))
}

/// Build a constant/type-alias signature: `= 3`, `= distinct int`.
fn build_const_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let value = value_text(node, source)?;
    Some(compact_signature(&format!("= {}", value), 120))
}

/// Build a bit field signature: `bit_field u8 { a: bool | 1 }`.
///
/// Taken from just after the `::`, because the `bit_field` keyword is an
/// anonymous token that a named-node scan would skip.
fn build_bit_field_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let binding = find_child_by_kind(node, "::")?;
    let text = source.get(binding.end_byte()..node.end_byte())?.trim();
    if text.is_empty() {
        return None;
    }
    Some(compact_signature(text, 120))
}

/// Build a typed constant signature: `: u64 : 0xff`.
///
/// Both the declared type and the value are kept, since the type is the reason
/// this form is used over a plain `::` constant.
fn build_typed_const_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let colon = find_child_by_kind(node, ":")?;
    let text = source.get(colon.start_byte()..node.end_byte())?.trim();
    if text.is_empty() {
        return None;
    }
    Some(compact_signature(text, 120))
}

/// Build a variable signature: `: int` when the type is annotated.
fn build_var_signature(node: tree_sitter::Node, source: &str) -> Option<String> {
    let ty = find_child_by_kind(node, "type")?;
    Some(compact_signature(
        &format!(": {}", node_text(&ty, source)),
        120,
    ))
}

/// Text of everything following the binding token, i.e. the declaration's value.
fn value_text(node: tree_sitter::Node, source: &str) -> Option<String> {
    let mut cursor = node.walk();
    if !cursor.goto_first_child() {
        return None;
    }
    let mut seen_binding = false;
    let mut start = None;
    loop {
        let child = cursor.node();
        if seen_binding && child.is_named() {
            start = Some(child.start_byte());
            break;
        }
        if matches!(child.kind(), "::" | ":" | ":=") {
            seen_binding = true;
        }
        if !cursor.goto_next_sibling() {
            break;
        }
    }
    let text = source.get(start?..node.end_byte())?.trim();
    if text.is_empty() {
        None
    } else {
        Some(text.to_string())
    }
}

/// Collect the text of direct children of a given kind.
fn named_children_of_kind(node: tree_sitter::Node, kind: &str, source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().kind() == kind {
                out.push(node_text(&cursor.node(), source));
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    out
}

/// Find a direct child node by kind name.
fn find_child_by_kind<'a>(
    node: tree_sitter::Node<'a>,
    kind: &str,
) -> Option<tree_sitter::Node<'a>> {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            if cursor.node().kind() == kind {
                return Some(cursor.node());
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
    None
}

/// Find the first descendant node by kind name (breadth-first).
fn find_descendant_by_kind<'a>(
    node: tree_sitter::Node<'a>,
    kind: &str,
) -> Option<tree_sitter::Node<'a>> {
    let mut queue = vec![node];
    while let Some(current) = queue.pop() {
        if current.kind() == kind {
            return Some(current);
        }
        let mut cursor = current.walk();
        if cursor.goto_first_child() {
            loop {
                queue.push(cursor.node());
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
    }
    None
}

/// Extract Odin doc comments (`//` lines or a `/* */` block preceding a declaration).
fn extract_odin_doc(node: &tree_sitter::Node, source: &str) -> Option<String> {
    let mut doc_lines = Vec::new();
    let mut sibling = node.prev_sibling();

    while let Some(sib) = sibling {
        match sib.kind() {
            "comment" => {
                let text = sib.utf8_text(source.as_bytes()).unwrap_or("").trim();
                if let Some(content) = text.strip_prefix("//") {
                    doc_lines.push(content.trim().to_string());
                    sibling = sib.prev_sibling();
                    continue;
                }
                break;
            }
            "block_comment" => {
                let text = sib.utf8_text(source.as_bytes()).unwrap_or("").trim();
                let content = text
                    .strip_prefix("/*")
                    .and_then(|t| t.strip_suffix("*/"))
                    .unwrap_or(text);
                doc_lines.push(content.trim().to_string());
                break;
            }
            _ => break,
        }
    }

    if doc_lines.is_empty() {
        return None;
    }

    doc_lines.reverse();
    Some(doc_lines.join("\n"))
}

fn node_text(node: &tree_sitter::Node, source: &str) -> String {
    source[node.byte_range()].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{Language, Parser};

    fn parse_odin(source: &str) -> Tree {
        let mut parser = Parser::new();
        let parsed = parser.parse_source(source, Language::Odin).unwrap();
        parsed.tree
    }

    #[test]
    fn test_extract_procedures() {
        let source = r#"package main

// Add returns the sum of two integers.
add :: proc(a: int, b: int) -> int {
    return a + b
}

helper :: proc(x: string) {
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "main.odin").unwrap();

        let procs: Vec<_> = symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Function)
            .collect();
        assert_eq!(procs.len(), 2);

        let add = procs.iter().find(|p| p.name == "add").unwrap();
        assert_eq!(add.visibility, Visibility::Public);
        let sig = add.signature.as_ref().expect("proc should have signature");
        assert_eq!(sig, "proc(a: int, b: int) -> int");
        assert_eq!(
            add.doc_comment.as_deref(),
            Some("Add returns the sum of two integers.")
        );

        assert!(procs.iter().any(|p| p.name == "helper"));
    }

    #[test]
    fn test_procedure_signature_excludes_body() {
        let source = r#"package main

run :: proc(count: int) -> (ok: bool, err: Error) {
    return true, nil
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "run.odin").unwrap();
        let sig = symbols[0].signature.as_ref().unwrap();
        assert_eq!(sig, "proc(count: int) -> (ok: bool, err: Error)");
        assert!(!sig.contains("return"), "body leaked into signature: {sig}");
    }

    #[test]
    fn test_extract_struct() {
        let source = r#"package main

// Vec2 is a two dimensional vector.
Vec2 :: struct {
    x: f32,
    y: f32,
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "vec.odin").unwrap();

        let vec2 = symbols.iter().find(|s| s.name == "Vec2").unwrap();
        assert_eq!(vec2.kind, SymbolKind::Struct);
        assert!(vec2.doc_comment.is_some());
        let sig = vec2.signature.as_ref().unwrap();
        assert!(sig.contains("x: f32"), "sig = {sig}");
        assert!(sig.contains("y: f32"), "sig = {sig}");
    }

    #[test]
    fn test_extract_generic_struct() {
        let source = r#"package main

Shape :: struct($T: typeid) {
    value: T,
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "shape.odin").unwrap();

        let shape = symbols.iter().find(|s| s.name == "Shape").unwrap();
        assert_eq!(shape.kind, SymbolKind::Struct);
        let sig = shape.signature.as_ref().unwrap();
        assert!(sig.contains("($T: typeid)"), "sig = {sig}");
        assert!(sig.contains("value: T"), "sig = {sig}");
    }

    #[test]
    fn test_extract_enum_and_union() {
        let source = r#"package main

Color :: enum {
    Red,
    Green,
}

Value :: union {
    int,
    string,
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "types.odin").unwrap();

        let color = symbols.iter().find(|s| s.name == "Color").unwrap();
        assert_eq!(color.kind, SymbolKind::Enum);
        let sig = color.signature.as_ref().unwrap();
        assert!(sig.contains("Red"), "sig = {sig}");
        assert!(sig.contains("Green"), "sig = {sig}");
        // The enum's own name must not be repeated as a variant.
        assert!(!sig.contains("Color"), "sig = {sig}");

        let value = symbols.iter().find(|s| s.name == "Value").unwrap();
        assert_eq!(value.kind, SymbolKind::Type);
        let sig = value.signature.as_ref().unwrap();
        assert!(sig.contains("int"), "sig = {sig}");
        assert!(sig.contains("string"), "sig = {sig}");
    }

    #[test]
    fn test_extract_constants_and_variables() {
        let source = r#"package main

MAX_RETRIES :: 3

global_count: int = 0
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "const.odin").unwrap();

        let max = symbols.iter().find(|s| s.name == "MAX_RETRIES").unwrap();
        assert_eq!(max.kind, SymbolKind::Variable);
        assert_eq!(max.signature.as_deref(), Some("= 3"));

        let count = symbols.iter().find(|s| s.name == "global_count").unwrap();
        assert_eq!(count.kind, SymbolKind::Variable);
        assert_eq!(count.signature.as_deref(), Some(": int"));
    }

    #[test]
    fn test_typed_constant() {
        // `name: Type : value` is a distinct grammar node from `name :: value`.
        let source = r#"package main

@(private)
FNV64A_OFFSET_BASIS: u64 : 0xcbf29ce484222325
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "hash.odin").unwrap();

        assert_eq!(symbols.len(), 1, "{:?}", symbols);
        let sym = &symbols[0];
        assert_eq!(sym.name, "FNV64A_OFFSET_BASIS");
        assert_eq!(sym.kind, SymbolKind::Variable);
        assert_eq!(sym.visibility, Visibility::Crate);
        let sig = sym.signature.as_ref().unwrap();
        assert!(sig.contains("u64"), "type should be kept: {sig}");
        assert!(sig.contains("0xcbf29ce484222325"), "sig = {sig}");
    }

    #[test]
    fn test_inferred_variable() {
        let source = r#"package main

count := 0

alias := some_other_symbol
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "vars.odin").unwrap();

        assert_eq!(symbols.len(), 2, "{:?}", symbols);
        let count = symbols.iter().find(|s| s.name == "count").unwrap();
        assert_eq!(count.kind, SymbolKind::Variable);
        assert_eq!(count.signature.as_deref(), Some("= 0"));

        // The right-hand side of `:=` is a value, not a second declared name.
        assert!(
            !symbols.iter().any(|s| s.name == "some_other_symbol"),
            "value mistaken for a name: {:?}",
            symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_bit_field() {
        let source = r#"package main

Flags :: bit_field u8 {
    a: bool | 1,
    b: u8   | 7,
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "flags.odin").unwrap();

        let flags = symbols.iter().find(|s| s.name == "Flags").unwrap();
        assert_eq!(flags.kind, SymbolKind::Struct);
        let sig = flags.signature.as_ref().unwrap();
        assert!(sig.contains("bit_field u8"), "sig = {sig}");
        assert!(sig.contains("a: bool"), "sig = {sig}");
    }

    #[test]
    fn test_multi_name_declarations() {
        let source = r#"package main

A, B :: 1, 2

x, y: int
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "multi.odin").unwrap();

        for name in ["A", "B", "x", "y"] {
            assert!(
                symbols.iter().any(|s| s.name == name),
                "missing {name} in {:?}",
                symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
            );
        }
        // The values (1, 2, int) must not be mistaken for names.
        assert_eq!(symbols.len(), 4, "{:?}", symbols);
    }

    #[test]
    fn test_type_alias_kinds() {
        let source = r#"package main

MyInt :: distinct int

Flags :: bit_set[Flag]

Matrix :: distinct [4][4]f32
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "alias.odin").unwrap();

        for name in ["MyInt", "Flags", "Matrix"] {
            let sym = symbols.iter().find(|s| s.name == name).unwrap();
            assert_eq!(sym.kind, SymbolKind::Type, "{name} should be a Type");
        }

        let myint = symbols.iter().find(|s| s.name == "MyInt").unwrap();
        assert_eq!(myint.signature.as_deref(), Some("= distinct int"));
    }

    #[test]
    fn test_visibility_from_attributes() {
        let source = r#"package main

public_proc :: proc() {}

@(private)
pkg_private :: proc() {}

@(private="file")
file_private :: proc() {}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "vis.odin").unwrap();

        let get = |n: &str| symbols.iter().find(|s| s.name == n).unwrap();
        assert_eq!(get("public_proc").visibility, Visibility::Public);
        assert_eq!(get("pkg_private").visibility, Visibility::Crate);
        assert_eq!(get("file_private").visibility, Visibility::Private);
    }

    #[test]
    fn test_local_declarations_are_skipped() {
        let source = r#"package main

outer :: proc() {
    inner :: proc() {}
    Local :: struct { a: int }
    count := 5
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "local.odin").unwrap();

        assert!(symbols.iter().any(|s| s.name == "outer"));
        assert!(
            !symbols.iter().any(|s| s.name == "inner"),
            "nested proc should not be a top-level symbol"
        );
        assert!(
            !symbols.iter().any(|s| s.name == "Local"),
            "local struct should not be a top-level symbol"
        );
    }

    #[test]
    fn test_when_and_foreign_declarations_are_kept() {
        let source = r#"package main

foreign import lib "system:c"

foreign lib {
    printf :: proc(fmt: cstring) -> i32 ---
}

when ODIN_OS == .Windows {
    win_only :: proc() {}
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "foreign.odin").unwrap();

        assert!(
            symbols.iter().any(|s| s.name == "printf"),
            "foreign proc should be kept: {:?}",
            symbols.iter().map(|s| &s.name).collect::<Vec<_>>()
        );
        assert!(
            symbols.iter().any(|s| s.name == "win_only"),
            "when-block proc should be kept"
        );

        let printf = symbols.iter().find(|s| s.name == "printf").unwrap();
        let sig = printf.signature.as_ref().unwrap();
        assert!(!sig.contains("---"), "sig should not include `---`: {sig}");
    }

    #[test]
    fn test_procedure_group() {
        let source = r#"package main

Grouped :: proc{add_int, add_f32}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "group.odin").unwrap();

        let grouped = symbols.iter().find(|s| s.name == "Grouped").unwrap();
        assert_eq!(grouped.kind, SymbolKind::Function);
        let sig = grouped.signature.as_ref().unwrap();
        assert!(sig.contains("add_int"), "sig = {sig}");
        assert!(sig.contains("add_f32"), "sig = {sig}");
        // Group members are not separate symbols.
        assert_eq!(symbols.len(), 1, "{:?}", symbols);
    }

    #[test]
    fn test_block_doc_comment() {
        let source = r#"package main

/* Does the thing. */
documented :: proc() {}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "doc.odin").unwrap();
        assert_eq!(symbols[0].doc_comment.as_deref(), Some("Does the thing."));
    }

    #[test]
    fn test_extract_imports() {
        let source = r#"package main

import "core:fmt"
import fm "core:fmt"
import "core:os"

foreign import lib "system:c"
"#;
        let tree = parse_odin(source);
        let imports = OdinExtractor::extract_imports(&tree, source).unwrap();

        assert!(imports
            .iter()
            .any(|i| i.path == "core:fmt" && i.alias.is_none()));
        assert!(imports
            .iter()
            .any(|i| i.path == "core:fmt" && i.alias.as_deref() == Some("fm")));
        assert!(imports.iter().any(|i| i.path == "core:os"));
        assert!(imports
            .iter()
            .any(|i| i.path == "system:c" && i.alias.as_deref() == Some("lib")));
    }

    #[test]
    fn test_mixed_odin_file() {
        let source = r#"package game

import "core:fmt"

// Entity is a thing in the world.
Entity :: struct {
    id:   int,
    name: string,
}

State :: enum {
    Idle,
    Running,
}

MAX_ENTITIES :: 1024

entities: [dynamic]Entity

@(private)
spawn :: proc(name: string) -> ^Entity {
    return nil
}

update :: proc(dt: f32) {
    fmt.println(dt)
}
"#;
        let tree = parse_odin(source);
        let symbols = OdinExtractor::extract_symbols(&tree, source, "game.odin").unwrap();

        let find = |n: &str| symbols.iter().find(|s| s.name == n);
        assert_eq!(find("Entity").unwrap().kind, SymbolKind::Struct);
        assert_eq!(find("State").unwrap().kind, SymbolKind::Enum);
        assert_eq!(find("MAX_ENTITIES").unwrap().kind, SymbolKind::Variable);
        assert_eq!(find("entities").unwrap().kind, SymbolKind::Variable);
        assert_eq!(find("spawn").unwrap().kind, SymbolKind::Function);
        assert_eq!(find("spawn").unwrap().visibility, Visibility::Crate);
        assert_eq!(find("update").unwrap().kind, SymbolKind::Function);

        let imports = OdinExtractor::extract_imports(&tree, source).unwrap();
        assert_eq!(imports.len(), 1);
        assert_eq!(imports[0].path, "core:fmt");
    }
}
