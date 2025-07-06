use super::traits::{Document, DocumentTemplate, DocumentValidationError};
use super::types::{DocumentId, DocumentType, Phase, Tag};
use super::metadata::DocumentMetadata;
use super::content::DocumentContent;
use super::helpers::FrontmatterParser;
use std::path::Path;
use gray_matter;
use chrono::{DateTime, Utc};
use tera::{Tera, Context};

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
            _ => Err(DocumentValidationError::InvalidContent(format!("Invalid risk level: {}", s))),
        }
    }
}

/// A Strategy document defines high-level approaches to achieve vision goals
#[derive(Debug)]
pub struct Strategy {
    core: super::traits::DocumentCore,
    risk_level: RiskLevel,
    stakeholders: Vec<String>,
    success_metrics: Vec<String>,
}

impl Strategy {
    /// Create a new Strategy document from parsed file data
    pub fn new(
        title: String,
        metadata: DocumentMetadata,
        content: DocumentContent,
        parent_id: Option<DocumentId>, // Usually a Vision
        blocked_by: Vec<DocumentId>,
        tags: Vec<Tag>,
        archived: bool,
        risk_level: RiskLevel,
        stakeholders: Vec<String>,
        success_metrics: Vec<String>,
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
            success_metrics,
        }
    }

    pub fn risk_level(&self) -> RiskLevel {
        self.risk_level
    }

    pub fn stakeholders(&self) -> &[String] {
        &self.stakeholders
    }

    pub fn success_metrics(&self) -> &[String] {
        &self.success_metrics
    }

    /// Create a Strategy document by reading and parsing a file
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, DocumentValidationError> {
        let raw_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e)))?;

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

        // Verify this is actually a strategy document
        let level = FrontmatterParser::extract_string(&fm_map, "level")?;
        if level != "strategy" {
            return Err(DocumentValidationError::InvalidContent(
                format!("Expected level 'strategy', found '{}'", level)
            ));
        }

        // Extract strategy-specific fields
        let parent_id = FrontmatterParser::extract_string(&fm_map, "parent").ok().map(DocumentId::from);
        let blocked_by = FrontmatterParser::extract_string_array(&fm_map, "blocked_by")
            .unwrap_or_default()
            .into_iter()
            .map(DocumentId::from)
            .collect();
        
        let risk_level = FrontmatterParser::extract_string(&fm_map, "risk_level")
            .and_then(|s| s.parse::<RiskLevel>())?;
        
        let stakeholders = FrontmatterParser::extract_string_array(&fm_map, "stakeholders")?;
        let success_metrics = FrontmatterParser::extract_string_array(&fm_map, "success_metrics")
            .unwrap_or_default();

        // Create metadata and content
        let metadata = DocumentMetadata::from_frontmatter(created_at, updated_at, exit_criteria_met);
        let content = DocumentContent::from_markdown(&parsed.content);

        Ok(Self::new(title, metadata, content, parent_id, blocked_by, tags, archived, 
                     risk_level, stakeholders, success_metrics))
    }

    /// Write the Strategy document to a file
    pub async fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), DocumentValidationError> {
        let content = self.to_content()?;
        std::fs::write(path.as_ref(), content)
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to write file: {}", e)))
    }

    /// Convert the Strategy document to its markdown string representation using templates
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
        context.insert("blocked_by", &self.blocked_by().iter().map(|id| id.to_string()).collect::<Vec<_>>());
        context.insert("risk_level", &self.risk_level.to_string());
        context.insert("stakeholders", &self.stakeholders);
        context.insert("success_metrics", &self.success_metrics);
        
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
            match (current_phase, phase) {
                (Shaping, Design) => true,
                (Design, Ready) => true,
                (Ready, Active) => true,
                (Active, Completed) => true,
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

        if self.success_metrics.is_empty() {
            return Err(DocumentValidationError::MissingRequiredField(
                "Strategies must have success metrics defined".to_string(),
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