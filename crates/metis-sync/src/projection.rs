//! Cross-workspace projection cache.
//!
//! Scans all `.metis/<prefix>/*.md` files (both owned and hydrated remote)
//! to build an in-memory index of documents, cross-workspace relationships,
//! and progress aggregations. Rebuilt after each sync operation.

use gray_matter::{engine::YAML, Matter, Pod};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use tracing::{debug, warn};

// ─── Types ───────────────────────────────────────────────────────────────────

/// A document extracted from disk with parsed frontmatter metadata.
#[derive(Debug, Clone)]
pub struct CachedDocument {
    /// Unique short code (e.g., "API-T-0001")
    pub short_code: String,
    /// Document title
    pub title: String,
    /// Document type: vision, strategy, initiative, task, adr
    pub document_type: String,
    /// Current phase (extracted from tags like "#phase/active")
    pub phase: String,
    /// Parent document short code (None if top-level or unset)
    pub parent: Option<String>,
    /// Documents this one is blocked by
    pub blocked_by: Vec<String>,
    /// Whether the document is archived
    pub archived: bool,
    /// Workspace prefix this document belongs to
    pub workspace: String,
    /// Whether this is an owned (writable) document or hydrated (read-only)
    pub owned: bool,
    /// Relative file path within .metis/ directory
    pub file_path: String,
}

/// Phase-based progress counts for children of a document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProgressSummary {
    pub backlog: usize,
    pub todo: usize,
    pub active: usize,
    pub completed: usize,
    pub blocked: usize,
    pub other: usize,
}

impl ProgressSummary {
    /// Total number of children across all phases.
    pub fn total(&self) -> usize {
        self.backlog + self.todo + self.active + self.completed + self.blocked + self.other
    }
}

/// Errors that can occur during projection cache building.
#[derive(Debug)]
pub struct ProjectionWarning {
    pub file_path: String,
    pub message: String,
}

/// In-memory projection cache indexing all documents across workspaces.
#[derive(Debug)]
pub struct ProjectionCache {
    /// All documents keyed by short_code
    documents: HashMap<String, CachedDocument>,
    /// Parent → children index (short_code → set of child short_codes)
    children_index: HashMap<String, BTreeSet<String>>,
    /// Blocked_by inverse index (blocker_short_code → set of blocked short_codes)
    blocks_index: HashMap<String, BTreeSet<String>>,
    /// Workspace → documents index (prefix → set of short_codes)
    workspace_index: BTreeMap<String, BTreeSet<String>>,
    /// Warnings from documents that couldn't be parsed
    pub warnings: Vec<ProjectionWarning>,
}

// ─── Implementation ──────────────────────────────────────────────────────────

impl ProjectionCache {
    /// Build a projection cache by scanning all documents on disk.
    ///
    /// Scans:
    /// 1. Owned workspace: `.metis/<owned_prefix>/` directory (hierarchical or flat)
    /// 2. Hydrated remotes: all other `.metis/<prefix>/` directories (flat)
    ///
    /// # Arguments
    ///
    /// * `metis_dir` - Path to the `.metis/` directory
    /// * `owned_prefix` - The owned workspace prefix (for marking documents as owned)
    pub fn build(metis_dir: &Path, owned_prefix: &str) -> Self {
        let mut cache = ProjectionCache {
            documents: HashMap::new(),
            children_index: HashMap::new(),
            blocks_index: HashMap::new(),
            workspace_index: BTreeMap::new(),
            warnings: Vec::new(),
        };

        // Discover workspace directories
        let workspaces = discover_workspaces(metis_dir, owned_prefix);

        debug!(
            workspace_count = workspaces.len(),
            owned = owned_prefix,
            "building projection cache"
        );

        // Scan each workspace
        for (prefix, dir_path) in &workspaces {
            let is_owned = prefix == owned_prefix;
            cache.scan_workspace(prefix, dir_path, is_owned);
        }

        // Build relationship indices
        cache.build_indices();

        debug!(
            documents = cache.documents.len(),
            workspaces = cache.workspace_index.len(),
            warnings = cache.warnings.len(),
            "projection cache built"
        );

        cache
    }

    /// Scan a workspace directory for .md files and parse each one.
    fn scan_workspace(&mut self, prefix: &str, dir_path: &Path, is_owned: bool) {
        let entries = match fs::read_dir(dir_path) {
            Ok(e) => e,
            Err(e) => {
                warn!(prefix, error = %e, "failed to read workspace directory");
                return;
            }
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            // Only process .md files
            let ext = path.extension().and_then(|e| e.to_str());
            if ext != Some("md") {
                continue;
            }

            let filename = match path.file_name().and_then(|n| n.to_str()) {
                Some(name) => name.to_string(),
                None => continue,
            };

            match parse_document(&path, prefix, is_owned, &filename) {
                Ok(doc) => {
                    let short_code = doc.short_code.clone();
                    let workspace = doc.workspace.clone();

                    // Handle duplicate short codes defensively
                    if self.documents.contains_key(&short_code) {
                        self.warnings.push(ProjectionWarning {
                            file_path: format!("{}/{}", prefix, filename),
                            message: format!(
                                "duplicate short_code '{}', keeping first occurrence",
                                short_code
                            ),
                        });
                        continue;
                    }

                    self.workspace_index
                        .entry(workspace)
                        .or_default()
                        .insert(short_code.clone());
                    self.documents.insert(short_code, doc);
                }
                Err(msg) => {
                    self.warnings.push(ProjectionWarning {
                        file_path: format!("{}/{}", prefix, filename),
                        message: msg,
                    });
                }
            }
        }
    }

    /// Build relationship indices from parsed documents.
    fn build_indices(&mut self) {
        for (short_code, doc) in &self.documents {
            // Parent → children index
            if let Some(ref parent) = doc.parent {
                self.children_index
                    .entry(parent.clone())
                    .or_default()
                    .insert(short_code.clone());
            }

            // Blocked_by → blocks index (inverse)
            for blocker in &doc.blocked_by {
                self.blocks_index
                    .entry(blocker.clone())
                    .or_default()
                    .insert(short_code.clone());
            }
        }
    }

    // ─── Query Methods ───────────────────────────────────────────────────

    /// Get a document by short code.
    pub fn get(&self, short_code: &str) -> Option<&CachedDocument> {
        self.documents.get(short_code)
    }

    /// Get all documents in the cache.
    pub fn all_documents(&self) -> impl Iterator<Item = &CachedDocument> {
        self.documents.values()
    }

    /// Total number of documents in the cache.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    /// Whether the cache is empty.
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// All workspace prefixes in the cache.
    pub fn workspaces(&self) -> Vec<&str> {
        self.workspace_index.keys().map(|s| s.as_str()).collect()
    }

    /// Find all children of a document across all workspaces.
    ///
    /// Returns documents whose `parent` field matches the given short code.
    pub fn children_of(&self, short_code: &str) -> Vec<&CachedDocument> {
        match self.children_index.get(short_code) {
            Some(children) => children
                .iter()
                .filter_map(|sc| self.documents.get(sc))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Find all documents blocked by a given document.
    ///
    /// Returns documents whose `blocked_by` list contains the given short code.
    pub fn blocks(&self, short_code: &str) -> Vec<&CachedDocument> {
        match self.blocks_index.get(short_code) {
            Some(blocked) => blocked
                .iter()
                .filter_map(|sc| self.documents.get(sc))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Compute progress summary for a document's children.
    ///
    /// Counts children by phase across all workspaces.
    pub fn progress(&self, short_code: &str) -> ProgressSummary {
        let children = self.children_of(short_code);
        let mut summary = ProgressSummary::default();

        for child in children {
            match child.phase.as_str() {
                "backlog" => summary.backlog += 1,
                "todo" => summary.todo += 1,
                "active" => summary.active += 1,
                "completed" => summary.completed += 1,
                "blocked" => summary.blocked += 1,
                _ => summary.other += 1,
            }
        }

        summary
    }

    /// Get all documents in a specific workspace.
    pub fn workspace_documents(&self, prefix: &str) -> Vec<&CachedDocument> {
        match self.workspace_index.get(prefix) {
            Some(codes) => codes
                .iter()
                .filter_map(|sc| self.documents.get(sc))
                .collect(),
            None => Vec::new(),
        }
    }

    /// Find upstream context for a workspace — all documents referenced by
    /// this workspace's documents via parent chains.
    ///
    /// Walks up the parent chain from each document in the workspace,
    /// collecting all referenced documents from OTHER workspaces.
    pub fn upstream_context(&self, prefix: &str) -> Vec<&CachedDocument> {
        let workspace_docs = match self.workspace_index.get(prefix) {
            Some(codes) => codes,
            None => return Vec::new(),
        };

        let mut upstream_codes: BTreeSet<String> = BTreeSet::new();
        let mut visited: BTreeSet<String> = BTreeSet::new();

        for code in workspace_docs {
            self.walk_parent_chain(code, prefix, &mut upstream_codes, &mut visited);
        }

        upstream_codes
            .iter()
            .filter_map(|sc| self.documents.get(sc))
            .collect()
    }

    /// Walk up the parent chain, collecting documents from other workspaces.
    fn walk_parent_chain(
        &self,
        short_code: &str,
        source_prefix: &str,
        upstream: &mut BTreeSet<String>,
        visited: &mut BTreeSet<String>,
    ) {
        if !visited.insert(short_code.to_string()) {
            return; // Already visited — prevents cycles
        }

        let doc = match self.documents.get(short_code) {
            Some(d) => d,
            None => return, // Unresolved reference — skip
        };

        // If this document is from a different workspace, it's upstream context
        if doc.workspace != source_prefix {
            upstream.insert(short_code.to_string());
        }

        // Continue walking up
        if let Some(ref parent) = doc.parent {
            self.walk_parent_chain(parent, source_prefix, upstream, visited);
        }
    }
}

// ─── Parsing ─────────────────────────────────────────────────────────────────

/// Known non-workspace entries in `.metis/` that should be skipped.
const RESERVED_NAMES: &[&str] = &[
    "archived",
    "strategies",
    "adrs",
    "backlog",
    "templates",
    "code-index.md",
    "code-index-hashes.json",
    "code-index-symbols.json",
    "config.toml",
    "metis.db",
    "metis.db-journal",
    "metis.db-wal",
    "metis.db-shm",
    ".gitignore",
    ".index-dirty",
];

/// Discover workspace directories in the `.metis/` folder.
///
/// Returns a list of (prefix, directory_path) tuples for all workspace folders.
/// A workspace folder is a non-hidden, non-reserved directory containing .md files.
fn discover_workspaces(metis_dir: &Path, owned_prefix: &str) -> Vec<(String, std::path::PathBuf)> {
    let mut workspaces = Vec::new();
    let reserved: BTreeSet<&str> = RESERVED_NAMES.iter().copied().collect();

    let entries = match fs::read_dir(metis_dir) {
        Ok(e) => e,
        Err(e) => {
            warn!(error = %e, "failed to read .metis directory");
            return workspaces;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let dirname = match path.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_string(),
            None => continue,
        };

        // Skip hidden directories
        if dirname.starts_with('.') {
            continue;
        }

        // Skip reserved directories
        if reserved.contains(dirname.as_str()) {
            continue;
        }

        workspaces.push((dirname, path));
    }

    // Ensure the owned workspace comes first for deterministic processing
    workspaces.sort_by(|(a, _), (b, _)| {
        if a == owned_prefix {
            std::cmp::Ordering::Less
        } else if b == owned_prefix {
            std::cmp::Ordering::Greater
        } else {
            a.cmp(b)
        }
    });

    workspaces
}

/// Parse a single markdown document file and extract frontmatter metadata.
fn parse_document(
    path: &Path,
    workspace: &str,
    is_owned: bool,
    filename: &str,
) -> Result<CachedDocument, String> {
    let content = fs::read_to_string(path).map_err(|e| format!("read error: {}", e))?;

    let matter = Matter::<YAML>::new();
    let parsed = matter.parse(&content);

    let data = parsed
        .data
        .ok_or_else(|| "no frontmatter found".to_string())?;

    let fm = match data {
        Pod::Hash(map) => map,
        _ => return Err("frontmatter is not a YAML mapping".to_string()),
    };

    let short_code = extract_string(&fm, "short_code")
        .ok_or_else(|| "missing or invalid 'short_code' field".to_string())?;

    let title = extract_string(&fm, "title").unwrap_or_default();

    let document_type = extract_string(&fm, "level")
        .ok_or_else(|| "missing or invalid 'level' field".to_string())?;

    let phase = extract_phase_from_tags(&fm);

    let parent = extract_string(&fm, "parent").filter(|s| s != "NULL" && !s.is_empty());

    let blocked_by = extract_string_array(&fm, "blocked_by");

    let archived = extract_bool(&fm, "archived").unwrap_or(false);

    Ok(CachedDocument {
        short_code,
        title,
        document_type,
        phase,
        parent,
        blocked_by,
        archived,
        workspace: workspace.to_string(),
        owned: is_owned,
        file_path: format!("{}/{}", workspace, filename),
    })
}

/// Extract a string value from a Pod HashMap.
fn extract_string(map: &HashMap<String, Pod>, key: &str) -> Option<String> {
    match map.get(key)? {
        Pod::String(s) => {
            let trimmed = s.trim().trim_matches('"').to_string();
            if trimmed.is_empty() || trimmed == "NULL" {
                None
            } else {
                Some(trimmed)
            }
        }
        _ => None,
    }
}

/// Extract a boolean from a Pod HashMap.
fn extract_bool(map: &HashMap<String, Pod>, key: &str) -> Option<bool> {
    match map.get(key)? {
        Pod::Boolean(b) => Some(*b),
        Pod::String(s) => match s.trim().to_lowercase().as_str() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        },
        _ => None,
    }
}

/// Extract the phase from the tags array.
///
/// Looks for tags like "#phase/active" and returns the phase portion.
fn extract_phase_from_tags(map: &HashMap<String, Pod>) -> String {
    if let Some(Pod::Array(tags)) = map.get("tags") {
        for tag in tags {
            if let Pod::String(s) = tag {
                let trimmed = s.trim().trim_matches('"');
                if let Some(phase) = trimmed.strip_prefix("#phase/") {
                    return phase.to_string();
                }
            }
        }
    }
    "unknown".to_string()
}

/// Extract a string array from a Pod HashMap.
///
/// Returns empty vec for NULL, empty, or missing values.
fn extract_string_array(map: &HashMap<String, Pod>, key: &str) -> Vec<String> {
    match map.get(key) {
        Some(Pod::Array(items)) => items
            .iter()
            .filter_map(|item| match item {
                Pod::String(s) => {
                    let trimmed = s.trim().trim_matches('"').to_string();
                    if trimmed.is_empty() || trimmed == "NULL" {
                        None
                    } else {
                        Some(trimmed)
                    }
                }
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    // ─── Helpers ─────────────────────────────────────────────────────────

    /// Create a .metis/ directory in a temp location.
    fn create_metis_dir() -> TempDir {
        TempDir::new().unwrap()
    }

    /// Write a document .md file with standard frontmatter.
    fn write_doc(metis_dir: &Path, prefix: &str, filename: &str, frontmatter: &str, body: &str) {
        let dir = metis_dir.join(prefix);
        fs::create_dir_all(&dir).unwrap();
        let content = format!("---\n{}\n---\n\n{}", frontmatter, body);
        fs::write(dir.join(filename), content).unwrap();
    }

    /// Standard task frontmatter.
    fn task_fm(short_code: &str, parent: &str, phase: &str) -> String {
        format!(
            r##"id: {}
level: task
title: "Task {}"
short_code: "{}"
created_at: 2026-01-01T00:00:00+00:00
updated_at: 2026-01-01T00:00:00+00:00
parent: {}
blocked_by: []
archived: false
tags:
  - "#task"
  - "#phase/{}"
exit_criteria_met: false
strategy_id: NULL
initiative_id: {}"##,
            short_code.to_lowercase().replace("-", "-"),
            short_code,
            short_code,
            parent,
            phase,
            parent,
        )
    }

    /// Standard initiative frontmatter.
    fn initiative_fm(short_code: &str, parent: &str, phase: &str) -> String {
        format!(
            r##"id: {}
level: initiative
title: "Initiative {}"
short_code: "{}"
created_at: 2026-01-01T00:00:00+00:00
updated_at: 2026-01-01T00:00:00+00:00
parent: {}
blocked_by: []
archived: false
tags:
  - "#initiative"
  - "#phase/{}"
exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: {}"##,
            short_code.to_lowercase().replace("-", "-"),
            short_code,
            short_code,
            parent,
            phase,
            short_code.to_lowercase(),
        )
    }

    /// Standard vision frontmatter.
    fn vision_fm(short_code: &str, phase: &str) -> String {
        format!(
            r##"id: {}
level: vision
title: "Vision {}"
short_code: "{}"
created_at: 2026-01-01T00:00:00+00:00
updated_at: 2026-01-01T00:00:00+00:00
archived: false
tags:
  - "#vision"
  - "#phase/{}"
exit_criteria_met: true"##,
            short_code.to_lowercase().replace("-", "-"),
            short_code,
            short_code,
            phase,
        )
    }

    /// Task frontmatter with blocked_by.
    fn blocked_task_fm(
        short_code: &str,
        parent: &str,
        phase: &str,
        blocked_by: &[&str],
    ) -> String {
        let blocked_list = if blocked_by.is_empty() {
            "[]".to_string()
        } else {
            let items: Vec<String> = blocked_by.iter().map(|s| format!("  - {}", s)).collect();
            format!("\n{}", items.join("\n"))
        };
        format!(
            r##"id: {}
level: task
title: "Task {}"
short_code: "{}"
created_at: 2026-01-01T00:00:00+00:00
updated_at: 2026-01-01T00:00:00+00:00
parent: {}
blocked_by: {}
archived: false
tags:
  - "#task"
  - "#phase/{}"
exit_criteria_met: false
strategy_id: NULL
initiative_id: {}"##,
            short_code.to_lowercase(),
            short_code,
            short_code,
            parent,
            blocked_list,
            phase,
            parent,
        )
    }

    // ─── Unit Tests — Cache Rebuild Scope ────────────────────────────────

    #[test]
    fn test_owned_workspace_only_single_workspace() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "# Vision",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-I-0001.md",
            &initiative_fm("API-I-0001", "API-V-0001", "active"),
            "# Initiative",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "# Task",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 3);
        assert!(cache.get("API-V-0001").is_some());
        assert!(cache.get("API-I-0001").is_some());
        assert!(cache.get("API-T-0001").is_some());

        // All should be owned
        for doc in cache.all_documents() {
            assert!(doc.owned, "{} should be owned", doc.short_code);
            assert_eq!(doc.workspace, "api");
        }
    }

    #[test]
    fn test_owned_plus_one_remote() {
        let metis_dir = create_metis_dir();

        // Owned workspace
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "# Vision",
        );

        // Hydrated remote
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "# Strat Vision",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 2);
        assert!(cache.get("API-V-0001").unwrap().owned);
        assert!(!cache.get("STRAT-V-0001").unwrap().owned);
    }

    #[test]
    fn test_owned_plus_multiple_remotes() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "alpha",
            "ALPHA-V-0001.md",
            &vision_fm("ALPHA-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-V-0001.md",
            &vision_fm("SRE-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 4);
        assert_eq!(cache.workspaces().len(), 4);

        assert!(cache.get("API-V-0001").unwrap().owned);
        assert!(!cache.get("STRAT-V-0001").unwrap().owned);
        assert!(!cache.get("ALPHA-V-0001").unwrap().owned);
        assert!(!cache.get("SRE-V-0001").unwrap().owned);
    }

    #[test]
    fn test_empty_remote_workspace() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        // Create empty remote workspace dir
        fs::create_dir_all(metis_dir.path().join("alpha")).unwrap();

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 1);
        assert!(cache.workspace_documents("alpha").is_empty());
    }

    #[test]
    fn test_non_document_files_skipped() {
        let metis_dir = create_metis_dir();

        // Create files that should NOT be indexed
        fs::write(metis_dir.path().join("config.toml"), "[upstream]").unwrap();
        fs::write(metis_dir.path().join("code-index.md"), "# Index").unwrap();

        // Reserved directories should be skipped
        fs::create_dir_all(metis_dir.path().join("archived")).unwrap();
        fs::write(
            metis_dir.path().join("archived/old.md"),
            "---\nshort_code: OLD-V-0001\nlevel: vision\ntags:\n  - \"#phase/published\"\n---\n",
        )
        .unwrap();

        // Actual workspace doc
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 1);
        assert!(cache.get("API-V-0001").is_some());
        assert!(cache.get("OLD-V-0001").is_none());
    }

    #[test]
    fn test_archived_documents_indexed() {
        let metis_dir = create_metis_dir();

        let fm = r##"id: api-v-0001
level: vision
title: "Archived Vision"
short_code: "API-V-0001"
created_at: 2026-01-01T00:00:00+00:00
updated_at: 2026-01-01T00:00:00+00:00
archived: true
tags:
  - "#vision"
  - "#phase/published"
exit_criteria_met: true"##;

        write_doc(metis_dir.path(), "api", "API-V-0001.md", fm, "");

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 1);
        let doc = cache.get("API-V-0001").unwrap();
        assert!(doc.archived);
    }

    // ─── Unit Tests — Cross-Workspace Relationships ──────────────────────

    #[test]
    fn test_parent_reference_across_workspaces() {
        let metis_dir = create_metis_dir();

        // Remote initiative
        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );

        // Owned task referencing remote parent
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let children = cache.children_of("WGR-I-0001");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].short_code, "API-T-0001");
    }

    #[test]
    fn test_multiple_children_across_workspaces() {
        let metis_dir = create_metis_dir();

        // Shared initiative
        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );

        // Tasks from different workspaces
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-T-0001.md",
            &task_fm("SRE-T-0001", "WGR-I-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "alpha",
            "ALPHA-T-0001.md",
            &task_fm("ALPHA-T-0001", "WGR-I-0001", "completed"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let children = cache.children_of("WGR-I-0001");
        assert_eq!(children.len(), 3);

        let codes: BTreeSet<&str> = children.iter().map(|d| d.short_code.as_str()).collect();
        assert!(codes.contains("API-T-0001"));
        assert!(codes.contains("SRE-T-0001"));
        assert!(codes.contains("ALPHA-T-0001"));
    }

    #[test]
    fn test_blocked_by_across_workspaces() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-T-0001.md",
            &task_fm("SRE-T-0001", "WGR-I-0001", "active"),
            "",
        );

        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &blocked_task_fm("API-T-0002", "WGR-I-0001", "blocked", &["SRE-T-0001"]),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let blocked = cache.blocks("SRE-T-0001");
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].short_code, "API-T-0002");
    }

    #[test]
    fn test_deep_parent_chain() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        // Task → Initiative → Vision chain
        let task = cache.get("API-T-0001").unwrap();
        assert_eq!(task.parent.as_deref(), Some("WGR-I-0001"));

        let initiative = cache.get("WGR-I-0001").unwrap();
        assert_eq!(initiative.parent.as_deref(), Some("STRAT-V-0001"));

        let vision = cache.get("STRAT-V-0001").unwrap();
        assert!(vision.parent.is_none());

        // Children chain
        let v_children = cache.children_of("STRAT-V-0001");
        assert_eq!(v_children.len(), 1);
        assert_eq!(v_children[0].short_code, "WGR-I-0001");

        let i_children = cache.children_of("WGR-I-0001");
        assert_eq!(i_children.len(), 1);
        assert_eq!(i_children[0].short_code, "API-T-0001");
    }

    #[test]
    fn test_orphaned_reference() {
        let metis_dir = create_metis_dir();

        // Task references a parent that doesn't exist
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "UNKNOWN-I-9999", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 1);
        let doc = cache.get("API-T-0001").unwrap();
        assert_eq!(doc.parent.as_deref(), Some("UNKNOWN-I-9999"));

        // children_of the missing parent should still work
        let children = cache.children_of("UNKNOWN-I-9999");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].short_code, "API-T-0001");
    }

    #[test]
    fn test_self_referencing_document() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-T-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        // Should still build without infinite loop
        assert_eq!(cache.len(), 1);

        // upstream_context should not loop
        let upstream = cache.upstream_context("api");
        assert!(upstream.is_empty()); // Self-ref in same workspace → not upstream
    }

    #[test]
    fn test_circular_blocked_by() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &blocked_task_fm("API-T-0001", "WGR-I-0001", "blocked", &["API-T-0002"]),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &blocked_task_fm("API-T-0002", "WGR-I-0001", "blocked", &["API-T-0001"]),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        // Both should build fine
        assert_eq!(cache.len(), 2);

        // blocks() should not loop
        let blocked_by_1 = cache.blocks("API-T-0001");
        assert_eq!(blocked_by_1.len(), 1);
        assert_eq!(blocked_by_1[0].short_code, "API-T-0002");

        let blocked_by_2 = cache.blocks("API-T-0002");
        assert_eq!(blocked_by_2.len(), 1);
        assert_eq!(blocked_by_2[0].short_code, "API-T-0001");
    }

    // ─── Unit Tests — Query Functions ────────────────────────────────────

    #[test]
    fn test_children_of_no_children() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        assert!(cache.children_of("API-V-0001").is_empty());
    }

    #[test]
    fn test_children_of_owned_only() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-I-0001.md",
            &initiative_fm("API-I-0001", "API-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &task_fm("API-T-0002", "API-I-0001", "active"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let children = cache.children_of("API-I-0001");
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn test_children_of_cross_workspace() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-T-0001.md",
            &task_fm("SRE-T-0001", "WGR-I-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "alpha",
            "ALPHA-T-0001.md",
            &task_fm("ALPHA-T-0001", "WGR-I-0001", "completed"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let children = cache.children_of("WGR-I-0001");
        assert_eq!(children.len(), 3);
    }

    #[test]
    fn test_progress_all_phases() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-I-0001.md",
            &initiative_fm("API-I-0001", "API-V-0001", "active"),
            "",
        );

        // 2 todo
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &task_fm("API-T-0002", "API-I-0001", "todo"),
            "",
        );
        // 1 active
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0003.md",
            &task_fm("API-T-0003", "API-I-0001", "active"),
            "",
        );
        // 3 completed
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0004.md",
            &task_fm("API-T-0004", "API-I-0001", "completed"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0005.md",
            &task_fm("API-T-0005", "API-I-0001", "completed"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0006.md",
            &task_fm("API-T-0006", "API-I-0001", "completed"),
            "",
        );
        // 1 blocked
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0007.md",
            &blocked_task_fm("API-T-0007", "API-I-0001", "blocked", &["API-T-0003"]),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let progress = cache.progress("API-I-0001");

        assert_eq!(
            progress,
            ProgressSummary {
                backlog: 0,
                todo: 2,
                active: 1,
                completed: 3,
                blocked: 1,
                other: 0,
            }
        );
        assert_eq!(progress.total(), 7);
    }

    #[test]
    fn test_progress_cross_workspace_aggregation() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );

        // Tasks from 3 workspaces
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "completed"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-T-0001.md",
            &task_fm("SRE-T-0001", "WGR-I-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "alpha",
            "ALPHA-T-0001.md",
            &task_fm("ALPHA-T-0001", "WGR-I-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let progress = cache.progress("WGR-I-0001");

        assert_eq!(progress.completed, 1);
        assert_eq!(progress.active, 1);
        assert_eq!(progress.todo, 1);
        assert_eq!(progress.total(), 3);
    }

    #[test]
    fn test_progress_no_children() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let progress = cache.progress("API-V-0001");
        assert_eq!(progress, ProgressSummary::default());
        assert_eq!(progress.total(), 0);
    }

    #[test]
    fn test_workspace_documents_owned() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let api_docs = cache.workspace_documents("api");
        assert_eq!(api_docs.len(), 2);
        for doc in &api_docs {
            assert_eq!(doc.workspace, "api");
        }
    }

    #[test]
    fn test_workspace_documents_remote() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-I-0001.md",
            &initiative_fm("STRAT-I-0001", "STRAT-V-0001", "active"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let strat_docs = cache.workspace_documents("strat");
        assert_eq!(strat_docs.len(), 2);
        for doc in &strat_docs {
            assert_eq!(doc.workspace, "strat");
            assert!(!doc.owned);
        }
    }

    #[test]
    fn test_workspace_documents_nonexistent() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        assert!(cache.workspace_documents("xyz").is_empty());
    }

    #[test]
    fn test_upstream_context_full_chain() {
        let metis_dir = create_metis_dir();

        // Remote upstream chain
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );

        // Owned tasks referencing remote initiative
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &task_fm("API-T-0002", "WGR-I-0001", "active"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        let upstream = cache.upstream_context("api");

        // Should include WGR-I-0001 and STRAT-V-0001 (both from strat workspace)
        let codes: BTreeSet<&str> = upstream.iter().map(|d| d.short_code.as_str()).collect();
        assert_eq!(codes.len(), 2);
        assert!(codes.contains("WGR-I-0001"));
        assert!(codes.contains("STRAT-V-0001"));
    }

    #[test]
    fn test_upstream_context_no_upstream() {
        let metis_dir = create_metis_dir();

        // All documents in same workspace — no upstream
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-I-0001.md",
            &initiative_fm("API-I-0001", "API-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");
        assert!(cache.upstream_context("api").is_empty());
    }

    // ─── Unit Tests — Owned vs Hydrated Distinction ──────────────────────

    #[test]
    fn test_owned_documents_marked_owned() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        for doc in cache.workspace_documents("api") {
            assert!(doc.owned);
        }
    }

    #[test]
    fn test_hydrated_documents_marked_not_owned() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        let doc = cache.get("STRAT-V-0001").unwrap();
        assert!(!doc.owned);
    }

    #[test]
    fn test_workspace_prefix_on_each_document() {
        let metis_dir = create_metis_dir();
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.get("API-V-0001").unwrap().workspace, "api");
        assert_eq!(cache.get("STRAT-V-0001").unwrap().workspace, "strat");
    }

    // ─── Edge Cases ──────────────────────────────────────────────────────

    #[test]
    fn test_corrupted_document_skipped() {
        let metis_dir = create_metis_dir();

        // Good document
        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        // Corrupted: no frontmatter
        let dir = metis_dir.path().join("api");
        fs::write(dir.join("API-T-BAD.md"), "# No frontmatter here").unwrap();

        // Corrupted: missing short_code
        fs::write(
            dir.join("API-T-BAD2.md"),
            "---\nlevel: task\ntags:\n  - \"#phase/todo\"\n---\n",
        )
        .unwrap();

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache.len(), 1);
        assert!(cache.get("API-V-0001").is_some());
        assert_eq!(cache.warnings.len(), 2);
    }

    #[test]
    fn test_duplicate_short_codes() {
        let metis_dir = create_metis_dir();

        // Same short_code in two workspaces
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "alpha",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "active"),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        // Should keep the first one (owned, since api is processed first)
        assert_eq!(cache.len(), 1);
        let doc = cache.get("API-T-0001").unwrap();
        assert!(doc.owned); // api (owned) was scanned first
        assert_eq!(cache.warnings.len(), 1);
    }

    #[test]
    fn test_cache_rebuild_idempotent() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-V-0001", "todo"),
            "",
        );

        let cache1 = ProjectionCache::build(metis_dir.path(), "api");
        let cache2 = ProjectionCache::build(metis_dir.path(), "api");

        assert_eq!(cache1.len(), cache2.len());

        for doc1 in cache1.all_documents() {
            let doc2 = cache2.get(&doc1.short_code).unwrap();
            assert_eq!(doc1.short_code, doc2.short_code);
            assert_eq!(doc1.phase, doc2.phase);
            assert_eq!(doc1.parent, doc2.parent);
            assert_eq!(doc1.owned, doc2.owned);
        }
    }

    #[test]
    fn test_empty_project() {
        let metis_dir = create_metis_dir();

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        assert!(cache.is_empty());
        assert_eq!(cache.len(), 0);
        assert!(cache.workspaces().is_empty());
        assert!(cache.children_of("ANYTHING").is_empty());
        assert!(cache.blocks("ANYTHING").is_empty());
        assert_eq!(cache.progress("ANYTHING"), ProgressSummary::default());
        assert!(cache.workspace_documents("api").is_empty());
        assert!(cache.upstream_context("api").is_empty());
    }

    // ─── Performance ─────────────────────────────────────────────────────

    #[test]
    fn test_performance_1000_documents() {
        let metis_dir = create_metis_dir();

        // 10 workspaces × 100 documents each
        for ws_idx in 0..10 {
            let prefix = format!("ws{}", ws_idx);
            let vision_sc = format!("WS{}-V-0001", ws_idx);

            write_doc(
                metis_dir.path(),
                &prefix,
                &format!("{}.md", vision_sc),
                &vision_fm(&vision_sc, "published"),
                "",
            );

            let init_sc = format!("WS{}-I-0001", ws_idx);
            write_doc(
                metis_dir.path(),
                &prefix,
                &format!("{}.md", init_sc),
                &initiative_fm(&init_sc, &vision_sc, "active"),
                "",
            );

            for t_idx in 1..=98 {
                let task_sc = format!("WS{}-T-{:04}", ws_idx, t_idx);
                write_doc(
                    metis_dir.path(),
                    &prefix,
                    &format!("{}.md", task_sc),
                    &task_fm(&task_sc, &init_sc, "todo"),
                    "",
                );
            }
        }

        let start = std::time::Instant::now();
        let cache = ProjectionCache::build(metis_dir.path(), "ws0");
        let elapsed = start.elapsed();

        assert_eq!(cache.len(), 1000);
        assert_eq!(cache.workspaces().len(), 10);
        assert!(
            elapsed.as_secs() < 1,
            "cache rebuild took {:?}, should be <1s",
            elapsed
        );
    }

    #[test]
    fn test_performance_5000_documents() {
        let metis_dir = create_metis_dir();

        // 10 workspaces × 500 documents each
        for ws_idx in 0..10 {
            let prefix = format!("ws{}", ws_idx);
            let vision_sc = format!("WS{}-V-0001", ws_idx);

            write_doc(
                metis_dir.path(),
                &prefix,
                &format!("{}.md", vision_sc),
                &vision_fm(&vision_sc, "published"),
                "",
            );

            let init_sc = format!("WS{}-I-0001", ws_idx);
            write_doc(
                metis_dir.path(),
                &prefix,
                &format!("{}.md", init_sc),
                &initiative_fm(&init_sc, &vision_sc, "active"),
                "",
            );

            for t_idx in 1..=498 {
                let task_sc = format!("WS{}-T-{:04}", ws_idx, t_idx);
                write_doc(
                    metis_dir.path(),
                    &prefix,
                    &format!("{}.md", task_sc),
                    &task_fm(&task_sc, &init_sc, "todo"),
                    "",
                );
            }
        }

        let start = std::time::Instant::now();
        let cache = ProjectionCache::build(metis_dir.path(), "ws0");
        let elapsed = start.elapsed();

        assert_eq!(cache.len(), 5000);
        assert!(
            elapsed.as_secs() < 5,
            "cache rebuild took {:?}, should be <5s",
            elapsed
        );
    }

    // ─── Integration-style Tests ─────────────────────────────────────────

    #[test]
    fn test_cache_reflects_disk_changes() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-V-0001.md",
            &vision_fm("API-V-0001", "published"),
            "",
        );

        let cache1 = ProjectionCache::build(metis_dir.path(), "api");
        assert_eq!(cache1.len(), 1);

        // Add a new document
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-V-0001", "todo"),
            "",
        );

        let cache2 = ProjectionCache::build(metis_dir.path(), "api");
        assert_eq!(cache2.len(), 2);
        assert!(cache2.get("API-T-0001").is_some());

        // Delete the document
        fs::remove_file(metis_dir.path().join("api/API-T-0001.md")).unwrap();

        let cache3 = ProjectionCache::build(metis_dir.path(), "api");
        assert_eq!(cache3.len(), 1);
        assert!(cache3.get("API-T-0001").is_none());
    }

    #[test]
    fn test_cache_consistency_after_phase_change() {
        let metis_dir = create_metis_dir();

        write_doc(
            metis_dir.path(),
            "api",
            "API-I-0001.md",
            &initiative_fm("API-I-0001", "API-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "todo"),
            "",
        );

        let cache1 = ProjectionCache::build(metis_dir.path(), "api");
        let progress1 = cache1.progress("API-I-0001");
        assert_eq!(progress1.todo, 1);
        assert_eq!(progress1.completed, 0);

        // "Complete" the task by rewriting the file
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "API-I-0001", "completed"),
            "",
        );

        let cache2 = ProjectionCache::build(metis_dir.path(), "api");
        let progress2 = cache2.progress("API-I-0001");
        assert_eq!(progress2.todo, 0);
        assert_eq!(progress2.completed, 1);
    }

    #[test]
    fn test_full_multi_workspace_scenario() {
        let metis_dir = create_metis_dir();

        // Strategy workspace — top-level vision + working group initiative
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-V-0001.md",
            &vision_fm("STRAT-V-0001", "published"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "WGR-I-0001.md",
            &initiative_fm("WGR-I-0001", "STRAT-V-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "strat",
            "STRAT-T-0001.md",
            &task_fm("STRAT-T-0001", "WGR-I-0001", "completed"),
            "",
        );

        // API workspace — tasks under the shared initiative
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0001.md",
            &task_fm("API-T-0001", "WGR-I-0001", "active"),
            "",
        );
        write_doc(
            metis_dir.path(),
            "api",
            "API-T-0002.md",
            &task_fm("API-T-0002", "WGR-I-0001", "todo"),
            "",
        );

        // SRE workspace — one task blocked by API
        write_doc(
            metis_dir.path(),
            "sre",
            "SRE-T-0001.md",
            &blocked_task_fm("SRE-T-0001", "WGR-I-0001", "blocked", &["API-T-0001"]),
            "",
        );

        let cache = ProjectionCache::build(metis_dir.path(), "api");

        // Total documents
        assert_eq!(cache.len(), 6);

        // Workspaces
        assert_eq!(cache.workspaces().len(), 3);

        // Progress on the shared initiative
        let progress = cache.progress("WGR-I-0001");
        assert_eq!(progress.completed, 1); // STRAT-T-0001
        assert_eq!(progress.active, 1); // API-T-0001
        assert_eq!(progress.todo, 1); // API-T-0002
        assert_eq!(progress.blocked, 1); // SRE-T-0001
        assert_eq!(progress.total(), 4);

        // Cross-workspace children
        let children = cache.children_of("WGR-I-0001");
        assert_eq!(children.len(), 4);

        // Blocked by
        let blocked = cache.blocks("API-T-0001");
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].short_code, "SRE-T-0001");

        // Upstream context for API workspace
        let upstream = cache.upstream_context("api");
        let codes: BTreeSet<&str> = upstream.iter().map(|d| d.short_code.as_str()).collect();
        assert!(codes.contains("WGR-I-0001"));
        assert!(codes.contains("STRAT-V-0001"));

        // Owned vs hydrated
        assert!(cache.get("API-T-0001").unwrap().owned);
        assert!(!cache.get("STRAT-V-0001").unwrap().owned);
        assert!(!cache.get("SRE-T-0001").unwrap().owned);
    }
}
