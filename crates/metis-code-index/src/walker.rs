//! Gitignore-aware source file walker.
//!
//! Uses the `ignore` crate to walk directories while respecting `.gitignore`,
//! `.git/info/exclude`, and global gitignore rules. Filters to supported
//! source file extensions and skips common non-source directories.

use std::path::{Path, PathBuf};

use ignore::WalkBuilder;

use crate::parser::Language;

/// A source file discovered during directory walking.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Absolute path to the file.
    pub path: PathBuf,
    /// Path relative to the walk root.
    pub relative_path: PathBuf,
    /// Detected programming language.
    pub language: Language,
}

/// Result of walking a directory for source files.
#[derive(Debug)]
pub struct WalkResult {
    /// The root directory that was walked.
    pub root: PathBuf,
    /// Discovered source files, sorted by relative path.
    pub files: Vec<SourceFile>,
}

impl WalkResult {
    /// Group files by language.
    pub fn by_language(&self) -> std::collections::HashMap<Language, Vec<&SourceFile>> {
        let mut map = std::collections::HashMap::new();
        for file in &self.files {
            map.entry(file.language).or_insert_with(Vec::new).push(file);
        }
        map
    }

    /// Get file count.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Directories to always skip, regardless of gitignore rules.
const SKIP_DIRS: &[&str] = &[
    "target",
    "node_modules",
    "__pycache__",
    ".git",
    "vendor",
    "dist",
    "build",
    ".tox",
    ".venv",
    "venv",
    ".mypy_cache",
    ".pytest_cache",
    ".next",
];

/// Walk a directory tree for source files, respecting gitignore rules.
///
/// Returns a sorted list of source files with their detected languages.
/// Hidden files/directories are skipped, along with common non-source
/// directories like `target/`, `node_modules/`, and `__pycache__/`.
pub fn walk_directory(root: &Path) -> Result<WalkResult, WalkError> {
    let root = root.canonicalize().map_err(|e| WalkError::IoError {
        path: root.to_path_buf(),
        source: e,
    })?;

    let walker = WalkBuilder::new(&root)
        .hidden(true) // skip hidden files/dirs
        .git_ignore(true) // respect .gitignore
        .git_global(true) // respect global gitignore
        .git_exclude(true) // respect .git/info/exclude
        .filter_entry(|entry| {
            // Skip known non-source directories
            if entry.file_type().is_some_and(|ft| ft.is_dir()) {
                if let Some(name) = entry.file_name().to_str() {
                    return !SKIP_DIRS.contains(&name);
                }
            }
            true
        })
        .build();

    let mut files = Vec::new();

    for entry in walker {
        let entry = entry.map_err(|e| WalkError::WalkError(e.to_string()))?;

        // Skip directories, only process files
        if !entry.file_type().is_some_and(|ft| ft.is_file()) {
            continue;
        }

        let path = entry.path();

        // Check if this is a supported source file
        if let Some(language) = Language::from_path(path) {
            let relative_path = path.strip_prefix(&root).unwrap_or(path).to_path_buf();

            files.push(SourceFile {
                path: path.to_path_buf(),
                relative_path,
                language,
            });
        }
    }

    // Sort by relative path for deterministic output
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(WalkResult { root, files })
}

/// Errors that can occur during directory walking.
#[derive(Debug, thiserror::Error)]
pub enum WalkError {
    #[error("Failed to access {path}: {source}")]
    IoError {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("Walk error: {0}")]
    WalkError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    /// Helper to create a test directory structure.
    fn create_test_project(dir: &Path) {
        // Source files
        let src = dir.join("src");
        fs::create_dir_all(&src).unwrap();
        fs::write(src.join("main.rs"), "fn main() {}").unwrap();
        fs::write(src.join("lib.rs"), "pub mod utils;").unwrap();

        let utils = src.join("utils");
        fs::create_dir_all(&utils).unwrap();
        fs::write(utils.join("mod.rs"), "pub fn helper() {}").unwrap();

        // Python files
        let scripts = dir.join("scripts");
        fs::create_dir_all(&scripts).unwrap();
        fs::write(scripts.join("build.py"), "def build(): pass").unwrap();

        // TypeScript files
        let ts_src = dir.join("frontend");
        fs::create_dir_all(&ts_src).unwrap();
        fs::write(ts_src.join("app.ts"), "function main() {}").unwrap();
        fs::write(ts_src.join("component.tsx"), "export function App() {}").unwrap();

        // Go files
        let go_src = dir.join("cmd");
        fs::create_dir_all(&go_src).unwrap();
        fs::write(go_src.join("main.go"), "package main").unwrap();

        // Non-source files (should be ignored)
        fs::write(dir.join("README.md"), "# Project").unwrap();
        fs::write(dir.join("Cargo.toml"), "[package]").unwrap();
    }

    #[test]
    fn test_walk_finds_source_files() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        let result = walk_directory(tmp.path()).unwrap();

        assert_eq!(result.file_count(), 7);

        // Check specific files exist
        let paths: Vec<String> = result
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(paths.iter().any(|p| p.contains("main.rs")));
        assert!(paths.iter().any(|p| p.contains("lib.rs")));
        assert!(paths.iter().any(|p| p.contains("build.py")));
        assert!(paths.iter().any(|p| p.contains("app.ts")));
        assert!(paths.iter().any(|p| p.contains("component.tsx")));
        assert!(paths.iter().any(|p| p.contains("main.go")));

        // Non-source files should NOT be present
        assert!(!paths.iter().any(|p| p.contains("README.md")));
        assert!(!paths.iter().any(|p| p.contains("Cargo.toml")));
    }

    #[test]
    fn test_walk_respects_gitignore() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        // Initialize a git repo so .gitignore is respected
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Create .gitignore that excludes the scripts directory
        fs::write(tmp.path().join(".gitignore"), "scripts/\n").unwrap();

        let result = walk_directory(tmp.path()).unwrap();

        let paths: Vec<String> = result
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        // scripts/build.py should be excluded by gitignore
        assert!(
            !paths.iter().any(|p| p.contains("build.py")),
            "build.py should be gitignored, found: {:?}",
            paths
        );

        // Other source files should still be present
        assert!(paths.iter().any(|p| p.contains("main.rs")));
    }

    #[test]
    fn test_walk_skips_target_directory() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        // Create a target directory with Rust files (should be skipped)
        let target = tmp.path().join("target").join("debug");
        fs::create_dir_all(&target).unwrap();
        fs::write(target.join("build_script.rs"), "fn main() {}").unwrap();

        let result = walk_directory(tmp.path()).unwrap();

        let paths: Vec<String> = result
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(
            !paths.iter().any(|p| p.contains("target")),
            "target/ files should be skipped, found: {:?}",
            paths
        );
    }

    #[test]
    fn test_walk_skips_node_modules() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        // Create node_modules with JS files (should be skipped)
        let nm = tmp.path().join("node_modules").join("lodash");
        fs::create_dir_all(&nm).unwrap();
        fs::write(nm.join("lodash.js"), "module.exports = {}").unwrap();

        let result = walk_directory(tmp.path()).unwrap();

        let paths: Vec<String> = result
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(
            !paths.iter().any(|p| p.contains("node_modules")),
            "node_modules/ files should be skipped, found: {:?}",
            paths
        );
    }

    #[test]
    fn test_walk_skips_pycache() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        // Create __pycache__ with .py files (should be skipped)
        let pycache = tmp.path().join("scripts").join("__pycache__");
        fs::create_dir_all(&pycache).unwrap();
        fs::write(pycache.join("build.cpython-311.py"), "compiled").unwrap();

        let result = walk_directory(tmp.path()).unwrap();

        let paths: Vec<String> = result
            .files
            .iter()
            .map(|f| f.relative_path.to_string_lossy().to_string())
            .collect();

        assert!(
            !paths.iter().any(|p| p.contains("__pycache__")),
            "__pycache__/ files should be skipped, found: {:?}",
            paths
        );
    }

    #[test]
    fn test_walk_by_language() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        let result = walk_directory(tmp.path()).unwrap();
        let by_lang = result.by_language();

        assert_eq!(
            by_lang.get(&Language::Rust).map(|v| v.len()).unwrap_or(0),
            3,
            "should find 3 Rust files"
        );
        assert_eq!(
            by_lang.get(&Language::Python).map(|v| v.len()).unwrap_or(0),
            1,
            "should find 1 Python file"
        );
        assert_eq!(
            by_lang
                .get(&Language::TypeScript)
                .map(|v| v.len())
                .unwrap_or(0),
            2,
            "should find 2 TypeScript files"
        );
        assert_eq!(
            by_lang.get(&Language::Go).map(|v| v.len()).unwrap_or(0),
            1,
            "should find 1 Go file"
        );
    }

    #[test]
    fn test_walk_sorted_output() {
        let tmp = tempfile::tempdir().unwrap();
        create_test_project(tmp.path());

        let result = walk_directory(tmp.path()).unwrap();

        let paths: Vec<&PathBuf> = result.files.iter().map(|f| &f.relative_path).collect();

        // Verify sorted order
        for window in paths.windows(2) {
            assert!(
                window[0] <= window[1],
                "files should be sorted: {:?} should come before {:?}",
                window[0],
                window[1]
            );
        }
    }

    #[test]
    fn test_walk_empty_directory() {
        let tmp = tempfile::tempdir().unwrap();

        let result = walk_directory(tmp.path()).unwrap();
        assert_eq!(result.file_count(), 0);
    }

    #[test]
    fn test_walk_nonexistent_directory() {
        let result = walk_directory(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_walk_language_detection() {
        let tmp = tempfile::tempdir().unwrap();

        // Create files with various extensions
        fs::write(tmp.path().join("main.rs"), "").unwrap();
        fs::write(tmp.path().join("app.py"), "").unwrap();
        fs::write(tmp.path().join("index.ts"), "").unwrap();
        fs::write(tmp.path().join("component.tsx"), "").unwrap();
        fs::write(tmp.path().join("util.js"), "").unwrap();
        fs::write(tmp.path().join("helper.jsx"), "").unwrap();
        fs::write(tmp.path().join("server.mjs"), "").unwrap();
        fs::write(tmp.path().join("config.cjs"), "").unwrap();
        fs::write(tmp.path().join("main.go"), "").unwrap();
        fs::write(tmp.path().join("types.pyi"), "").unwrap();

        let result = walk_directory(tmp.path()).unwrap();
        assert_eq!(result.file_count(), 10, "should find all 10 source files");

        // Verify language detection
        for file in &result.files {
            let ext = file.path.extension().unwrap().to_str().unwrap();
            let expected = Language::from_extension(ext).unwrap();
            assert_eq!(file.language, expected, "language mismatch for {ext}");
        }
    }
}
