use super::traits::{DocumentViewer, ViewerError};
use std::path::PathBuf;
use std::process::Command;

/// VSCode viewer backend that opens documents via the `code` CLI.
pub struct VscodeViewer {
    /// Cached availability check result
    available: bool,
}

impl Default for VscodeViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl VscodeViewer {
    pub fn new() -> Self {
        let available = Self::check_available();
        Self { available }
    }

    fn check_available() -> bool {
        Command::new("which")
            .arg("code")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    }
}

impl DocumentViewer for VscodeViewer {
    fn open(&self, paths: &[PathBuf], background: bool) -> Result<(), ViewerError> {
        if !self.available {
            return Err(ViewerError::NotAvailable {
                viewer: "VSCode".to_string(),
                reason: "'code' command not found on PATH".to_string(),
            });
        }

        if background && cfg!(target_os = "macos") {
            // Use `open -g` to open in VSCode without stealing focus
            let mut cmd = Command::new("open");
            cmd.arg("-g").arg("-a").arg("Visual Studio Code");
            for path in paths {
                cmd.arg(path);
            }
            cmd.spawn().map_err(|e| ViewerError::OpenFailed {
                viewer: "VSCode".to_string(),
                reason: e.to_string(),
            })?;
        } else {
            let mut cmd = Command::new("code");
            cmd.arg("--reuse-window");
            for path in paths {
                cmd.arg(path);
            }
            cmd.spawn().map_err(|e| ViewerError::OpenFailed {
                viewer: "VSCode".to_string(),
                reason: e.to_string(),
            })?;
        }

        Ok(())
    }

    fn is_open(&self, _path: &PathBuf) -> Result<bool, ViewerError> {
        // VSCode CLI doesn't provide a reliable way to check if a specific file
        // is open in a tab. The `--reuse-window` flag handles the "don't spawn
        // new windows" concern, and VSCode itself won't duplicate tabs for the
        // same file. So we return false and let VSCode handle dedup internally.
        Ok(false)
    }

    fn name(&self) -> &str {
        "VSCode"
    }

    fn is_available(&self) -> bool {
        self.available
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vscode_viewer_name() {
        let viewer = VscodeViewer { available: false };
        assert_eq!(viewer.name(), "VSCode");
    }

    #[test]
    fn test_vscode_viewer_not_available_returns_error() {
        let viewer = VscodeViewer { available: false };
        let result = viewer.open(&[PathBuf::from("/tmp/test.md")], false);
        assert!(result.is_err());
        match result.unwrap_err() {
            ViewerError::NotAvailable { viewer, .. } => assert_eq!(viewer, "VSCode"),
            _ => panic!("Expected NotAvailable error"),
        }
    }

    #[test]
    fn test_is_open_always_false() {
        let viewer = VscodeViewer { available: false };
        // VSCode handles tab dedup internally
        assert!(!viewer.is_open(&PathBuf::from("/tmp/test.md")).unwrap());
    }
}
