//! `metis index` subcommand — generate `.metis/code-index.md`.
//!
//! Orchestrates the full code-indexing pipeline: walk source files,
//! parse with tree-sitter, extract symbols, and write the markdown index.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use anyhow::Result;
use clap::Args;

use metis_code_index::parser::{Language, ParsedFile, Parser};
use metis_code_index::symbols::Symbol;
use metis_code_index::walker::walk_directory;
use metis_code_index::{
    format_index, GoExtractor, PythonExtractor, RustExtractor, TypeScriptExtractor,
};

use crate::workspace;

#[derive(Args)]
pub struct IndexCommand {
    /// Only generate the project structure tree, skip symbol extraction
    #[arg(long)]
    pub structure_only: bool,

    /// Enable incremental indexing (only re-index changed files)
    #[arg(long)]
    pub incremental: bool,
}

impl IndexCommand {
    pub async fn execute(&self) -> Result<()> {
        let start = Instant::now();

        // Validate workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }

        let metis_dir = metis_dir.unwrap();
        let project_root = metis_dir.parent().ok_or_else(|| {
            anyhow::anyhow!("Cannot determine project root from .metis directory")
        })?;

        if self.incremental {
            println!("Note: --incremental is accepted but not yet implemented (full index will be generated).");
        }

        // Step 1: Walk source files
        println!("Scanning source files...");
        let walk_result = walk_directory(project_root)
            .map_err(|e| anyhow::anyhow!("Failed to walk directory: {}", e))?;

        println!("  Found {} source files", walk_result.file_count());

        // Report languages
        let by_lang = walk_result.by_language();
        let mut lang_summary: Vec<(&str, usize)> = by_lang
            .iter()
            .map(|(lang, files)| (lang.name(), files.len()))
            .collect();
        lang_summary.sort_by(|a, b| b.1.cmp(&a.1));
        for (name, count) in &lang_summary {
            println!("    {}: {} files", name, count);
        }

        // Step 2: Parse and extract symbols
        let mut symbols_by_file: BTreeMap<PathBuf, Vec<Symbol>> = BTreeMap::new();

        if !self.structure_only {
            println!("Extracting symbols...");
            let mut parser = Parser::new();
            let mut errors = 0;

            for file in &walk_result.files {
                let rel_path_str = file.relative_path.to_string_lossy().to_string();

                match parser.parse_file(&file.path) {
                    Ok(parsed) => {
                        match extract_symbols_for_language(file.language, &parsed, &rel_path_str) {
                            Ok(symbols) if !symbols.is_empty() => {
                                symbols_by_file.insert(file.relative_path.clone(), symbols);
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "Failed to extract symbols from {}: {}",
                                    rel_path_str,
                                    e
                                );
                                errors += 1;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        tracing::warn!("Failed to parse {}: {}", rel_path_str, e);
                        errors += 1;
                    }
                }
            }

            let symbol_count: usize = symbols_by_file.values().map(|v| v.len()).sum();
            println!(
                "  Extracted {} symbols from {} files",
                symbol_count,
                symbols_by_file.len()
            );
            if errors > 0 {
                println!("  {} files had parse errors (skipped)", errors);
            }
        } else {
            println!("Skipping symbol extraction (--structure-only)");
        }

        // Step 3: Generate and write the index
        let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let content = format_index(&walk_result, &symbols_by_file, &timestamp);

        let output_path = metis_dir.join("code-index.md");
        std::fs::write(&output_path, content)?;

        let elapsed = start.elapsed();
        println!(
            "\nIndex written to {} ({:.1}s)",
            output_path.display(),
            elapsed.as_secs_f64()
        );

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use std::fs;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_index_no_workspace() {
        let tmp = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(tmp.path()).unwrap();

        let cmd = IndexCommand {
            structure_only: false,
            incremental: false,
        };
        let result = cmd.execute().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_index_generates_file() {
        let tmp = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(tmp.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Index Test".to_string()),
            prefix: None,
            preset: None,
            strategies: None,
            initiatives: None,
        };
        init_cmd.execute().await.unwrap();

        // Create a sample source file
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(
            src_dir.join("main.rs"),
            "pub fn main() {\n    println!(\"hello\");\n}\n\npub struct Config {\n    pub name: String,\n}\n",
        )
        .unwrap();

        let cmd = IndexCommand {
            structure_only: false,
            incremental: false,
        };
        cmd.execute().await.unwrap();

        let index_path = tmp.path().join(".metis/code-index.md");
        assert!(index_path.exists(), "code-index.md should be created");

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("# Code Index"));
        assert!(content.contains("## Project Structure"));
        assert!(content.contains("## Key Symbols"));
        assert!(content.contains("main.rs"));

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_index_structure_only() {
        let tmp = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(tmp.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Structure Only Test".to_string()),
            prefix: None,
            preset: None,
            strategies: None,
            initiatives: None,
        };
        init_cmd.execute().await.unwrap();

        // Create a source file
        let src_dir = tmp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();
        fs::write(src_dir.join("lib.rs"), "pub fn hello() {}\n").unwrap();

        let cmd = IndexCommand {
            structure_only: true,
            incremental: false,
        };
        cmd.execute().await.unwrap();

        let index_path = tmp.path().join(".metis/code-index.md");
        assert!(index_path.exists());

        let content = fs::read_to_string(&index_path).unwrap();
        assert!(content.contains("## Project Structure"));
        // With structure_only, Key Symbols section exists but should have no file entries
        assert!(content.contains("## Key Symbols"));
        assert!(
            !content.contains("### src/lib.rs"),
            "structure-only should not include symbol sections per file"
        );

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }

    #[tokio::test]
    async fn test_index_incremental_flag_accepted() {
        let tmp = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();
        std::env::set_current_dir(tmp.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Incremental Test".to_string()),
            prefix: None,
            preset: None,
            strategies: None,
            initiatives: None,
        };
        init_cmd.execute().await.unwrap();

        let cmd = IndexCommand {
            structure_only: false,
            incremental: true,
        };
        // Should succeed (incremental is a no-op for now)
        cmd.execute().await.unwrap();

        let index_path = tmp.path().join(".metis/code-index.md");
        assert!(index_path.exists());

        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }
    }
}
