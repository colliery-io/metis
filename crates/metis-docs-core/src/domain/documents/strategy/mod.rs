use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use super::metadata::DocumentMetadata;
use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use chrono::Utc;
use gray_matter;
use std::path::Path;
use tera::{Context, Tera};

/// Risk level for strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
            RiskLevel::Critical => write!(f, "critical"),
        }
    }
}

impl std::str::FromStr for RiskLevel {
    type Err = DocumentValidationError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "low" => Ok(RiskLevel::Low),
            "medium" => Ok(RiskLevel::Medium),
            "high" => Ok(RiskLevel::High),
            "critical" => Ok(RiskLevel::Critical),
            _ => Err(DocumentValidationError::InvalidContent(format!(
                "Invalid risk level: {}",
                s
            ))),
        }
    }
}

/// A Strategy document defines high-level approaches to achieve vision goals
#[derive(Debug)]
pub struct Strategy {
    core: super::traits::DocumentCore,
    risk_level: RiskLevel,
    stakeholders: Vec<String>,
}

impl Strategy {
    /// Create a new Strategy document with content rendered from template
    pub fn new(
        title: String,
        parent_id: Option<DocumentId>, // Usually a Vision
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        risk_level: RiskLevel,
        stakeholders: Vec<String>,
    ) -> Result<Self, DocumentValidationError> {
        // Create fresh metadata
        let metadata = DocumentMetadata::new();

        // Render the content template
        let template_content = include_str!("content.md");
        let mut tera = Tera::default();
        tera.add_raw_template("strategy_content", template_content)
            .map_err(|e| {
                DocumentValidationError::InvalidContent(format!("Template error: {}", e))
            })?;

        let mut context = Context::new();
        context.insert("title", &title);

        let rendered_content = tera.render("strategy_content", &context).map_err(|e| {
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
            risk_level,
            stakeholders,
        })
    }

    /// Create a Strategy document from existing data (used when loading from file)
    #[allow(clippy::too_many_arguments)]
    pub fn from_parts(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>,
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        risk_level: RiskLevel,
        stakeholders: Vec<String>,
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
            risk_level,
            stakeholders,
        }
    }

    pub fn risk_level(&self) -> RiskLevel {
        self.risk_level
    }

    pub fn stakeholders(&self) -> &[String] {
        &self.stakeholders
    }

    /// Get the next phase in the Strategy sequence
    fn next_phase_in_sequence(current: Phase) -> Option<Phase> {
        use Phase::*;
        match current {
            Shaping => Some(Design),
            Design => Some(Ready),
            Ready => Some(Active),
            Active => Some(Completed),
            Completed => None, // Final phase
            _ => None,         // Invalid phase for Strategy
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

    /// Create a Strategy document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e))
        })?;

        Self::from_content(&raw_content)
    }

    /// Create a Strategy document from raw file content string
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

        // Verify this is actually a strategy document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "strategy" {
            return Err(DocumentValidationError::InvalidContent(format!(
                "Expected level 'strategy', found '{}'",
                level
            )));
        }

        // Extract strategy-specific fields
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent")
            .ok()
            .map(DocumentId::from);
        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();

        let risk_level = FrontmatterParser::extract_string(&fm_map, "risk_level")
            .and_then(|s| s.parse::<RiskLevel>())?;

        let stakeholders = FrontmatterParser::extract_string_array(&fm_map, "stakeholders")?;

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
            risk_level,
            stakeholders,
        ))
    }

    /// Write the Strategy document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content).map_err(|e| {
            DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e))
        })
    }

    /// Convert the Strategy document to its markdown string representation using templates
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

        // Format blocked_by as YAML list
        let blocked_by_list: Vec<String> =
            self.blocked_by().iter().map(|id| id.to_string()).collect();
        context.insert("blocked_by", &blocked_by_list);
        context.insert("risk_level", &self.risk_level.to_string());
        context.insert("stakeholders", &self.stakeholders);

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

    // Helper methods
}

impl Document for Strategy {
    // id() uses default implementation from trait

    fn document_type(&self) -> DocumentType {
        DocumentType::Strategy
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
                (Shaping, Design) | (Design, Ready) | (Ready, Active) | (Active, Completed)
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
        // Strategy-specific validation rules
        if self.title().trim().is_empty() {
            return Err(DocumentValidationError::InvalidTitle(
                "Strategy title cannot be empty".to_string(),
            ));
        }

        if self.stakeholders.is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Strategies must have at least one stakeholder".to_string(),
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
    async fn test_strategy_new() {
        let strategy = Strategy::new(
            "Test Strategy".to_string(),
            Some(DocumentId::from("parent-vision".to_string())),
            Vec::new(),
            vec![
                Tag::Label("strategy".to_string()),
                Tag::Phase(Phase::Shaping),
            ],
            false,
            RiskLevel::Medium,
            vec!["stakeholder1".to_string(), "stakeholder2".to_string()],
        )
        .expect("Failed to create strategy");

        assert_eq!(strategy.title(), "Test Strategy");
        assert_eq!(strategy.document_type(), DocumentType::Strategy);
        assert!(!strategy.archived());
        assert_eq!(strategy.risk_level(), RiskLevel::Medium);
        assert_eq!(strategy.stakeholders().len(), 2);

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-strategy.md");

        strategy.to_file(&file_path).await.unwrap();
        let loaded_strategy = Strategy::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_strategy.title(), strategy.title());
        assert_eq!(loaded_strategy.phase().unwrap(), strategy.phase().unwrap());
        assert_eq!(loaded_strategy.content().body, strategy.content().body);
        assert_eq!(loaded_strategy.archived(), strategy.archived());
        assert_eq!(loaded_strategy.risk_level(), strategy.risk_level());
        assert_eq!(loaded_strategy.stakeholders(), strategy.stakeholders());
    }

    #[tokio::test]
    async fn test_strategy_from_content() {
        let content = r##"---
id: test-strategy
level: strategy
title: "Test Strategy"
created_at: 2025-01-01T00:00:00Z
updated_at: 2025-01-01T00:00:00Z
parent: parent-vision
blocked_by: []
archived: false

tags:
  - "#strategy"
  - "#phase/shaping"

exit_criteria_met: false
risk_level: medium
stakeholders: ["stakeholder1", "stakeholder2"]
---

# Test Strategy

## Vision Alignment

This strategy aligns with our vision.

## Current State

We are here.

## Acceptance Criteria

- [ ] Strategy is clearly defined
- [ ] Stakeholders are identified
"##;

        let strategy = Strategy::from_content(content).unwrap();

        assert_eq!(strategy.title(), "Test Strategy");
        assert_eq!(strategy.document_type(), DocumentType::Strategy);
        assert!(!strategy.archived());
        assert_eq!(strategy.risk_level(), RiskLevel::Medium);
        assert_eq!(strategy.stakeholders().len(), 2);
        assert_eq!(strategy.phase().unwrap(), Phase::Shaping);

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-strategy.md");

        strategy.to_file(&file_path).await.unwrap();
        let loaded_strategy = Strategy::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_strategy.title(), strategy.title());
        assert_eq!(loaded_strategy.phase().unwrap(), strategy.phase().unwrap());
        assert_eq!(loaded_strategy.content().body, strategy.content().body);
        assert_eq!(loaded_strategy.risk_level(), strategy.risk_level());
        assert_eq!(loaded_strategy.stakeholders(), strategy.stakeholders());
    }

    #[tokio::test]
    async fn test_strategy_phase_transitions() {
        let mut strategy = Strategy::new(
            "Test Strategy".to_string(),
            None,
            Vec::new(),
            vec![Tag::Phase(Phase::Shaping)],
            false,
            RiskLevel::Medium,
            Vec::new(),
        )
        .expect("Failed to create strategy");

        assert!(strategy.can_transition_to(Phase::Design));
        assert!(!strategy.can_transition_to(Phase::Completed));

        // Auto-transition from Shaping should go to Design
        let new_phase = strategy.transition_phase(None).unwrap();
        assert_eq!(new_phase, Phase::Design);
        assert_eq!(strategy.phase().unwrap(), Phase::Design);

        // Round-trip test after transition
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-strategy.md");
        strategy.to_file(&file_path).await.unwrap();
        let loaded_strategy = Strategy::from_file(&file_path).await.unwrap();
        assert_eq!(loaded_strategy.phase().unwrap(), Phase::Design);
    }

    #[tokio::test]
    async fn test_strategy_validation() {
        let strategy = Strategy::new(
            "Test Strategy".to_string(),
            None,
            Vec::new(),
            vec![
                Tag::Label("strategy".to_string()),
                Tag::Phase(Phase::Shaping),
            ],
            false,
            RiskLevel::High,
            vec!["key-stakeholder".to_string()],
        )
        .expect("Failed to create strategy");

        assert!(strategy.validate().is_ok());
        assert_eq!(strategy.risk_level(), RiskLevel::High);

        // Round-trip test: write to file and read back
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test-strategy.md");

        strategy.to_file(&file_path).await.unwrap();
        let loaded_strategy = Strategy::from_file(&file_path).await.unwrap();

        assert_eq!(loaded_strategy.title(), strategy.title());
        assert_eq!(loaded_strategy.risk_level(), strategy.risk_level());
        assert_eq!(loaded_strategy.stakeholders(), strategy.stakeholders());
        assert!(loaded_strategy.validate().is_ok());
    }
}
