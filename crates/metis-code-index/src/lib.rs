//! metis-code-index: Code indexing via tree-sitter for AI agent navigation.
//!
//! This crate provides symbol extraction from source code using tree-sitter
//! grammars. It supports multiple languages and is designed to produce
//! compressed structural indexes for AI coding agents.
//!
//! Vendored from colliery-io/muninn/crates/muninn-graph with storage,
//! graph building, and file watching removed.

pub mod formatter;
pub mod lang;
pub mod parser;
pub mod symbols;
pub mod walker;

pub use formatter::{format_index, write_index_file};
pub use lang::go::GoExtractor;
pub use lang::python::PythonExtractor;
pub use lang::rust::RustExtractor;
pub use lang::typescript::TypeScriptExtractor;
pub use parser::{Language, ParseError, ParsedFile, Parser};
pub use symbols::{Symbol, SymbolKind, Visibility};
pub use walker::{walk_directory, SourceFile, WalkError, WalkResult};
