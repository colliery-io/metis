//! Document viewer abstraction layer
//!
//! Provides a trait for viewer backends and a dispatcher that routes
//! open requests to the configured viewer, with fallback chain on failure.

pub mod dispatcher;
pub mod sys_editor;
pub mod traits;
pub mod vscode;

pub use dispatcher::ViewerDispatcher;
pub use sys_editor::SysEditorViewer;
pub use traits::DocumentViewer;
pub use vscode::VscodeViewer;
