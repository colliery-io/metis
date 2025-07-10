use chrono::{DateTime, Utc};
use metis_docs_core::{
    Application, Database, DocumentType,
    dal::database::models::Document,
};
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
    pub title: String,
    pub document_type: String,
    pub phase: String,
    pub filepath: String,
    pub parent_id: Option<String>,
    pub frontmatter_json: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub content_hash: String,
    pub archived: bool,
}

impl From<Document> for DocumentSummary {
    fn from(doc: Document) -> Self {
        // Parse frontmatter JSON
        let frontmatter: serde_json::Value = serde_json::from_str(&doc.frontmatter_json)
            .unwrap_or_else(|_| serde_json::json!({}));
        
        Self {
            id: doc.id,
            title: doc.title,
            document_type: doc.document_type,
            phase: doc.phase,
            filepath: doc.filepath,
            parent_id: doc.parent_id,
            frontmatter_json: frontmatter,
            created_at: doc.created_at,
            updated_at: doc.updated_at,
            content_hash: doc.content_hash,
            archived: doc.archived,
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
        let db_path = project_path.join("metis.db");

        match Database::new(db_path.to_str().unwrap()) {
            Ok(db) => {
                let mut app = Application::new(db);
                
                // Query documents using the Application's database service
                let documents = app.with_database(|db_service| {
                    // Get all documents from the repository
                    let mut all_docs = Vec::new();
                    
                    // Filter by type if specified
                    if let Some(type_str) = &self.document_type {
                        match db_service.find_by_type(type_str) {
                            Ok(docs) => all_docs.extend(docs),
                            Err(e) => {
                                eprintln!("Failed to find documents by type: {}", e);
                            }
                        }
                    } else {
                        // Get all document types
                        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
                            if let Ok(docs) = db_service.find_by_type(doc_type) {
                                all_docs.extend(docs);
                            }
                        }
                    }
                    
                    // Filter by phase if specified
                    if let Some(phase_filter) = &self.phase {
                        all_docs.retain(|doc| doc.phase.eq_ignore_ascii_case(phase_filter));
                    }
                    
                    // Filter out archived documents
                    all_docs.retain(|doc| !doc.archived);
                    
                    all_docs
                });

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
                    "project_path": self.project_path,
                    "filters": {
                        "document_type": self.document_type,
                        "phase": self.phase
                    }
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
        let db_path = project_path.join("metis.db");

        match Database::new(db_path.to_str().unwrap()) {
            Ok(db) => {
                let mut app = Application::new(db);
                
                // For now, we'll implement a simple search by loading documents and searching their content
                // In the future, this should use a proper full-text search implementation
                let documents = app.with_database(|db_service| {
                    let mut all_docs = Vec::new();
                    
                    // Get all document types or filtered by type
                    if let Some(type_str) = &self.document_type {
                        if let Ok(docs) = db_service.find_by_type(type_str) {
                            all_docs.extend(docs);
                        }
                    } else {
                        for doc_type in ["vision", "strategy", "initiative", "task", "adr"] {
                            if let Ok(docs) = db_service.find_by_type(doc_type) {
                                all_docs.extend(docs);
                            }
                        }
                    }
                    
                    // Filter out archived documents
                    all_docs.retain(|doc| !doc.archived);
                    
                    // Simple search: check if query appears in title or content
                    let query_lower = self.query.to_lowercase();
                    all_docs.retain(|doc| {
                        doc.title.to_lowercase().contains(&query_lower) ||
                        doc.content.as_ref().map_or(false, |content| 
                            content.to_lowercase().contains(&query_lower)
                        )
                    });
                    
                    all_docs
                });

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
                    "error": format!("Failed to connect to database: {}", e)
                });

                Ok(CallToolResult::text_content(vec![TextContent::from(
                    serde_json::to_string_pretty(&error_response).map_err(CallToolError::new)?,
                )]))
            }
        }
    }
}
