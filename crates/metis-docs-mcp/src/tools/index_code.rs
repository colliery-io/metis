//! `index_code` MCP tool — generate code index for AI agent navigation.
//!
//! Exposes the code-indexing pipeline as an MCP tool so AI agents can
//! trigger index generation programmatically.

use crate::formatting::ToolOutput;
use metis_code_index::parser::{Language, ParsedFile, Parser};
use metis_code_index::symbols::Symbol;
use metis_code_index::walker::walk_directory;
use metis_code_index::{
    format_index, GoExtractor, HashManifest, PythonExtractor, RustExtractor, SymbolCache,
    TypeScriptExtractor,
};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[mcp_tool(
    name = "index_code",
    description = "Generate a code index for AI agent navigation. Walks source files, parses with tree-sitter, extracts symbols, and writes .metis/code-index.md. Supports Rust, Python, TypeScript, JavaScript, and Go.",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = false
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct IndexCodeTool {
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Only generate the project structure tree, skip symbol extraction (default: false)
    #[serde(default)]
    pub structure_only: Option<bool>,
    /// Enable incremental indexing — only re-index changed files using content hashes (default: false)
    #[serde(default)]
    pub incremental: Option<bool>,
}

impl IndexCodeTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let start = Instant::now();
        let metis_dir = Path::new(&self.project_path);

        // Validate .metis directory exists
        if !metis_dir.exists() || !metis_dir.is_dir() {
            return Ok(ToolOutput::new()
                .header("Index Code Error")
                .text(&format!(
                    "Directory not found: {}. Run `initialize_project` first.",
                    self.project_path
                ))
                .build_result());
        }

        let project_root = metis_dir.parent().ok_or_else(|| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Cannot determine project root from .metis directory",
            ))
        })?;

        let structure_only = self.structure_only.unwrap_or(false);
        let incremental = self.incremental.unwrap_or(false);

        let hash_path = metis_dir.join("code-index-hashes.json");
        let symbol_cache_path = metis_dir.join("code-index-symbols.json");

        // Step 1: Walk source files
        let walk_result = walk_directory(project_root).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to walk directory: {}", e),
            ))
        })?;

        let file_count = walk_result.file_count();

        // Collect language stats
        let by_lang = walk_result.by_language();
        let mut lang_summary: Vec<(String, usize)> = by_lang
            .iter()
            .map(|(lang, files)| (lang.name().to_string(), files.len()))
            .collect();
        lang_summary.sort_by(|a, b| b.1.cmp(&a.1));

        // Step 2: Parse and extract symbols
        let symbols_by_file: BTreeMap<PathBuf, Vec<Symbol>>;
        let mut parse_errors = 0;
        let mut incremental_stats: Option<(usize, usize, usize)> = None;

        if structure_only {
            symbols_by_file = BTreeMap::new();
        } else if incremental {
            let manifest = HashManifest::load(&hash_path).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to load hash manifest: {}", e),
                ))
            })?;
            let diff = manifest.diff(&walk_result);

            if diff.changed_count() == 0 && diff.deleted_count() == 0 {
                // No changes — load cached symbols
                let cache = SymbolCache::load(&symbol_cache_path).map_err(|e| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to load symbol cache: {}", e),
                    ))
                })?;
                incremental_stats = Some((0, diff.unchanged_count(), 0));
                symbols_by_file = cache.to_path_map();
            } else {
                incremental_stats = Some((
                    diff.changed_count(),
                    diff.unchanged_count(),
                    diff.deleted_count(),
                ));

                let mut symbol_cache = SymbolCache::load(&symbol_cache_path).map_err(|e| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to load symbol cache: {}", e),
                    ))
                })?;

                // Parse only changed files
                let mut parser = Parser::new();
                let mut new_symbols: BTreeMap<PathBuf, Vec<Symbol>> = BTreeMap::new();

                for file in &diff.changed {
                    let rel_path_str = file.relative_path.to_string_lossy().to_string();
                    match parser.parse_file(&file.path) {
                        Ok(parsed) => {
                            match extract_symbols_for_language(
                                file.language,
                                &parsed,
                                &rel_path_str,
                            ) {
                                Ok(symbols) if !symbols.is_empty() => {
                                    new_symbols.insert(file.relative_path.clone(), symbols);
                                }
                                Err(_) => parse_errors += 1,
                                _ => {}
                            }
                        }
                        Err(_) => parse_errors += 1,
                    }
                }

                symbol_cache.update(&new_symbols, &diff.deleted);

                let mut updated_manifest = manifest;
                updated_manifest.update(&diff);
                updated_manifest.save(&hash_path).map_err(|e| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to save hash manifest: {}", e),
                    ))
                })?;
                symbol_cache.save(&symbol_cache_path).map_err(|e| {
                    CallToolError::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to save symbol cache: {}", e),
                    ))
                })?;

                symbols_by_file = symbol_cache.to_path_map();
            }
        } else {
            // Full index
            let mut parser = Parser::new();
            let mut extracted: BTreeMap<PathBuf, Vec<Symbol>> = BTreeMap::new();

            for file in &walk_result.files {
                let rel_path_str = file.relative_path.to_string_lossy().to_string();
                match parser.parse_file(&file.path) {
                    Ok(parsed) => {
                        match extract_symbols_for_language(file.language, &parsed, &rel_path_str) {
                            Ok(symbols) if !symbols.is_empty() => {
                                extracted.insert(file.relative_path.clone(), symbols);
                            }
                            Err(_) => parse_errors += 1,
                            _ => {}
                        }
                    }
                    Err(_) => parse_errors += 1,
                }
            }

            // Save hash manifest and symbol cache for future incremental runs
            let manifest = HashManifest::from_walk_result(&walk_result);
            manifest.save(&hash_path).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to save hash manifest: {}", e),
                ))
            })?;
            let cache = SymbolCache::from_path_map(&extracted);
            cache.save(&symbol_cache_path).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to save symbol cache: {}", e),
                ))
            })?;

            symbols_by_file = extracted;
        }

        // Step 3: Write the index (preserving existing summaries)
        let output_path = metis_dir.join("code-index.md");
        let existing_content = std::fs::read_to_string(&output_path).ok();
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let content = format_index(
            &walk_result,
            &symbols_by_file,
            &timestamp,
            existing_content.as_deref(),
        );
        std::fs::write(&output_path, content).map_err(|e| {
            CallToolError::new(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to write code-index.md: {}", e),
            ))
        })?;

        let elapsed = start.elapsed();
        let symbol_count: usize = symbols_by_file.values().map(|v| v.len()).sum();

        // Build response
        let mut output = ToolOutput::new().header("Code Index Generated");

        let mut stats = vec![
            vec!["Files indexed".to_string(), file_count.to_string()],
            vec!["Symbols extracted".to_string(), symbol_count.to_string()],
            vec!["Time".to_string(), format!("{:.1}s", elapsed.as_secs_f64())],
            vec!["Output".to_string(), output_path.display().to_string()],
        ];

        if parse_errors > 0 {
            stats.push(vec!["Parse errors".to_string(), parse_errors.to_string()]);
        }

        if structure_only {
            stats.push(vec![
                "Mode".to_string(),
                "structure-only (symbols skipped)".to_string(),
            ]);
        }

        if let Some((changed, unchanged, deleted)) = incremental_stats {
            stats.push(vec!["Mode".to_string(), "incremental".to_string()]);
            stats.push(vec!["Files re-indexed".to_string(), changed.to_string()]);
            stats.push(vec!["Files skipped".to_string(), unchanged.to_string()]);
            if deleted > 0 {
                stats.push(vec!["Files removed".to_string(), deleted.to_string()]);
            }
        }

        output = output.table(&["Metric", "Value"], stats);

        if !lang_summary.is_empty() {
            output = output.subheader("Languages Detected");
            let lang_rows: Vec<Vec<String>> = lang_summary
                .iter()
                .map(|(name, count)| vec![name.clone(), count.to_string()])
                .collect();
            output = output.table(&["Language", "Files"], lang_rows);
        }

        Ok(output.build_result())
    }
}

/// Dispatch symbol extraction to the appropriate language extractor.
fn extract_symbols_for_language(
    language: Language,
    parsed: &ParsedFile,
    file_path: &str,
) -> Result<Vec<Symbol>, String> {
    let tree = &parsed.tree;
    let source = &parsed.source;
    match language {
        Language::Rust => RustExtractor::extract_symbols(tree, source, file_path),
        Language::Python => PythonExtractor::extract_symbols(tree, source, file_path),
        Language::TypeScript => {
            TypeScriptExtractor::extract_symbols(tree, source, file_path, language)
        }
        Language::JavaScript => {
            TypeScriptExtractor::extract_symbols(tree, source, file_path, language)
        }
        Language::Go => GoExtractor::extract_symbols(tree, source, file_path),
    }
}
