use crate::commands::list::OutputFormat;
use crate::workspace;
use anyhow::Result;
use clap::Args;
use metis_core::{Application, Database, Result as MetisResult};
use serde::Serialize;

#[derive(Args)]
pub struct StatusCommand {
    /// Include archived documents in the status view
    #[arg(long)]
    pub include_archived: bool,

    /// Output format (table, compact, json)
    #[arg(short = 'f', long, value_enum, default_value = "table")]
    pub format: OutputFormat,
}

/// JSON-serializable status row for output
#[derive(Serialize)]
struct StatusOutput {
    code: String,
    title: String,
    #[serde(rename = "type")]
    doc_type: String,
    phase: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    blocked_by: String,
    updated: String,
}

impl StatusCommand {
    // Helper methods to reduce complexity

    /// Get all document types to query
    fn get_document_types() -> &'static [&'static str] {
        &["vision", "strategy", "initiative", "task", "adr"]
    }

    /// Initialize database connection from workspace
    async fn connect_to_database(
    ) -> Result<metis_core::dal::database::repository::DocumentRepository> {
        let (workspace_exists, metis_dir) = workspace::has_metis_vault();
        if !workspace_exists {
            anyhow::bail!("Not in a Metis workspace. Run 'metis init' to create one.");
        }
        let metis_dir = metis_dir.unwrap();

        let db_path = metis_dir.join("metis.db");
        let db = Database::new(db_path.to_str().unwrap())
            .map_err(|e| anyhow::anyhow!("Database connection failed: {}", e))?;
        Ok(db.into_repository())
    }

    /// Fetch and filter documents from repository
    async fn fetch_documents(
        &self,
        repo: &mut metis_core::dal::database::repository::DocumentRepository,
    ) -> MetisResult<Vec<metis_core::dal::database::models::Document>> {
        let mut all_docs = Vec::new();

        // Collect all documents
        for doc_type in Self::get_document_types() {
            let mut docs = repo.find_by_type(doc_type)?;
            all_docs.append(&mut docs);
        }

        // Filter archived if needed
        if !self.include_archived {
            all_docs.retain(|doc| !doc.archived);
        }

        Ok(all_docs)
    }

    /// Sort documents by actionability and recency
    fn sort_documents_by_priority(&self, docs: &mut [metis_core::dal::database::models::Document]) {
        docs.sort_by(|a, b| {
            let a_priority = self.get_action_priority(a);
            let b_priority = self.get_action_priority(b);

            match a_priority.cmp(&b_priority) {
                std::cmp::Ordering::Equal => {
                    // If same priority, sort by most recently updated
                    b.updated_at
                        .partial_cmp(&a.updated_at)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                other => other,
            }
        });
    }

    /// Count documents by phase for insights
    fn count_documents_by_phase(
        &self,
        documents: &[metis_core::dal::database::models::Document],
    ) -> (usize, usize, usize) {
        let blocked_count = documents.iter().filter(|d| d.phase == "blocked").count();
        let todo_count = documents.iter().filter(|d| d.phase == "todo").count();
        let active_count = documents.iter().filter(|d| d.phase == "active").count();
        (blocked_count, todo_count, active_count)
    }

    pub async fn execute(&self) -> Result<()> {
        // 1. Validate workspace and sync before reading
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
        let mut repo = Self::connect_to_database().await?;

        // 4. Fetch and sort documents
        let mut documents = self.fetch_documents(&mut repo).await?;
        self.sort_documents_by_priority(&mut documents);

        // 5. Display results
        if documents.is_empty() {
            match self.format {
                OutputFormat::Json => println!("[]"),
                _ => println!("No documents found in workspace."),
            }
            return Ok(());
        }

        match self.format {
            OutputFormat::Table => self.display_table(&documents),
            OutputFormat::Compact => self.display_compact(&documents),
            OutputFormat::Json => self.display_json(&documents),
        }
        Ok(())
    }

    fn get_action_priority(&self, doc: &metis_core::dal::database::models::Document) -> u8 {
        // Lower numbers = higher priority (more actionable)
        match doc.phase.as_str() {
            "blocked" => 0,                          // Most urgent - things blocking other work
            "todo" => 1,                             // Ready to start
            "discussion" => 2,                       // Needs decision
            "active" => 3,                           // Currently being worked on
            "discovery" | "shaping" | "design" => 4, // Needs planning/refinement
            "ready" | "decompose" => 5,              // Staged for work
            "review" => 6,                           // Waiting for review
            "decided" | "published" | "completed" => 7, // Done but recent
            _ => 8,                                  // Other states
        }
    }

    /// Display status as a human-readable table
    fn display_table(&self, documents: &[metis_core::dal::database::models::Document]) {
        println!("\nWORKSPACE STATUS\n");

        println!(
            "{:<14} {:<35} {:<12} {:<12} {:<18} {:<12}",
            "Code", "Title", "Type", "Phase", "Blocked By", "Updated"
        );
        println!("{}", "-".repeat(105));

        for doc in documents {
            println!(
                "{:<14} {:<35} {:<12} {:<12} {:<18} {:<12}",
                doc.short_code,
                self.truncate_string(&doc.title, 33),
                doc.document_type,
                doc.phase,
                self.extract_blocked_by_info(doc),
                chrono::DateTime::from_timestamp(doc.updated_at as i64, 0)
                    .map(|dt| self.format_relative_time(dt))
                    .unwrap_or_else(|| "Unknown".to_string())
            );
        }

        println!("\nTotal: {} documents", documents.len());

        // Summary insights
        self.display_insights(documents);
    }

    /// Display status in compact format (one line per document)
    fn display_compact(&self, documents: &[metis_core::dal::database::models::Document]) {
        for doc in documents {
            let blocked_by = self.extract_blocked_by_info(doc);
            if blocked_by.is_empty() {
                println!("{} {} {}", doc.short_code, doc.phase, doc.title);
            } else {
                println!(
                    "{} {} {} [blocked by: {}]",
                    doc.short_code, doc.phase, doc.title, blocked_by
                );
            }
        }
    }

    /// Display status as JSON array
    fn display_json(&self, documents: &[metis_core::dal::database::models::Document]) {
        let output: Vec<StatusOutput> = documents
            .iter()
            .map(|doc| StatusOutput {
                code: doc.short_code.clone(),
                title: doc.title.clone(),
                doc_type: doc.document_type.clone(),
                phase: doc.phase.clone(),
                blocked_by: self.extract_blocked_by_info(doc),
                updated: chrono::DateTime::from_timestamp(doc.updated_at as i64, 0)
                    .map(|dt| self.format_relative_time(dt))
                    .unwrap_or_else(|| "Unknown".to_string()),
            })
            .collect();

        match serde_json::to_string_pretty(&output) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }

    fn extract_blocked_by_info(&self, doc: &metis_core::dal::database::models::Document) -> String {
        if doc.phase != "blocked" {
            return String::new();
        }

        // Parse frontmatter JSON to get blocked_by information
        if let Ok(frontmatter) = serde_json::from_str::<serde_json::Value>(&doc.frontmatter_json) {
            if let Some(blocked_by) = frontmatter.get("blocked_by") {
                if let Some(array) = blocked_by.as_array() {
                    let blocking_docs: Vec<String> = array
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| s.to_string())
                        .collect();

                    if !blocking_docs.is_empty() {
                        return self.truncate_string(&blocking_docs.join(", "), 18);
                    }
                }
            }
        }

        "Unknown".to_string()
    }

    fn format_relative_time(&self, dt: chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(dt);

        if diff.num_days() > 0 {
            if diff.num_days() == 1 {
                "1 day ago".to_string()
            } else if diff.num_days() < 7 {
                format!("{} days ago", diff.num_days())
            } else if diff.num_days() < 30 {
                format!("{} weeks ago", diff.num_days() / 7)
            } else {
                format!("{} months ago", diff.num_days() / 30)
            }
        } else if diff.num_hours() > 0 {
            if diff.num_hours() == 1 {
                "1 hour ago".to_string()
            } else {
                format!("{} hours ago", diff.num_hours())
            }
        } else if diff.num_minutes() > 0 {
            if diff.num_minutes() == 1 {
                "1 minute ago".to_string()
            } else {
                format!("{} minutes ago", diff.num_minutes())
            }
        } else {
            "Just now".to_string()
        }
    }

    fn display_insights(&self, documents: &[metis_core::dal::database::models::Document]) {
        let (blocked_count, todo_count, active_count) = self.count_documents_by_phase(documents);

        if blocked_count > 0 || todo_count > 0 {
            println!("ACTIONABLE ITEMS:");
            if blocked_count > 0 {
                println!("  [!] {} blocked documents need unblocking", blocked_count);
            }
            if todo_count > 0 {
                println!("  [*] {} documents ready to start", todo_count);
            }
            if active_count > 0 {
                println!("  [~] {} documents in progress", active_count);
            }
        }
    }

    fn truncate_string(&self, s: &str, max_len: usize) -> String {
        if s.len() <= max_len {
            s.to_string()
        } else {
            format!("{}…", &s[..max_len.saturating_sub(1)])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::InitCommand;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_status_command_no_workspace() {
        let temp_dir = tempdir().unwrap();
        let original_dir = std::env::current_dir().ok();

        // Change to temp directory without workspace
        if std::env::set_current_dir(temp_dir.path()).is_err() {
            return; // Skip test if we can't change directory
        }

        let cmd = StatusCommand {
            include_archived: false,
            format: OutputFormat::Table,
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
    async fn test_status_command_empty_workspace() {
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
            upstream: None,
            workspace_prefix: None,
            team: None,
            prefix: None,
        };
        init_cmd.execute().await.unwrap();

        let cmd = StatusCommand {
            include_archived: false,
            format: OutputFormat::Table,
        };

        let result = cmd.execute().await;

        // Always restore original directory first
        if let Some(original) = original_dir {
            let _ = std::env::set_current_dir(&original);
        }

        // Should succeed and show at least the vision document created by init
        assert!(result.is_ok());
    }

    #[test]
    fn test_action_priority() {
        let cmd = StatusCommand {
            include_archived: false,
            format: OutputFormat::Table,
        };

        // Create mock documents with different phases
        let blocked_doc = metis_core::dal::database::models::Document {
            filepath: "/test.md".to_string(),
            id: "test-1".to_string(),
            title: "Test".to_string(),
            document_type: "task".to_string(),
            created_at: 0.0,
            updated_at: 0.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "hash".to_string(),
            frontmatter_json: "{}".to_string(),
            content: None,
            phase: "blocked".to_string(),
            strategy_id: Some("test-strategy".to_string()),
            initiative_id: Some("test-initiative".to_string()),
            short_code: "TEST-T-0001".to_string(),
        };

        let todo_doc = metis_core::dal::database::models::Document {
            phase: "todo".to_string(),
            ..blocked_doc.clone()
        };

        let completed_doc = metis_core::dal::database::models::Document {
            phase: "completed".to_string(),
            ..blocked_doc.clone()
        };

        // Blocked should have highest priority (lowest number)
        assert!(cmd.get_action_priority(&blocked_doc) < cmd.get_action_priority(&todo_doc));
        assert!(cmd.get_action_priority(&todo_doc) < cmd.get_action_priority(&completed_doc));
    }
}
