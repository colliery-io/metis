use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::dal::database::models::Document;
use metis_core::{Application, Database};

#[derive(Args)]
pub struct SearchCommand {
    /// Search query for full-text search across document content
    pub query: String,

    /// Maximum number of results to show
    #[arg(short = 'l', long, default_value = "20")]
    pub limit: usize,
}

impl SearchCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Initialize the database and application
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database: {}", e))?;
        let mut app = Application::new(database);

        // 3. Perform full-text search
        let results = self.perform_search(&mut app, &self.query)?;

        // 4. Limit results
        let limited_results: Vec<_> = results.into_iter().take(self.limit).collect();

        // 5. Display results
        self.display_results(&limited_results)?;

        Ok(())
    }

    fn perform_search(&self, app: &mut Application, query: &str) -> Result<Vec<Document>> {
        app.with_database(|db_service| db_service.search_documents(query))
            .map_err(|e| anyhow::anyhow!("Search failed: {}", e))
    }

    fn display_results(&self, documents: &[Document]) -> Result<()> {
        if documents.is_empty() {
            println!("No documents found for query: \"{}\"", self.query);
            return Ok(());
        }

        self.display_table(documents)?;
        Ok(())
    }

    fn display_table(&self, documents: &[Document]) -> Result<()> {
        // Print header
        println!("{:<50} {:<12} {:<120}", "ID", "Type", "Path");
        println!("{}", "-".repeat(182));

        for doc in documents {
            let id = truncate(&doc.id, 49);
            let doc_type = truncate(&doc.document_type, 11);

            // Extract relative path from .metis directory
            let relative_path = if let Some(metis_pos) = doc.filepath.find(".metis/") {
                &doc.filepath[metis_pos + 7..] // Skip ".metis/"
            } else {
                &doc.filepath
            };
            let path = truncate(relative_path, 119);

            println!("{:<50} {:<12} {:<120}", id, doc_type, path);
        }

        println!(
            "\nFound {} document(s) for \"{}\"",
            documents.len(),
            self.query
        );
        Ok(())
    }
}

// Helper functions
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
