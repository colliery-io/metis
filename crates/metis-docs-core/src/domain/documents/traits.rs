use super::types::{DocumentId, DocumentType, Phase, Tag};
use super::metadata::DocumentMetadata;
use super::content::DocumentContent;
use chrono::{DateTime, Utc};

/// Core document trait that all document types must implement
pub trait Document {
    /// Get the unique identifier for this document (derived from title)
    fn id(&self) -> DocumentId {
        DocumentId::from_title(self.title())
    }
    
    /// Get the document type
    fn document_type(&self) -> DocumentType;
    
    /// Get the document title
    fn title(&self) -> &str;
    
    /// Get the document metadata
    fn metadata(&self) -> &DocumentMetadata;
    
    /// Get the document content
    fn content(&self) -> &DocumentContent;
    
    /// Get access to the core document data
    fn core(&self) -> &DocumentCore;
    
    /// Get the document tags
    fn tags(&self) -> &[Tag] {
        &self.core().tags
    }
    
    /// Get the current phase of the document (parsed from tags)
    fn phase(&self) -> Result<Phase, DocumentValidationError> {
        // Find the first Phase tag in the tags list
        for tag in self.tags() {
            if let Tag::Phase(phase) = tag {
                return Ok(*phase);
            }
        }
        // No phase tag found - this is an error
        Err(DocumentValidationError::MissingPhaseTag)
    }
    
    /// Check if this document can transition to the given phase
    fn can_transition_to(&self, phase: Phase) -> bool;
    
    /// Check if this document is archived
    fn archived(&self) -> bool {
        self.core().archived
    }
    
    /// Get the parent document ID if this document has a parent
    fn parent_id(&self) -> Option<&DocumentId>;
    
    /// Get IDs of documents that block this one
    fn blocked_by(&self) -> &[DocumentId];
    
    /// Validate the document according to its type-specific rules
    fn validate(&self) -> Result<(), DocumentValidationError>;
    
    /// Check if exit criteria are met
    fn exit_criteria_met(&self) -> bool;
    
    /// Get the template for rendering this document type
    fn template(&self) -> DocumentTemplate;
    
    /// Get the frontmatter template for this document type
    fn frontmatter_template(&self) -> &'static str;
    
    /// Get the content template for this document type
    fn content_template(&self) -> &'static str;
    
    /// Get the acceptance criteria template for this document type
    fn acceptance_criteria_template(&self) -> &'static str;
}

/// Template information for a document
pub struct DocumentTemplate {
    pub frontmatter: &'static str,
    pub content: &'static str,
    pub acceptance_criteria: &'static str,
    pub file_extension: &'static str,
}

/// Common document data that all document types share
#[derive(Debug)]
pub struct DocumentCore {
    pub title: String,
    pub metadata: DocumentMetadata,
    pub content: DocumentContent,
    pub parent_id: Option<DocumentId>,
    pub blocked_by: Vec<DocumentId>,
    pub tags: Vec<Tag>,
    pub archived: bool,
}

/// Validation errors for documents
#[derive(Debug, PartialEq, thiserror::Error)]
pub enum DocumentValidationError {
    #[error("Invalid title: {0}")]
    InvalidTitle(String),
    
    #[error("Invalid parent: {0}")]
    InvalidParent(String),
    
    #[error("Invalid phase transition from {from:?} to {to:?}")]
    InvalidPhaseTransition { from: Phase, to: Phase },
    
    #[error("Missing required field: {0}")]
    MissingRequiredField(String),
    
    #[error("Invalid content: {0}")]
    InvalidContent(String),
    
    #[error("Missing phase tag in document")]
    MissingPhaseTag,
}