use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A Vision document represents the high-level direction and goals
#[derive(Debug)]
pub struct Vision {
    core: super::traits::DocumentCore,
}

impl Vision {
    /// Create a new Vision document with content rendered from template
    pub fn new(
        title: String,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Result<Self, DocumentValidationError> {
        // Create fresh metadata
        let metadata = DocumentMetadata::new();

        // Render the content template
        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("vision_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("vision_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,        // Visions have no parents
                blocked_by: Vec::new(), // Visions cannot be blocked
                tags,
                archived,
            },
        })
    }

    /// Create a Vision document from existing data (used when loading from file)
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Self {
        Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id: None,        // Visions have no parents
                blocked_by: Vec::new(), // Visions cannot be blocked
                tags,
                archived,
            },
        }
    }

    /// Create a Vision document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        Self::from_content(&raw_content)
    }

    /// Create a Vision document from raw file content string
    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        // Parse frontmatter and content
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);

        // Extract frontmatter data
        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

        // Parse frontmatter into structured data
        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => {
                return Err(DocumentValidationError::InvalidContent(
                    "Frontmatter must be a hash/map".to_string(),
                ))
            }
        };

        // Extract required fields
        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        // Parse timestamps
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met =
            FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        // Parse tags
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        // Verify this is actually a vision document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "vision" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'vision', found '{}'",
                level
            )));
        }

        // Create metadata and content
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(title, metadata, content, tags, archived))
    }

    /// Get the next phase in the Vision sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Draft => Some(Review),
            Review => Some(Published),
            Published => None, // Final phase
            _ => None,         // Invalid phase for Vision
        }
    }

    /// Update the phase tag in the document's tags
    fn update_phase_tag(&mut self, new_phase: Phase) {
        // Remove any existing phase tags
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        // Add the new phase tag
        self.core.tags.push(Tag::Phase(new_phase));
        // Update timestamp
        self.core.metadata.updated_at = chrono::Utc::now();
    }

    /// Write the Vision document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Vision document to its markdown string representation using templates
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();

        // Add the frontmatter template to Tera
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        // Create context with all document data
        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );

        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        // Render frontmatter
        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        // Use the actual content body
        let content_body = &self.content().body;

        // Use actual acceptance criteria if present, otherwise empty string
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
        } else {
            String::new()
        };

        // Combine everything
        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
    }
}

impl Document for Vision {
    // id() is provided by the trait with default implementation

    fn document_type(&self) -> DocumentType {
        DocumentType::Vision
    }

    fn title(&self) -> &str {
        &self.core.title
    }

    fn metadata(&self) -> &DocumentMetadata {
        &self.core.metadata
    }

    fn content(&self) -> &DocumentContent {
        &self.core.content
    }

    fn core(&self) -> &super::traits::DocumentCore {
        &self.core
    }

    fn can_transition_to(&self, phase: Phase) -> bool {
        if let Ok(current_phase) = self.phase() {
            use Phase::*;
            matches!(
                (current_phase, phase),
                (Draft, Review) | (Review, Published)
            )
        } else {
            false // Can't transition if we can't determine current phase
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        None // Visions never have parents
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[] // Visions can never be blocked
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        // Vision-specific validation rules
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Vision title cannot be empty".to_string(),
            ));
        }

        if self.parent_id().is_some() {
            return Err(DocumentValidationError::InvalidParent(
                "Visions cannot have parents".to_string(),
            ));
        }

        if !self.blocked_by().is_empty() {
            return Err(DocumentValidationError::InvalidContent(
                "Visions cannot be blocked by other documents".to_string(),
            ));
        }

        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
        // Check if all acceptance criteria checkboxes are checked
        // This would typically parse the content for checkbox completion
        // For now, return false as a placeholder
        false
    }

    fn template(&self) -> DocumentTemplate {
        DocumentTemplate {
            frontmatter: self.frontmatter_template(),
            content: self.content_template(),
            acceptance_criteria: self.acceptance_criteria_template(),
            file_extension: "md",
        }
    }

    fn frontmatter_template(&self) -> &'static str {
        include_str!("frontmatter.yaml")
    }

    fn content_template(&self) -> &'static str {
        include_str!("content.md")
    }

    fn acceptance_criteria_template(&self) -> &'static str {
        include_str!("acceptance_criteria.md")
    }

    fn transition_phase(
        &mut self,
        target_phase: Option<Phase>,
    ) -> Result<Phase, DocumentValidationError> {
        let current_phase = self.phase()?;

        let new_phase = match target_phase {
            Some(phase) => {
                // Validate the transition is allowed
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => {
                // Auto-transition to next phase in sequence
                match Self::next_phase_in_sequence(current_phase) {
                    Some(next) => next,
                    None => return Ok(current_phase), // Already at final phase
                }
            }
        };

        self.update_phase_tag(new_phase);
        Ok(new_phase)
    }

    fn core_mut(&mut self) -> &mut super::traits::DocumentCore {
        &mut self.core
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::documents::traits::DocumentValidationError;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_vision_from_content() {
        let content = r##"---
id: test-vision
level: vision
title: "Test Vision"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false

tags:
  - "#vision"
  - "#phase/draft"

exit_criteria_met: false
---

# Test Vision

## Purpose

This is a test vision for our system.

## Current State

We are here.

## Future State

We want to be there.

## Acceptance Criteria

- [ ] Purpose is clearly defined
- [ ] Current and future states are documented
"##;

        let vision = Vision::from_content(content).unwrap();

        assert_eq!(vision.title(), "Test Vision");
        assert_eq!(vision.document_type(), DocumentType::Vision);
        assert!(!vision.archived());
        assert_eq!(vision.tags().len(), 2);
        assert_eq!(vision.phase().unwrap(), Phase::Draft);
        assert!(vision.content().has_acceptance_criteria());

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");

        vision.to_file(&file_path).await.unwrap();
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_vision.title(), vision.title());
        assert_eq!(loaded_vision.phase().unwrap(), vision.phase().unwrap());
        assert_eq!(loaded_vision.content().body, vision.content().body);
        assert_eq!(loaded_vision.archived(), vision.archived());
        assert_eq!(loaded_vision.tags().len(), vision.tags().len());
    }

    #[test]
    fn test_vision_invalid_level() {
        let content = r##"---
id: test-doc
level: strategy
title: "Test Strategy"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
---

# Test Strategy
"##;

        let result = Vision::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'vision'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_vision_missing_title() {
        let content = r##"---
id: test-vision
level: vision
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
---

Some content without a title in frontmatter.
"##;

        let result = Vision::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::MissingRequiredField(field) => {
                assert_eq!(field, "title");
            }
            _ => panic!("Expected MissingRequiredField error"),
        }
    }

    #[tokio::test]
    async fn test_vision_tag_parsing() {
        let content = r##"---
id: test-vision
level: vision
title: "Test Vision"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
tags:
  - "#vision"
  - "#phase/review"
  - "#high-priority"
  - "urgent"
exit_criteria_met: false
---

# Test Vision
"##;

        let vision = Vision::from_content(content).unwrap();
        let tags = vision.tags();

        assert_eq!(tags.len(), 4);
        assert!(tags.contains(&Tag::Label("vision".to_string())));
        assert!(tags.contains(&Tag::Phase(Phase::Review)));
        assert!(tags.contains(&Tag::Label("high-priority".to_string())));
        assert!(tags.contains(&Tag::Label("urgent".to_string())));

        assert_eq!(vision.phase().unwrap(), Phase::Review);

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");

        vision.to_file(&file_path).await.unwrap();
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_vision.title(), vision.title());
        assert_eq!(loaded_vision.phase().unwrap(), vision.phase().unwrap());
        assert_eq!(loaded_vision.tags().len(), vision.tags().len());

        // Verify specific tags survived the round-trip
        let loaded_tags = loaded_vision.tags();
        assert!(loaded_tags.contains(&Tag::Label("vision".to_string())));
        assert!(loaded_tags.contains(&Tag::Phase(Phase::Review)));
        assert!(loaded_tags.contains(&Tag::Label("high-priority".to_string())));
        assert!(loaded_tags.contains(&Tag::Label("urgent".to_string())));
    }

    #[tokio::test]
    async fn test_vision_validation() {
        let vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Label("vision".to_string()), Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        assert!(vision.validate().is_ok());
        assert_eq!(vision.parent_id(), None);
        assert_eq!(vision.blocked_by().len(), 0);

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");

        // Write to file
        vision.to_file(&file_path).await.unwrap();

        // Read back from file
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();

        // Verify all fields match
        assert_eq!(loaded_vision.title(), vision.title());
        assert_eq!(loaded_vision.phase().unwrap(), vision.phase().unwrap());
        assert_eq!(loaded_vision.content().body, vision.content().body);
        assert_eq!(loaded_vision.archived(), vision.archived());
        assert_eq!(loaded_vision.tags().len(), vision.tags().len());
    }

    #[tokio::test]
    async fn test_vision_phase_transitions() {
        let vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        assert!(vision.can_transition_to(Phase::Review));
        assert!(!vision.can_transition_to(Phase::Published));
        assert!(!vision.can_transition_to(Phase::Active));

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");

        vision.to_file(&file_path).await.unwrap();
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_vision.title(), vision.title());
        assert_eq!(loaded_vision.phase().unwrap(), vision.phase().unwrap());
        assert!(loaded_vision.can_transition_to(Phase::Review));
        assert!(!loaded_vision.can_transition_to(Phase::Published));
        assert!(!loaded_vision.can_transition_to(Phase::Active));
    }

    #[tokio::test]
    async fn test_vision_transition_phase_auto() {
        let mut vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        // Auto-transition from Draft should go to Review
        let new_phase = vision.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);
        assert_eq!(vision.phase().unwrap(), Phase::Review);

        // Round-trip test after first transition
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");
        vision.to_file(&file_path).await.unwrap();
        let mut loaded_vision = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision.phase().unwrap(), Phase::Review);

        // Auto-transition from Review should go to Published
        let new_phase = loaded_vision.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
        assert_eq!(loaded_vision.phase().unwrap(), Phase::Published);

        // Round-trip test after second transition
        loaded_vision.to_file(&file_path).await.unwrap();
        let mut loaded_vision2 = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision2.phase().unwrap(), Phase::Published);

        // Auto-transition from Published should stay at Published (final phase)
        let new_phase = loaded_vision2.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
        assert_eq!(loaded_vision2.phase().unwrap(), Phase::Published);

        // Final round-trip test
        loaded_vision2.to_file(&file_path).await.unwrap();
        let loaded_vision3 = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision3.phase().unwrap(), Phase::Published);
    }

    #[tokio::test]
    async fn test_vision_transition_phase_explicit() {
        let mut vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        // Explicit transition from Draft to Review
        let new_phase = vision.transition_phase(Some(Phase::Review)).unwrap();
        assert_eq!(new_phase, Phase::Review);
        assert_eq!(vision.phase().unwrap(), Phase::Review);

        // Round-trip test after first transition
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");
        vision.to_file(&file_path).await.unwrap();
        let mut loaded_vision = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision.phase().unwrap(), Phase::Review);

        // Explicit transition from Review to Published
        let new_phase = loaded_vision
            .transition_phase(Some(Phase::Published))
            .unwrap();
        assert_eq!(new_phase, Phase::Published);
        assert_eq!(loaded_vision.phase().unwrap(), Phase::Published);

        // Final round-trip test
        loaded_vision.to_file(&file_path).await.unwrap();
        let loaded_vision2 = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision2.phase().unwrap(), Phase::Published);
    }

    #[tokio::test]
    async fn test_vision_transition_phase_invalid() {
        let mut vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        // Invalid transition from Draft to Published (must go through Review)
        let result = vision.transition_phase(Some(Phase::Published));
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidPhaseTransition { from, to } => {
                assert_eq!(from, Phase::Draft);
                assert_eq!(to, Phase::Published);
            }
            _ => panic!("Expected InvalidPhaseTransition error"),
        }

        // Should still be in Draft phase
        assert_eq!(vision.phase().unwrap(), Phase::Draft);

        // Round-trip test to ensure vision is still valid after failed transition
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");
        vision.to_file(&file_path).await.unwrap();
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_vision.phase().unwrap(), Phase::Draft);
    }

    #[tokio::test]
    async fn test_vision_update_section() {
        // First create a vision with the template
        let mut vision = Vision::new(
            "Test Vision".to_string(),
            vec![Tag::Phase(Phase::Draft)],
            false,
        )
        .expect("Failed to create vision");

        // Then update its content to have specific test content
        vision.core_mut().content = DocumentContent::new(
            "## Purpose\n\nOriginal purpose\n\n## Current State\n\nOriginal state",
        );

        // Replace existing section
        vision
            .update_section("Updated purpose content", "Purpose", false)
            .unwrap();
        let content = vision.content().body.clone();
        assert!(content.contains("## Purpose\n\nUpdated purpose content"));
        assert!(!content.contains("Original purpose"));

        // Append to existing section
        vision
            .update_section("Additional state info", "Current State", true)
            .unwrap();
        let content = vision.content().body.clone();
        assert!(content.contains("Original state"));
        assert!(content.contains("Additional state info"));

        // Add new section
        vision
            .update_section("Future vision details", "Future State", false)
            .unwrap();
        let content = vision.content().body.clone();
        assert!(content.contains("## Future State\n\nFuture vision details"));

        // Round-trip test to ensure all updates persist
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-vision.md");
        vision.to_file(&file_path).await.unwrap();
        let loaded_vision = Vision::from_file(&file_path).await.unwrap();

        let loaded_content = loaded_vision.content().body.clone();
        assert!(loaded_content.contains("## Purpose\n\nUpdated purpose content"));
        assert!(loaded_content.contains("Original state"));
        assert!(loaded_content.contains("Additional state info"));
        assert!(loaded_content.contains("## Future State\n\nFuture vision details"));
    }
}
