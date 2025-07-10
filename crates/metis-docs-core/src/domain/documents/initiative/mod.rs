use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// Complexity level for initiatives
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Complexity {
    S,  // Small
    M,  // Medium
    L,  // Large
    XL, // Extra Large
}

impl std::fmt::Display for Complexity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Complexity::S => write!(f, "S"),
            Complexity::M => write!(f, "M"),
            Complexity::L => write!(f, "L"),
            Complexity::XL => write!(f, "XL"),
        }
    }
}

impl std::str::FromStr for Complexity {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_uppercase().as_str() {
            "S" => Ok(Complexity::S),
            "M" => Ok(Complexity::M),
            "L" => Ok(Complexity::L),
            "XL" => Ok(Complexity::XL),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Invalid complexity: {}",
                s
            ))),
        }
    }
}

/// An Initiative document represents a concrete implementation approach for a strategy
#[derive(Debug)]
pub struct Initiative {
    core: super::traits::DocumentCore,
    estimated_complexity: Complexity,
}

impl Initiative {
    /// Create a new Initiative document with content rendered from template
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>, // Usually a Strategy
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        estimated_complexity: Complexity,
    ) -> Result<Self, DocumentValidationError> {
        // Create fresh metadata
        let metadata = DocumentMetadata::new();

        // Render the content template
        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("initiative_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("initiative_content", &context).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Template render error: {}", e))
        })?;

        let content = DocumentContent::new(&rendered_content);

        Ok(Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
            },
            estimated_complexity,
        })
    }

    /// Create an Initiative document from existing data (used when loading from file)
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        estimated_complexity: Complexity,
    ) -> Self {
        Self {
            core: super::traits::DocumentCore {
                title,
                metadata,
                content,
                parent_id,
                blocked_by,
                tags,
                archived,
            },
            estimated_complexity,
        }
    }

    pub fn estimated_complexity(&self) -> Complexity {
        self.estimated_complexity
    }

    /// Get the next phase in the Initiative sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Discovery => Some(Design),
            Design => Some(Ready),
            Ready => Some(Decompose),
            Decompose => Some(Active),
            Active => Some(Completed),
            Completed => None, // Final phase
            _ => None,         // Invalid phase for Initiative
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

    /// Create an Initiative document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        Self::from_content(&raw_content)
    }

    /// Create an Initiative document from raw file content string
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

        // Verify this is actually an initiative document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "initiative" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'initiative', found '{}'",
                level
            )));
        }

        // Extract initiative-specific fields
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent")
            .ok()
            .map(DocumentId::from);
        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        let estimated_complexity =
            FrontmatterParser::extract_string(&fm_map, "estimated_complexity")
                .and_then(|s| s.parse::<Complexity>())?;

        // Create metadata and content
        let metadata =
            DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::from_parts(
            title,
            metadata,
            content,
            parent_id,
            blocked_by,
            tags,
            archived,
            estimated_complexity,
        ))
    }

    /// Write the Initiative document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Initiative document to its markdown string representation using templates
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
        context.insert(
            "parent_id",
            &self
                .parent_id()
                .map(|id| id.to_string())
                .unwrap_or_default(),
        );
        let blocked_by_list: Vec<String> =
            self.blocked_by().iter().map(|id| id.to_string()).collect();
        context.insert("blocked_by", &blocked_by_list);
        context.insert(
            "estimated_complexity",
            &self.estimated_complexity.to_string(),
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

impl Document for Initiative {
    // id() uses default implementation from trait

    fn document_type(&self) -> DocumentType {
        DocumentType::Initiative
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
                (Discovery, Design)
                    | (Design, Ready)
                    | (Ready, Decompose)
                    | (Decompose, Active)
                    | (Active, Completed)
            )
        } else {
            false // Can't transition if we can't determine current phase
        }
    }

    fn parent_id(&self) -> Option<&DocumentId> {
        self.core.parent_id.as_ref()
    }

    fn blocked_by(&self) -> &[DocumentId] {
        &self.core.blocked_by
    }

    fn validate(&self) -> Result<(), DocumentValidationError> {
        // Initiative-specific validation rules
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Initiative title cannot be empty".to_string(),
            ));
        }

        // Parent should typically be a Strategy
        if self.parent_id().is_none() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Initiatives should have a parent Strategy".to_string(),
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
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_initiative_from_content() {
        let content = r##"---
id: test-initiative
level: initiative
title: "Test Initiative"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
parent: strategy-001
blocked_by: []
estimated_complexity: "L"

tags:
  - "#initiative"
  - "#phase/discovery"

exit_criteria_met: false
---

# Test Initiative

## Purpose

This is a test initiative for our system.

## Approach

We will implement this step by step.

## Acceptance Criteria

- [ ] Purpose is clearly defined
- [ ] Approach is documented
"##;

        let initiative = Initiative::from_content(content).unwrap();

        assert_eq!(initiative.title(), "Test Initiative");
        assert_eq!(initiative.document_type(), DocumentType::Initiative);
        assert!(!initiative.archived());
        assert_eq!(initiative.tags().len(), 2);
        assert_eq!(initiative.phase().unwrap(), Phase::Discovery);
        assert_eq!(initiative.estimated_complexity(), Complexity::L);
        assert!(initiative.content().has_acceptance_criteria());

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-initiative.md");

        initiative.to_file(&file_path).await.unwrap();
        let loaded_initiative = Initiative::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_initiative.title(), initiative.title());
        assert_eq!(
            loaded_initiative.phase().unwrap(),
            initiative.phase().unwrap()
        );
        assert_eq!(loaded_initiative.content().body, initiative.content().body);
        assert_eq!(loaded_initiative.archived(), initiative.archived());
        assert_eq!(
            loaded_initiative.estimated_complexity(),
            initiative.estimated_complexity()
        );
        assert_eq!(loaded_initiative.tags().len(), initiative.tags().len());
    }

    #[test]
    fn test_initiative_complexity_parsing() {
        assert_eq!("S".parse::<Complexity>().unwrap(), Complexity::S);
        assert_eq!("M".parse::<Complexity>().unwrap(), Complexity::M);
        assert_eq!("L".parse::<Complexity>().unwrap(), Complexity::L);
        assert_eq!("XL".parse::<Complexity>().unwrap(), Complexity::XL);
        assert_eq!("s".parse::<Complexity>().unwrap(), Complexity::S); // Case insensitive
        assert!("invalid".parse::<Complexity>().is_err());
    }

    #[test]
    fn test_initiative_invalid_level() {
        let content = r##"---
id: test-doc
level: strategy
title: "Test Strategy"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
archived: false
estimated_complexity: "M"
tags:
  - "#strategy"
  - "#phase/shaping"
exit_criteria_met: false
---

# Test Strategy
"##;

        let result = Initiative::from_content(content);
        assert!(result.is_err());
        match result.unwrap_err() {
            DocumentValidationError::InvalidContent(msg) => {
                assert!(msg.contains("Expected level 'initiative'"));
            }
            _ => panic!("Expected InvalidContent error"),
        }
    }

    #[tokio::test]
    async fn test_initiative_validation() {
        let initiative = Initiative::new(
            "Test Initiative".to_string(),
            Some(DocumentId::from("parent-strategy")),
            vec![],
            vec![
                Tag::Label("initiative".to_string()),
                Tag::Phase(Phase::Discovery),
            ],
            false,
            Complexity::M,
        )
        .expect("Failed to create initiative");

        assert!(initiative.validate().is_ok());

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-initiative.md");

        initiative.to_file(&file_path).await.unwrap();
        let loaded_initiative = Initiative::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_initiative.title(), initiative.title());
        assert_eq!(
            loaded_initiative.estimated_complexity(),
            initiative.estimated_complexity()
        );
        assert!(loaded_initiative.validate().is_ok());

        // Test validation failure - no parent
        let initiative_no_parent = Initiative::new(
            "Test Initiative".to_string(),
            None, // No parent
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
        )
        .expect("Failed to create initiative");

        assert!(initiative_no_parent.validate().is_err());
    }

    #[tokio::test]
    async fn test_initiative_phase_transitions() {
        let initiative = Initiative::new(
            "Test Initiative".to_string(),
            Some(DocumentId::from("parent-strategy")),
            vec![],
            vec![Tag::Phase(Phase::Discovery)],
            false,
            Complexity::M,
        )
        .expect("Failed to create initiative");

        assert!(initiative.can_transition_to(Phase::Design));
        assert!(!initiative.can_transition_to(Phase::Active));
        assert!(!initiative.can_transition_to(Phase::Completed));

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-initiative.md");

        initiative.to_file(&file_path).await.unwrap();
        let loaded_initiative = Initiative::from_file(&file_path).await.unwrap();

        assert_eq!(
            loaded_initiative.phase().unwrap(),
            initiative.phase().unwrap()
        );
        assert!(loaded_initiative.can_transition_to(Phase::Design));
        assert!(!loaded_initiative.can_transition_to(Phase::Active));
        assert!(!loaded_initiative.can_transition_to(Phase::Completed));
    }

    #[test]
    fn test_complexity_display() {
        assert_eq!(Complexity::S.to_string(), "S");
        assert_eq!(Complexity::M.to_string(), "M");
        assert_eq!(Complexity::L.to_string(), "L");
        assert_eq!(Complexity::XL.to_string(), "XL");
    }
}
