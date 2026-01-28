use crate::formatting::ToolOutput;
use metis_core::application::services::workspace::WorkspaceDetectionService;
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult},
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
    /// Path to the .metis folder (e.g., "/Users/me/my-project/.metis"). Must end with .metis
    pub project_path: String,
    /// Include archived documents in results (defaults to false)
    #[serde(default)]
    pub include_archived: Option<bool>,
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

        // List all documents (respecting include_archived flag, defaults to false)
        let include_archived = self.include_archived.unwrap_or(false);
        let mut documents = self.list_all_documents(&mut repo, include_archived)?;
        let total_count = documents.len();

        // Build formatted output
        let mut output = ToolOutput::new().header(&format!("Documents ({} total)", total_count));

        if total_count == 0 {
            output = output.text("No documents found.");
        } else {
            // Sort by type order, then by short_code
            let type_order_map: HashMap<&str, usize> = [
                ("vision", 0),
                ("strategy", 1),
                ("initiative", 2),
                ("task", 3),
                ("adr", 4),
            ]
            .into_iter()
            .collect();

            documents.sort_by(|a, b| {
                let a_order = type_order_map.get(a.document_type.as_str()).unwrap_or(&999);
                let b_order = type_order_map.get(b.document_type.as_str()).unwrap_or(&999);
                a_order
                    .cmp(b_order)
                    .then_with(|| a.short_code.cmp(&b.short_code))
            });

            // Build single table with all documents
            let rows: Vec<Vec<String>> = documents
                .iter()
                .map(|doc| {
                    vec![
                        doc.document_type.clone(),
                        doc.short_code.clone(),
                        doc.title.clone(),
                        doc.phase.clone(),
                    ]
                })
                .collect();

            output = output.table(&["Type", "Code", "Title", "Phase"], rows);
        }

        Ok(output.build_result())
    }

    fn list_all_documents(
        &self,
        repo: &mut metis_core::dal::database::repository::DocumentRepository,
        include_archived: bool,
    ) -> Result<Vec<metis_core::dal::database::models::Document>, CallToolError> {
        let mut all_docs = Vec::new();

        // Collect all document types
        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
            let mut docs = if include_archived {
                repo.find_by_type(doc_type)
            } else {
                repo.find_by_type_unarchived(doc_type)
            }
            .map_err(|e| {
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
