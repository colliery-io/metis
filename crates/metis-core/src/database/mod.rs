//! Database operations for document storage and retrieval

pub mod query;

use crate::{Document, DocumentType, MetisError, Result};
use chrono::{DateTime, Utc};
use gray_matter;
use sha2::{Digest, Sha256};
use sqlx::{migrate::MigrateDatabase, SqlitePool};
use std::fs;
use std::path::Path;

// Re-export query types
pub use query::{QueryService, Relationship, RelationshipDirection, SearchResult};

#[derive(Clone)]
pub struct DocumentStore {
    pool: SqlitePool,
}

impl DocumentStore {
    /// Create a new DocumentStore with automatic database creation and migrations
    pub async fn new(database_url: &str) -> Result<Self> {
        // Create database if it doesn't exist
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;

        // Run migrations
        sqlx::migrate!("./src/migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    /// Create a DocumentStore from an existing pool (useful for testing)
    pub async fn from_pool(pool: SqlitePool) -> Result<Self> {
        let store = Self { pool };

        // Run migrations
        sqlx::migrate!("./src/migrations").run(&store.pool).await?;

        Ok(store)
    }

    /// Get the underlying database pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Create a QueryService using this DocumentStore's database pool
    pub fn query_service(&self) -> QueryService {
        QueryService::new(self.pool.clone())
    }

    /// Store a document by reading and parsing the file at the given path
    pub async fn store_document(&self, filepath: &Path) -> Result<Document> {
        // Read file contents
        let raw_content = fs::read_to_string(filepath).map_err(MetisError::Io)?;

        // Parse frontmatter and content
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&raw_content);

        // Extract frontmatter as JSON
        let frontmatter: serde_json::Value = if let Some(data) = parsed.data {
            // gray_matter returns a Pod which we need to convert to serde_json::Value
            match data {
                gray_matter::Pod::Hash(map) => {
                    let mut json_map = serde_json::Map::new();
                    for (key, value) in map {
                        let json_value = match value {
                            gray_matter::Pod::String(s) => serde_json::Value::String(s),
                            gray_matter::Pod::Integer(i) => {
                                serde_json::Value::Number(serde_json::Number::from(i))
                            }
                            gray_matter::Pod::Float(f) => serde_json::Value::Number(
                                serde_json::Number::from_f64(f)
                                    .unwrap_or(serde_json::Number::from(0)),
                            ),
                            gray_matter::Pod::Boolean(b) => serde_json::Value::Bool(b),
                            gray_matter::Pod::Array(arr) => {
                                let json_arr: Vec<serde_json::Value> = arr
                                    .into_iter()
                                    .map(|item| match item {
                                        gray_matter::Pod::String(s) => serde_json::Value::String(s),
                                        _ => serde_json::Value::String(format!("{:?}", item)),
                                    })
                                    .collect();
                                serde_json::Value::Array(json_arr)
                            }
                            _ => serde_json::Value::String(format!("{:?}", value)),
                        };
                        json_map.insert(key, json_value);
                    }
                    serde_json::Value::Object(json_map)
                }
                _ => serde_json::Value::Object(serde_json::Map::new()),
            }
        } else {
            serde_json::Value::Object(serde_json::Map::new())
        };

        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(raw_content.as_bytes());
        let content_hash = format!("{:x}", hasher.finalize());

        // Extract document metadata from frontmatter
        let id = frontmatter
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MetisError::MissingRequiredField {
                field: "id".to_string(),
            })?
            .to_string();

        let document_type_str = frontmatter
            .get("level")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MetisError::MissingRequiredField {
                field: "level".to_string(),
            })?;
        let document_type: DocumentType = document_type_str.parse()?;

        let status = frontmatter
            .get("status")
            .and_then(|v| v.as_str())
            .ok_or_else(|| MetisError::MissingRequiredField {
                field: "status".to_string(),
            })?
            .to_string();

        // Force parent_id to None for vision and ADR documents since they can't have parents
        let parent_id = if document_type_str == "vision" || document_type_str == "adr" {
            None
        } else {
            frontmatter
                .get("parent")
                .and_then(|v| if v.is_null() { None } else { v.as_str() })
                .filter(|s| !s.trim().is_empty())
                .map(|s| s.to_string())
        };

        let exit_criteria_met = frontmatter
            .get("exit_criteria_met")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let created_at = frontmatter
            .get("created_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        let updated_at = frontmatter
            .get("updated_at")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc))
            .unwrap_or_else(Utc::now);

        // Get file metadata for new fields
        let metadata = std::fs::metadata(filepath).map_err(MetisError::Io)?;
        let file_size = metadata.len() as i64;
        let file_modified_at = metadata
            .modified()
            .map_err(MetisError::Io)?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs_f64();

        // Create Document struct
        let document = Document {
            id: id.clone(),
            filepath: filepath.to_string_lossy().to_string(),
            document_type: document_type.clone(),
            level: document_type,
            status,
            parent_id,
            created_at,
            updated_at,
            content_hash,
            frontmatter: frontmatter.clone(),
            exit_criteria_met,
            content: Some(parsed.content),
            file_size: Some(file_size),
            file_modified_at: Some(file_modified_at),
        };

        // Store in database - create bindings to avoid temporary value issues
        let doc_type_str = document.document_type.to_string();
        let level_str = document.level.to_string();
        let created_at_timestamp = document.created_at.timestamp_millis() as f64 / 1000.0;
        let updated_at_timestamp = document.updated_at.timestamp_millis() as f64 / 1000.0;
        let frontmatter_json = serde_json::to_string(&document.frontmatter)?;

        sqlx::query!(
            r#"
            INSERT OR REPLACE INTO documents 
            (id, filepath, document_type, level, status, parent_id, 
             created_at, updated_at, content_hash, frontmatter_json, 
             exit_criteria_met, content, file_size, file_modified_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            document.id,
            document.filepath,
            doc_type_str,
            level_str,
            document.status,
            document.parent_id,
            created_at_timestamp,
            updated_at_timestamp,
            document.content_hash,
            frontmatter_json,
            document.exit_criteria_met,
            document.content,
            document.file_size,
            document.file_modified_at
        )
        .execute(&self.pool)
        .await?;

        // Extract and store properties for efficient queries
        self.update_properties(&document.id, &frontmatter).await?;

        Ok(document)
    }

    /// Get a document by ID
    pub async fn get_document(&self, document_id: &str) -> Result<Option<Document>> {
        let record = sqlx::query!(
            r#"
            SELECT id, filepath, document_type, level, status, parent_id, 
                   created_at, updated_at, content_hash, frontmatter_json, 
                   exit_criteria_met, content, file_size, file_modified_at
            FROM documents 
            WHERE id = ?
            "#,
            document_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = record {
            let document_type: DocumentType = row.document_type.parse()?;
            let level: DocumentType = row.level.parse()?;
            let frontmatter: serde_json::Value = serde_json::from_str(&row.frontmatter_json)?;
            let created_at =
                DateTime::from_timestamp(row.created_at as i64, 0).unwrap_or_else(Utc::now);
            let updated_at =
                DateTime::from_timestamp(row.updated_at as i64, 0).unwrap_or_else(Utc::now);

            Ok(Some(Document {
                id: row.id.unwrap_or_default(),
                filepath: row.filepath,
                document_type,
                level,
                status: row.status,
                parent_id: row.parent_id,
                created_at,
                updated_at,
                content_hash: row.content_hash,
                frontmatter,
                exit_criteria_met: row.exit_criteria_met.unwrap_or(false),
                content: row.content,
                file_size: row.file_size,
                file_modified_at: row.file_modified_at,
            }))
        } else {
            Ok(None)
        }
    }

    /// Update an existing document by re-reading the file
    pub async fn update_document(&self, filepath: &Path) -> Result<Document> {
        // For updates, we just re-store the document which will replace it
        self.store_document(filepath).await
    }

    /// Delete a document by ID
    pub async fn delete_document(&self, document_id: &str) -> Result<bool> {
        let result = sqlx::query!("DELETE FROM documents WHERE id = ?", document_id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Extract frontmatter properties and store them for efficient querying
    async fn update_properties(
        &self,
        document_id: &str,
        frontmatter: &serde_json::Value,
    ) -> Result<()> {
        // First, delete existing properties for this document
        sqlx::query!(
            "DELETE FROM document_properties WHERE document_id = ?",
            document_id
        )
        .execute(&self.pool)
        .await?;

        // Extract properties from frontmatter
        if let serde_json::Value::Object(map) = frontmatter {
            for (key, value) in map {
                let (prop_value, prop_type) = match value {
                    serde_json::Value::String(s) => (s.clone(), "text".to_string()),
                    serde_json::Value::Number(n) => (n.to_string(), "number".to_string()),
                    serde_json::Value::Bool(b) => (b.to_string(), "boolean".to_string()),
                    serde_json::Value::Array(_) => (value.to_string(), "array".to_string()),
                    _ => (value.to_string(), "text".to_string()),
                };

                sqlx::query!(
                    r#"
                    INSERT INTO document_properties 
                    (document_id, property_name, property_value, property_type)
                    VALUES (?1, ?2, ?3, ?4)
                    "#,
                    document_id,
                    key,
                    prop_value,
                    prop_type
                )
                .execute(&self.pool)
                .await?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::SqlitePool;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    async fn create_test_store() -> DocumentStore {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        DocumentStore::from_pool(pool).await.unwrap()
    }

    fn create_test_document_file(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    #[tokio::test]
    async fn test_document_store_creation() {
        let pool = SqlitePool::connect(":memory:").await.unwrap();
        let store = DocumentStore::from_pool(pool).await.unwrap();

        // Verify tables were created by checking the schema
        let tables: Vec<(String,)> =
            sqlx::query_as("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .fetch_all(store.pool())
                .await
                .unwrap();

        let table_names: Vec<String> = tables.into_iter().map(|(name,)| name).collect();

        assert!(table_names.contains(&"documents".to_string()));
        assert!(table_names.contains(&"document_relationships".to_string()));
        assert!(table_names.contains(&"document_properties".to_string()));
        assert!(table_names.contains(&"document_search".to_string()));
    }

    #[tokio::test]
    async fn test_store_document() {
        let store = create_test_store().await;

        let document_content = r##"---
id: test-vision-document
level: vision
status: draft
phase: shaping
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
tags:
  - "#vision"
  - "#phase/shaping"
---

# Test Vision Document

This is a test vision document for testing purposes.

## Goals

- Test goal 1
- Test goal 2
"##;

        let temp_file = create_test_document_file(document_content);
        let result = store.store_document(temp_file.path()).await;

        assert!(result.is_ok());
        let document = result.unwrap();

        assert_eq!(document.id, "test-vision-document");
        assert_eq!(document.document_type, DocumentType::Vision);
        assert_eq!(document.status, "draft");
        // Phase is now in tags, not a separate field
        assert!(document.content.is_some());
        assert!(document
            .content
            .as_ref()
            .unwrap()
            .contains("Test Vision Document"));
    }

    #[tokio::test]
    async fn test_get_document() {
        let store = create_test_store().await;

        let document_content = r#"---
id: test-get-document
level: strategy
status: active
phase: design
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
---

# Test Strategy Document

Content for get test.
"#;

        let temp_file = create_test_document_file(document_content);

        // Store the document first
        store.store_document(temp_file.path()).await.unwrap();

        // Now get it
        let result = store.get_document("test-get-document").await;
        assert!(result.is_ok());

        let document = result.unwrap();
        assert!(document.is_some());

        let doc = document.unwrap();
        assert_eq!(doc.id, "test-get-document");
        assert_eq!(doc.document_type, DocumentType::Strategy);
        assert_eq!(doc.status, "active");
        // Phase is now in tags, not a separate field
    }

    #[tokio::test]
    async fn test_get_nonexistent_document() {
        let store = create_test_store().await;

        let result = store.get_document("nonexistent-document").await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_update_document() {
        let store = create_test_store().await;

        let initial_content = r#"---
id: test-update-document
level: task
status: todo
phase: todo
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
---

# Initial Content
"#;

        let updated_content = r#"---
id: test-update-document
level: task
status: doing
phase: doing
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T19:00:00Z
exit_criteria_met: false
---

# Updated Content

This content has been updated.
"#;

        let temp_file = create_test_document_file(initial_content);

        // Store initial document
        let initial_doc = store.store_document(temp_file.path()).await.unwrap();
        assert_eq!(initial_doc.status, "todo");

        // Update the file content
        fs::write(temp_file.path(), updated_content).unwrap();

        // Update the document
        let updated_doc = store.update_document(temp_file.path()).await.unwrap();
        assert_eq!(updated_doc.status, "doing");
        assert!(updated_doc
            .content
            .as_ref()
            .unwrap()
            .contains("Updated Content"));

        // Verify it was updated in the database
        let retrieved_doc = store
            .get_document("test-update-document")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(retrieved_doc.status, "doing");
    }

    #[tokio::test]
    async fn test_delete_document() {
        let store = create_test_store().await;

        let document_content = r#"---
id: test-delete-document
level: adr
status: draft
phase: shaping
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
---

# Document to Delete
"#;

        let temp_file = create_test_document_file(document_content);

        // Store the document
        store.store_document(temp_file.path()).await.unwrap();

        // Verify it exists
        let doc = store.get_document("test-delete-document").await.unwrap();
        assert!(doc.is_some());

        // Delete it
        let deleted = store.delete_document("test-delete-document").await.unwrap();
        assert!(deleted);

        // Verify it's gone
        let doc_after = store.get_document("test-delete-document").await.unwrap();
        assert!(doc_after.is_none());

        // Try to delete again - should return false
        let deleted_again = store.delete_document("test-delete-document").await.unwrap();
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_properties_extraction() {
        let store = create_test_store().await;

        let document_content = r##"---
id: test-properties-document
level: initiative
status: active
phase: design
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
technical_lead: john.doe
estimated_complexity: l
priority: high
tags:
  - "#initiative"
  - "#phase/design"
---

# Test Properties Document
"##;

        let temp_file = create_test_document_file(document_content);

        // Store the document
        store.store_document(temp_file.path()).await.unwrap();

        // Check that properties were extracted
        let properties: Vec<(String, String, String)> = sqlx::query_as(
            "SELECT property_name, property_value, property_type FROM document_properties WHERE document_id = ? ORDER BY property_name"
        )
        .bind("test-properties-document")
        .fetch_all(store.pool())
        .await
        .unwrap();

        // Should have extracted multiple properties
        assert!(!properties.is_empty());

        // Check for specific properties
        let technical_lead = properties
            .iter()
            .find(|(name, _, _)| name == "technical_lead");
        assert!(technical_lead.is_some());
        assert_eq!(technical_lead.unwrap().1, "john.doe");

        let complexity = properties
            .iter()
            .find(|(name, _, _)| name == "estimated_complexity");
        assert!(complexity.is_some());
        assert_eq!(complexity.unwrap().1, "l");
    }

    #[tokio::test]
    async fn test_missing_required_fields() {
        let store = create_test_store().await;

        let document_content = r#"---
status: draft
phase: shaping
---

# Document Missing Required Fields
"#;

        let temp_file = create_test_document_file(document_content);

        // Should fail due to missing id field
        let result = store.store_document(temp_file.path()).await;
        assert!(result.is_err());

        if let Err(MetisError::MissingRequiredField { field }) = result {
            assert_eq!(field, "id");
        } else {
            panic!("Expected MissingRequiredField error");
        }
    }

    #[tokio::test]
    async fn test_content_hash_calculation() {
        let store = create_test_store().await;

        let document_content = r#"---
id: test-hash-document
level: vision
status: draft
phase: shaping
created_at: 2025-07-02T18:00:00Z
updated_at: 2025-07-02T18:00:00Z
exit_criteria_met: false
---

# Test Hash Document

Content for hash testing.
"#;

        let temp_file = create_test_document_file(document_content);

        // Store the document
        let doc1 = store.store_document(temp_file.path()).await.unwrap();

        // Store the same document again
        let doc2 = store.store_document(temp_file.path()).await.unwrap();

        // Content hashes should be the same
        assert_eq!(doc1.content_hash, doc2.content_hash);
        assert!(!doc1.content_hash.is_empty());
    }
}
