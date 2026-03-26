//! Tracks when documents were last read to prevent stale edits.
//!
//! The MCP server maintains this in-memory map for the session.
//! When a document is read via `read_document`, the current time is recorded.
//! When `edit_document` is called, the file's mtime is compared against the
//! last-read time. If the file was modified externally since the last read,
//! the edit is rejected.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::SystemTime;

/// In-memory tracker for document read timestamps.
/// Thread-safe via internal `Mutex`.
#[derive(Debug)]
pub struct DocumentReadTracker {
    last_read: Mutex<HashMap<PathBuf, SystemTime>>,
}

impl DocumentReadTracker {
    pub fn new() -> Self {
        Self {
            last_read: Mutex::new(HashMap::new()),
        }
    }

    /// Record that a document was just read. Called after successful `read_document`.
    pub fn record_read(&self, path: &Path) {
        let canonical = path.to_path_buf();
        let mut map = self.last_read.lock().unwrap();
        map.insert(canonical, SystemTime::now());
    }

    /// Check whether a document can be safely edited.
    ///
    /// Returns:
    /// - `Ok(())` if the document was read and hasn't been modified since
    /// - `Err(ReadGuardError::NeverRead)` if the document was never read in this session
    /// - `Err(ReadGuardError::StaleRead)` if the file was modified after the last read
    /// - `Err(ReadGuardError::MtimeUnavailable)` if we can't stat the file
    pub fn check_edit_allowed(&self, path: &Path) -> Result<(), ReadGuardError> {
        let canonical = path.to_path_buf();
        let map = self.last_read.lock().unwrap();

        let last_read = map.get(&canonical).ok_or(ReadGuardError::NeverRead)?;

        // Get file mtime
        let metadata = std::fs::metadata(path).map_err(|e| ReadGuardError::MtimeUnavailable {
            reason: e.to_string(),
        })?;

        let mtime = metadata
            .modified()
            .map_err(|e| ReadGuardError::MtimeUnavailable {
                reason: e.to_string(),
            })?;

        // Allow a small tolerance (1 second) for filesystem granularity
        let tolerance = std::time::Duration::from_secs(1);
        if mtime > *last_read + tolerance {
            return Err(ReadGuardError::StaleRead {
                last_read: *last_read,
                file_mtime: mtime,
            });
        }

        Ok(())
    }

    /// Update the last-read timestamp after a successful edit.
    /// This prevents the guard from rejecting subsequent edits to the same file
    /// in the same session (since our own write updated the mtime).
    pub fn record_edit(&self, path: &Path) {
        // Reuse record_read — same semantics
        self.record_read(path);
    }
}

impl Default for DocumentReadTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub enum ReadGuardError {
    /// Document was never read in this session.
    NeverRead,
    /// Document was modified after the last read.
    StaleRead {
        last_read: SystemTime,
        file_mtime: SystemTime,
    },
    /// Could not check file mtime.
    MtimeUnavailable { reason: String },
}

impl std::fmt::Display for ReadGuardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReadGuardError::NeverRead => {
                write!(
                    f,
                    "Document must be read before editing. Use `read_document` first, then retry the edit."
                )
            }
            ReadGuardError::StaleRead { .. } => {
                write!(
                    f,
                    "Document was modified externally since your last read. \
                     Use `read_document` to get the current content, then retry the edit."
                )
            }
            ReadGuardError::MtimeUnavailable { reason } => {
                write!(f, "Could not check file modification time: {}", reason)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_then_edit_succeeds() {
        let tracker = DocumentReadTracker::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "initial content").unwrap();

        tracker.record_read(file.path());

        // Edit immediately should succeed
        assert!(tracker.check_edit_allowed(file.path()).is_ok());
    }

    #[test]
    fn test_edit_without_read_fails() {
        let tracker = DocumentReadTracker::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "content").unwrap();

        let result = tracker.check_edit_allowed(file.path());
        assert!(matches!(result, Err(ReadGuardError::NeverRead)));
    }

    #[test]
    fn test_edit_after_external_modify_fails() {
        let tracker = DocumentReadTracker::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "initial content").unwrap();

        tracker.record_read(file.path());

        // Wait a moment and modify the file externally
        std::thread::sleep(std::time::Duration::from_secs(2));
        writeln!(file, "modified externally").unwrap();
        file.flush().unwrap();

        let result = tracker.check_edit_allowed(file.path());
        assert!(matches!(result, Err(ReadGuardError::StaleRead { .. })));
    }

    #[test]
    fn test_record_edit_allows_subsequent_edits() {
        let tracker = DocumentReadTracker::new();
        let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "content").unwrap();

        tracker.record_read(file.path());
        // Simulate our own edit updating mtime
        tracker.record_edit(file.path());

        assert!(tracker.check_edit_allowed(file.path()).is_ok());
    }
}
