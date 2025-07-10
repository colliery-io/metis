use crate::dal::database::models::*;
use crate::dal::database::schema;
use crate::{MetisError, Result};
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

/// Data access repository for document operations
pub struct DocumentRepository {
    connection: SqliteConnection,
}

impl DocumentRepository {
    pub fn new(connection: SqliteConnection) -> Self {
        Self { connection }
    }

    /// Insert a new document into the database
    pub fn create_document(&mut self, doc: NewDocument) -> Result<Document> {
        use schema::documents::dsl::*;

        diesel::insert_into(documents)
            .values(&doc)
            .returning(Document::as_returning())
            .get_result(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Find a document by its filepath
    pub fn find_by_filepath(&mut self, file_path: &str) -> Result<Option<Document>> {
        use schema::documents::dsl::*;

        documents
            .filter(filepath.eq(file_path))
            .first(&mut self.connection)
            .optional()
            .map_err(MetisError::Database)
    }

    /// Find a document by its ID
    pub fn find_by_id(&mut self, document_id: &str) -> Result<Option<Document>> {
        use schema::documents::dsl::*;

        documents
            .filter(id.eq(document_id))
            .first(&mut self.connection)
            .optional()
            .map_err(MetisError::Database)
    }

    /// Update an existing document
    pub fn update_document(&mut self, file_path: &str, doc: &Document) -> Result<Document> {
        use schema::documents::dsl::*;

        diesel::update(documents.filter(filepath.eq(file_path)))
            .set(doc)
            .returning(Document::as_returning())
            .get_result(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Delete a document and all its relationships
    pub fn delete_document(&mut self, file_path: &str) -> Result<bool> {
        use schema::documents::dsl::*;

        let deleted_count = diesel::delete(documents.filter(filepath.eq(file_path)))
            .execute(&mut self.connection)
            .map_err(MetisError::Database)?;

        Ok(deleted_count > 0)
    }

    /// Find all children of a document
    pub fn find_children(&mut self, parent_document_id: &str) -> Result<Vec<Document>> {
        use schema::document_relationships::dsl::*;
        use schema::documents::dsl::*;

        documents
            .inner_join(document_relationships.on(id.eq(child_id)))
            .filter(parent_id.eq(parent_document_id))
            .select(Document::as_select())
            .load(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Find the parent of a document
    pub fn find_parent(&mut self, child_document_id: &str) -> Result<Option<Document>> {
        use schema::document_relationships::dsl::*;
        use schema::documents::dsl::*;

        documents
            .inner_join(document_relationships.on(id.eq(parent_id)))
            .filter(child_id.eq(child_document_id))
            .select(Document::as_select())
            .first(&mut self.connection)
            .optional()
            .map_err(MetisError::Database)
    }

    /// Create a parent-child relationship
    pub fn create_relationship(&mut self, relationship: DocumentRelationship) -> Result<()> {
        use schema::document_relationships::dsl::*;

        diesel::insert_into(document_relationships)
            .values(&relationship)
            .execute(&mut self.connection)
            .map_err(MetisError::Database)?;

        Ok(())
    }

    /// Search documents using FTS
    pub fn search_documents(&mut self, query: &str) -> Result<Vec<Document>> {
        // For SQLite FTS, we need to use sql_query for the MATCH operator
        diesel::sql_query(
            "
            SELECT d.* FROM documents d
            INNER JOIN document_search ds ON d.filepath = ds.document_filepath
            WHERE document_search MATCH ?
        ",
        )
        .bind::<diesel::sql_types::Text, _>(query)
        .load::<Document>(&mut self.connection)
        .map_err(MetisError::Database)
    }

    /// Get all documents of a specific type
    pub fn find_by_type(&mut self, doc_type: &str) -> Result<Vec<Document>> {
        use schema::documents::dsl::*;

        documents
            .filter(document_type.eq(doc_type))
            .order(updated_at.desc())
            .load(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Get documents with specific tags
    pub fn find_by_tag(&mut self, tag_name: &str) -> Result<Vec<Document>> {
        use schema::document_tags::dsl::*;
        use schema::documents::dsl::*;

        documents
            .inner_join(document_tags.on(filepath.eq(document_filepath)))
            .filter(tag.eq(tag_name))
            .select(Document::as_select())
            .load(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Get documents in a specific phase
    pub fn find_by_phase(&mut self, phase_name: &str) -> Result<Vec<Document>> {
        use schema::documents::dsl::*;

        documents
            .filter(phase.eq(phase_name))
            .order(updated_at.desc())
            .load(&mut self.connection)
            .map_err(MetisError::Database)
    }

    /// Get documents by type and phase
    pub fn find_by_type_and_phase(
        &mut self,
        doc_type: &str,
        phase_name: &str,
    ) -> Result<Vec<Document>> {
        use schema::documents::dsl::*;

        documents
            .filter(document_type.eq(doc_type))
            .filter(phase.eq(phase_name))
            .order(updated_at.desc())
            .load(&mut self.connection)
            .map_err(MetisError::Database)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::Database;

    fn setup_test_repository() -> DocumentRepository {
        let db = Database::new(":memory:").expect("Failed to create test database");
        db.into_repository()
    }

    fn create_test_document() -> NewDocument {
        NewDocument {
            filepath: "/test/doc.md".to_string(),
            id: "test-doc-1".to_string(),
            title: "Test Document".to_string(),
            document_type: "vision".to_string(),
            created_at: 1609459200.0, // 2021-01-01
            updated_at: 1609459200.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "abc123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: Some("Test content".to_string()),
            phase: "draft".to_string(),
        }
    }

    #[test]
    fn test_create_and_find_document() {
        let mut repo = setup_test_repository();

        let new_doc = create_test_document();
        let created = repo
            .create_document(new_doc)
            .expect("Failed to create document");

        assert_eq!(created.filepath, "/test/doc.md");
        assert_eq!(created.title, "Test Document");
        assert_eq!(created.document_type, "vision");

        // Test find by filepath
        let found = repo
            .find_by_filepath("/test/doc.md")
            .expect("Failed to find document")
            .expect("Document not found");
        assert_eq!(found.id, "test-doc-1");

        // Test find by id
        let found_by_id = repo
            .find_by_id("test-doc-1")
            .expect("Failed to find document")
            .expect("Document not found");
        assert_eq!(found_by_id.filepath, "/test/doc.md");
    }

    #[test]
    fn test_update_document() {
        let mut repo = setup_test_repository();

        let new_doc = create_test_document();
        let mut created = repo
            .create_document(new_doc)
            .expect("Failed to create document");

        // Update the document
        created.title = "Updated Title".to_string();
        created.updated_at = 1609462800.0; // 1 hour later

        let updated = repo
            .update_document("/test/doc.md", &created)
            .expect("Failed to update document");

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.updated_at, 1609462800.0);
    }

    #[test]
    fn test_delete_document() {
        let mut repo = setup_test_repository();

        let new_doc = create_test_document();
        repo.create_document(new_doc)
            .expect("Failed to create document");

        // Delete the document
        let deleted = repo
            .delete_document("/test/doc.md")
            .expect("Failed to delete document");
        assert!(deleted);

        // Verify it's gone
        let found = repo
            .find_by_filepath("/test/doc.md")
            .expect("Failed to search for document");
        assert!(found.is_none());

        // Try to delete non-existent document
        let deleted_again = repo
            .delete_document("/test/doc.md")
            .expect("Failed to delete document");
        assert!(!deleted_again);
    }

    #[test]
    fn test_document_relationships() {
        let mut repo = setup_test_repository();

        // Create parent document
        let parent_doc = NewDocument {
            filepath: "/parent.md".to_string(),
            id: "parent-1".to_string(),
            title: "Parent Document".to_string(),
            document_type: "strategy".to_string(),
            created_at: 1609459200.0,
            updated_at: 1609459200.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "parent123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: Some("Parent content".to_string()),
            phase: "shaping".to_string(),
        };
        repo.create_document(parent_doc)
            .expect("Failed to create parent");

        // Create child document
        let child_doc = NewDocument {
            filepath: "/child.md".to_string(),
            id: "child-1".to_string(),
            title: "Child Document".to_string(),
            document_type: "initiative".to_string(),
            created_at: 1609459200.0,
            updated_at: 1609459200.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "child123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: Some("Child content".to_string()),
            phase: "discovery".to_string(),
        };
        repo.create_document(child_doc)
            .expect("Failed to create child");

        // Create relationship
        let relationship = DocumentRelationship {
            child_id: "child-1".to_string(),
            parent_id: "parent-1".to_string(),
            child_filepath: "/child.md".to_string(),
            parent_filepath: "/parent.md".to_string(),
        };
        repo.create_relationship(relationship)
            .expect("Failed to create relationship");

        // Test find children
        let children = repo
            .find_children("parent-1")
            .expect("Failed to find children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child-1");

        // Test find parent
        let parent = repo
            .find_parent("child-1")
            .expect("Failed to find parent")
            .expect("Parent not found");
        assert_eq!(parent.id, "parent-1");
    }

    #[test]
    fn test_find_by_type() {
        let mut repo = setup_test_repository();

        // Create documents of different types
        let vision_doc = NewDocument {
            document_type: "vision".to_string(),
            filepath: "/vision.md".to_string(),
            id: "vision-1".to_string(),
            title: "Vision Doc".to_string(),
            created_at: 1609459200.0,
            updated_at: 1609459200.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "vision123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: None,
            phase: "draft".to_string(),
        };

        let strategy_doc = NewDocument {
            document_type: "strategy".to_string(),
            filepath: "/strategy.md".to_string(),
            id: "strategy-1".to_string(),
            title: "Strategy Doc".to_string(),
            created_at: 1609462800.0, // Later timestamp
            updated_at: 1609462800.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "strategy123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: None,
            phase: "shaping".to_string(),
        };

        repo.create_document(vision_doc)
            .expect("Failed to create vision");
        repo.create_document(strategy_doc)
            .expect("Failed to create strategy");

        // Test find by type
        let visions = repo.find_by_type("vision").expect("Failed to find visions");
        assert_eq!(visions.len(), 1);
        assert_eq!(visions[0].document_type, "vision");

        let strategies = repo
            .find_by_type("strategy")
            .expect("Failed to find strategies");
        assert_eq!(strategies.len(), 1);
        assert_eq!(strategies[0].document_type, "strategy");

        // Verify ordering (newest first)
        let _all_docs = repo.find_by_type("vision").expect("Failed to find docs");
        // Since we only have one vision, we can't test ordering here
        // But the query should work
    }

    #[test]
    fn test_document_not_found() {
        let mut repo = setup_test_repository();

        let found = repo
            .find_by_filepath("/nonexistent.md")
            .expect("Failed to search for document");
        assert!(found.is_none());

        let found_by_id = repo
            .find_by_id("nonexistent")
            .expect("Failed to search for document");
        assert!(found_by_id.is_none());
    }
}
