use crate::commands::list::OutputFormat;
use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::dal::database::models::Document;
use metis_core::{Application, Database};
use serde::Serialize;

#[derive(Args)]
pub struct SearchCommand {
    /// Search query for full-text search across document content
    pub query: String,

    /// Maximum number of results to show
    #[arg(short = 'l', long, default_value = "20")]
    pub limit: usize,

    /// Output format (table, compact, json)
    #[arg(short = 'f', long, value_enum, default_value = "table")]
    pub format: OutputFormat,
}

/// JSON-serializable search result for output
#[derive(Serialize)]
struct SearchResultOutput {
    code: String,
    title: String,
    #[serde(rename = "type")]
    doc_type: String,
}

impl SearchCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Sync before searching to catch external edits
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database for sync: {}", e))?;
        let app = Application::new(database);
        app.sync_directory(&metis_dir)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync workspace: {}", e))?;

        // 3. Initialize the database and application for search
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let mut app = Application::new(database);

        // 4. Perform full-text search
        let results = self.perform_search(&mut app, &self.query)?;

        // 5. Limit results
        let limited_results: Vec<_> = results.into_iter().take(self.limit).collect();

        // 6. Display results based on format
        if limited_results.is_empty() {
            match self.format {
                OutputFormat::Json => println!("[]"),
                _ => println!("No documents found for query: \"{}\"", self.query),
            }
            return Ok(());
        }

        match self.format {
            OutputFormat::Table => self.display_table(&limited_results),
            OutputFormat::Compact => self.display_compact(&limited_results),
            OutputFormat::Json => self.display_json(&limited_results),
        }

        Ok(())
    }

    fn perform_search(&self, app: &mut Application, query: &str) -> Result<Vec<Document>> {
        app.with_database(|db_service| db_service.search_documents(query))
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
    }

    /// Display results as a human-readable table
    /// Columns match MCP search_documents: Code, Title, Type
    fn display_table(&self, documents: &[Document]) {
        println!("\n{:<14} {:<60} {:<12}", "Code", "Title", "Type");
        println!("{}", "-".repeat(88));

        for doc in documents {
            println!(
                "{:<14} {:<60} {:<12}",
                doc.short_code,
                truncate(&doc.title, 58),
                doc.document_type
            );
        }

        println!(
            "\nFound {} document(s) for \"{}\"",
            documents.len(),
            self.query
        );
    }

    /// Display results in compact format (one line per document)
    /// Format: CODE TYPE TITLE
    fn display_compact(&self, documents: &[Document]) {
        for doc in documents {
            println!("{} {} {}", doc.short_code, doc.document_type, doc.title);
        }
    }

    /// Display results as JSON array
    fn display_json(&self, documents: &[Document]) {
        let output: Vec<SearchResultOutput> = documents
            .iter()
            .map(|doc| SearchResultOutput {
                code: doc.short_code.clone(),
                title: doc.title.clone(),
                doc_type: doc.document_type.clone(),
            })
            .collect();

        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }
}

// Helper function
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("short", 10), "short");
        assert_eq!(truncate("this is a very long string", 10), "this is...");
        assert_eq!(truncate("exactly_10", 10), "exactly_10");
    }
}
