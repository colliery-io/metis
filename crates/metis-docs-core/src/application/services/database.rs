use crate::dal::database::{models::*, repository::DocumentRepository};
use crate::domain::documents::types::DocumentType;
use crate::Result;

/// Database service - handles all database CRUD operations
pub struct DatabaseService {
    repository: DocumentRepository,
}

impl DatabaseService {
    pub fn new(repository: DocumentRepository) -> Self {
        Self { repository }
    }

    /// Create a new document in the database
    pub fn create_document(&mut self, document: NewDocument) -> Result<Document> {
        self.repository.create_document(document)
    }

    /// Find a document by filepath
    pub fn find_by_filepath(&mut self, filepath: &str) -> Result<Option<Document>> {
        self.repository.find_by_filepath(filepath)
    }

    /// Find a document by ID
    pub fn find_by_id(&mut self, id: &str) -> Result<Option<Document>> {
        self.repository.find_by_id(id)
    }

    /// Update an existing document
    pub fn update_document(&mut self, filepath: &str, document: &Document) -> Result<Document> {
        self.repository.update_document(filepath, document)
    }

    /// Delete a document from the database
    pub fn delete_document(&mut self, filepath: &str) -> Result<bool> {
        self.repository.delete_document(filepath)
    }

    /// Search documents using full-text search
    pub fn search_documents(&mut self, query: &str) -> Result<Vec<Document>> {
        self.repository.search_documents(query)
    }

    /// Get all documents of a specific type
    pub fn find_by_type(&mut self, doc_type: DocumentType) -> Result<Vec<Document>> {
        let type_str = doc_type.to_string();
        self.repository.find_by_type(&type_str)
    }

    /// Get documents with a specific tag
    pub fn find_by_tag(&mut self, tag: &str) -> Result<Vec<Document>> {
        self.repository.find_by_tag(tag)
    }

    /// Get all children of a document
    pub fn find_children(&mut self, parent_id: &str) -> Result<Vec<Document>> {
        self.repository.find_children(parent_id)
    }

    /// Get the parent of a document
    pub fn find_parent(&mut self, child_id: &str) -> Result<Option<Document>> {
        self.repository.find_parent(child_id)
    }

    /// Create a parent-child relationship
    pub fn create_relationship(&mut self, parent_id: &str, child_id: &str, parent_filepath: &str, child_filepath: &str) -> Result<()> {
        let relationship = DocumentRelationship {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
            parent_filepath: parent_filepath.to_string(),
            child_filepath: child_filepath.to_string(),
        };
        self.repository.create_relationship(relationship)
    }

    /// Check if a document exists by filepath
    pub fn document_exists(&mut self, filepath: &str) -> Result<bool> {
        Ok(self.repository.find_by_filepath(filepath)?.is_some())
    }

    /// Get document count by type
    pub fn count_by_type(&mut self, doc_type: DocumentType) -> Result<usize> {
        let docs = self.repository.find_by_type(&doc_type.to_string())?;
        Ok(docs.len())
    }

    /// Get all document IDs and their filepaths (useful for validation)
    pub fn get_all_id_filepath_pairs(&mut self) -> Result<Vec<(String, String)>> {
        // This would need a custom query in the repository
        // For now, we'll use find_by_type for each type
        let mut pairs = Vec::new();
        
        for doc_type in [
            DocumentType::Vision,
            DocumentType::Strategy,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr,
        ] {
            let docs = self.repository.find_by_type(&doc_type.to_string())?;
            for doc in docs {
                pairs.push((doc.id, doc.filepath));
            }
        }
        
        Ok(pairs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::Database;
    

    fn setup_service() -> DatabaseService {
        let db = Database::new(":memory:").expect("Failed to create test database");
        DatabaseService::new(db.into_repository())
    }

    fn create_test_document() -> NewDocument {
        NewDocument {
            filepath: "/test/doc.md".to_string(),
            id: "test-doc-1".to_string(),
            title: "Test Document".to_string(),
            document_type: "vision".to_string(),
            created_at: 1609459200.0,
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
    fn test_database_service_crud() {
        let mut service = setup_service();
        
        // Create
        let new_doc = create_test_document();
        let created = service.create_document(new_doc).expect("Failed to create");
        assert_eq!(created.id, "test-doc-1");
        
        // Read
        let found = service.find_by_id("test-doc-1")
            .expect("Failed to find")
            .expect("Document not found");
        assert_eq!(found.filepath, "/test/doc.md");
        
        // Update
        let mut updated_doc = found.clone();
        updated_doc.title = "Updated Title".to_string();
        let updated = service.update_document("/test/doc.md", &updated_doc)
            .expect("Failed to update");
        assert_eq!(updated.title, "Updated Title");
        
        // Delete
        let deleted = service.delete_document("/test/doc.md")
            .expect("Failed to delete");
        assert!(deleted);
        
        // Verify deleted
        assert!(!service.document_exists("/test/doc.md").expect("Failed to check existence"));
    }

    #[test]
    fn test_database_service_relationships() {
        let mut service = setup_service();
        
        // Create parent and child documents
        let parent = NewDocument {
            id: "parent-1".to_string(),
            filepath: "/parent.md".to_string(),
            document_type: "strategy".to_string(),
            ..create_test_document()
        };
        
        let child = NewDocument {
            id: "child-1".to_string(),
            filepath: "/child.md".to_string(),
            document_type: "initiative".to_string(),
            ..create_test_document()
        };
        
        service.create_document(parent).expect("Failed to create parent");
        service.create_document(child).expect("Failed to create child");
        
        // Create relationship
        service.create_relationship("parent-1", "child-1", "/parent.md", "/child.md")
            .expect("Failed to create relationship");
        
        // Test find children
        let children = service.find_children("parent-1").expect("Failed to find children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child-1");
        
        // Test find parent
        let parent = service.find_parent("child-1")
            .expect("Failed to find parent")
            .expect("Parent not found");
        assert_eq!(parent.id, "parent-1");
    }
}