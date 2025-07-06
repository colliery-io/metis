use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use super::metadata::DocumentMetadata;
use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use std::path::Path;
use gray_matter;
use chrono::{DateTime, Utc};
use tera::{Tera, Context};

/// A Vision document represents the high-level direction and goals
#[derive(Debug)]
pub struct Vision {
    core: super::traits::DocumentCore,
}

impl Vision {
    /// Create a new Vision document from parsed file data
    pub fn new(
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
                parent_id: None, // Visions have no parents
                blocked_by: Vec::new(), // Visions cannot be blocked
                tags,
                archived,
            },
        }
    }

    /// Create a Vision document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e)))?;

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
            _ => return Err(DocumentValidationError::InvalidContent("Frontmatter must be a hash/map".to_string())),
        };

        // Extract required fields
        let title = FrontmatterParser::extract_string(&fm_map, "title")?;
        let archived = FrontmatterParser::extract_bool(&fm_map, "archived").unwrap_or(false);

        // Parse timestamps
        let created_at = FrontmatterParser::extract_datetime(&fm_map, "created_at")?;
        let updated_at = FrontmatterParser::extract_datetime(&fm_map, "updated_at")?;
        let exit_criteria_met = FrontmatterParser::extract_bool(&fm_map, "exit_criteria_met").unwrap_or(false);

        // Parse tags
        let tags = FrontmatterParser::extract_tags(&fm_map)?;

        // Verify this is actually a vision document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "vision" {
            return Err(DocumentValidationError::InvalidContent(
                format!("Expected level 'vision', found '{}'", level)
            ));
        }

        // Create metadata and content
        let metadata = DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::new(title, metadata, content, tags, archived))
    }



    /// Write the Vision document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e)))
    }

    /// Convert the Vision document to its markdown string representation using templates
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();
        
        // Add the templates to Tera
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        tera.add_raw_template("content", self.content_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        tera.add_raw_template("acceptance_criteria", self.acceptance_criteria_template())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Template error: {}", e)))?;
        
        // Create context with all document data
        let mut context = Context::new();
        context.insert("slug", &self.id().to_string());
        context.insert("title", self.title());
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived());
        context.insert("exit_criteria_met", &self.metadata().exit_criteria_met);
        
        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);
        
        // Render frontmatter
        let frontmatter = tera.render("frontmatter", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e)))?;
        
        // Add content body and acceptance criteria to context
        context.insert("body", &self.content().body);
        context.insert("acceptance_criteria_content", &self.content().acceptance_criteria.as_deref().unwrap_or(""));
        
        // Render content
        let content_body = tera.render("content", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Content render error: {}", e)))?;
        
        // Render acceptance criteria
        let acceptance_criteria = tera.render("acceptance_criteria", &context)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Acceptance criteria render error: {}", e)))?;
        
        // Combine everything
        Ok(format!("---\n{}---\n\n{}\n\n{}", frontmatter, content_body, acceptance_criteria))
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
            match (current_phase, phase) {
                (Draft, Review) => true,
                (Review, Published) => true,
                _ => false,
            }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vision_from_content() {
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

    #[test]
    fn test_vision_tag_parsing() {
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
    }

    #[test]
    fn test_vision_validation() {
        let vision = Vision::new(
            "Test Vision".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            vec![Tag::Label("vision".to_string()), Tag::Phase(Phase::Draft)],
            false,
        );

        assert!(vision.validate().is_ok());
        assert_eq!(vision.parent_id(), None);
        assert_eq!(vision.blocked_by().len(), 0);
    }

    #[test]
    fn test_vision_phase_transitions() {
        let vision = Vision::new(
            "Test Vision".to_string(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            vec![Tag::Phase(Phase::Draft)],
            false,
        );

        assert!(vision.can_transition_to(Phase::Review));
        assert!(!vision.can_transition_to(Phase::Published));
        assert!(!vision.can_transition_to(Phase::Active));
    }
}