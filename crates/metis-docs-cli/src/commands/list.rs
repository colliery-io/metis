use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{Application, Database, Result as MetisResult};

#[derive(Args)]
pub struct ListCommand {
    /// Document type to filter by (vision, strategy, initiative, task, adr)
    #[arg(short = 't', long)]
    pub document_type: Option<String>,

    /// Phase to filter by (draft, active, completed, etc.)
    #[arg(short = 'p', long)]
    pub phase: Option<String>,

    /// Show all documents regardless of type
    #[arg(short = 'a', long)]
    pub all: bool,

    /// Include archived documents in the list
    #[arg(long)]
    pub include_archived: bool,
}

impl ListCommand {
    pub async fn execute(&self) -> Result<()> {
        // 1. Validate we're in a metis workspace
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        // 2. Sync before reading to catch external edits
        let db_path = metis_dir.join("metis.db");
        let database = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Failed to open database for sync: {}", e))?;
        let app = Application::new(database);
        app.sync_directory(&metis_dir)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to sync workspace: {}", e))?;

        // 3. Connect to database
        let db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Database connection failed: {}", e))?;
        let mut repo = db.into_repository();

        // 4. Query documents based on filters
        let documents = if self.all {
            // Show all documents
            self.list_all_documents(&mut repo).await?
        } else if let Some(doc_type) = &self.document_type {
            if let Some(phase) = &self.phase {
                // Filter by both type and phase
                repo.find_by_type_and_phase(doc_type, phase)
                    .map_err(|e| anyhow::anyhow!("Database query failed: {}", e))?
            } else {
                // Filter by type only
                repo.find_by_type(doc_type)
                    .map_err(|e| anyhow::anyhow!("Database query failed: {}", e))?
            }
        } else if let Some(phase) = &self.phase {
            // Filter by phase only
            repo.find_by_phase(phase)
                .map_err(|e| anyhow::anyhow!("Database query failed: {}", e))?
        } else {
            // Default: show all documents
            self.list_all_documents(&mut repo).await?
        };

        // 5. Display results
        if documents.is_empty() {
            println!("No documents found matching the criteria.");
            return Ok(());
        }

        self.display_documents(&documents);

        Ok(())
    }

    async fn list_all_documents(
        &self,
        repo: &mut metis_core::dal::database::repository::DocumentRepository,
    ) -> MetisResult<Vec<metis_core::dal::database::models::Document>> {
        // For listing all documents, we can query each type
        let mut all_docs = Vec::new();

        // Collect all document types
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            let mut docs = repo.find_by_type(doc_type)?;
            all_docs.append(&mut docs);
        }

        // Filter out archived documents unless requested
        if !self.include_archived {
            all_docs.retain(|doc| !doc.archived);
        }

        // Sort by updated_at descending
        all_docs.sort_by(|a, b| {
            b.updated_at
                .partial_cmp(&a.updated_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_docs)
    }

    fn display_documents(&self, documents: &[metis_core::dal::database::models::Document]) {
        println!(
            "\n{:<15} {:<30} {:<15} {:<15} {:<20}",
            "TYPE", "TITLE", "PHASE", "ID", "UPDATED"
        );
        println!("{}", "-".repeat(95));

        for doc in documents {
            let updated = chrono::DateTime::from_timestamp(doc.updated_at as i64, 0)
                .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_else(|| "Unknown".to_string());

            println!(
                "{:<15} {:<30} {:<15} {:<15} {:<20}",
                doc.document_type,
                self.truncate_string(&doc.title, 28),
                doc.phase,
                self.truncate_string(&doc.id, 13),
                updated
            );
        }

        println!("\nTotal: {} documents", documents.len());
    }

    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}...", &s[..max_len.saturating_sub(3)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_list_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return; // Skip test if we can't change directory
        }

        let cmd = ListCommand {
            document_type: None,
            phase: None,
            all: false,
            include_archived: false,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Not in a Metis workspace"));
    }

    #[tokio::test]
    async fn test_list_command_empty_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory
        std::env::set_current_dir(temp_dir.path()).unwrap();

        // Create workspace
        let init_cmd = InitCommand {
            name: Some("Test Project".to_string()),
            preset: None,
            strategies: None,
            initiatives: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let cmd = ListCommand {
            document_type: None,
            phase: None,
            all: true,
            include_archived: false,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        // Should succeed but show no documents (except the vision.md created by init)
        assert!(result.is_ok());
    }
}
