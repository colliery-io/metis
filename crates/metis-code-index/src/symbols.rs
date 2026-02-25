//! Symbol types and classification for code intelligence.
//!
//! This module defines the core data types for representing code symbols
//! extracted from source files.

use serde::{Deserialize, Serialize};

/// The kind of symbol extracted from source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    /// A source file (container for other symbols)
    File,
    /// A logical grouping (Rust mod, Python module)
    Module,
    /// An OOP class definition
    Class,
    /// A data structure (Rust struct, C struct)
    Struct,
    /// An abstract interface (Rust trait, Python ABC, TypeScript interface)
    Interface,
    /// An enumeration type
    Enum,
    /// A standalone function
    Function,
    /// A method attached to a type
    Method,
    /// A module-level variable or constant
    Variable,
    /// A type alias or typedef
    Type,
    /// A macro definition
    Macro,
}

impl SymbolKind {
    /// Returns the string representation used in node IDs and queries.
    pub fn as_str(&self) -> &'static str {
        match self {
            SymbolKind::File => "file",
            SymbolKind::Module => "module",
            SymbolKind::Class => "class",
            SymbolKind::Struct => "struct",
            SymbolKind::Interface => "interface",
            SymbolKind::Enum => "enum",
            SymbolKind::Function => "function",
            SymbolKind::Method => "method",
            SymbolKind::Variable => "variable",
            SymbolKind::Type => "type",
            SymbolKind::Macro => "macro",
        }
    }

    /// Returns true if this symbol kind represents a type definition.
    pub fn is_type_definition(&self) -> bool {
        matches!(
            self,
            SymbolKind::Class
                | SymbolKind::Struct
                | SymbolKind::Interface
                | SymbolKind::Enum
                | SymbolKind::Type
        )
    }

    /// Returns true if this symbol kind represents a callable.
    pub fn is_callable(&self) -> bool {
        matches!(
            self,
            SymbolKind::Function | SymbolKind::Method | SymbolKind::Macro
        )
    }
}

/// Visibility/accessibility of a symbol.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum Visibility {
    /// Publicly accessible (Rust `pub`, Python public)
    Public,
    /// Private to the containing scope (Rust default, Python `_prefix`)
    #[default]
    Private,
    /// Visible within the crate/package (Rust `pub(crate)`)
    Crate,
    /// Restricted visibility with a path (Rust `pub(in path)`)
    Restricted(String),
}

/// A symbol extracted from source code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    /// The symbol's name (e.g., function name, struct name)
    pub name: String,

    /// The kind of symbol
    pub kind: SymbolKind,

    /// File path relative to repository root
    pub file_path: String,

    /// Starting line number (1-indexed)
    pub start_line: usize,

    /// Ending line number (1-indexed, inclusive)
    pub end_line: usize,

    /// The symbol's signature (e.g., function signature, struct fields summary)
    pub signature: Option<String>,

    /// Fully qualified name (e.g., `crate::module::StructName::method`)
    pub qualified_name: Option<String>,

    /// Documentation comment extracted from source
    pub doc_comment: Option<String>,

    /// Visibility/accessibility of the symbol
    pub visibility: Visibility,
}

impl Symbol {
    /// Create a new symbol with required fields.
    pub fn new(
        name: impl Into<String>,
        kind: SymbolKind,
        file_path: impl Into<String>,
        start_line: usize,
        end_line: usize,
    ) -> Self {
        Self {
            name: name.into(),
            kind,
            file_path: file_path.into(),
            start_line,
            end_line,
            signature: None,
            qualified_name: None,
            doc_comment: None,
            visibility: Visibility::default(),
        }
    }

    /// Set the signature.
    pub fn with_signature(mut self, signature: impl Into<String>) -> Self {
        self.signature = Some(signature.into());
        self
    }

    /// Set the qualified name.
    pub fn with_qualified_name(mut self, qualified_name: impl Into<String>) -> Self {
        self.qualified_name = Some(qualified_name.into());
        self
    }

    /// Set the doc comment.
    pub fn with_doc_comment(mut self, doc_comment: impl Into<String>) -> Self {
        self.doc_comment = Some(doc_comment.into());
        self
    }

    /// Set the visibility.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Generate a stable ID for this symbol.
    ///
    /// The ID is constructed from file path, kind, name, and line number
    /// to ensure uniqueness and stability across rebuilds.
    pub fn id(&self) -> String {
        let safe_path = self.file_path.replace(['/', '\\', '.', ':'], "_");
        format!(
            "{}__{}__{}__{}",
            safe_path,
            self.kind.as_str(),
            self.name,
            self.start_line
        )
    }

    /// Returns the number of lines this symbol spans.
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }

    /// Returns a location string for display (file:line-line).
    pub fn location(&self) -> String {
        format!("{}:{}-{}", self.file_path, self.start_line, self.end_line)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_kind_as_str() {
        assert_eq!(SymbolKind::Function.as_str(), "function");
        assert_eq!(SymbolKind::Struct.as_str(), "struct");
        assert_eq!(SymbolKind::Interface.as_str(), "interface");
    }

    #[test]
    fn test_symbol_kind_classification() {
        assert!(SymbolKind::Struct.is_type_definition());
        assert!(SymbolKind::Class.is_type_definition());
        assert!(!SymbolKind::Function.is_type_definition());

        assert!(SymbolKind::Function.is_callable());
        assert!(SymbolKind::Method.is_callable());
        assert!(!SymbolKind::Struct.is_callable());
    }

    #[test]
    fn test_symbol_creation() {
        let sym = Symbol::new(
            "parse_config",
            SymbolKind::Function,
            "src/config.rs",
            10,
            25,
        )
        .with_signature("pub fn parse_config(path: &Path) -> Result<Config>")
        .with_visibility(Visibility::Public);

        assert_eq!(sym.name, "parse_config");
        assert_eq!(sym.kind, SymbolKind::Function);
        assert_eq!(sym.start_line, 10);
        assert_eq!(sym.end_line, 25);
        assert_eq!(sym.line_count(), 16);
        assert_eq!(sym.visibility, Visibility::Public);
    }

    #[test]
    fn test_symbol_id_generation() {
        let sym = Symbol::new("MyStruct", SymbolKind::Struct, "src/lib.rs", 5, 20);
        assert_eq!(sym.id(), "src_lib_rs__struct__MyStruct__5");
    }

    #[test]
    fn test_symbol_location() {
        let sym = Symbol::new("foo", SymbolKind::Function, "src/main.rs", 100, 150);
        assert_eq!(sym.location(), "src/main.rs:100-150");
    }

    #[test]
    fn test_visibility_default() {
        assert_eq!(Visibility::default(), Visibility::Private);
    }

    #[test]
    fn test_symbol_serialization() {
        let sym = Symbol::new("test_fn", SymbolKind::Function, "test.rs", 1, 5)
            .with_visibility(Visibility::Public);

        let json = serde_json::to_string(&sym).expect("serialize");
        let deserialized: Symbol = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, sym.name);
        assert_eq!(deserialized.kind, sym.kind);
        assert_eq!(deserialized.visibility, sym.visibility);
    }

    #[test]
    fn test_symbol_kind_serialization() {
        let kind = SymbolKind::Interface;
        let json = serde_json::to_string(&kind).expect("serialize");
        assert_eq!(json, "\"interface\"");

        let deserialized: SymbolKind = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, SymbolKind::Interface);
    }

    #[test]
    fn test_visibility_serialization() {
        let vis = Visibility::Restricted("crate::internal".to_string());
        let json = serde_json::to_string(&vis).expect("serialize");

        let deserialized: Visibility = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, vis);
    }
}
