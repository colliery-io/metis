use super::traits::{DocumentViewer, ViewerError};
use std::path::PathBuf;
use std::process::Command;
use tracing::info;

/// Known TUI editors that require a terminal to run.
const TUI_EDITORS: &[&str] = &["vim", "nvim", "vi", "nano", "emacs", "micro", "helix", "hx"];

/// System editor viewer backend.
///
/// Resolution order:
/// 1. `$EDITOR` environment variable
/// 2. `open` (macOS) / `xdg-open` (Linux) as OS default fallback
pub struct SysEditorViewer {
    editor: Option<String>,
}

impl Default for SysEditorViewer {
    fn default() -> Self {
        Self::new()
    }
}

impl SysEditorViewer {
    pub fn new() -> Self {
        let editor = std::env::var("EDITOR").ok();
        Self { editor }
    }

    /// Check if the resolved editor is a TUI editor that needs a terminal.
    fn is_tui_editor(editor: &str) -> bool {
        // Extract the binary name from the full path
        let bin_name = std::path::Path::new(editor)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(editor);

        TUI_EDITORS.contains(&bin_name)
    }

    /// Open files using the OS default handler (`open` on macOS, `xdg-open` on Linux).
    fn open_with_os_default(paths: &[PathBuf], background: bool) -> Result<(), ViewerError> {
        for path in paths {
            if cfg!(target_os = "macos") {
                let mut cmd = Command::new("open");
                if background {
                    cmd.arg("-g");
                }
                cmd.arg(path);
                cmd.spawn().map_err(|e| ViewerError::OpenFailed {
                    viewer: "System Editor".to_string(),
                    reason: format!("Failed to run 'open': {}", e),
                })?;
            } else {
                Command::new("xdg-open").arg(path).spawn().map_err(|e| {
                    ViewerError::OpenFailed {
                        viewer: "System Editor".to_string(),
                        reason: format!("Failed to run 'xdg-open': {}", e),
                    }
                })?;
            }
        }
        Ok(())
    }

    /// Open files using the configured $EDITOR.
    fn open_with_editor(editor: &str, paths: &[PathBuf]) -> Result<(), ViewerError> {
        if Self::is_tui_editor(editor) {
            // TUI editors need a terminal. On macOS, use `open -a Terminal` to spawn one.
            if cfg!(target_os = "macos") {
                for path in paths {
                    Command::new("open")
                        .arg("-a")
                        .arg("Terminal")
                        .arg(path)
                        .spawn()
                        .map_err(|e| ViewerError::OpenFailed {
                            viewer: "System Editor".to_string(),
                            reason: format!(
                                "Failed to open TUI editor '{}' in terminal: {}",
                                editor, e
                            ),
                        })?;
                }
            } else {
                // On Linux, try common terminal emulators
                let terminals = ["x-terminal-emulator", "gnome-terminal", "xterm"];
                let mut opened = false;

                for terminal in &terminals {
                    for path in paths {
                        let result = Command::new(terminal)
                            .arg("-e")
                            .arg(editor)
                            .arg(path)
                            .spawn();

                        if result.is_ok() {
                            opened = true;
                            break;
                        }
                    }
                    if opened {
                        break;
                    }
                }

                if !opened {
                    return Err(ViewerError::OpenFailed {
                        viewer: "System Editor".to_string(),
                        reason: format!(
                            "TUI editor '{}' requires a terminal, but no terminal emulator found. \
                             Set $EDITOR to a GUI editor or use 'code' viewer instead.",
                            editor
                        ),
                    });
                }
            }
        } else {
            // GUI editor — spawn detached with all files
            let mut cmd = Command::new(editor);
            for path in paths {
                cmd.arg(path);
            }
            cmd.spawn().map_err(|e| ViewerError::OpenFailed {
                viewer: "System Editor".to_string(),
                reason: format!("Failed to run '{}': {}", editor, e),
            })?;
        }

        Ok(())
    }
}

impl DocumentViewer for SysEditorViewer {
    fn open(&self, paths: &[PathBuf], background: bool) -> Result<(), ViewerError> {
        match &self.editor {
            Some(editor) => {
                info!("Opening with $EDITOR: {}", editor);
                Self::open_with_editor(editor, paths)
            }
            None => {
                info!("No $EDITOR set, using OS default handler");
                Self::open_with_os_default(paths, background)
            }
        }
    }

    fn is_open(&self, _path: &PathBuf) -> Result<bool, ViewerError> {
        // We don't have a reliable way to check if a file is open in an arbitrary
        // system editor. Return false and let the editor handle it.
        Ok(false)
    }

    fn name(&self) -> &str {
        "System Editor"
    }

    fn is_available(&self) -> bool {
        // Always available — at minimum, OS default handler (open/xdg-open) exists
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sys_editor_name() {
        let viewer = SysEditorViewer { editor: None };
        assert_eq!(viewer.name(), "System Editor");
    }

    #[test]
    fn test_sys_editor_always_available() {
        let viewer = SysEditorViewer { editor: None };
        assert!(viewer.is_available());
    }

    #[test]
    fn test_is_open_always_false() {
        let viewer = SysEditorViewer { editor: None };
        assert!(!viewer.is_open(&PathBuf::from("/tmp/test.md")).unwrap());
    }

    #[test]
    fn test_tui_editor_detection() {
        assert!(SysEditorViewer::is_tui_editor("vim"));
        assert!(SysEditorViewer::is_tui_editor("nvim"));
        assert!(SysEditorViewer::is_tui_editor("nano"));
        assert!(SysEditorViewer::is_tui_editor("/usr/bin/vim"));
        assert!(SysEditorViewer::is_tui_editor("/usr/local/bin/nvim"));

        assert!(!SysEditorViewer::is_tui_editor("code"));
        assert!(!SysEditorViewer::is_tui_editor("subl"));
        assert!(!SysEditorViewer::is_tui_editor("/usr/bin/code"));
    }
}
