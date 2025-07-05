use chrono::{DateTime, Utc};
use metis_core::{Document, DocumentStore, DocumentType, SearchResult};
use rust_mcp_sdk::{
    macros::{mcp_tool, JsonSchema},
    schema::{schema_utils::CallToolError, CallToolResult, TextContent},
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::str::FromStr;

/// Document summary without content field for list operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSummary {
    pub id: String,
    pub filepath: String,
    pub document_type: DocumentType,
    pub level: DocumentType,
    pub status: String,
    pub parent_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content_hash: String,
    pub frontmatter: serde_json::Value,
    pub exit_criteria_met: bool,
    pub file_size: Option<i64>,
    pub file_modified_at: Option<f64>,
}

impl From<Document> for DocumentSummary {
    fn from(doc: Document) -> Self {
        Self {
            id: doc.id,
            filepath: doc.filepath,
            document_type: doc.document_type,
            level: doc.level,
            status: doc.status,
            parent_id: doc.parent_id,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            content_hash: doc.content_hash,
            frontmatter: doc.frontmatter,
            exit_criteria_met: doc.exit_criteria_met,
            file_size: doc.file_size,
            file_modified_at: doc.file_modified_at,
        }
    }
}

/// Search result summary without content field for search operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultSummary {
    pub document: DocumentSummary,
    pub rank: f64,
    pub snippet: String,
}

impl From<SearchResult> for SearchResultSummary {
    fn from(result: SearchResult) -> Self {
        Self {
            document: DocumentSummary::from(result.document),
            rank: result.rank,
            snippet: result.snippet,
        }
    }
}

#[mcp_tool(
    name = "list_documents",
    description = "List documents in a project with optional filtering",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct ListDocumentsTool {
    /// Path to the .metis folder to list documents from
    pub project_path: String,

    /// Filter by document type (vision, strategy, initiative, task, adr)
    pub document_type: Option<String>,

    /// Filter by phase (draft, review, published, etc.)
    pub phase: Option<String>,

    /// Maximum number of results to return
    pub limit: Option<u32>,
}

impl ListDocumentsTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);
        let db_path = project_path.join(".metis.db");

        match DocumentStore::new(db_path.to_str().unwrap()).await {
            Ok(store) => {
                let query_service = store.query_service();

                // Parse document type filter if provided
                let doc_type_filter = if let Some(type_str) = &self.document_type {
                    match DocumentType::from_str(type_str) {
                        Ok(doc_type) => Some(doc_type),
                        Err(_) => return Ok(CallToolResult::text_content(
                            vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                                "error": format!("Invalid document type '{}'. Must be: vision, strategy, initiative, task, adr", type_str)
                            })).map_err(CallToolError::new)?)]
                        ))
                    }
                } else {
                    None
                };

                // Query documents
                let documents =
                    if let Some(doc_type) = doc_type_filter {
                        match query_service.find_documents_by_type(doc_type).await {
                            Ok(docs) => docs,
                            Err(e) => {
                                return Ok(CallToolResult::text_content(vec![TextContent::from(
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "error": format!("Failed to query documents: {}", e)
                                    }))
                                    .map_err(CallToolError::new)?,
                                )]))
                            }
                        }
                    } else if let Some(phase) = &self.phase {
                        match query_service.find_documents_by_phase(phase).await {
                        Ok(docs) => docs,
                        Err(e) => return Ok(CallToolResult::text_content(
                            vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                                "error": format!("Failed to query documents by phase: {}", e)
                            })).map_err(CallToolError::new)?)]
                        ))
                    }
                    } else {
                        // List all documents - use a generic query approach
                        match query_service
                            .find_documents_by_type(DocumentType::Vision)
                            .await
                        {
                            Ok(mut all_docs) => {
                                // Get documents of other types and combine
                                for doc_type in [
                                    DocumentType::Strategy,
                                    DocumentType::Initiative,
                                    DocumentType::Task,
                                    DocumentType::Adr,
                                ] {
                                    if let Ok(docs) =
                                        query_service.find_documents_by_type(doc_type).await
                                    {
                                        all_docs.extend(docs);
                                    }
                                }
                                all_docs
                            }
                            Err(e) => {
                                return Ok(CallToolResult::text_content(vec![TextContent::from(
                                    serde_json::to_string_pretty(&serde_json::json!({
                                        "error": format!("Failed to query documents: {}", e)
                                    }))
                                    .map_err(CallToolError::new)?,
                                )]))
                            }
                        }
                    };

                // Apply limit if specified and convert to summaries
                let limited_docs: Vec<DocumentSummary> = if let Some(limit) = self.limit {
                    documents
                        .into_iter()
                        .take(limit as usize)
                        .map(DocumentSummary::from)
                        .collect()
                } else {
                    documents.into_iter().map(DocumentSummary::from).collect()
                };

                let response = serde_json::json!({
                    "message": format!("Found {} documents", limited_docs.len()),
                    "documents": limited_docs,
                    "project_path": self.project_path
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                )]))
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to connect to database: {}", e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}

#[mcp_tool(
    name = "search_documents",
    description = "Search documents by content with optional filtering",
    idempotent_hint = true,
    destructive_hint = false,
    open_world_hint = false,
    read_only_hint = true
)]
#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SearchDocumentsTool {
    /// Path to the .metis folder to search documents in
    pub project_path: String,

    /// Search query to match against document content
    pub query: String,

    /// Filter by document type (vision, strategy, initiative, task, adr)
    pub document_type: Option<String>,

    /// Maximum number of results to return
    pub limit: Option<u32>,
}

impl SearchDocumentsTool {
    pub async fn call_tool(&self) -> std::result::Result<CallToolResult, CallToolError> {
        let project_path = PathBuf::from(&self.project_path);
        let db_path = project_path.join(".metis.db");

        match DocumentStore::new(db_path.to_str().unwrap()).await {
            Ok(store) => {
                let query_service = store.query_service();

                // Perform the search with a reasonable default limit
                let limit = self.limit.unwrap_or(100) as usize;
                match query_service.search_content(&self.query, limit).await {
                    Ok(documents) => {
                        // Filter by document type if specified
                        let filtered_docs = if let Some(type_str) = &self.document_type {
                            match DocumentType::from_str(type_str) {
                                Ok(doc_type) => {
                                    documents.into_iter()
                                        .filter(|doc| doc.document.document_type == doc_type)
                                        .collect()
                                }
                                Err(_) => return Ok(CallToolResult::text_content(
                                    vec![TextContent::from(serde_json::to_string_pretty(&serde_json::json!({
                                        "error": format!("Invalid document type '{}'. Must be: vision, strategy, initiative, task, adr", type_str)
                                    })).map_err(CallToolError::new)?)]
                                ))
                            }
                        } else {
                            documents
                        };

                        // Apply limit if specified and convert to summaries
                        let limited_docs: Vec<SearchResultSummary> = if let Some(limit) = self.limit
                        {
                            filtered_docs
                                .into_iter()
                                .take(limit as usize)
                                .map(SearchResultSummary::from)
                                .collect()
                        } else {
                            filtered_docs
                                .into_iter()
                                .map(SearchResultSummary::from)
                                .collect()
                        };

                        let response = serde_json::json!({
                            "message": format!("Found {} documents matching '{}'", limited_docs.len(), self.query),
                            "documents": limited_docs,
                            "query": self.query,
                            "project_path": self.project_path
                        });

                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&response).map_err(CallToolError::new)?,
                        )]))
                    }
                    Err(e) => {
                        let error_response = serde_json::json!({
                            "error": format!("Search failed: {}", e)
                        });

                        Ok(CallToolResult::text_content(vec![TextContent::from(
                            serde_json::to_string_pretty(&error_response)
                                .map_err(CallToolError::new)?,
                        )]))
                    }
                }
            }
            Err(e) => {
                let error_response = serde_json::json!({
                    "error": format!("Failed to connect to database: {}", e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
