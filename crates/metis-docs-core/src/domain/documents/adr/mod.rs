use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use super::metadata::DocumentMetadata;
use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use std::path::Path;
use gray_matter;
use chrono::{DateTime, Utc};
use tera::{Tera, Context};

/// An ADR (Architecture Decision Record) documents architectural decisions
#[derive(Debug)]
pub struct Adr {
    core: super::traits::DocumentCore,
    number: u32,
    decision_maker: String,
    decision_date: DateTime<Utc>,
}

impl Adr {
    /// Create a new ADR document from parsed file data
    pub fn new(
        number: u32,
        title: String,
        decision_maker: String,
        decision_date: DateTime<Utc>,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
    ) -> Self {
        Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id, // ADRs can reference the work that generated the need for decision
                blocked_by: Vec::new(), // ADRs cannot be blocked
                tags,
                archived,
            },
            number,
            decision_maker,
            decision_date,
        }
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn decision_maker(&self) -> &str {
        &self.decision_maker
    }

    pub fn decision_date(&self) -> DateTime<Utc> {
        self.decision_date
    }

    /// Create an ADR document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e)))?;

        Self::from_content(&raw_content)
    }

    /// Create an ADR document from raw file content string
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

        // Verify this is actually an ADR document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "adr" {
            return Err(DocumentValidationError::InvalidContent(
                format!("Expected level 'adr', found '{}'", level)
            ));
        }

        // Extract ADR-specific fields
        let number = FrontmatterParser::extract_integer(&fm_map, "number")? as u32;
        let decision_maker = FrontmatterParser::extract_string(&fm_map, "decision_maker")?;
        let decision_date = FrontmatterParser::extract_datetime(&fm_map, "decision_date")?;
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent").ok().map(DocumentId::from);

        // Create metadata and content
        let metadata = DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::new(number, title, decision_maker, decision_date, metadata, content, 
                     parent_id, tags, archived))
    }

    /// Write the ADR document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e)))
    }

    /// Convert the ADR document to its markdown string representation using templates
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
        context.insert("parent_id", &self.parent_id().map(|id| id.to_string()).unwrap_or_default());
        context.insert("number", &self.number);
        context.insert("decision_maker", &self.decision_maker);
        context.insert("decision_date", &self.decision_date.to_rfc3339());
        
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

impl Document for Adr {
    /// ADRs have special ID format: number-slug
    fn id(&self) -> DocumentId {
        let slug = DocumentId::from_title(self.title()).to_string();
        DocumentId::new(&format!("{:03}-{}", self.number, slug))
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Adr
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
                (Draft, Discussion) => true,
                (Discussion, Decided) => true,
                (Decided, Superseded) => true,
                _ => false,
            }
        } else {
            false // Can't transition if we can't determine current phase
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[] // ADRs can never be blocked
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        // ADR-specific validation rules
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "ADR title cannot be empty".to_string(),
            ));
        }

        if self.decision_maker.trim().is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "decision_maker is required for ADRs".to_string(),
            ));
        }

        if !self.blocked_by().is_empty() {
            return Err(DocumentValidationError::InvalidContent(
                "ADRs cannot be blocked by other documents".to_string(),
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
    fn test_adr_from_content() {
        let content = r##"---
id: test-adr
level: adr
title: "Use React for Frontend"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
number: 1
decision_maker: "Architecture Team"
decision_date: 2025-01-01T12:00:00Z
parent: initiative-001

tags:
  - "#adr"
  - "#phase/decided"

exit_criteria_met: false
---

# ADR-001: Use React for Frontend

## Context

We need to choose a frontend framework for our application.

## Decision

We will use React as our frontend framework.

## Consequences

This will allow us to leverage a large ecosystem.
"##;

        let adr = Adr::from_content(content).unwrap();
        
        assert_eq!(adr.title(), "Use React for Frontend");
        assert_eq!(adr.document_type(), DocumentType::Adr);
        assert!(!adr.archived());
        assert_eq!(adr.tags().len(), 2);
        assert_eq!(adr.phase().unwrap(), Phase::Decided);
        assert_eq!(adr.number(), 1);
        assert_eq!(adr.decision_maker(), "Architecture Team");
        assert_eq!(adr.id().to_string(), "001-use-react-for-frontend");
    }


    #[test]
    fn test_adr_special_id_format() {
        let adr = Adr::new(
            42,
            "Use Docker for Containerization".to_string(),
            "DevOps Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        assert_eq!(adr.id().to_string(), "042-use-docker-for-containerization");
        assert_eq!(adr.number(), 42);
    }

    #[test]
    fn test_adr_invalid_level() {
        let content = r##"---
id: test-doc
level: strategy
title: "Test Strategy"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
number: 1
decision_maker: "Team"
decision_date: 2025-01-01T12:00:00Z
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
---

# Test Strategy
"##;

        let result = Adr::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'adr'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_adr_validation() {
        let adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        assert!(adr.validate().is_ok());
        
        // Test validation failure - empty decision maker
        let adr_no_decision_maker = Adr::new(
            1,
            "Test Decision".to_string(),
            "".to_string(), // Empty decision maker
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );
        
        assert!(adr_no_decision_maker.validate().is_err());
    }

    #[test]
    fn test_adr_cannot_be_blocked() {
        let adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        assert_eq!(adr.blocked_by().len(), 0);
        assert!(adr.validate().is_ok());
    }

    #[test]
    fn test_adr_phase_transitions() {
        let discussion_adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Discussion)],
            false,
        );

        assert!(discussion_adr.can_transition_to(Phase::Decided));
        assert!(!discussion_adr.can_transition_to(Phase::Active));
        assert!(!discussion_adr.can_transition_to(Phase::Draft));

        let decided_adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        assert!(decided_adr.can_transition_to(Phase::Superseded));
        assert!(!decided_adr.can_transition_to(Phase::Discussion));
    }

    #[test]
    fn test_adr_number_formatting() {
        let adr1 = Adr::new(
            1,
            "First Decision".to_string(),
            "Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        let adr42 = Adr::new(
            42,
            "Another Decision".to_string(),
            "Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        let adr999 = Adr::new(
            999,
            "Big Decision".to_string(),
            "Team".to_string(),
            Utc::now(),
            DocumentMetadata::new(),
            DocumentContent::new("Test content"),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
        );

        assert_eq!(adr1.id().to_string(), "001-first-decision");
        assert_eq!(adr42.id().to_string(), "042-another-decision");
        assert_eq!(adr999.id().to_string(), "999-big-decision");
    }
}