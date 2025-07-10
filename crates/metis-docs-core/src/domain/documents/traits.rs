use super::content::DocumentContent;
use super::metadata::DocumentMetadata;
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;

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

    /// Transition to the next phase in sequence, or to a specific phase if provided
    fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError>;

    /// Update a specific section (H2 heading) in the document content
    fn update_section(
        &mut self,
        content: &str,
        heading: &str,
        append: bool,
    ) -> Result<(), DocumentValidationError> {
        let lines: Vec<&str> = self.core().content.body.lines().collect();
        let target_heading = format!("## {}", heading);

        // Find the section start
        let section_start = lines.iter().position(|line| line.trim() == target_heading);

        let new_body = if let Some(section_start) = section_start {
            // Section exists, update it
            let section_end = lines[section_start + 1..]
                .iter()
                .position(|line| line.trim_start().starts_with("## "))
                .map(|pos| section_start + 1 + pos)
                .unwrap_or(lines.len());

            // Build the updated content
            let mut updated_lines = Vec::new();

            // Add content before the section
            updated_lines.extend_from_slice(&lines[..section_start + 1]);

            if append {
                // For append mode, keep existing content and add new content
                updated_lines.extend_from_slice(&lines[section_start + 1..section_end]);
                if !content.trim().is_empty() {
                    if section_end > section_start + 1 {
                        updated_lines.push(""); // Add blank line before new content
                    }
                    for line in content.lines() {
                        updated_lines.push(line);
                    }
                }
            } else {
                // For replace mode, replace section content entirely
                if !content.trim().is_empty() {
                    updated_lines.push(""); // Empty line after heading
                    for line in content.lines() {
                        updated_lines.push(line);
                    }
                }
            }

            // Add content after the section
            if section_end < lines.len() {
                updated_lines.push(""); // Empty line before next section
                updated_lines.extend_from_slice(&lines[section_end..]);
            }

            updated_lines.join("\n")
        } else {
            // Section doesn't exist, add new section
            let mut updated_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();

            if !updated_lines.is_empty() {
                updated_lines.push("".to_string()); // Empty line before new section
            }
            updated_lines.push(target_heading);
            if !content.trim().is_empty() {
                updated_lines.push("".to_string()); // Empty line after heading
                for line in content.lines() {
                    updated_lines.push(line.to_string());
                }
            }

            updated_lines.join("\n")
        };

        self.update_content_body(new_body)
    }

    /// Helper method for update_section to actually mutate the content
    fn update_content_body(&mut self, new_body: String) -> Result<(), DocumentValidationError> {
        // We need mutable access to core, which requires each document type to provide access
        let core = self.core_mut();
        core.content.body = new_body;
        core.metadata.updated_at = Utc::now();
        Ok(())
    }

    /// Get mutable access to the document core (needed for updates)
    fn core_mut(&mut self) -> &mut DocumentCore;

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
