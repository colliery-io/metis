//! Query operations for document discovery and search

use crate::{Document, DocumentType, RelationshipType, Result};
use sqlx::SqlitePool;

/// Search result with ranking and snippet
#[derive(Debug, Clone, serde::Serialize)]
pub struct SearchResult {
    pub document: Document,
    pub rank: f64,
    pub snippet: String,
}

/// Direction for relationship queries
#[derive(Debug, Clone)]
pub enum RelationshipDirection {
    /// Relationships where this document is the source (outgoing)
    Outgoing,
    /// Relationships where this document is the target (incoming)
    Incoming,
    /// All relationships involving this document
    Both,
}

/// Document relationship with metadata
#[derive(Debug, Clone, serde::Serialize)]
pub struct Relationship {
    pub from_id: String,
    pub to_id: String,
    pub relationship_type: RelationshipType,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Query service for document operations
#[derive(Clone)]
pub struct QueryService {
    pool: SqlitePool,
}

impl QueryService {
    /// Create a new QueryService with the given database pool
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Find documents by document type
    pub async fn find_documents_by_type(&self, doc_type: DocumentType) -> Result<Vec<Document>> {
        self.find_documents_by_type_paginated(doc_type, None, None)
            .await
    }

    /// Find documents by document type with pagination
    pub async fn find_documents_by_type_paginated(
        &self,
        doc_type: DocumentType,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Document>> {
        let doc_type_str = doc_type.to_string();

        let sql = format!(
            r#"
            SELECT id, filepath, document_type, level, status, parent_id, 
                   created_at, updated_at, content_hash, frontmatter_json, 
                   exit_criteria_met, content, file_size, file_modified_at
            FROM documents 
            WHERE document_type = ?
            ORDER BY updated_at DESC
            {}
            "#,
            match (limit, offset) {
                (Some(l), Some(o)) => format!("LIMIT {} OFFSET {}", l, o),
                (Some(l), None) => format!("LIMIT {}", l),
                (None, Some(o)) => format!("OFFSET {}", o),
                (None, None) => String::new(),
            }
        );

        let records = sqlx::query(&sql)
            .bind(&doc_type_str)
            .fetch_all(&self.pool)
            .await?;

        let mut documents = Vec::new();
        for row in records {
            use sqlx::Row;
            documents.push(self.record_to_document(
                row.try_get::<Option<String>, _>("id")?.unwrap_or_default(),
                row.try_get("filepath")?,
                row.try_get("document_type")?,
                row.try_get("level")?,
                row.try_get("status")?,
                row.try_get("parent_id")?,
                row.try_get("created_at")?,
                row.try_get("updated_at")?,
                row.try_get("content_hash")?,
                row.try_get("frontmatter_json")?,
                row.try_get("exit_criteria_met")?,
                row.try_get("content")?,
                row.try_get("file_size")?,
                row.try_get("file_modified_at")?,
            )?);
        }
        Ok(documents)
    }

    /// Find documents by phase tag (e.g., "draft", "review", "published")
    pub async fn find_documents_by_phase(&self, phase: &str) -> Result<Vec<Document>> {
        let phase_tag = format!("#phase/{}", phase);
        let search_pattern = format!("%{}%", phase_tag);
        let records = sqlx::query!(
            r#"
            SELECT id, filepath, document_type, level, status, parent_id, 
                   created_at, updated_at, content_hash, frontmatter_json, 
                   exit_criteria_met, content, file_size, file_modified_at
            FROM documents 
            WHERE frontmatter_json LIKE ?
            ORDER BY updated_at DESC
            "#,
            search_pattern
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in records {
            documents.push(self.record_to_document(
                row.id.unwrap_or_default(),
                row.filepath,
                row.document_type,
                row.level,
                row.status,
                row.parent_id,
                row.created_at,
                row.updated_at,
                row.content_hash,
                row.frontmatter_json,
                row.exit_criteria_met,
                row.content,
                row.file_size,
                row.file_modified_at,
            )?);
        }
        Ok(documents)
    }

    /// Find documents by parent ID
    pub async fn find_documents_by_parent(&self, parent_id: &str) -> Result<Vec<Document>> {
        let records = sqlx::query!(
            r#"
            SELECT id, filepath, document_type, level, status, parent_id, 
                   created_at, updated_at, content_hash, frontmatter_json, 
                   exit_criteria_met, content, file_size, file_modified_at
            FROM documents 
            WHERE parent_id = ?
            ORDER BY updated_at DESC
            "#,
            parent_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in records {
            documents.push(self.record_to_document(
                row.id.unwrap_or_default(),
                row.filepath,
                row.document_type,
                row.level,
                row.status,
                row.parent_id,
                row.created_at,
                row.updated_at,
                row.content_hash,
                row.frontmatter_json,
                row.exit_criteria_met,
                row.content,
                row.file_size,
                row.file_modified_at,
            )?);
        }
        Ok(documents)
    }

    /// Find orphaned documents (documents with parent_id that doesn't exist)
    pub async fn find_orphaned_documents(&self) -> Result<Vec<Document>> {
        let records = sqlx::query!(
            r#"
            SELECT d.id, d.filepath, d.document_type, d.level, d.status, d.parent_id, 
                   d.created_at, d.updated_at, d.content_hash, d.frontmatter_json, 
                   d.exit_criteria_met, d.content, d.file_size, d.file_modified_at
            FROM documents d
            LEFT JOIN documents p ON d.parent_id = p.id
            WHERE d.parent_id IS NOT NULL AND p.id IS NULL
            ORDER BY d.updated_at DESC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in records {
            documents.push(self.record_to_document(
                row.id.unwrap_or_default(),
                row.filepath,
                row.document_type,
                row.level,
                row.status,
                row.parent_id,
                row.created_at,
                row.updated_at,
                row.content_hash,
                row.frontmatter_json,
                row.exit_criteria_met,
                row.content,
                row.file_size,
                row.file_modified_at,
            )?);
        }
        Ok(documents)
    }

    /// Get all property names used in documents
    pub async fn get_all_property_names(&self) -> Result<Vec<String>> {
        let records = sqlx::query!(
            "SELECT DISTINCT property_name FROM document_properties ORDER BY property_name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records.into_iter().map(|row| row.property_name).collect())
    }

    /// Get all values for a specific property with usage counts
    pub async fn get_property_values(&self, prop_name: &str) -> Result<Vec<(String, usize)>> {
        let records = sqlx::query!(
            r#"
            SELECT property_value, COUNT(*) as count 
            FROM document_properties 
            WHERE property_name = ? 
            GROUP BY property_value 
            ORDER BY count DESC, property_value
            "#,
            prop_name
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records
            .into_iter()
            .map(|row| (row.property_value.unwrap_or_default(), row.count as usize))
            .collect())
    }

    /// Search documents by property value (equals only for now)
    pub async fn search_by_property(&self, prop_name: &str, value: &str) -> Result<Vec<Document>> {
        let records = sqlx::query!(
            r#"
            SELECT DISTINCT d.id, d.filepath, d.document_type, d.level, d.status, d.parent_id, 
                           d.created_at, d.updated_at, d.content_hash, d.frontmatter_json, 
                           d.exit_criteria_met, d.content, d.file_size, d.file_modified_at
            FROM documents d
            JOIN document_properties p ON d.id = p.document_id
            WHERE p.property_name = ? AND p.property_value = ?
            ORDER BY d.updated_at DESC
            "#,
            prop_name,
            value
        )
        .fetch_all(&self.pool)
        .await?;

        let mut documents = Vec::new();
        for row in records {
            documents.push(self.record_to_document(
                row.id.unwrap_or_default(),
                row.filepath,
                row.document_type,
                row.level,
                row.status,
                row.parent_id,
                row.created_at,
                row.updated_at,
                row.content_hash,
                row.frontmatter_json,
                row.exit_criteria_met,
                row.content,
                row.file_size,
                row.file_modified_at,
            )?);
        }
        Ok(documents)
    }

    /// Full-text search using FTS5 with ranking and snippets
    pub async fn search_content(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        use sqlx::Row;

        let records = sqlx::query(
            r#"
            SELECT d.id, d.filepath, d.document_type, d.level, d.status, d.parent_id, 
                   d.created_at, d.updated_at, d.content_hash, d.frontmatter_json, 
                   d.exit_criteria_met, d.content, d.file_size, d.file_modified_at,
                   bm25(document_search) as rank,
                   snippet(document_search, -1, '<mark>', '</mark>', '...', 64) as snippet
            FROM document_search
            JOIN documents d ON document_search.document_id = d.id
            WHERE document_search MATCH ?
            ORDER BY rank
            LIMIT ?
            "#,
        )
        .bind(query)
        .bind(limit as i64)
        .fetch_all(&self.pool)
        .await?;

        let mut results = Vec::new();
        for row in records {
            let document = self.record_to_document(
                row.try_get::<Option<String>, _>("id")?.unwrap_or_default(),
                row.try_get("filepath")?,
                row.try_get("document_type")?,
                row.try_get("level")?,
                row.try_get("status")?,
                row.try_get("parent_id")?,
                row.try_get("created_at")?,
                row.try_get("updated_at")?,
                row.try_get("content_hash")?,
                row.try_get("frontmatter_json")?,
                row.try_get("exit_criteria_met")?,
                row.try_get("content")?,
                row.try_get("file_size")?,
                row.try_get("file_modified_at")?,
            )?;

            let search_result = SearchResult {
                document,
                rank: row.try_get::<f64, _>("rank").unwrap_or(0.0),
                snippet: row.try_get::<String, _>("snippet").unwrap_or_default(),
            };
            results.push(search_result);
        }
        Ok(results)
    }

    /// Get relationships for a document
    pub async fn get_relationships(
        &self,
        document_id: &str,
        direction: RelationshipDirection,
    ) -> Result<Vec<Relationship>> {
        use sqlx::Row;

        let (sql, bind_params): (String, Vec<&str>) = match direction {
            RelationshipDirection::Outgoing => (
                "SELECT from_id, to_id, relationship_type, created_at FROM document_relationships WHERE from_id = ? ORDER BY created_at DESC".to_string(),
                vec![document_id]
            ),
            RelationshipDirection::Incoming => (
                "SELECT from_id, to_id, relationship_type, created_at FROM document_relationships WHERE to_id = ? ORDER BY created_at DESC".to_string(),
                vec![document_id]
            ),
            RelationshipDirection::Both => (
                "SELECT from_id, to_id, relationship_type, created_at FROM document_relationships WHERE from_id = ? OR to_id = ? ORDER BY created_at DESC".to_string(),
                vec![document_id, document_id]
            ),
        };

        let mut query = sqlx::query(&sql);
        for param in bind_params {
            query = query.bind(param);
        }

        let records = query.fetch_all(&self.pool).await?;

        let mut relationships = Vec::new();
        for row in records {
            let relationship_type_str: String = row.try_get("relationship_type")?;
            let relationship_type: RelationshipType = match relationship_type_str.as_str() {
                "parent" => RelationshipType::Parent,
                "blocks" => RelationshipType::Blocks,
                "supersedes" => RelationshipType::Supersedes,
                "related" => RelationshipType::Related,
                _ => continue, // Skip unknown relationship types
            };

            let created_at_timestamp: f64 = row.try_get("created_at")?;
            let created_at = chrono::DateTime::from_timestamp(created_at_timestamp as i64, 0)
                .unwrap_or_else(chrono::Utc::now);

            relationships.push(Relationship {
                from_id: row.try_get("from_id")?,
                to_id: row.try_get("to_id")?,
                relationship_type,
                created_at,
            });
        }

        Ok(relationships)
    }

    /// Helper function to convert record fields to Document
    #[allow(clippy::too_many_arguments)]
    fn record_to_document(
        &self,
        id: String,
        filepath: String,
        document_type_str: String,
        level_str: String,
        status: String,
        parent_id: Option<String>,
        created_at: f64,
        updated_at: f64,
        content_hash: String,
        frontmatter_json: String,
        exit_criteria_met: Option<bool>,
        content: Option<String>,
        file_size: Option<i64>,
        file_modified_at: Option<f64>,
    ) -> Result<Document> {
        use chrono::{DateTime, Utc};

        let document_type: DocumentType = document_type_str.parse()?;
        let level: DocumentType = level_str.parse()?;
        let frontmatter: serde_json::Value = serde_json::from_str(&frontmatter_json)?;
        let created_at_dt = DateTime::from_timestamp(created_at as i64, 0).unwrap_or_else(Utc::now);
        let updated_at_dt = DateTime::from_timestamp(updated_at as i64, 0).unwrap_or_else(Utc::now);

        Ok(Document {
            id,
            filepath,
            document_type,
            level,
            status,
            parent_id,
            created_at: created_at_dt,
            updated_at: updated_at_dt,
            content_hash,
            frontmatter,
            exit_criteria_met: exit_criteria_met.unwrap_or(false),
            content,
            file_size,
            file_modified_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DocumentStore;
    use sqlx::SqlitePool;
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn create_test_setup() -> (DocumentStore, QueryService) {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let store = DocumentStore::from_pool(pool.clone()).await.unwrap();
        let query_service = QueryService::new(pool);
        (store, query_service)
    }

    fn create_test_document_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    #[tokio::test]
    async fn test_find_documents_by_type() {
        let (store, query_service) = create_test_setup().await;

        // Create test documents of different types
        let vision_content = r##"---
id: test-vision
level: vision
status: draft
tags:
  - "#phase/shaping"
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Vision
"##;

        let strategy_content = r##"---
id: test-strategy
level: strategy
status: active
tags:
  - "#phase/design"
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Strategy
"##;

        let vision_file = create_test_document_file(vision_content);
        let strategy_file = create_test_document_file(strategy_content);

        store.store_document(vision_file.path()).await.unwrap();
        store.store_document(strategy_file.path()).await.unwrap();

        // Query for vision documents
        let visions = query_service
            .find_documents_by_type(DocumentType::Vision)
            .await
            .unwrap();
        assert_eq!(visions.len(), 1);
        assert_eq!(visions[0].id, "test-vision");

        // Query for strategy documents
        let strategies = query_service
            .find_documents_by_type(DocumentType::Strategy)
            .await
            .unwrap();
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0].id, "test-strategy");
    }

    #[tokio::test]
    async fn test_find_documents_by_phase() {
        let (store, query_service) = create_test_setup().await;

        let shaping_content = r##"---
id: test-shaping
level: task
status: todo
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
tags:
  - "#phase/shaping"
---

# Shaping Task
"##;

        let design_content = r##"---
id: test-design
level: task
status: active
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
tags:
  - "#phase/design"
---

# Design Task
"##;

        let shaping_file = create_test_document_file(shaping_content);
        let design_file = create_test_document_file(design_content);

        store.store_document(shaping_file.path()).await.unwrap();
        store.store_document(design_file.path()).await.unwrap();

        // Query by phase
        let shaping_docs = query_service
            .find_documents_by_phase("shaping")
            .await
            .unwrap();
        assert_eq!(shaping_docs.len(), 1);
        assert_eq!(shaping_docs[0].id, "test-shaping");

        let design_docs = query_service
            .find_documents_by_phase("design")
            .await
            .unwrap();
        assert_eq!(design_docs.len(), 1);
        assert_eq!(design_docs[0].id, "test-design");
    }

    #[tokio::test]
    async fn test_search_by_property() {
        let (store, query_service) = create_test_setup().await;

        let doc_content = r##"---
id: test-properties
level: initiative
status: active
tags:
  - "#phase/design"
technical_lead: john.doe
priority: high
estimated_complexity: l
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Properties
"##;

        let doc_file = create_test_document_file(doc_content);
        store.store_document(doc_file.path()).await.unwrap();

        // Search by exact match
        let results = query_service
            .search_by_property("technical_lead", "john.doe")
            .await
            .unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "test-properties");

        // Search for non-match
        let results = query_service
            .search_by_property("technical_lead", "jane.doe")
            .await
            .unwrap();
        assert_eq!(results.len(), 0);
    }

    #[tokio::test]
    async fn test_get_property_names_and_values() {
        let (store, query_service) = create_test_setup().await;

        let doc_content = r##"---
id: test-props
level: task
status: todo
tags:
  - "#phase/todo"
priority: high
estimated_hours: 8
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Test Props
"##;

        let doc_file = create_test_document_file(doc_content);
        store.store_document(doc_file.path()).await.unwrap();

        // Get all property names
        let prop_names = query_service.get_all_property_names().await.unwrap();
        assert!(prop_names.contains(&"priority".to_string()));
        assert!(prop_names.contains(&"estimated_hours".to_string()));

        // Get values for specific property
        let priority_values = query_service.get_property_values("priority").await.unwrap();
        assert_eq!(priority_values.len(), 1);
        assert_eq!(priority_values[0].0, "high");
        assert_eq!(priority_values[0].1, 1); // count
    }

    #[tokio::test]
    async fn test_search_content_fts() {
        let (store, query_service) = create_test_setup().await;

        let doc1_content = r##"---
id: test-search-1
level: vision
status: draft
tags:
  - "#phase/shaping"
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Document Management Vision

This vision outlines our approach to document management using Rust and SQLite.
"##;

        let doc2_content = r##"---
id: test-search-2
level: strategy
status: active
tags:
  - "#phase/design"
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Storage Strategy

Our storage strategy focuses on SQLite performance and indexing capabilities.
"##;

        let doc1_file = create_test_document_file(doc1_content);
        let doc2_file = create_test_document_file(doc2_content);

        store.store_document(doc1_file.path()).await.unwrap();
        store.store_document(doc2_file.path()).await.unwrap();

        // Search for documents containing "SQLite"
        let results = query_service.search_content("SQLite", 10).await.unwrap();
        assert_eq!(results.len(), 2);

        // Verify search results have proper structure
        for result in &results {
            assert!(!result.document.id.is_empty());
            // BM25 scores can be negative, so just check it's a valid number
            assert!(!result.rank.is_nan());
            assert!(!result.snippet.is_empty());
        }

        // Search for specific term that should match only one document
        let mgmt_results = query_service
            .search_content("management", 10)
            .await
            .unwrap();
        assert_eq!(mgmt_results.len(), 1);
        assert_eq!(mgmt_results[0].document.id, "test-search-1");

        // Search for term that doesn't exist
        let no_results = query_service
            .search_content("nonexistent", 10)
            .await
            .unwrap();
        assert_eq!(no_results.len(), 0);
    }

    #[tokio::test]
    async fn test_get_relationships() {
        let (store, query_service) = create_test_setup().await;

        // Create test documents
        let doc1_content = r##"---
id: parent-doc
level: strategy
status: active
tags:
  - "#phase/design"
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Parent Document
"##;

        let doc2_content = r##"---
id: child-doc
level: task
status: todo
tags:
  - "#phase/todo"
parent: parent-doc
created_at: 2025-07-03T10:00:00Z
updated_at: 2025-07-03T10:00:00Z
exit_criteria_met: false
---

# Child Document
"##;

        let doc1_file = create_test_document_file(doc1_content);
        let doc2_file = create_test_document_file(doc2_content);

        store.store_document(doc1_file.path()).await.unwrap();
        store.store_document(doc2_file.path()).await.unwrap();

        // Manually insert a relationship for testing
        sqlx::query(
            "INSERT INTO document_relationships (from_id, to_id, relationship_type, created_at) VALUES (?, ?, ?, ?)"
        )
        .bind("parent-doc")
        .bind("child-doc")
        .bind("blocks")
        .bind(chrono::Utc::now().timestamp() as f64)
        .execute(store.pool())
        .await
        .unwrap();

        // Test outgoing relationships
        let outgoing = query_service
            .get_relationships("parent-doc", RelationshipDirection::Outgoing)
            .await
            .unwrap();
        assert_eq!(outgoing.len(), 1);
        assert_eq!(outgoing[0].from_id, "parent-doc");
        assert_eq!(outgoing[0].to_id, "child-doc");

        // Test incoming relationships
        let incoming = query_service
            .get_relationships("child-doc", RelationshipDirection::Incoming)
            .await
            .unwrap();
        assert_eq!(incoming.len(), 1);
        assert_eq!(incoming[0].from_id, "parent-doc");
        assert_eq!(incoming[0].to_id, "child-doc");

        // Test both directions
        let both = query_service
            .get_relationships("parent-doc", RelationshipDirection::Both)
            .await
            .unwrap();
        assert_eq!(both.len(), 1);

        // Test no relationships
        let none = query_service
            .get_relationships("nonexistent", RelationshipDirection::Both)
            .await
            .unwrap();
        assert_eq!(none.len(), 0);
    }
}
