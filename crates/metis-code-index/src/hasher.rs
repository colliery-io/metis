//! Content hash tracking for incremental re-indexing.
//!
//! Stores BLAKE3 hashes of source files in a JSON manifest so that
//! `metis index --incremental` can skip unchanged files.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::symbols::Symbol;
use crate::walker::{SourceFile, WalkResult};

/// Hash manifest stored at `.metis/code-index-hashes.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HashManifest {
    /// Map of relative file paths to their content hashes.
    pub files: BTreeMap<String, String>,
}

/// Result of comparing current files against a previous hash manifest.
#[derive(Debug)]
pub struct IncrementalDiff {
    /// Files that are new or have changed content.
    pub changed: Vec<SourceFile>,
    /// Files that are unchanged (hash matches).
    pub unchanged: Vec<SourceFile>,
    /// Relative paths of files that were in the manifest but no longer exist.
    pub deleted: Vec<String>,
}

impl IncrementalDiff {
    /// Number of files that need re-indexing.
    pub fn changed_count(&self) -> usize {
        self.changed.len()
    }

    /// Number of files skipped.
    pub fn unchanged_count(&self) -> usize {
        self.unchanged.len()
    }

    /// Number of files removed.
    pub fn deleted_count(&self) -> usize {
        self.deleted.len()
    }
}

impl HashManifest {
    /// Load a manifest from a JSON file. Returns empty manifest if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse hash manifest: {}", e),
            )
        })
    }

    /// Save the manifest to a JSON file.
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)
    }

    /// Compute the BLAKE3 hash of a file's contents.
    pub fn hash_file(path: &Path) -> Result<String, std::io::Error> {
        let contents = std::fs::read(path)?;
        Ok(blake3::hash(&contents).to_hex().to_string())
    }

    /// Compare current files against this manifest to determine what changed.
    ///
    /// Computes hashes for all files in `walk_result` and returns a diff
    /// showing which files are new/changed, unchanged, or deleted.
    pub fn diff(&self, walk_result: &WalkResult) -> IncrementalDiff {
        let mut changed = Vec::new();
        let mut unchanged = Vec::new();
        let mut seen_paths = std::collections::HashSet::new();

        for file in &walk_result.files {
            let rel_path = file.relative_path.to_string_lossy().to_string();
            seen_paths.insert(rel_path.clone());

            match Self::hash_file(&file.path) {
                Ok(hash) => {
                    if self.files.get(&rel_path).map(|h| h.as_str()) == Some(hash.as_str()) {
                        unchanged.push(file.clone());
                    } else {
                        changed.push(file.clone());
                    }
                }
                Err(_) => {
                    // Can't hash the file — treat as changed so it gets processed
                    changed.push(file.clone());
                }
            }
        }

        let deleted: Vec<String> = self
            .files
            .keys()
            .filter(|path| !seen_paths.contains(path.as_str()))
            .cloned()
            .collect();

        IncrementalDiff {
            changed,
            unchanged,
            deleted,
        }
    }

    /// Build a fresh manifest from a walk result by hashing all files.
    pub fn from_walk_result(walk_result: &WalkResult) -> Self {
        let mut files = BTreeMap::new();
        for file in &walk_result.files {
            if let Ok(hash) = Self::hash_file(&file.path) {
                let rel_path = file.relative_path.to_string_lossy().to_string();
                files.insert(rel_path, hash);
            }
        }
        Self { files }
    }

    /// Update the manifest with hashes from changed files and remove deleted paths.
    pub fn update(&mut self, diff: &IncrementalDiff) {
        // Remove deleted files
        for path in &diff.deleted {
            self.files.remove(path);
        }

        // Update hashes for changed files
        for file in &diff.changed {
            if let Ok(hash) = Self::hash_file(&file.path) {
                let rel_path = file.relative_path.to_string_lossy().to_string();
                self.files.insert(rel_path, hash);
            }
        }
    }

    /// Get the set of directories that contain changed or deleted files.
    pub fn affected_directories(diff: &IncrementalDiff) -> std::collections::BTreeSet<PathBuf> {
        let mut dirs = std::collections::BTreeSet::new();

        for file in &diff.changed {
            if let Some(parent) = file.relative_path.parent() {
                dirs.insert(parent.to_path_buf());
            }
        }

        for path in &diff.deleted {
            let p = PathBuf::from(path);
            if let Some(parent) = p.parent() {
                dirs.insert(parent.to_path_buf());
            }
        }

        dirs
    }
}

/// Cached symbol data stored at `.metis/code-index-symbols.json`.
///
/// Persists extracted symbols so incremental re-indexing can skip
/// unchanged files without re-parsing them.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SymbolCache {
    /// Map of relative file paths to their extracted symbols.
    pub files: BTreeMap<String, Vec<Symbol>>,
}

impl SymbolCache {
    /// Load a symbol cache from a JSON file. Returns empty cache if file doesn't exist.
    pub fn load(path: &Path) -> Result<Self, std::io::Error> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Failed to parse symbol cache: {}", e),
            )
        })
    }

    /// Save the symbol cache to a JSON file (compact format).
    pub fn save(&self, path: &Path) -> Result<(), std::io::Error> {
        let content = serde_json::to_string(self)?;
        std::fs::write(path, content)
    }

    /// Convert to a `BTreeMap<PathBuf, Vec<Symbol>>` for use with `format_index`.
    pub fn to_path_map(&self) -> BTreeMap<PathBuf, Vec<Symbol>> {
        self.files
            .iter()
            .map(|(path, symbols)| (PathBuf::from(path), symbols.clone()))
            .collect()
    }

    /// Build from a `BTreeMap<PathBuf, Vec<Symbol>>`.
    pub fn from_path_map(map: &BTreeMap<PathBuf, Vec<Symbol>>) -> Self {
        let files = map
            .iter()
            .map(|(path, symbols)| (path.to_string_lossy().to_string(), symbols.clone()))
            .collect();
        Self { files }
    }

    /// Update cache: add/replace changed file symbols and remove deleted file entries.
    pub fn update(&mut self, changed: &BTreeMap<PathBuf, Vec<Symbol>>, deleted: &[String]) {
        for path in deleted {
            self.files.remove(path);
        }
        for (path, symbols) in changed {
            self.files
                .insert(path.to_string_lossy().to_string(), symbols.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Language;
    use std::fs;

    fn make_source_file(root: &Path, rel: &str, lang: Language) -> SourceFile {
        SourceFile {
            path: root.join(rel),
            relative_path: PathBuf::from(rel),
            language: lang,
        }
    }

    #[test]
    fn test_hash_file() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("test.rs");
        fs::write(&file, "fn main() {}").unwrap();

        let hash = HashManifest::hash_file(&file).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // BLAKE3 hex is 64 chars

        // Same content produces same hash
        let hash2 = HashManifest::hash_file(&file).unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_hash_changes_with_content() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("test.rs");

        fs::write(&file, "fn main() {}").unwrap();
        let hash1 = HashManifest::hash_file(&file).unwrap();

        fs::write(&file, "fn main() { println!(\"hello\"); }").unwrap();
        let hash2 = HashManifest::hash_file(&file).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_manifest_save_load() {
        let tmp = tempfile::tempdir().unwrap();
        let manifest_path = tmp.path().join("hashes.json");

        let mut manifest = HashManifest::default();
        manifest
            .files
            .insert("src/main.rs".to_string(), "abc123".to_string());
        manifest
            .files
            .insert("src/lib.rs".to_string(), "def456".to_string());

        manifest.save(&manifest_path).unwrap();

        let loaded = HashManifest::load(&manifest_path).unwrap();
        assert_eq!(loaded.files.len(), 2);
        assert_eq!(loaded.files["src/main.rs"], "abc123");
        assert_eq!(loaded.files["src/lib.rs"], "def456");
    }

    #[test]
    fn test_load_nonexistent_returns_empty() {
        let manifest = HashManifest::load(Path::new("/nonexistent/path.json")).unwrap();
        assert!(manifest.files.is_empty());
    }

    #[test]
    fn test_from_walk_result() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();
        fs::write(tmp.path().join("lib.rs"), "pub mod utils;").unwrap();

        let walk_result = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![
                make_source_file(tmp.path(), "main.rs", Language::Rust),
                make_source_file(tmp.path(), "lib.rs", Language::Rust),
            ],
        };

        let manifest = HashManifest::from_walk_result(&walk_result);
        assert_eq!(manifest.files.len(), 2);
        assert!(manifest.files.contains_key("main.rs"));
        assert!(manifest.files.contains_key("lib.rs"));
    }

    #[test]
    fn test_diff_all_new() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();

        let walk_result = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![make_source_file(tmp.path(), "main.rs", Language::Rust)],
        };

        let empty_manifest = HashManifest::default();
        let diff = empty_manifest.diff(&walk_result);

        assert_eq!(diff.changed_count(), 1);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
    }

    #[test]
    fn test_diff_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();

        let walk_result = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![make_source_file(tmp.path(), "main.rs", Language::Rust)],
        };

        let manifest = HashManifest::from_walk_result(&walk_result);
        let diff = manifest.diff(&walk_result);

        assert_eq!(diff.changed_count(), 0);
        assert_eq!(diff.unchanged_count(), 1);
        assert_eq!(diff.deleted_count(), 0);
    }

    #[test]
    fn test_diff_modified() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();

        let walk_result = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![make_source_file(tmp.path(), "main.rs", Language::Rust)],
        };

        let manifest = HashManifest::from_walk_result(&walk_result);

        // Modify the file
        fs::write(tmp.path().join("main.rs"), "fn main() { updated }").unwrap();

        let diff = manifest.diff(&walk_result);

        assert_eq!(diff.changed_count(), 1);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.deleted_count(), 0);
    }

    #[test]
    fn test_diff_deleted() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("main.rs"), "fn main() {}").unwrap();

        let walk_result_full = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![make_source_file(tmp.path(), "main.rs", Language::Rust)],
        };

        let manifest = HashManifest::from_walk_result(&walk_result_full);

        // Walk result with no files (simulating deletion)
        let walk_result_empty = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![],
        };

        let diff = manifest.diff(&walk_result_empty);

        assert_eq!(diff.changed_count(), 0);
        assert_eq!(diff.unchanged_count(), 0);
        assert_eq!(diff.deleted_count(), 1);
        assert_eq!(diff.deleted[0], "main.rs");
    }

    #[test]
    fn test_diff_mixed_scenario() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("keep.rs"), "unchanged").unwrap();
        fs::write(tmp.path().join("modify.rs"), "original").unwrap();
        fs::write(tmp.path().join("delete.rs"), "will be deleted").unwrap();

        let walk_result_initial = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![
                make_source_file(tmp.path(), "delete.rs", Language::Rust),
                make_source_file(tmp.path(), "keep.rs", Language::Rust),
                make_source_file(tmp.path(), "modify.rs", Language::Rust),
            ],
        };

        let manifest = HashManifest::from_walk_result(&walk_result_initial);

        // Modify one file, add a new one, "delete" one by not including it
        fs::write(tmp.path().join("modify.rs"), "modified content").unwrap();
        fs::write(tmp.path().join("new.rs"), "brand new").unwrap();

        let walk_result_updated = WalkResult {
            root: tmp.path().to_path_buf(),
            files: vec![
                make_source_file(tmp.path(), "keep.rs", Language::Rust),
                make_source_file(tmp.path(), "modify.rs", Language::Rust),
                make_source_file(tmp.path(), "new.rs", Language::Rust),
            ],
        };

        let diff = manifest.diff(&walk_result_updated);

        assert_eq!(diff.unchanged_count(), 1); // keep.rs
        assert_eq!(diff.changed_count(), 2); // modify.rs + new.rs
        assert_eq!(diff.deleted_count(), 1); // delete.rs
    }

    #[test]
    fn test_update_manifest() {
        let tmp = tempfile::tempdir().unwrap();
        fs::write(tmp.path().join("keep.rs"), "unchanged").unwrap();
        fs::write(tmp.path().join("modify.rs"), "original").unwrap();
        fs::write(tmp.path().join("new.rs"), "brand new").unwrap();

        let mut manifest = HashManifest::default();
        manifest
            .files
            .insert("keep.rs".to_string(), "keep_hash".to_string());
        manifest
            .files
            .insert("modify.rs".to_string(), "old_hash".to_string());
        manifest
            .files
            .insert("deleted.rs".to_string(), "del_hash".to_string());

        let diff = IncrementalDiff {
            changed: vec![
                make_source_file(tmp.path(), "modify.rs", Language::Rust),
                make_source_file(tmp.path(), "new.rs", Language::Rust),
            ],
            unchanged: vec![make_source_file(tmp.path(), "keep.rs", Language::Rust)],
            deleted: vec!["deleted.rs".to_string()],
        };

        manifest.update(&diff);

        assert_eq!(manifest.files.len(), 3); // keep + modify + new (deleted removed)
        assert_eq!(manifest.files["keep.rs"], "keep_hash"); // unchanged
        assert_ne!(manifest.files["modify.rs"], "old_hash"); // updated
        assert!(manifest.files.contains_key("new.rs")); // added
        assert!(!manifest.files.contains_key("deleted.rs")); // removed
    }

    #[test]
    fn test_affected_directories() {
        let tmp = tempfile::tempdir().unwrap();

        let diff = IncrementalDiff {
            changed: vec![
                make_source_file(tmp.path(), "src/main.rs", Language::Rust),
                make_source_file(tmp.path(), "src/utils/helper.rs", Language::Rust),
                make_source_file(tmp.path(), "tests/test.rs", Language::Rust),
            ],
            unchanged: vec![],
            deleted: vec!["lib/old.rs".to_string()],
        };

        let dirs = HashManifest::affected_directories(&diff);
        assert!(dirs.contains(&PathBuf::from("src")));
        assert!(dirs.contains(&PathBuf::from("src/utils")));
        assert!(dirs.contains(&PathBuf::from("tests")));
        assert!(dirs.contains(&PathBuf::from("lib")));
    }

    // SymbolCache tests

    fn make_symbol(name: &str, file: &str) -> Symbol {
        use crate::symbols::SymbolKind;
        Symbol::new(name, SymbolKind::Function, file, 1, 10)
    }

    #[test]
    fn test_symbol_cache_save_load() {
        let tmp = tempfile::tempdir().unwrap();
        let cache_path = tmp.path().join("symbols.json");

        let mut cache = SymbolCache::default();
        cache.files.insert(
            "src/main.rs".to_string(),
            vec![make_symbol("main", "src/main.rs")],
        );

        cache.save(&cache_path).unwrap();

        let loaded = SymbolCache::load(&cache_path).unwrap();
        assert_eq!(loaded.files.len(), 1);
        assert_eq!(loaded.files["src/main.rs"].len(), 1);
        assert_eq!(loaded.files["src/main.rs"][0].name, "main");
    }

    #[test]
    fn test_symbol_cache_load_nonexistent() {
        let cache = SymbolCache::load(Path::new("/nonexistent/cache.json")).unwrap();
        assert!(cache.files.is_empty());
    }

    #[test]
    fn test_symbol_cache_roundtrip_path_map() {
        let mut map = BTreeMap::new();
        map.insert(
            PathBuf::from("src/lib.rs"),
            vec![make_symbol("init", "src/lib.rs")],
        );
        map.insert(
            PathBuf::from("src/main.rs"),
            vec![make_symbol("main", "src/main.rs")],
        );

        let cache = SymbolCache::from_path_map(&map);
        assert_eq!(cache.files.len(), 2);

        let roundtripped = cache.to_path_map();
        assert_eq!(roundtripped.len(), 2);
        assert!(roundtripped.contains_key(&PathBuf::from("src/lib.rs")));
        assert!(roundtripped.contains_key(&PathBuf::from("src/main.rs")));
    }

    #[test]
    fn test_symbol_cache_update() {
        let mut cache = SymbolCache::default();
        cache.files.insert(
            "keep.rs".to_string(),
            vec![make_symbol("keep_fn", "keep.rs")],
        );
        cache.files.insert(
            "modify.rs".to_string(),
            vec![make_symbol("old_fn", "modify.rs")],
        );
        cache.files.insert(
            "delete.rs".to_string(),
            vec![make_symbol("del_fn", "delete.rs")],
        );

        let mut changed = BTreeMap::new();
        changed.insert(
            PathBuf::from("modify.rs"),
            vec![make_symbol("new_fn", "modify.rs")],
        );
        changed.insert(
            PathBuf::from("added.rs"),
            vec![make_symbol("add_fn", "added.rs")],
        );

        cache.update(&changed, &["delete.rs".to_string()]);

        assert_eq!(cache.files.len(), 3); // keep + modify + added
        assert_eq!(cache.files["keep.rs"][0].name, "keep_fn");
        assert_eq!(cache.files["modify.rs"][0].name, "new_fn");
        assert_eq!(cache.files["added.rs"][0].name, "add_fn");
        assert!(!cache.files.contains_key("delete.rs"));
    }
}
