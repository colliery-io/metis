use crate::dal::git::GitRepo;
use crate::{MetisError, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

/// Storage backend determines where .metis/ files are read from and written to.
pub enum StorageBackend {
    /// Direct filesystem operations (used when on main/master or outside a git repo)
    Local,
    /// Git-backed overlay: reads from main's tree, writes to a pending overlay directory.
    /// Pending changes are flushed to main on git commit via a post-commit hook.
    /// Stores paths instead of GitRepo to avoid Send/Sync issues with git2::Repository.
    GitOverlay {
        /// The .metis/ workspace directory (absolute path) — used to re-discover the git repo
        workspace_dir: PathBuf,
        /// Directory for pending writes (e.g., .metis/.pending/)
        overlay_dir: PathBuf,
    },
}

/// Filesystem operations service
/// Handles reading/writing documents to disk and computing file hashes.
/// Dispatches through a `StorageBackend` to support branch-independent .metis/ storage.
pub struct FilesystemService {
    backend: StorageBackend,
}

impl FilesystemService {
    /// Create a new FilesystemService with automatic backend detection.
    /// If inside a git repo and not on main/master, uses GitOverlay backend.
    /// Otherwise, uses Local backend.
    pub fn new<P: AsRef<Path>>(workspace_path: P) -> Self {
        let workspace = workspace_path.as_ref();

        // Try to detect git repo and branch
        if let Some(git_repo) = GitRepo::discover(workspace) {
            if !git_repo.is_on_main() {
                let overlay_dir = workspace.join(".pending");
                return Self {
                    backend: StorageBackend::GitOverlay {
                        workspace_dir: workspace.to_path_buf(),
                        overlay_dir,
                    },
                };
            }
        }

        Self {
            backend: StorageBackend::Local,
        }
    }

    /// Create a FilesystemService with the Local backend (for tests or non-git contexts).
    pub fn local() -> Self {
        Self {
            backend: StorageBackend::Local,
        }
    }

    /// Check if this service is using the GitOverlay backend.
    pub fn is_git_overlay(&self) -> bool {
        matches!(self.backend, StorageBackend::GitOverlay { .. })
    }

    /// Open the git repo for the current workspace. Called lazily per-operation.
    fn open_git_repo(&self) -> Option<GitRepo> {
        match &self.backend {
            StorageBackend::GitOverlay { workspace_dir, .. } => {
                GitRepo::discover(workspace_dir)
            }
            StorageBackend::Local => None,
        }
    }

    /// Canonicalize a path, falling back to the original if canonicalization fails.
    fn canonical(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    /// Convert an absolute file path to a repo-relative tree path for git2 blob lookup.
    /// e.g., `/project/.metis/initiatives/FOO/initiative.md` → `.metis/initiatives/FOO/initiative.md`
    fn to_tree_path(&self, path: &Path, git_repo: &GitRepo) -> Option<String> {
        match &self.backend {
            StorageBackend::GitOverlay {
                workspace_dir,
                ..
            } => {
                let repo_root = Self::canonical(git_repo.workdir()?);
                let canon_path = Self::canonical(path);
                canon_path
                    .strip_prefix(&repo_root)
                    .ok()
                    .map(|p| p.to_string_lossy().to_string())
                    .or_else(|| {
                        let ws_parent = Self::canonical(
                            workspace_dir.parent()?,
                        );
                        canon_path
                            .strip_prefix(&ws_parent)
                            .ok()
                            .map(|p| p.to_string_lossy().to_string())
                    })
            }
            StorageBackend::Local => None,
        }
    }

    /// Convert an absolute path to its overlay-relative path.
    /// e.g., `/project/.metis/initiatives/FOO/initiative.md` → `.pending/initiatives/FOO/initiative.md`
    fn to_overlay_path(&self, path: &Path) -> Option<PathBuf> {
        match &self.backend {
            StorageBackend::GitOverlay {
                workspace_dir,
                overlay_dir,
                ..
            } => {
                // Canonicalize to handle macOS /var vs /private/var
                let canon_ws = Self::canonical(workspace_dir);
                let canon_path = Self::canonical(path);
                // Try canonicalized first, fall back to raw
                let relative = canon_path
                    .strip_prefix(&canon_ws)
                    .or_else(|_| path.strip_prefix(workspace_dir))
                    .ok()?;
                Some(overlay_dir.join(relative))
            }
            StorageBackend::Local => None,
        }
    }

    /// Read file contents from main's tree via git2, with no overlay check.
    /// This is the pure git read path — overlay merging is added in T-0120.
    fn read_from_git(&self, path: &Path) -> Result<String> {
        match &self.backend {
            StorageBackend::GitOverlay { .. } => {
                let git_repo = self.open_git_repo().ok_or_else(|| {
                    MetisError::FileSystem("Cannot open git repository".to_string())
                })?;

                let tree_path = self.to_tree_path(path, &git_repo).ok_or_else(|| {
                    MetisError::FileSystem(format!(
                        "Cannot resolve tree path for: {}",
                        path.display()
                    ))
                })?;

                git_repo.read_blob(&tree_path).ok_or_else(|| {
                    MetisError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("File not found in main's tree: {}", tree_path),
                    ))
                })
            }
            StorageBackend::Local => unreachable!(),
        }
    }

    /// Check if a file has been tombstoned (deleted in the overlay).
    fn is_tombstoned(&self, path: &Path) -> bool {
        if let Some(overlay_path) = self.to_overlay_path(path) {
            let tombstone = overlay_path.with_extension(
                overlay_path
                    .extension()
                    .map(|e| format!("{}.deleted", e.to_string_lossy()))
                    .unwrap_or_else(|| "deleted".to_string()),
            );
            tombstone.exists()
        } else {
            false
        }
    }

    /// Read file contents
    pub fn read_file<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        match &self.backend {
            StorageBackend::Local => fs::read_to_string(path).map_err(MetisError::Io),
            StorageBackend::GitOverlay { .. } => {
                let path = path.as_ref();

                // Check tombstone first — if deleted in overlay, it doesn't exist
                if self.is_tombstoned(path) {
                    return Err(MetisError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("File deleted in overlay: {}", path.display()),
                    )));
                }

                // Check overlay first — local writes take precedence
                if let Some(overlay_path) = self.to_overlay_path(path) {
                    if overlay_path.exists() {
                        return fs::read_to_string(&overlay_path).map_err(MetisError::Io);
                    }
                }

                // Fall back to git tree
                self.read_from_git(path)
            }
        }
    }

    /// Write file contents
    pub fn write_file<P: AsRef<Path>>(&self, path: P, content: &str) -> Result<()> {
        match &self.backend {
            StorageBackend::Local => {
                if let Some(parent) = path.as_ref().parent() {
                    fs::create_dir_all(parent).map_err(MetisError::Io)?;
                }
                fs::write(path, content).map_err(MetisError::Io)
            }
            StorageBackend::GitOverlay { .. } => {
                let path = path.as_ref();
                let overlay_path = self.to_overlay_path(path).ok_or_else(|| {
                    MetisError::FileSystem(format!(
                        "Cannot resolve overlay path for: {}",
                        path.display()
                    ))
                })?;

                // Ensure parent directory exists in overlay
                if let Some(parent) = overlay_path.parent() {
                    fs::create_dir_all(parent).map_err(MetisError::Io)?;
                }

                // Remove any tombstone for this file
                let tombstone = overlay_path.with_extension(
                    overlay_path
                        .extension()
                        .map(|e| format!("{}.deleted", e.to_string_lossy()))
                        .unwrap_or_else(|| "deleted".to_string()),
                );
                if tombstone.exists() {
                    let _ = fs::remove_file(&tombstone);
                }

                fs::write(&overlay_path, content).map_err(MetisError::Io)
            }
        }
    }

    /// Check if file exists
    pub fn file_exists<P: AsRef<Path>>(&self, path: P) -> bool {
        match &self.backend {
            StorageBackend::Local => path.as_ref().exists(),
            StorageBackend::GitOverlay { .. } => {
                let path = path.as_ref();

                // Tombstoned = deleted in overlay
                if self.is_tombstoned(path) {
                    return false;
                }

                // Check overlay
                if let Some(overlay_path) = self.to_overlay_path(path) {
                    if overlay_path.exists() {
                        return true;
                    }
                }

                // Check git tree
                if let Some(git_repo) = self.open_git_repo() {
                    if let Some(tree_path) = self.to_tree_path(path, &git_repo) {
                        git_repo.blob_exists(&tree_path)
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Compute SHA256 hash of file contents
    pub fn compute_file_hash<P: AsRef<Path>>(&self, path: P) -> Result<String> {
        let contents = self.read_file(path)?;
        Ok(Self::compute_content_hash(&contents))
    }

    /// Compute SHA256 hash of string content (stateless — no backend dispatch needed)
    pub fn compute_content_hash(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    /// Get file modification time as Unix timestamp
    pub fn get_file_mtime<P: AsRef<Path>>(&self, path: P) -> Result<f64> {
        match &self.backend {
            StorageBackend::Local => {
                let metadata = fs::metadata(path).map_err(MetisError::Io)?;
                let mtime = metadata
                    .modified()
                    .map_err(MetisError::Io)?
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|_| MetisError::ValidationFailed {
                        message: "Invalid file modification time".to_string(),
                    })?;
                Ok(mtime.as_secs_f64())
            }
            StorageBackend::GitOverlay { .. } => {
                // Use main's HEAD commit time as the best available proxy
                let git_repo = self.open_git_repo().ok_or_else(|| {
                    MetisError::ValidationFailed {
                        message: "Cannot open git repository".to_string(),
                    }
                })?;
                git_repo.main_head_commit_time().ok_or_else(|| {
                    MetisError::ValidationFailed {
                        message: "Cannot determine main branch commit time".to_string(),
                    }
                })
            }
        }
    }

    /// Create directories recursively
    pub fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match &self.backend {
            StorageBackend::Local => {
                fs::create_dir_all(path).map_err(MetisError::Io)
            }
            StorageBackend::GitOverlay { .. } => {
                // No-op: git doesn't track directories, and write_file creates parents in the overlay
                Ok(())
            }
        }
    }

    /// Rename/move a file
    pub fn rename_file<P: AsRef<Path>, Q: AsRef<Path>>(&self, from: P, to: Q) -> Result<()> {
        match &self.backend {
            StorageBackend::Local => {
                // Ensure destination parent exists
                if let Some(parent) = to.as_ref().parent() {
                    fs::create_dir_all(parent).map_err(MetisError::Io)?;
                }
                fs::rename(from, to).map_err(MetisError::Io)
            }
            StorageBackend::GitOverlay { .. } => {
                // Read from source (overlay or git), write to new location in overlay, tombstone old
                let content = self.read_file(&from)?;
                self.write_file(&to, &content)?;
                self.delete_file(&from)?;
                Ok(())
            }
        }
    }

    /// Delete a file
    pub fn delete_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        match &self.backend {
            StorageBackend::Local => fs::remove_file(path).map_err(MetisError::Io),
            StorageBackend::GitOverlay { .. } => {
                let path = path.as_ref();
                let overlay_path = self.to_overlay_path(path).ok_or_else(|| {
                    MetisError::FileSystem(format!(
                        "Cannot resolve overlay path for: {}",
                        path.display()
                    ))
                })?;

                // Remove the overlay file if it exists
                if overlay_path.exists() {
                    fs::remove_file(&overlay_path).map_err(MetisError::Io)?;
                }

                // Create a tombstone so reads don't fall through to main's tree
                let tombstone = overlay_path.with_extension(
                    overlay_path
                        .extension()
                        .map(|e| format!("{}.deleted", e.to_string_lossy()))
                        .unwrap_or_else(|| "deleted".to_string()),
                );
                if let Some(parent) = tombstone.parent() {
                    fs::create_dir_all(parent).map_err(MetisError::Io)?;
                }
                fs::write(&tombstone, "").map_err(MetisError::Io)
            }
        }
    }

    /// List all markdown files in a directory recursively
    pub fn find_markdown_files<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<String>> {
        match &self.backend {
            StorageBackend::Local => Self::find_markdown_files_local(dir),
            StorageBackend::GitOverlay {
                workspace_dir,
                overlay_dir,
                ..
            } => {
                let dir_path = dir.as_ref();
                let git_repo = self.open_git_repo().ok_or_else(|| {
                    MetisError::FileSystem("Cannot open git repository".to_string())
                })?;
                let repo_root_raw = git_repo.workdir().ok_or_else(|| {
                    MetisError::FileSystem("Cannot determine repo root".to_string())
                })?;
                let repo_root = Self::canonical(repo_root_raw);
                let dir_canon = Self::canonical(dir_path);

                // Convert the directory to a tree prefix for git2 lookup
                let tree_prefix = dir_canon
                    .strip_prefix(&repo_root)
                    .ok()
                    .map(|p| {
                        let s = p.to_string_lossy().to_string();
                        if s.ends_with('/') || s.is_empty() {
                            s
                        } else {
                            format!("{}/", s)
                        }
                    })
                    .unwrap_or_default();

                // Get files from main's tree
                let git_files = git_repo.list_markdown_files(&tree_prefix);

                // Convert to absolute paths and collect into a set
                use std::collections::HashSet;
                let mut result_set: HashSet<String> = HashSet::new();

                for relative_path in &git_files {
                    let absolute = repo_root.join(relative_path);
                    let abs_str = absolute.to_string_lossy().to_string();

                    // Check if this file is tombstoned
                    if !self.is_tombstoned(&absolute) {
                        result_set.insert(abs_str);
                    }
                }

                // Add overlay files
                let overlay_prefix = dir_path
                    .strip_prefix(workspace_dir)
                    .ok()
                    .map(|p| overlay_dir.join(p))
                    .unwrap_or_else(|| overlay_dir.clone());

                if overlay_prefix.exists() {
                    use walkdir::WalkDir;
                    for entry in WalkDir::new(&overlay_prefix).follow_links(true).into_iter().flatten() {
                        if entry.file_type().is_file() {
                            if let Some(name) = entry.file_name().to_str() {
                                // Skip tombstones and non-md files
                                if name.ends_with(".deleted") || !name.ends_with(".md") {
                                    continue;
                                }
                                if name == "code-index.md" {
                                    continue;
                                }
                                // Convert overlay path back to "real" absolute path
                                if let Ok(rel) = entry.path().strip_prefix(overlay_dir) {
                                    let absolute = workspace_dir.join(rel);
                                    result_set.insert(absolute.to_string_lossy().to_string());
                                }
                            }
                        }
                    }
                }

                let mut files: Vec<String> = result_set.into_iter().collect();
                files.sort();
                Ok(files)
            }
        }
    }

    /// Local implementation of find_markdown_files (also used by tests)
    fn find_markdown_files_local<P: AsRef<Path>>(dir: P) -> Result<Vec<String>> {
        use walkdir::WalkDir;

        let mut files = Vec::new();

        for entry in WalkDir::new(dir).follow_links(true) {
            let entry = entry
                .map_err(|e| MetisError::Io(std::io::Error::other(format!("Walk error: {}", e))))?;

            if entry.file_type().is_file() {
                if let Some(path_str) = entry.path().to_str() {
                    if path_str.ends_with(".md") {
                        if entry.file_name() == "code-index.md" {
                            continue;
                        }
                        files.push(path_str.to_string());
                    }
                }
            }
        }

        Ok(files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn local_fs() -> FilesystemService {
        FilesystemService::local()
    }

    #[test]
    fn test_write_and_read_file() {
        let fs = local_fs();
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.md");

        let content = "# Test Document\n\nThis is test content.";

        fs.write_file(&file_path, content).expect("Failed to write file");
        let read_content = fs.read_file(&file_path).expect("Failed to read file");
        assert_eq!(content, read_content);
        assert!(fs.file_exists(&file_path));
    }

    #[test]
    fn test_compute_hashes() {
        let fs = local_fs();
        let content = "# Test Document\n\nThis is test content.";

        let hash1 = FilesystemService::compute_content_hash(content);
        let hash2 = FilesystemService::compute_content_hash(content);
        assert_eq!(hash1, hash2);

        let different_content = "# Different Document\n\nThis is different content.";
        let hash3 = FilesystemService::compute_content_hash(different_content);
        assert_ne!(hash1, hash3);

        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("test.md");
        fs.write_file(&file_path, content).expect("Failed to write file");

        let file_hash = fs.compute_file_hash(&file_path).expect("Failed to compute file hash");
        assert_eq!(hash1, file_hash);
    }

    #[test]
    fn test_file_operations() {
        let fs = local_fs();
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let file_path = temp_dir.path().join("subdir").join("test.md");

        let content = "# Test Document";

        fs.write_file(&file_path, content).expect("Failed to write file");
        assert!(fs.file_exists(&file_path));

        let mtime = fs.get_file_mtime(&file_path).expect("Failed to get mtime");
        assert!(mtime > 0.0);

        fs.delete_file(&file_path).expect("Failed to delete file");
        assert!(!fs.file_exists(&file_path));
    }

    #[test]
    fn test_find_markdown_files() {
        let fs = local_fs();
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        let files = vec![
            "doc1.md",
            "subdir/doc2.md",
            "subdir/nested/doc3.md",
            "not_markdown.txt",
        ];

        for file in &files {
            let file_path = base_path.join(file);
            fs.write_file(&file_path, "# Test").expect("Failed to write file");
        }

        let found_files = fs.find_markdown_files(base_path)
            .expect("Failed to find markdown files");

        assert_eq!(found_files.len(), 3);

        for file in &found_files {
            assert!(file.ends_with(".md"));
        }
    }

    #[test]
    fn test_find_markdown_files_excludes_code_index() {
        let fs = local_fs();
        let temp_dir = tempdir().expect("Failed to create temp dir");
        let base_path = temp_dir.path();

        let files = vec![
            "vision.md",
            "code-index.md",
            "initiatives/init-1/initiative.md",
        ];

        for file in &files {
            let file_path = base_path.join(file);
            fs.write_file(&file_path, "# Test").expect("Failed to write file");
        }

        let found_files = fs.find_markdown_files(base_path)
            .expect("Failed to find markdown files");

        assert_eq!(found_files.len(), 2);
        assert!(found_files.iter().all(|f| !f.contains("code-index.md")));
    }

    // --- GitOverlay tests ---

    use git2::{Repository, Signature};

    /// Create a git repo with .metis/ files on main, switch to feature branch,
    /// return a FilesystemService with GitOverlay backend.
    fn setup_overlay_test() -> (tempfile::TempDir, FilesystemService) {
        let dir = tempdir().unwrap();
        let repo = Repository::init(dir.path()).unwrap();

        // Create .metis/ workspace with a document
        let metis_dir = dir.path().join(".metis");
        fs::create_dir_all(metis_dir.join("initiatives/METIS-I-0001")).unwrap();
        fs::write(
            metis_dir.join("initiatives/METIS-I-0001/initiative.md"),
            "# Original Initiative\n\nOriginal content.",
        )
        .unwrap();
        fs::write(metis_dir.join("config.toml"), "[project]\nprefix = \"METIS\"")
            .unwrap();

        // Commit on main
        let sig = Signature::now("Test", "test@test.com").unwrap();
        let tree_id = {
            let mut index = repo.index().unwrap();
            index
                .add_all(["*"].iter(), git2::IndexAddOption::DEFAULT, None)
                .unwrap();
            index.write().unwrap();
            index.write_tree().unwrap()
        };
        {
            let tree = repo.find_tree(tree_id).unwrap();
            repo.commit(Some("HEAD"), &sig, &sig, "initial", &tree, &[])
                .unwrap();
        }

        // Switch to feature branch
        {
            let commit = repo.head().unwrap().peel_to_commit().unwrap();
            repo.branch("feature/test", &commit, false).unwrap();
        }
        repo.set_head("refs/heads/feature/test").unwrap();

        let fs_service = FilesystemService::new(&metis_dir);
        assert!(fs_service.is_git_overlay());

        (dir, fs_service)
    }

    #[test]
    fn test_overlay_read_from_main() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        let content = fs
            .read_file(metis_dir.join("initiatives/METIS-I-0001/initiative.md"))
            .unwrap();
        assert!(content.contains("Original Initiative"));
    }

    #[test]
    fn test_overlay_write_then_read() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");
        let file_path = metis_dir.join("initiatives/METIS-I-0001/initiative.md");

        // Write to overlay
        fs.write_file(&file_path, "# Modified Initiative\n\nNew content.")
            .unwrap();

        // Read should return overlay content, not git content
        let content = fs.read_file(&file_path).unwrap();
        assert!(content.contains("Modified Initiative"));
        assert!(!content.contains("Original Initiative"));
    }

    #[test]
    fn test_overlay_write_new_file() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");
        let new_file = metis_dir.join("initiatives/METIS-I-0002/initiative.md");

        assert!(!fs.file_exists(&new_file));

        fs.write_file(&new_file, "# New Initiative").unwrap();

        assert!(fs.file_exists(&new_file));
        let content = fs.read_file(&new_file).unwrap();
        assert_eq!(content, "# New Initiative");
    }

    #[test]
    fn test_overlay_delete_creates_tombstone() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");
        let file_path = metis_dir.join("initiatives/METIS-I-0001/initiative.md");

        // File exists on main
        assert!(fs.file_exists(&file_path));

        // Delete it
        fs.delete_file(&file_path).unwrap();

        // Now it shouldn't exist (tombstone blocks fallthrough to main)
        assert!(!fs.file_exists(&file_path));

        // Reading should fail
        assert!(fs.read_file(&file_path).is_err());
    }

    #[test]
    fn test_overlay_write_after_delete_removes_tombstone() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");
        let file_path = metis_dir.join("initiatives/METIS-I-0001/initiative.md");

        // Delete then re-create
        fs.delete_file(&file_path).unwrap();
        assert!(!fs.file_exists(&file_path));

        fs.write_file(&file_path, "# Resurrected").unwrap();
        assert!(fs.file_exists(&file_path));

        let content = fs.read_file(&file_path).unwrap();
        assert!(content.contains("Resurrected"));
    }

    #[test]
    fn test_overlay_find_markdown_files_from_main() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        let files = fs.find_markdown_files(&metis_dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("initiative.md"));
    }

    #[test]
    fn test_overlay_find_markdown_files_with_overlay_additions() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        // Add a new file in overlay
        fs.write_file(
            metis_dir.join("initiatives/METIS-I-0002/initiative.md"),
            "# New",
        )
        .unwrap();

        let files = fs.find_markdown_files(&metis_dir).unwrap();
        assert_eq!(files.len(), 2);
    }

    #[test]
    fn test_overlay_find_markdown_files_excludes_tombstoned() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        fs.delete_file(metis_dir.join("initiatives/METIS-I-0001/initiative.md"))
            .unwrap();

        let files = fs.find_markdown_files(&metis_dir).unwrap();
        assert_eq!(files.len(), 0);
    }

    #[test]
    fn test_overlay_find_markdown_files_mixed() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        // Delete existing, add new
        fs.delete_file(metis_dir.join("initiatives/METIS-I-0001/initiative.md"))
            .unwrap();
        fs.write_file(
            metis_dir.join("initiatives/METIS-I-0002/initiative.md"),
            "# New",
        )
        .unwrap();

        let files = fs.find_markdown_files(&metis_dir).unwrap();
        assert_eq!(files.len(), 1);
        assert!(files[0].contains("METIS-I-0002"));
    }

    #[test]
    fn test_overlay_file_exists_checks_both() {
        let (dir, fs) = setup_overlay_test();
        let metis_dir = dir.path().join(".metis");

        // File on main should exist
        assert!(fs.file_exists(
            metis_dir.join("initiatives/METIS-I-0001/initiative.md")
        ));

        // Non-existent file should not exist
        assert!(!fs.file_exists(metis_dir.join("nonexistent.md")));

        // File only in overlay should exist
        fs.write_file(metis_dir.join("new.md"), "# New").unwrap();
        assert!(fs.file_exists(metis_dir.join("new.md")));
    }
}
