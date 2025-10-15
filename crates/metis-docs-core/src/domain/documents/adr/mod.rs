use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// An ADR (Architecture Decision Record) documents architectural decisions
#[derive(Debug)]
pub struct Adr {
    core: super::traits::DocumentCore,
    number: u32,
    decision_maker: String,
    decision_date: Option<chrono::DateTime<Utc>>,
}

impl Adr {
    /// Create a new ADR document with content rendered from template
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        number: u32,
        title: String,
        decision_maker: String,
        decision_date: Option<chrono::DateTime<Utc>>,
        parent_id: Option<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        short_code: String,
    ) -> Result<Self, DocumentValidationError> {
        // Create fresh metadata
        let metadata = DocumentMetadata::new(short_code);

        // Render the content template
        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("adr_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("number", &number);
        context.insert("title", &title);

        let rendered_content = tera.render("adr_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id, // ADRs can reference the work that generated the need for decision
                blocked_by: Vec::new(), // ADRs cannot be blocked
                tags,
                archived,
                strategy_id: None,   // ADRs are not part of strategies
                initiative_id: None, // ADRs are not part of initiatives
            },
            number,
            decision_maker,
            decision_date,
        })
    }

    /// Create an ADR document from existing data (used when loading from file)
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        number: u32,
        title: String,
        decision_maker: String,
        decision_date: Option<chrono::DateTime<Utc>>,
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
                strategy_id: None,   // ADRs are not part of strategies
                initiative_id: None, // ADRs are not part of initiatives
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

    pub fn decision_date(&self) -> Option<chrono::DateTime<Utc>> {
        self.decision_date
    }

    /// Create an ADR document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

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

        // Verify this is actually an ADR document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "adr" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'adr', found '{}'",
                level
            )));
        }

        // Extract ADR-specific fields
        let number = FrontmatterParser::extract_integer(&fm_map, "number")? as u32;
        let decision_maker =
            FrontmatterParser::extract_string(&fm_map, "decision_maker").unwrap_or_default();
        let decision_date = FrontmatterParser::extract_datetime(&fm_map, "decision_date").ok();
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
            number,
            title,
            decision_maker,
            decision_date,
            metadata,
            content,
            parent_id,
            tags,
            archived,
        ))
    }

    /// Get the next phase in the ADR sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Draft => Some(Discussion),
            Discussion => Some(Decided),
            Decided => Some(Superseded),
            Superseded => None, // Final phase
            _ => None,          // Invalid phase for ADR
        }
    }

    /// Update the phase tag in the document's tags
    fn update_phase_tag(&mut self, new_phase: Phase) {
        // Remove any existing phase tags
        self.core.tags.retain(|tag| !matches!(tag, Tag::Phase(_)));
        // Add the new phase tag
        self.core.tags.push(Tag::Phase(new_phase));
        // Update timestamp
        self.core.metadata.updated_at = Utc::now();
    }

    /// Write the ADR document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the ADR document to its markdown string representation using templates
    pub fn to_content(&self) -> Result<String, DocumentValidationError> {
        let mut tera = Tera::default();

        // Add the frontmatter template to Tera
        tera.add_raw_template("frontmatter", self.frontmatter_template())
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        // Create context with all document data
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
        context.insert("number", &self.number);
        context.insert("formatted_number", &format!("{:03}", self.number));
        context.insert("decision_maker", &self.decision_maker);
        context.insert(
            "decision_date",
            &self
                .decision_date
                .map(|d| d.to_rfc3339())
                .unwrap_or_default(),
        );

        // Convert tags to strings
        let tag_strings: Vec<String> = self.tags().iter().map(|tag| tag.to_str()).collect();
        context.insert("tags", &tag_strings);

        // Add lineage fields (NULL for ADR documents)
        context.insert("strategy_id", "NULL");
        context.insert("initiative_id", "NULL");

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
            matches!(
                (current_phase, phase),
                (Draft, Discussion) | (Discussion, Decided) | (Decided, Superseded)
            )
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
short_code: TEST-A-9001

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
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0042".to_string(),
        )
        .unwrap();

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
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert!(adr.validate().is_ok());

        // Test validation failure - empty decision maker
        let adr_no_decision_maker = Adr::new(
            1,
            "Test Decision".to_string(),
            "".to_string(), // Empty decision maker
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert!(adr_no_decision_maker.validate().is_err());
    }

    #[test]
    fn test_adr_cannot_be_blocked() {
        let adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert_eq!(adr.blocked_by().len(), 0);
        assert!(adr.validate().is_ok());
    }

    #[test]
    fn test_adr_phase_transitions() {
        let discussion_adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Discussion)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert!(discussion_adr.can_transition_to(Phase::Decided));
        assert!(!discussion_adr.can_transition_to(Phase::Active));
        assert!(!discussion_adr.can_transition_to(Phase::Draft));

        let decided_adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert!(decided_adr.can_transition_to(Phase::Superseded));
        assert!(!decided_adr.can_transition_to(Phase::Discussion));
    }

    #[test]
    fn test_adr_number_formatting() {
        let adr1 = Adr::new(
            1,
            "First Decision".to_string(),
            "Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        let adr42 = Adr::new(
            42,
            "Another Decision".to_string(),
            "Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        let adr999 = Adr::new(
            999,
            "Big Decision".to_string(),
            "Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Decided)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        assert_eq!(adr1.id().to_string(), "001-first-decision");
        assert_eq!(adr42.id().to_string(), "042-another-decision");
        assert_eq!(adr999.id().to_string(), "999-big-decision");
    }

    #[test]
    fn test_adr_transition_phase_auto() {
        let mut adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        // Auto-transition from Draft should go to Discussion
        let new_phase = adr.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Discussion);
        assert_eq!(adr.phase().unwrap(), Phase::Discussion);

        // Auto-transition from Discussion should go to Decided
        let new_phase = adr.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Decided);
        assert_eq!(adr.phase().unwrap(), Phase::Decided);

        // Auto-transition from Decided should go to Superseded
        let new_phase = adr.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Superseded);
        assert_eq!(adr.phase().unwrap(), Phase::Superseded);

        // Auto-transition from Superseded should stay at Superseded (final phase)
        let new_phase = adr.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Superseded);
        assert_eq!(adr.phase().unwrap(), Phase::Superseded);
    }

    #[test]
    fn test_adr_transition_phase_explicit() {
        let mut adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Discussion)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        // Explicit transition from Discussion to Decided
        let new_phase = adr.transition_phase(Some(Phase::Decided)).unwrap();
        assert_eq!(new_phase, Phase::Decided);
        assert_eq!(adr.phase().unwrap(), Phase::Decided);

        // Explicit transition from Decided to Superseded
        let new_phase = adr.transition_phase(Some(Phase::Superseded)).unwrap();
        assert_eq!(new_phase, Phase::Superseded);
        assert_eq!(adr.phase().unwrap(), Phase::Superseded);
    }

    #[test]
    fn test_adr_transition_phase_invalid() {
        let mut adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        // Invalid transition from Draft to Decided (must go through Discussion)
        let result = adr.transition_phase(Some(Phase::Decided));
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidPhaseTransition { from, to } => {
                assert_eq!(from, Phase::Draft);
                assert_eq!(to, Phase::Decided);
            }
            _ => panic!("Expected InvalidPhaseTransition error"),
        }

        // Should still be in Draft phase
        assert_eq!(adr.phase().unwrap(), Phase::Draft);
    }

    #[test]
    fn test_adr_update_section() {
        // First create an ADR with the template
        let mut adr = Adr::new(
            1,
            "Test Decision".to_string(),
            "Architecture Team".to_string(),
            Some(Utc::now()),
            None,
            vec![Tag::Phase(Phase::Draft)],
            false,
            "TEST-A-0101".to_string(),
        )
        .unwrap();

        // Then update its content to have specific test content
        adr.core_mut().content = DocumentContent::new(
            "## Context\n\nOriginal context\n\n## Decision\n\nOriginal decision",
        );

        // Replace existing section
        adr.update_section("Updated context information", "Context", false)
            .unwrap();
        let content = adr.content().body.clone();
        assert!(content.contains("## Context\n\nUpdated context information"));
        assert!(!content.contains("Original context"));

        // Append to existing section
        adr.update_section("Additional decision rationale", "Decision", true)
            .unwrap();
        let content = adr.content().body.clone();
        assert!(content.contains("Original decision"));
        assert!(content.contains("Additional decision rationale"));

        // Add new section
        adr.update_section("Impact analysis details", "Consequences", false)
            .unwrap();
        let content = adr.content().body.clone();
        assert!(content.contains("## Consequences\n\nImpact analysis details"));
    }
}
