use super::traits::{DocumentViewer, ViewerError};
use metis_core::domain::configuration::{ViewerBackend, ViewerConfig};
use std::path::PathBuf;
use tracing::{info, warn};

/// Dispatches open requests to the configured viewer backend,
/// with fallback chain on failure.
///
/// Resolution order:
/// 1. Explicit viewer override (if provided)
/// 2. Configured default from config.toml `[viewer].default`
/// 3. $EDITOR environment variable (resolved to sys_editor backend)
///
/// Fallback chain on failure: configured viewer → sys_editor → error
pub struct ViewerDispatcher {
    backends: Vec<Box<dyn DocumentViewer>>,
    config: ViewerConfig,
}

impl ViewerDispatcher {
    pub fn new(config: ViewerConfig, backends: Vec<Box<dyn DocumentViewer>>) -> Self {
        Self { backends, config }
    }

    /// Open documents in the appropriate viewer.
    ///
    /// Uses "look before you leap" — checks `is_open` before calling `open`
    /// for each path. Paths already open are skipped.
    ///
    /// If `viewer_override` is provided, it takes precedence over config.
    pub fn open(
        &self,
        paths: &[PathBuf],
        viewer_override: Option<&ViewerBackend>,
    ) -> Result<OpenResult, ViewerError> {
        if paths.is_empty() {
            return Ok(OpenResult {
                opened: vec![],
                skipped: vec![],
                viewer_used: "none".to_string(),
            });
        }

        // Determine which backend to try first
        let preferred = viewer_override.or(self.config.default.as_ref());

        // Build ordered list of backends to try
        let backend_order = self.resolve_backend_order(preferred);

        for backend in &backend_order {
            if !backend.is_available() {
                info!(
                    "Viewer '{}' not available, trying next in fallback chain",
                    backend.name()
                );
                continue;
            }

            // Look before you leap: filter out already-open paths
            let mut to_open = Vec::new();
            let mut skipped = Vec::new();

            for path in paths {
                match backend.is_open(path) {
                    Ok(true) => {
                        info!("Skipping '{}' — already open in {}", path.display(), backend.name());
                        skipped.push(path.clone());
                    }
                    Ok(false) => {
                        to_open.push(path.clone());
                    }
                    Err(e) => {
                        // If we can't check, assume not open and try to open
                        warn!(
                            "Could not check open status for '{}' in {}: {}",
                            path.display(),
                            backend.name(),
                            e
                        );
                        to_open.push(path.clone());
                    }
                }
            }

            if to_open.is_empty() {
                return Ok(OpenResult {
                    opened: vec![],
                    skipped,
                    viewer_used: backend.name().to_string(),
                });
            }

            match backend.open(&to_open) {
                Ok(()) => {
                    info!(
                        "Opened {} document(s) in {}",
                        to_open.len(),
                        backend.name()
                    );
                    return Ok(OpenResult {
                        opened: to_open,
                        skipped,
                        viewer_used: backend.name().to_string(),
                    });
                }
                Err(e) => {
                    warn!(
                        "Failed to open in '{}': {}. Trying next backend...",
                        backend.name(),
                        e
                    );
                    continue;
                }
            }
        }

        Err(ViewerError::NoViewerAvailable)
    }

    /// Whether proactive opening is suppressed by config.
    pub fn is_proactive_opening_suppressed(&self) -> bool {
        self.config.suppress_proactive_ticket_opening
    }

    /// Resolve the ordered list of backends to try.
    /// Preferred backend goes first, then sys_editor as fallback.
    fn resolve_backend_order(
        &self,
        preferred: Option<&ViewerBackend>,
    ) -> Vec<&dyn DocumentViewer> {
        let mut order: Vec<&dyn DocumentViewer> = Vec::new();

        // If there's a preferred backend, put it first
        if let Some(pref) = preferred {
            let pref_name = match pref {
                ViewerBackend::Code => "VSCode",
                ViewerBackend::SysEditor => "System Editor",
                ViewerBackend::Gui => "Metis GUI",
            };

            if let Some(backend) = self.backends.iter().find(|b| b.name() == pref_name) {
                order.push(backend.as_ref());
            }
        }

        // Add sys_editor as fallback (if not already in the list)
        if let Some(sys_editor) = self.backends.iter().find(|b| b.name() == "System Editor") {
            if !order.iter().any(|b| b.name() == "System Editor") {
                order.push(sys_editor.as_ref());
            }
        }

        // Add any remaining backends as last-resort fallbacks
        for backend in &self.backends {
            if !order.iter().any(|b| b.name() == backend.name()) {
                order.push(backend.as_ref());
            }
        }

        order
    }
}

/// Result of an open operation.
#[derive(Debug)]
pub struct OpenResult {
    /// Paths that were successfully opened.
    pub opened: Vec<PathBuf>,
    /// Paths that were skipped because they were already open.
    pub skipped: Vec<PathBuf>,
    /// Name of the viewer backend that was used.
    pub viewer_used: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    /// Stub viewer for testing
    struct StubViewer {
        name: String,
        available: bool,
        open_succeeds: bool,
        files_open: Vec<PathBuf>,
        open_called: Arc<AtomicBool>,
    }

    impl StubViewer {
        fn new(name: &str, available: bool, open_succeeds: bool) -> Self {
            Self {
                name: name.to_string(),
                available,
                open_succeeds,
                files_open: vec![],
                open_called: Arc::new(AtomicBool::new(false)),
            }
        }

        fn with_open_files(mut self, files: Vec<PathBuf>) -> Self {
            self.files_open = files;
            self
        }
    }

    impl DocumentViewer for StubViewer {
        fn open(&self, _paths: &[PathBuf]) -> Result<(), ViewerError> {
            self.open_called.store(true, Ordering::SeqCst);
            if self.open_succeeds {
                Ok(())
            } else {
                Err(ViewerError::OpenFailed {
                    viewer: self.name.clone(),
                    reason: "stub failure".to_string(),
                })
            }
        }

        fn is_open(&self, path: &PathBuf) -> Result<bool, ViewerError> {
            Ok(self.files_open.contains(path))
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    #[test]
    fn test_dispatcher_uses_configured_backend() {
        let config = ViewerConfig {
            default: Some(ViewerBackend::Code),
            suppress_proactive_ticket_opening: false,
        };

        let vscode = StubViewer::new("VSCode", true, true);
        let open_called = vscode.open_called.clone();
        let sys_editor = StubViewer::new("System Editor", true, true);

        let dispatcher = ViewerDispatcher::new(
            config,
            vec![Box::new(vscode), Box::new(sys_editor)],
        );

        let result = dispatcher.open(&[PathBuf::from("/tmp/test.md")], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.viewer_used, "VSCode");
        assert!(open_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatcher_fallback_on_failure() {
        let config = ViewerConfig {
            default: Some(ViewerBackend::Code),
            suppress_proactive_ticket_opening: false,
        };

        let vscode = StubViewer::new("VSCode", true, false); // fails
        let sys_editor = StubViewer::new("System Editor", true, true); // succeeds
        let sys_open_called = sys_editor.open_called.clone();

        let dispatcher = ViewerDispatcher::new(
            config,
            vec![Box::new(vscode), Box::new(sys_editor)],
        );

        let result = dispatcher.open(&[PathBuf::from("/tmp/test.md")], None);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.viewer_used, "System Editor");
        assert!(sys_open_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatcher_fallback_on_unavailable() {
        let config = ViewerConfig {
            default: Some(ViewerBackend::Code),
            suppress_proactive_ticket_opening: false,
        };

        let vscode = StubViewer::new("VSCode", false, true); // not available
        let sys_editor = StubViewer::new("System Editor", true, true);

        let dispatcher = ViewerDispatcher::new(
            config,
            vec![Box::new(vscode), Box::new(sys_editor)],
        );

        let result = dispatcher.open(&[PathBuf::from("/tmp/test.md")], None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().viewer_used, "System Editor");
    }

    #[test]
    fn test_dispatcher_skips_already_open() {
        let config = ViewerConfig {
            default: Some(ViewerBackend::Code),
            suppress_proactive_ticket_opening: false,
        };

        let open_path = PathBuf::from("/tmp/already_open.md");
        let vscode = StubViewer::new("VSCode", true, true)
            .with_open_files(vec![open_path.clone()]);
        let open_called = vscode.open_called.clone();

        let dispatcher = ViewerDispatcher::new(config, vec![Box::new(vscode)]);

        let result = dispatcher.open(&[open_path.clone()], None).unwrap();
        assert!(result.opened.is_empty());
        assert_eq!(result.skipped.len(), 1);
        // open() should NOT have been called since everything was already open
        assert!(!open_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatcher_no_backends_available() {
        let config = ViewerConfig::default();
        let dispatcher = ViewerDispatcher::new(config, vec![]);

        let result = dispatcher.open(&[PathBuf::from("/tmp/test.md")], None);
        assert!(result.is_err());
    }

    #[test]
    fn test_dispatcher_viewer_override() {
        let config = ViewerConfig {
            default: Some(ViewerBackend::Code),
            suppress_proactive_ticket_opening: false,
        };

        let vscode = StubViewer::new("VSCode", true, true);
        let sys_editor = StubViewer::new("System Editor", true, true);
        let sys_open_called = sys_editor.open_called.clone();

        let dispatcher = ViewerDispatcher::new(
            config,
            vec![Box::new(vscode), Box::new(sys_editor)],
        );

        // Override to sys_editor even though config says code
        let result = dispatcher
            .open(&[PathBuf::from("/tmp/test.md")], Some(&ViewerBackend::SysEditor))
            .unwrap();
        assert_eq!(result.viewer_used, "System Editor");
        assert!(sys_open_called.load(Ordering::SeqCst));
    }

    #[test]
    fn test_dispatcher_empty_paths() {
        let config = ViewerConfig::default();
        let dispatcher = ViewerDispatcher::new(config, vec![]);

        let result = dispatcher.open(&[], None).unwrap();
        assert!(result.opened.is_empty());
        assert!(result.skipped.is_empty());
    }

    #[test]
    fn test_suppress_proactive_opening() {
        let config = ViewerConfig {
            default: None,
            suppress_proactive_ticket_opening: true,
        };

        let dispatcher = ViewerDispatcher::new(config, vec![]);
        assert!(dispatcher.is_proactive_opening_suppressed());
    }

    #[test]
    fn test_env_fallback_when_no_default() {
        // When no default is configured, the dispatcher should still
        // try sys_editor (which reads $EDITOR internally)
        let config = ViewerConfig {
            default: None,
            suppress_proactive_ticket_opening: false,
        };

        let sys_editor = StubViewer::new("System Editor", true, true);
        let dispatcher = ViewerDispatcher::new(config, vec![Box::new(sys_editor)]);

        let result = dispatcher.open(&[PathBuf::from("/tmp/test.md")], None);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().viewer_used, "System Editor");
    }
}
