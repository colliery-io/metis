//! Language-specific extractors.
//!
//! Each language module provides extraction logic for symbols, imports,
//! and call relationships from parsed syntax trees.

pub mod go;
pub mod python;
pub mod rust;
pub mod typescript;

pub use go::GoExtractor;
pub use python::PythonExtractor;
pub use rust::RustExtractor;
pub use typescript::TypeScriptExtractor;
