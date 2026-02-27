//! Layout mapping between local hierarchical and central flat document layouts.
//!
//! Local Metis stores documents in a hierarchy:
//!   `.metis/vision.md`
//!   `.metis/strategies/{STRAT}/initiatives/{INIT}/tasks/{TASK}.md`
//!   `.metis/adrs/{ADR}.md`
//!   `.metis/backlog/{category}/{TASK}.md`
//!
//! Central stores documents flat-by-workspace:
//!   `prefix/PREFIX-V-0001.md`
//!   `prefix/PREFIX-T-0042.md`
//!
//! This module provides the mapping between these two layouts.

use crate::{MetisError, Result};
use gray_matter::{engine::YAML, Matter};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// A document extracted from the local hierarchy, ready for central flat layout.
#[derive(Debug, Clone)]
pub struct FlatDocument {
    /// The short code (e.g., "API-T-0001")
    pub short_code: String,
    /// The filename for central storage (e.g., "API-T-0001.md")
    pub filename: String,
    /// Full file content including frontmatter
    pub content: String,
}

/// Result of a flatten operation.
#[derive(Debug)]
pub struct FlattenResult {
    /// Documents successfully flattened
    pub documents: Vec<FlatDocument>,
    /// Files that were skipped (not Metis documents)
    pub skipped: Vec<PathBuf>,
    /// Files that had errors during processing
    pub errors: Vec<(PathBuf, String)>,
}

/// Result of reading flat documents from a directory.
#[derive(Debug)]
pub struct ReadFlatResult {
    /// Documents successfully read
    pub documents: Vec<FlatDocument>,
    /// Files that were skipped (not Metis documents)
    pub skipped: Vec<PathBuf>,
    /// Files that had errors during processing
    pub errors: Vec<(PathBuf, String)>,
}

// Files that should never be included in the central layout
const EXCLUDED_FILES: &[&str] = &[
    "config.toml",
    "metis.db",
    "metis.db-journal",
    "metis.db-wal",
    "metis.db-shm",
    "code-index.md",
    "code-index-hashes.json",
    "code-index-symbols.json",
];

// Directories that should never be traversed for flattening
const EXCLUDED_DIRS: &[&str] = &["archived"];

/// Extract the `short_code` field from a markdown file's YAML frontmatter.
///
/// Returns `None` if the file has no frontmatter, no `short_code` field,
/// or the frontmatter is malformed.
pub fn extract_short_code(content: &str) -> Option<String> {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);
    let frontmatter = parsed.data?;

    if let gray_matter::Pod::Hash(map) = frontmatter {
        if let Some(gray_matter::Pod::String(sc)) = map.get("short_code") {
            let trimmed = sc.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }
    None
}

/// Extract the `level` (document type) field from a markdown file's YAML frontmatter.
pub fn extract_level(content: &str) -> Option<String> {
    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(content);
    let frontmatter = parsed.data?;

    if let gray_matter::Pod::Hash(map) = frontmatter {
        // Try "level" first, then "document_type", then "type"
        for key in &["level", "document_type", "type"] {
            if let Some(gray_matter::Pod::String(s)) = map.get(*key) {
                let trimmed = s.trim().to_string();
                if !trimmed.is_empty() {
                    return Some(trimmed);
                }
            }
        }
    }
    None
}

/// Flatten the local workspace hierarchy into a list of flat documents.
///
/// Walks the `.metis/` directory, finds all Metis document `.md` files
/// (identified by having a `short_code` in their frontmatter), and returns
/// them as flat documents ready for the central repo layout.
///
/// # Arguments
/// * `workspace_dir` - Path to the `.metis/` directory
///
/// # Returns
/// A `FlattenResult` containing documents, skipped files, and errors.
pub fn flatten_workspace(workspace_dir: &Path) -> Result<FlattenResult> {
    if !workspace_dir.exists() || !workspace_dir.is_dir() {
        return Err(MetisError::FileSystem(format!(
            "Workspace directory not found: {}",
            workspace_dir.display()
        )));
    }

    let mut documents: BTreeMap<String, FlatDocument> = BTreeMap::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    for entry in WalkDir::new(workspace_dir)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            // Skip excluded directories
            if e.file_type().is_dir() {
                let name = e.file_name().to_string_lossy();
                // Skip hidden directories (except the root .metis itself)
                if name.starts_with('.') && e.depth() > 0 {
                    return false;
                }
                // Skip excluded directories
                if EXCLUDED_DIRS.contains(&name.as_ref()) {
                    return false;
                }
                return true;
            }
            true
        })
    {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                errors.push((
                    PathBuf::from(workspace_dir),
                    format!("Walk error: {}", err),
                ));
                continue;
            }
        };

        // Only process files
        if !entry.file_type().is_file() {
            continue;
        }

        let path = entry.path();
        let filename = entry.file_name().to_string_lossy().to_string();

        // Only process .md files
        if !filename.ends_with(".md") {
            continue;
        }

        // Skip excluded files
        if EXCLUDED_FILES.contains(&filename.as_str()) {
            continue;
        }

        // Skip hidden files
        if filename.starts_with('.') {
            continue;
        }

        // Read the file
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(err) => {
                errors.push((path.to_path_buf(), format!("Read error: {}", err)));
                continue;
            }
        };

        // Extract short_code from frontmatter
        match extract_short_code(&content) {
            Some(short_code) => {
                let flat_filename = format!("{}.md", short_code);
                let doc = FlatDocument {
                    short_code: short_code.clone(),
                    filename: flat_filename,
                    content,
                };
                // Use BTreeMap to deduplicate by short_code (first wins)
                if documents.contains_key(&short_code) {
                    errors.push((
                        path.to_path_buf(),
                        format!(
                            "Duplicate short code '{}' — already found, skipping this file",
                            short_code
                        ),
                    ));
                } else {
                    documents.insert(short_code, doc);
                }
            }
            None => {
                // Not a Metis document (no short_code in frontmatter)
                skipped.push(path.to_path_buf());
            }
        }
    }

    Ok(FlattenResult {
        documents: documents.into_values().collect(),
        skipped,
        errors,
    })
}

/// Read flat documents from a directory (e.g., a hydrated remote workspace folder
/// or a workspace folder extracted from central).
///
/// # Arguments
/// * `source_dir` - Path to the directory containing flat `.md` files
///
/// # Returns
/// A `ReadFlatResult` containing documents, skipped files, and errors.
pub fn read_flat_documents(source_dir: &Path) -> Result<ReadFlatResult> {
    if !source_dir.exists() || !source_dir.is_dir() {
        return Err(MetisError::FileSystem(format!(
            "Source directory not found: {}",
            source_dir.display()
        )));
    }

    let mut documents = Vec::new();
    let mut skipped = Vec::new();
    let mut errors = Vec::new();

    let entries = std::fs::read_dir(source_dir).map_err(|e| {
        MetisError::FileSystem(format!(
            "Failed to read directory {}: {}",
            source_dir.display(),
            e
        ))
    })?;

    for entry in entries {
        let entry = match entry {
            Ok(e) => e,
            Err(err) => {
                errors.push((
                    source_dir.to_path_buf(),
                    format!("Directory entry error: {}", err),
                ));
                continue;
            }
        };

        let path = entry.path();

        // Only process files
        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => continue,
        };

        // Only process .md files
        if !filename.ends_with(".md") {
            continue;
        }

        // Skip hidden files
        if filename.starts_with('.') {
            continue;
        }

        // Read the file
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(err) => {
                errors.push((path.clone(), format!("Read error: {}", err)));
                continue;
            }
        };

        // Extract short_code from frontmatter
        match extract_short_code(&content) {
            Some(short_code) => {
                documents.push(FlatDocument {
                    short_code: short_code.clone(),
                    filename: format!("{}.md", short_code),
                    content,
                });
            }
            None => {
                skipped.push(path);
            }
        }
    }

    // Sort by short_code for deterministic ordering
    documents.sort_by(|a, b| a.short_code.cmp(&b.short_code));

    Ok(ReadFlatResult {
        documents,
        skipped,
        errors,
    })
}

/// Write flat documents to a target directory.
///
/// Each document is written as `target_dir/SHORT-CODE.md`.
/// Existing files are overwritten. The target directory is created if it doesn't exist.
///
/// # Arguments
/// * `documents` - The flat documents to write
/// * `target_dir` - The directory to write to (e.g., `.metis/api/` for hydration,
///   or a temp dir for building git trees)
///
/// # Returns
/// The number of documents written.
pub fn write_flat_documents(documents: &[FlatDocument], target_dir: &Path) -> Result<usize> {
    std::fs::create_dir_all(target_dir).map_err(|e| {
        MetisError::FileSystem(format!(
            "Failed to create directory {}: {}",
            target_dir.display(),
            e
        ))
    })?;

    let mut count = 0;
    for doc in documents {
        let file_path = target_dir.join(&doc.filename);
        std::fs::write(&file_path, &doc.content).map_err(|e| {
            MetisError::FileSystem(format!(
                "Failed to write {}: {}",
                file_path.display(),
                e
            ))
        })?;
        count += 1;
    }

    Ok(count)
}

/// Remove files from a directory that are NOT in the given set of documents.
///
/// Used during hydration to remove documents that were deleted/archived upstream.
/// Only removes `.md` files — other files are left alone.
///
/// # Returns
/// The list of files that were removed.
pub fn remove_stale_files(
    target_dir: &Path,
    current_documents: &[FlatDocument],
) -> Result<Vec<PathBuf>> {
    if !target_dir.exists() {
        return Ok(Vec::new());
    }

    let current_filenames: std::collections::HashSet<&str> = current_documents
        .iter()
        .map(|d| d.filename.as_str())
        .collect();

    let mut removed = Vec::new();

    let entries = std::fs::read_dir(target_dir).map_err(|e| {
        MetisError::FileSystem(format!(
            "Failed to read directory {}: {}",
            target_dir.display(),
            e
        ))
    })?;

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let filename = match path.file_name() {
            Some(f) => f.to_string_lossy().to_string(),
            None => continue,
        };

        // Only consider .md files for removal
        if !filename.ends_with(".md") {
            continue;
        }

        if !current_filenames.contains(filename.as_str()) {
            std::fs::remove_file(&path).map_err(|e| {
                MetisError::FileSystem(format!(
                    "Failed to remove stale file {}: {}",
                    path.display(),
                    e
                ))
            })?;
            removed.push(path);
        }
    }

    Ok(removed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_document_content(short_code: &str, level: &str, parent: Option<&str>) -> String {
        let parent_line = match parent {
            Some(p) => format!("parent: {}", p),
            None => "parent:".to_string(),
        };
        let mut s = String::new();
        s.push_str("---\n");
        s.push_str("id: test-doc\n");
        s.push_str(&format!("level: {}\n", level));
        s.push_str("title: \"Test Document\"\n");
        s.push_str(&format!("short_code: \"{}\"\n", short_code));
        s.push_str("created_at: 2026-01-01T00:00:00+00:00\n");
        s.push_str("updated_at: 2026-01-01T00:00:00+00:00\n");
        s.push_str(&format!("{}\n", parent_line));
        s.push_str("blocked_by: []\n");
        s.push_str("archived: false\n");
        s.push_str("tags:\n");
        s.push_str(&format!("  - \"#{}\"\n", level));
        s.push_str("  - \"#phase/active\"\n");
        s.push_str("exit_criteria_met: false\n");
        s.push_str("strategy_id: NULL\n");
        s.push_str("initiative_id: NULL\n");
        s.push_str("---\n\n");
        s.push_str("# Test Document\n\nSome content here.\n");
        s
    }

    fn setup_local_hierarchy(dir: &Path) {
        // Create the standard Metis directory structure
        // Vision
        let vision_content = make_document_content("TEST-V-0001", "vision", None);
        std::fs::write(dir.join("vision.md"), &vision_content).unwrap();

        // ADRs
        let adr_dir = dir.join("adrs");
        std::fs::create_dir_all(&adr_dir).unwrap();
        let adr_content = make_document_content("TEST-A-0001", "adr", None);
        std::fs::write(adr_dir.join("TEST-A-0001.md"), &adr_content).unwrap();

        // Strategy/Initiative/Task hierarchy
        let tasks_dir = dir.join("strategies/NULL/initiatives/TEST-I-0001/tasks");
        std::fs::create_dir_all(&tasks_dir).unwrap();

        let init_content =
            make_document_content("TEST-I-0001", "initiative", Some("TEST-V-0001"));
        std::fs::write(
            dir.join("strategies/NULL/initiatives/TEST-I-0001/initiative.md"),
            &init_content,
        )
        .unwrap();

        let task1_content =
            make_document_content("TEST-T-0001", "task", Some("TEST-I-0001"));
        std::fs::write(tasks_dir.join("TEST-T-0001.md"), &task1_content).unwrap();

        let task2_content =
            make_document_content("TEST-T-0002", "task", Some("TEST-I-0001"));
        std::fs::write(tasks_dir.join("TEST-T-0002.md"), &task2_content).unwrap();

        // Backlog
        let backlog_dir = dir.join("backlog/bugs");
        std::fs::create_dir_all(&backlog_dir).unwrap();
        let bug_content = make_document_content("TEST-T-0003", "task", None);
        std::fs::write(backlog_dir.join("TEST-T-0003.md"), &bug_content).unwrap();

        // Non-document files (should be excluded)
        std::fs::write(dir.join("config.toml"), "[project]\nprefix = \"TEST\"\n").unwrap();
        std::fs::write(dir.join("code-index.md"), "# Code Index\n").unwrap();
    }

    // ============================================================
    // extract_short_code tests
    // ============================================================

    #[test]
    fn test_extract_short_code_valid() {
        let content = make_document_content("PROJ-T-0001", "task", None);
        assert_eq!(extract_short_code(&content), Some("PROJ-T-0001".to_string()));
    }

    #[test]
    fn test_extract_short_code_no_frontmatter() {
        let content = "# Just a markdown file\nNo frontmatter here.";
        assert_eq!(extract_short_code(content), None);
    }

    #[test]
    fn test_extract_short_code_no_short_code_field() {
        let content = "---\ntitle: \"Test\"\nlevel: task\n---\n# Content\n";
        assert_eq!(extract_short_code(content), None);
    }

    #[test]
    fn test_extract_short_code_empty_short_code() {
        let content = "---\nshort_code: \"\"\nlevel: task\n---\n# Content\n";
        assert_eq!(extract_short_code(content), None);
    }

    // ============================================================
    // extract_level tests
    // ============================================================

    #[test]
    fn test_extract_level_valid() {
        let content = make_document_content("PROJ-T-0001", "task", None);
        assert_eq!(extract_level(&content), Some("task".to_string()));
    }

    #[test]
    fn test_extract_level_vision() {
        let content = make_document_content("PROJ-V-0001", "vision", None);
        assert_eq!(extract_level(&content), Some("vision".to_string()));
    }

    #[test]
    fn test_extract_level_no_frontmatter() {
        assert_eq!(extract_level("# No frontmatter"), None);
    }

    // ============================================================
    // Flatten tests
    // ============================================================

    #[test]
    fn test_flatten_workspace_simple() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();
        setup_local_hierarchy(ws_dir);

        let result = flatten_workspace(ws_dir).unwrap();

        // Should find: vision, ADR, initiative, 2 tasks, 1 backlog task = 6 documents
        assert_eq!(result.documents.len(), 6);

        let short_codes: Vec<&str> = result.documents.iter().map(|d| d.short_code.as_str()).collect();
        assert!(short_codes.contains(&"TEST-V-0001"));
        assert!(short_codes.contains(&"TEST-A-0001"));
        assert!(short_codes.contains(&"TEST-I-0001"));
        assert!(short_codes.contains(&"TEST-T-0001"));
        assert!(short_codes.contains(&"TEST-T-0002"));
        assert!(short_codes.contains(&"TEST-T-0003"));
    }

    #[test]
    fn test_flatten_excludes_non_document_files() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();
        setup_local_hierarchy(ws_dir);

        let result = flatten_workspace(ws_dir).unwrap();

        // config.toml and code-index.md should be skipped (no short_code)
        let filenames: Vec<&str> = result.documents.iter().map(|d| d.filename.as_str()).collect();
        assert!(!filenames.contains(&"config.toml"));
        assert!(!filenames.contains(&"code-index.md"));
    }

    #[test]
    fn test_flatten_preserves_content() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();
        setup_local_hierarchy(ws_dir);

        let result = flatten_workspace(ws_dir).unwrap();

        let vision = result
            .documents
            .iter()
            .find(|d| d.short_code == "TEST-V-0001")
            .unwrap();
        assert!(vision.content.contains("short_code: \"TEST-V-0001\""));
        assert!(vision.content.contains("# Test Document"));
    }

    #[test]
    fn test_flatten_filename_format() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();
        setup_local_hierarchy(ws_dir);

        let result = flatten_workspace(ws_dir).unwrap();

        for doc in &result.documents {
            assert_eq!(doc.filename, format!("{}.md", doc.short_code));
        }
    }

    #[test]
    fn test_flatten_empty_workspace() {
        let temp = tempdir().unwrap();
        let result = flatten_workspace(temp.path()).unwrap();
        assert!(result.documents.is_empty());
    }

    #[test]
    fn test_flatten_nonexistent_dir() {
        let result = flatten_workspace(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_flatten_skips_hidden_files() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();

        // Create a hidden .md file
        std::fs::write(
            ws_dir.join(".hidden-doc.md"),
            make_document_content("TEST-T-9999", "task", None),
        )
        .unwrap();

        let result = flatten_workspace(ws_dir).unwrap();
        assert!(result.documents.is_empty());
    }

    #[test]
    fn test_flatten_skips_archived_directory() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();

        // Create a normal doc
        let content = make_document_content("TEST-V-0001", "vision", None);
        std::fs::write(ws_dir.join("vision.md"), &content).unwrap();

        // Create an archived doc (should be excluded from flatten — archived dir is skipped)
        let archived_dir = ws_dir.join("archived/strategies/NULL/initiatives/TEST-I-OLD/tasks");
        std::fs::create_dir_all(&archived_dir).unwrap();
        std::fs::write(
            archived_dir.join("TEST-T-OLD.md"),
            make_document_content("TEST-T-OLD", "task", None),
        )
        .unwrap();

        let result = flatten_workspace(ws_dir).unwrap();
        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.documents[0].short_code, "TEST-V-0001");
    }

    #[test]
    fn test_flatten_handles_corrupted_frontmatter() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();

        // Valid document
        std::fs::write(
            ws_dir.join("good.md"),
            make_document_content("TEST-V-0001", "vision", None),
        )
        .unwrap();

        // Corrupted frontmatter (will be skipped, not cause an error)
        std::fs::write(
            ws_dir.join("bad.md"),
            "---\ninvalid: yaml: : broken\n---\n# Bad\n",
        )
        .unwrap();

        let result = flatten_workspace(ws_dir).unwrap();
        assert_eq!(result.documents.len(), 1);
        // The bad file should be in skipped (no valid short_code extracted)
        assert!(!result.skipped.is_empty());
    }

    #[test]
    fn test_flatten_reports_duplicate_short_codes() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();

        let content = make_document_content("TEST-T-0001", "task", None);

        // Write same short code in two different locations
        std::fs::write(ws_dir.join("doc1.md"), &content).unwrap();
        let subdir = ws_dir.join("subdir");
        std::fs::create_dir_all(&subdir).unwrap();
        std::fs::write(subdir.join("doc2.md"), &content).unwrap();

        let result = flatten_workspace(ws_dir).unwrap();
        // One should succeed, one should be in errors
        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.errors.len(), 1);
        assert!(result.errors[0].1.contains("Duplicate short code"));
    }

    // ============================================================
    // Read flat documents tests
    // ============================================================

    #[test]
    fn test_read_flat_documents() {
        let temp = tempdir().unwrap();
        let flat_dir = temp.path();

        // Write some flat documents
        std::fs::write(
            flat_dir.join("API-V-0001.md"),
            make_document_content("API-V-0001", "vision", None),
        )
        .unwrap();
        std::fs::write(
            flat_dir.join("API-T-0001.md"),
            make_document_content("API-T-0001", "task", Some("API-I-0001")),
        )
        .unwrap();

        let result = read_flat_documents(flat_dir).unwrap();
        assert_eq!(result.documents.len(), 2);

        // Should be sorted by short_code
        assert_eq!(result.documents[0].short_code, "API-T-0001");
        assert_eq!(result.documents[1].short_code, "API-V-0001");
    }

    #[test]
    fn test_read_flat_documents_skips_non_md() {
        let temp = tempdir().unwrap();
        let flat_dir = temp.path();

        std::fs::write(
            flat_dir.join("API-V-0001.md"),
            make_document_content("API-V-0001", "vision", None),
        )
        .unwrap();
        std::fs::write(flat_dir.join("config.toml"), "[project]\n").unwrap();
        std::fs::write(flat_dir.join("notes.txt"), "some notes").unwrap();

        let result = read_flat_documents(flat_dir).unwrap();
        assert_eq!(result.documents.len(), 1);
    }

    #[test]
    fn test_read_flat_documents_empty_dir() {
        let temp = tempdir().unwrap();
        let result = read_flat_documents(temp.path()).unwrap();
        assert!(result.documents.is_empty());
    }

    #[test]
    fn test_read_flat_documents_nonexistent_dir() {
        let result = read_flat_documents(Path::new("/nonexistent/path"));
        assert!(result.is_err());
    }

    #[test]
    fn test_read_flat_documents_skips_non_metis_md() {
        let temp = tempdir().unwrap();
        let flat_dir = temp.path();

        // Metis document
        std::fs::write(
            flat_dir.join("API-V-0001.md"),
            make_document_content("API-V-0001", "vision", None),
        )
        .unwrap();
        // Regular markdown (no frontmatter with short_code)
        std::fs::write(flat_dir.join("README.md"), "# README\n").unwrap();

        let result = read_flat_documents(flat_dir).unwrap();
        assert_eq!(result.documents.len(), 1);
        assert_eq!(result.skipped.len(), 1);
    }

    // ============================================================
    // Write flat documents tests
    // ============================================================

    #[test]
    fn test_write_flat_documents() {
        let temp = tempdir().unwrap();
        let target_dir = temp.path().join("api");

        let docs = vec![
            FlatDocument {
                short_code: "API-V-0001".to_string(),
                filename: "API-V-0001.md".to_string(),
                content: make_document_content("API-V-0001", "vision", None),
            },
            FlatDocument {
                short_code: "API-T-0001".to_string(),
                filename: "API-T-0001.md".to_string(),
                content: make_document_content("API-T-0001", "task", Some("API-I-0001")),
            },
        ];

        let count = write_flat_documents(&docs, &target_dir).unwrap();
        assert_eq!(count, 2);

        // Verify files exist
        assert!(target_dir.join("API-V-0001.md").exists());
        assert!(target_dir.join("API-T-0001.md").exists());

        // Verify content
        let content = std::fs::read_to_string(target_dir.join("API-V-0001.md")).unwrap();
        assert!(content.contains("short_code: \"API-V-0001\""));
    }

    #[test]
    fn test_write_flat_documents_creates_dir() {
        let temp = tempdir().unwrap();
        let target_dir = temp.path().join("nested/deep/path");

        let docs = vec![FlatDocument {
            short_code: "API-V-0001".to_string(),
            filename: "API-V-0001.md".to_string(),
            content: "test".to_string(),
        }];

        let count = write_flat_documents(&docs, &target_dir).unwrap();
        assert_eq!(count, 1);
        assert!(target_dir.join("API-V-0001.md").exists());
    }

    #[test]
    fn test_write_flat_documents_overwrites_existing() {
        let temp = tempdir().unwrap();
        let target_dir = temp.path();

        // Write initial content
        std::fs::write(target_dir.join("API-V-0001.md"), "old content").unwrap();

        let docs = vec![FlatDocument {
            short_code: "API-V-0001".to_string(),
            filename: "API-V-0001.md".to_string(),
            content: "new content".to_string(),
        }];

        write_flat_documents(&docs, target_dir).unwrap();
        let content = std::fs::read_to_string(target_dir.join("API-V-0001.md")).unwrap();
        assert_eq!(content, "new content");
    }

    #[test]
    fn test_write_flat_documents_empty_list() {
        let temp = tempdir().unwrap();
        let count = write_flat_documents(&[], temp.path()).unwrap();
        assert_eq!(count, 0);
    }

    // ============================================================
    // Remove stale files tests
    // ============================================================

    #[test]
    fn test_remove_stale_files() {
        let temp = tempdir().unwrap();
        let target_dir = temp.path();

        // Write 3 files
        std::fs::write(target_dir.join("API-V-0001.md"), "keep").unwrap();
        std::fs::write(target_dir.join("API-T-0001.md"), "keep").unwrap();
        std::fs::write(target_dir.join("API-T-0099.md"), "stale").unwrap();

        // Current documents only have 2
        let current = vec![
            FlatDocument {
                short_code: "API-V-0001".to_string(),
                filename: "API-V-0001.md".to_string(),
                content: "keep".to_string(),
            },
            FlatDocument {
                short_code: "API-T-0001".to_string(),
                filename: "API-T-0001.md".to_string(),
                content: "keep".to_string(),
            },
        ];

        let removed = remove_stale_files(target_dir, &current).unwrap();
        assert_eq!(removed.len(), 1);
        assert!(!target_dir.join("API-T-0099.md").exists());
        assert!(target_dir.join("API-V-0001.md").exists());
        assert!(target_dir.join("API-T-0001.md").exists());
    }

    #[test]
    fn test_remove_stale_files_preserves_non_md() {
        let temp = tempdir().unwrap();
        let target_dir = temp.path();

        std::fs::write(target_dir.join("API-V-0001.md"), "keep").unwrap();
        std::fs::write(target_dir.join("config.toml"), "preserve").unwrap();

        let current = vec![FlatDocument {
            short_code: "API-V-0001".to_string(),
            filename: "API-V-0001.md".to_string(),
            content: "keep".to_string(),
        }];

        let removed = remove_stale_files(target_dir, &current).unwrap();
        assert!(removed.is_empty());
        assert!(target_dir.join("config.toml").exists());
    }

    #[test]
    fn test_remove_stale_files_nonexistent_dir() {
        let removed =
            remove_stale_files(Path::new("/nonexistent/path"), &[]).unwrap();
        assert!(removed.is_empty());
    }

    // ============================================================
    // Roundtrip tests
    // ============================================================

    #[test]
    fn test_roundtrip_flatten_write_read() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path().join("workspace");
        let flat_dir = temp.path().join("flat");

        std::fs::create_dir_all(&ws_dir).unwrap();
        setup_local_hierarchy(&ws_dir);

        // Flatten
        let flat_result = flatten_workspace(&ws_dir).unwrap();
        assert_eq!(flat_result.documents.len(), 6);

        // Write flat
        write_flat_documents(&flat_result.documents, &flat_dir).unwrap();

        // Read flat
        let read_result = read_flat_documents(&flat_dir).unwrap();
        assert_eq!(read_result.documents.len(), 6);

        // Verify all short codes survived
        let original_codes: std::collections::HashSet<&str> = flat_result
            .documents
            .iter()
            .map(|d| d.short_code.as_str())
            .collect();
        let read_codes: std::collections::HashSet<&str> = read_result
            .documents
            .iter()
            .map(|d| d.short_code.as_str())
            .collect();
        assert_eq!(original_codes, read_codes);

        // Verify content is preserved byte-for-byte
        for orig in &flat_result.documents {
            let read = read_result
                .documents
                .iter()
                .find(|d| d.short_code == orig.short_code)
                .unwrap();
            assert_eq!(orig.content, read.content);
        }
    }

    #[test]
    fn test_roundtrip_unicode_content() {
        let temp = tempdir().unwrap();
        let ws_dir = temp.path();

        let content = format!(
            "---\nshort_code: \"TEST-T-0001\"\nlevel: task\ntitle: \"Unicode Test 日本語 🎉\"\nparent:\nblocked_by: []\narchived: false\ntags:\n  - \"#task\"\n---\n\n# 日本語テスト 🎉\n\nContent with émojis and spëcial chars: àáâãäå\n"
        );
        std::fs::write(ws_dir.join("unicode.md"), &content).unwrap();

        let flat_result = flatten_workspace(ws_dir).unwrap();
        assert_eq!(flat_result.documents.len(), 1);

        let flat_dir = temp.path().join("flat");
        write_flat_documents(&flat_result.documents, &flat_dir).unwrap();

        let read_result = read_flat_documents(&flat_dir).unwrap();
        assert_eq!(read_result.documents[0].content, content);
    }
}
