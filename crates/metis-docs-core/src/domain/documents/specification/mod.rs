use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// A Specification captures system-level or project-level design:
/// PRDs, requirements, system context, and architecture framing.
/// Specifications are attached documents (children of Vision or Initiative),
/// not hierarchy nodes — the Vision → Initiative → Task chain is unchanged.
#[derive(Debug)]
pub struct Specification {
    core: super::traits::DocumentCore,
}

impl Specification {
    /// Create a new Specification document with content rendered from template
    pub fn new(
        title: String,
        parent_id: DocumentId,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        let template_content = include_str!("content.md");
        Self::new_with_template(title, parent_id, tags, archived, short_code, template_content)
    }

    /// Create a new Specification document with a custom template
    pub fn new_with_template(
        title: String,
        parent_id: DocumentId,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
        template_content: &str,
    ) -> Result<Self, DocumentValidationError> {
        let metadata = DocumentMetadata::new(short_code);

        // Render the content template
        let mut tera = Tera::default();
        tera.add_raw_template("spec_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("spec_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id: Some(parent_id),
                blocked_by: Vec::new(),
                tags,
                archived,
                initiative_id: None, // Specifications are not part of the initiative hierarchy
            },
        })
    }

    /// Create a Specification from existing data (used when loading from file)
    pub fn from_parts(
        title: String,
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
                parent_id,
                blocked_by: Vec::new(),
                tags,
                archived,
                initiative_id: None,
            },
        }
    }

    /// Create a Specification document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        Self::from_content(&raw_content)
    }

    /// Create a Specification document from raw file content string
    pub fn from_content(raw_content: &str) -> Result<Self, DocumentValidationError> {
        let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(raw_content);

        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;

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

        // Verify this is actually a specification document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "specification" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'specification', found '{}'",
                level
            )));
        }

        // Extract parent
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent")
            .ok()
            .map(DocumentId::from);

        // Create metadata and content
        let short_code = FrontmatterParser::extract_string(&fm_map, "short_code")?;
        let metadata = DocumentMetadata::from_frontmatter(
            created_at,
            updated_at,
            exit_criteria_met,
            short_code,
        );
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title, metadata, content, parent_id, tags, archived,
        ))
    }

    /// Get the next phase in the Specification sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Discovery => Some(Drafting),
            Drafting => Some(Review),
            Review => Some(Published),
            Published => None, // Final phase (but content remains editable)
            _ => None,         // Invalid phase for Specification
        }
    }

    /// Update the phase tag in the document's tags
    fn update_phase_tag(&mut self, new_phase: Phase) {
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        self.core.tags.push(Tag::Phase(new_phase));
        self.core.metadata.updated_at = Utc::now();
    }

    /// Write the Specification document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Specification document to its markdown string representation
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();

        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("slug", &DocumentId::title_to_slug(self.title()));
        context.insert("title", self.title());
        context.insert("short_code", &self.metadata().short_code);
        context.insert("created_at", &self.metadata().created_at.to_rfc3339());
        context.insert("updated_at", &self.metadata().updated_at.to_rfc3339());
        context.insert("archived", &self.archived().to_string());
        context.insert(
            "exit_criteria_met",
            &self.metadata().exit_criteria_met.to_string(),
        );
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );

        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        // Add lineage fields (NULL for Specification documents)
        context.insert("initiative_id", "NULL");

        // Render frontmatter
        let frontmatter = tera.render("frontmatter", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Frontmatter render error: {}", e))
        })?;

        // Use the actual content body
        let content_body = &self.content().body;

        // Use actual acceptance criteria if present
        let acceptance_criteria = if let Some(ac) = &self.content().acceptance_criteria {
            format!("\n\n## Acceptance Criteria\n\n{}", ac)
        } else {
            String::new()
        };

        Ok(format!(
            "---\n{}\n---\n\n{}{}",
            frontmatter.trim_end(),
            content_body,
            acceptance_criteria
        ))
    }
}

impl Document for Specification {
    fn id(&self) -> DocumentId {
        DocumentId::from_title(self.title())
    }

    fn document_type(&self) -> DocumentType {
        DocumentType::Specification
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
            DocumentType::Specification.can_transition(current_phase, phase)
        } else {
            false
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &[] // Specifications cannot be blocked
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Specification title cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    fn exit_criteria_met(&self) -> bool {
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
                if !self.can_transition_to(phase) {
                    return Err(DocumentValidationError::InvalidPhaseTransition {
                        from: current_phase,
                        to: phase,
                    });
                }
                phase
            }
            None => {
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

    #[test]
    fn test_specification_from_content() {
        let content = r##"---
id: system-design-spec
level: specification
title: "System Design Specification"
short_code: TEST-S-0001
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
parent: TEST-V-0001
archived: false

tags:
  - "#specification"
  - "#phase/discovery"

exit_criteria_met: false
---

# System Design Specification

## Overview

This is the system design specification.

## Requirements

### Functional Requirements

| ID | Requirement | Rationale |
|----|-------------|-----------|
| REQ-1.1.1 | Support user authentication | Core system need |
"##;

        let spec = Specification::from_content(content).unwrap();

        assert_eq!(spec.title(), "System Design Specification");
        assert_eq!(spec.document_type(), DocumentType::Specification);
        assert!(!spec.archived());
        assert_eq!(spec.phase().unwrap(), Phase::Discovery);
        assert_eq!(
            spec.parent_id().unwrap().to_string(),
            "TEST-V-0001"
        );
    }

    #[test]
    fn test_specification_invalid_level() {
        let content = r##"---
id: test-doc
level: adr
title: "Test ADR"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
number: 1
decision_maker: "Team"
short_code: TEST-A-0001
tags:
  - "#adr"
  - "#phase/draft"
exit_criteria_met: false
---

# Test ADR
"##;

        let result = Specification::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'specification'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[test]
    fn test_specification_validation() {
        let spec = Specification::new(
            "Test Specification".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        assert!(spec.validate().is_ok());
    }

    #[test]
    fn test_specification_empty_title_validation() {
        let spec = Specification::from_parts(
            "".to_string(),
            DocumentMetadata::new("TEST-S-0001".to_string()),
            DocumentContent::new("content"),
            Some(DocumentId::new("TEST-V-0001")),
            vec![Tag::Phase(Phase::Discovery)],
            false,
        );

        assert!(spec.validate().is_err());
    }

    #[test]
    fn test_specification_cannot_be_blocked() {
        let spec = Specification::new(
            "Test Specification".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        assert_eq!(spec.blocked_by().len(), 0);
    }

    #[test]
    fn test_specification_phase_transitions() {
        let spec = Specification::new(
            "Test Specification".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        assert!(spec.can_transition_to(Phase::Drafting));
        assert!(!spec.can_transition_to(Phase::Review)); // Can't skip
        assert!(!spec.can_transition_to(Phase::Published)); // Can't skip
        assert!(!spec.can_transition_to(Phase::Active)); // Invalid phase

        let drafting_spec = Specification::from_parts(
            "Test".to_string(),
            DocumentMetadata::new("TEST-S-0001".to_string()),
            DocumentContent::new("content"),
            Some(DocumentId::new("TEST-V-0001")),
            vec![Tag::Phase(Phase::Drafting)],
            false,
        );

        assert!(drafting_spec.can_transition_to(Phase::Review));
        assert!(!drafting_spec.can_transition_to(Phase::Published));

        let review_spec = Specification::from_parts(
            "Test".to_string(),
            DocumentMetadata::new("TEST-S-0001".to_string()),
            DocumentContent::new("content"),
            Some(DocumentId::new("TEST-V-0001")),
            vec![Tag::Phase(Phase::Review)],
            false,
        );

        assert!(review_spec.can_transition_to(Phase::Published));
        assert!(!review_spec.can_transition_to(Phase::Discovery));
    }

    #[test]
    fn test_specification_transition_phase_auto() {
        let mut spec = Specification::new(
            "Test Specification".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        let new_phase = spec.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Drafting);
        assert_eq!(spec.phase().unwrap(), Phase::Drafting);

        let new_phase = spec.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Review);
        assert_eq!(spec.phase().unwrap(), Phase::Review);

        let new_phase = spec.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
        assert_eq!(spec.phase().unwrap(), Phase::Published);

        // Auto-transition from Published should stay at Published (final phase)
        let new_phase = spec.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Published);
    }

    #[test]
    fn test_specification_transition_phase_invalid() {
        let mut spec = Specification::new(
            "Test Specification".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![Tag::Phase(Phase::Discovery)],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        // Invalid: skip from Discovery to Review
        let result = spec.transition_phase(Some(Phase::Review));
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidPhaseTransition { from, to } => {
                assert_eq!(from, Phase::Discovery);
                assert_eq!(to, Phase::Review);
            }
            _ => panic!("Expected InvalidPhaseTransition error"),
        }

        // Should still be in Discovery
        assert_eq!(spec.phase().unwrap(), Phase::Discovery);
    }

    #[test]
    fn test_specification_to_content_roundtrip() {
        let spec = Specification::new(
            "System Design".to_string(),
            DocumentId::new("TEST-V-0001"),
            vec![
                Tag::Label("specification".to_string()),
                Tag::Phase(Phase::Discovery),
            ],
            false,
            "TEST-S-0001".to_string(),
        )
        .unwrap();

        let content = spec.to_content().unwrap();

        // Parse back
        let spec2 = Specification::from_content(&content).unwrap();
        assert_eq!(spec2.title(), "System Design");
        assert_eq!(spec2.document_type(), DocumentType::Specification);
        assert_eq!(spec2.phase().unwrap(), Phase::Discovery);
        assert_eq!(
            spec2.parent_id().unwrap().to_string(),
            "TEST-V-0001"
        );
    }
}
