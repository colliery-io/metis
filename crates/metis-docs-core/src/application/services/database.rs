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
    pub fn create_relationship(
        &mut self,
        parent_id: &str,
        child_id: &str,
        parent_filepath: &str,
        child_filepath: &str,
    ) -> Result<()> {
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

    /// Get all documents belonging to a strategy
    pub fn find_by_strategy_id(&mut self, strategy_id: &str) -> Result<Vec<Document>> {
        self.repository.find_by_strategy_id(strategy_id)
    }

    /// Get all documents belonging to an initiative
    pub fn find_by_initiative_id(&mut self, initiative_id: &str) -> Result<Vec<Document>> {
        self.repository.find_by_initiative_id(initiative_id)
    }

    /// Get all documents in a strategy hierarchy (strategy + its initiatives + their tasks)
    pub fn find_strategy_hierarchy(&mut self, strategy_id: &str) -> Result<Vec<Document>> {
        self.repository.find_strategy_hierarchy(strategy_id)
    }

    /// Get all documents in an initiative hierarchy (initiative + its tasks)
    pub fn find_initiative_hierarchy(&mut self, initiative_id: &str) -> Result<Vec<Document>> {
        self.repository.find_initiative_hierarchy(initiative_id)
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
            strategy_id: None,
            initiative_id: None,
        }
    }

    fn create_test_document_with_lineage(
        id: &str, 
        doc_type: &str, 
        filepath: &str,
        strategy_id: Option<String>,
        initiative_id: Option<String>
    ) -> NewDocument {
        NewDocument {
            filepath: filepath.to_string(),
            id: id.to_string(),
            title: format!("Test {}", doc_type),
            document_type: doc_type.to_string(),
            created_at: 1609459200.0,
            updated_at: 1609459200.0,
            archived: false,
            exit_criteria_met: false,
            file_hash: "abc123".to_string(),
            frontmatter_json: "{}".to_string(),
            content: Some("Test content".to_string()),
            phase: "draft".to_string(),
            strategy_id,
            initiative_id,
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
        let found = service
            .find_by_id("test-doc-1")
            .expect("Failed to find")
            .expect("Document not found");
        assert_eq!(found.filepath, "/test/doc.md");

        // Update
        let mut updated_doc = found.clone();
        updated_doc.title = "Updated Title".to_string();
        let updated = service
            .update_document("/test/doc.md", &updated_doc)
            .expect("Failed to update");
        assert_eq!(updated.title, "Updated Title");

        // Delete
        let deleted = service
            .delete_document("/test/doc.md")
            .expect("Failed to delete");
        assert!(deleted);

        // Verify deleted
        assert!(!service
            .document_exists("/test/doc.md")
            .expect("Failed to check existence"));
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

        service
            .create_document(parent)
            .expect("Failed to create parent");
        service
            .create_document(child)
            .expect("Failed to create child");

        // Create relationship
        service
            .create_relationship("parent-1", "child-1", "/parent.md", "/child.md")
            .expect("Failed to create relationship");

        // Test find children
        let children = service
            .find_children("parent-1")
            .expect("Failed to find children");
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, "child-1");

        // Test find parent
        let parent = service
            .find_parent("child-1")
            .expect("Failed to find parent")
            .expect("Parent not found");
        assert_eq!(parent.id, "parent-1");
    }

    #[test]
    fn test_lineage_queries() {
        let mut service = setup_service();

        // Create strategy
        let strategy = create_test_document_with_lineage(
            "strategy-1", 
            "strategy", 
            "/strategies/strategy-1/strategy.md",
            None, 
            None
        );
        service.create_document(strategy).expect("Failed to create strategy");

        // Create initiative under strategy
        let initiative = create_test_document_with_lineage(
            "initiative-1", 
            "initiative", 
            "/strategies/strategy-1/initiatives/initiative-1/initiative.md",
            Some("strategy-1".to_string()), 
            None
        );
        service.create_document(initiative).expect("Failed to create initiative");

        // Create tasks under initiative
        let task1 = create_test_document_with_lineage(
            "task-1", 
            "task", 
            "/strategies/strategy-1/initiatives/initiative-1/tasks/task-1.md",
            Some("strategy-1".to_string()), 
            Some("initiative-1".to_string())
        );
        let task2 = create_test_document_with_lineage(
            "task-2", 
            "task", 
            "/strategies/strategy-1/initiatives/initiative-1/tasks/task-2.md",
            Some("strategy-1".to_string()), 
            Some("initiative-1".to_string())
        );
        service.create_document(task1).expect("Failed to create task1");
        service.create_document(task2).expect("Failed to create task2");

        // Test find by strategy ID
        let strategy_docs = service.find_by_strategy_id("strategy-1").expect("Failed to find by strategy");
        assert_eq!(strategy_docs.len(), 3); // initiative + 2 tasks

        // Test find by initiative ID
        let initiative_docs = service.find_by_initiative_id("initiative-1").expect("Failed to find by initiative");
        assert_eq!(initiative_docs.len(), 2); // 2 tasks

        // Test strategy hierarchy (should include strategy itself + its children)
        let strategy_hierarchy = service.find_strategy_hierarchy("strategy-1").expect("Failed to find strategy hierarchy");
        assert_eq!(strategy_hierarchy.len(), 4); // strategy + initiative + 2 tasks

        // Test initiative hierarchy (should include initiative itself + its tasks)
        let initiative_hierarchy = service.find_initiative_hierarchy("initiative-1").expect("Failed to find initiative hierarchy");
        assert_eq!(initiative_hierarchy.len(), 3); // initiative + 2 tasks

        // Verify document types in strategy hierarchy
        let doc_types: Vec<&str> = strategy_hierarchy.iter().map(|d| d.document_type.as_str()).collect();
        assert!(doc_types.contains(&"strategy"));
        assert!(doc_types.contains(&"initiative"));
        assert!(doc_types.iter().filter(|&&t| t == "task").count() == 2);
    }
}
