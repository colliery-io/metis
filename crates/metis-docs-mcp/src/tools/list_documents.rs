use crate::formatting::ToolOutput;
use metis_core::application::services::workspace::WorkspaceDetectionService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[mcp_tool(
    name = "list_documents",
    description = "List documents in a project with optional filtering. Returns document details including unique short codes (format: PREFIX-TYPE-NNNN).",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsTool {
    /// Path to the .metis folder to list documents from
    pub project_path: String,
}

impl ListDocumentsTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let metis_dir = Path::new(&self.project_path);

        // Prepare workspace (validates, creates/updates database, syncs)
        let detection_service = WorkspaceDetectionService::new();
        let db = detection_service
            .prepare_workspace(metis_dir)
            .await
            .map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            })?;

        let mut repo = db.into_repository();

        // List all documents
        let documents = self.list_all_documents(&mut repo)?;
        let total_count = documents.len();

        // Group documents by type
        let mut by_type: HashMap<String, Vec<_>> = HashMap::new();
        for doc in documents {
            by_type
                .entry(doc.document_type.clone())
                .or_default()
                .push(doc);
        }

        // Build formatted output
        let mut output = ToolOutput::new().header(&format!("Documents ({} total)", total_count));

        // Order: vision, strategy, initiative, task, adr
        let type_order = ["vision", "strategy", "initiative", "task", "adr"];
        let type_labels = [
            ("vision", "Vision"),
            ("strategy", "Strategies"),
            ("initiative", "Initiatives"),
            ("task", "Tasks"),
            ("adr", "ADRs"),
        ]
        .into_iter()
        .collect::<HashMap<_, _>>();

        for doc_type in type_order {
            if let Some(docs) = by_type.get(doc_type) {
                if !docs.is_empty() {
                    let label = type_labels.get(doc_type).unwrap_or(&doc_type);
                    output = output.subheader(label);

                    let rows: Vec<Vec<String>> = docs
                        .iter()
                        .map(|doc| {
                            vec![
                                doc.short_code.clone(),
                                doc.title.clone(),
                                doc.phase.clone(),
                            ]
                        })
                        .collect();

                    output = output.table(&["Code", "Title", "Phase"], rows);
                }
            }
        }

        if total_count == 0 {
            output = output.text("No documents found.");
        }

        Ok(CallToolResult::text_content(vec![TextContent::from(
            output.build(),
        )]))
    }

    fn list_all_documents(
        &self,
        repo: &mut metis_core::dal::database::repository::DocumentRepository,
    ) -> Result<Vec<metis_core::dal::database::models::Document>, CallToolError> {
        let mut all_docs = Vec::new();

        // Collect all document types
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            let mut docs = repo.find_by_type(doc_type).map_err(|e| {
                CallToolError::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to query {} documents: {}", doc_type, e),
                ))
            })?;
            all_docs.append(&mut docs);
        }

        // Sort by updated_at descending
        all_docs.sort_by(|a, b| {
            b.updated_at
                .partial_cmp(&a.updated_at)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(all_docs)
    }
}
