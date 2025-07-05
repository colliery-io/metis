//! Document context and related types for template rendering

use crate::{DocumentType, MetisError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Context for document creation containing all template variables
#[derive(Debug, Clone, Serialize)]
pub struct DocumentContext {
    // Core fields for all documents
    pub title: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub parent_title: Option<String>,
    pub blocked_by: Vec<String>,

    // Document-type specific fields
    pub risk_level: Option<RiskLevel>,        // Strategy
    pub stakeholders: Vec<String>,            // Strategy, Vision
    pub technical_lead: Option<String>,       // Initiative
    pub complexity: Option<Complexity>,       // Initiative
    pub decision_maker: Option<String>,       // ADR
    pub decision_date: Option<DateTime<Utc>>, // ADR
    pub number: Option<u32>,                  // ADR
}

/// Risk level for strategies
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Complexity level for initiatives
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    S,  // Small
    M,  // Medium
    L,  // Large
    XL, // Extra Large
}

impl DocumentContext {
    /// Create a new DocumentContext with required fields
    pub fn new(title: String) -> Self {
        let slug = Self::title_to_slug(&title);
        let now = Utc::now();

        Self {
            title,
            slug,
            created_at: now,
            updated_at: now,
            parent_title: None,
            blocked_by: Vec::new(),
            risk_level: None,
            stakeholders: Vec::new(),
            technical_lead: None,
            complexity: None,
            decision_maker: None,
            decision_date: None,
            number: None,
        }
    }

    /// Validate context for a specific document type
    pub fn validate_for_type(&self, doc_type: &DocumentType) -> Result<()> {
        match doc_type {
            DocumentType::Strategy => {
                if self.risk_level.is_none() {
                    return Err(MetisError::MissingRequiredField {
                        field: "risk_level".to_string(),
                    });
                }
            }
            DocumentType::Initiative => {
                if self.complexity.is_none() {
                    return Err(MetisError::MissingRequiredField {
                        field: "complexity".to_string(),
                    });
                }
            }
            DocumentType::Adr => {
                if self.decision_maker.is_none() {
                    return Err(MetisError::MissingRequiredField {
                        field: "decision_maker".to_string(),
                    });
                }
                if self.number.is_none() {
                    return Err(MetisError::MissingRequiredField {
                        field: "number".to_string(),
                    });
                }
            }
            DocumentType::Vision | DocumentType::Task => {
                // No additional required fields
            }
        }
        Ok(())
    }

    /// Convert title to URL-friendly slug
    pub fn title_to_slug(title: &str) -> String {
        title
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Builder pattern methods for setting optional fields
    pub fn with_parent(mut self, parent_title: String) -> Self {
        self.parent_title = Some(parent_title);
        self
    }

    pub fn with_blocked_by(mut self, blocked_by: Vec<String>) -> Self {
        self.blocked_by = blocked_by;
        self
    }

    pub fn with_risk_level(mut self, risk_level: RiskLevel) -> Self {
        self.risk_level = Some(risk_level);
        self
    }

    pub fn with_stakeholders(mut self, stakeholders: Vec<String>) -> Self {
        self.stakeholders = stakeholders;
        self
    }

    pub fn with_technical_lead(mut self, technical_lead: String) -> Self {
        self.technical_lead = Some(technical_lead);
        self
    }

    pub fn with_complexity(mut self, complexity: Complexity) -> Self {
        self.complexity = Some(complexity);
        self
    }

    pub fn with_decision_maker(mut self, decision_maker: String) -> Self {
        self.decision_maker = Some(decision_maker);
        self
    }

    pub fn with_decision_date(mut self, decision_date: DateTime<Utc>) -> Self {
        self.decision_date = Some(decision_date);
        self
    }

    pub fn with_number(mut self, number: u32) -> Self {
        self.number = Some(number);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    #[test]
    fn test_title_to_slug() {
        assert_eq!(
            DocumentContext::title_to_slug("Core Document Management Library"),
            "core-document-management-library"
        );
        assert_eq!(
            DocumentContext::title_to_slug("ADR-001: Document Format"),
            "adr-001-document-format"
        );
        assert_eq!(
            DocumentContext::title_to_slug("Storage & Indexing System"),
            "storage-indexing-system"
        );
    }

    #[test]
    fn test_new_context() {
        let ctx = DocumentContext::new("Test Document".to_string());
        assert_eq!(ctx.title, "Test Document");
        assert_eq!(ctx.slug, "test-document");
        assert!(ctx.created_at <= Utc::now());
        assert!(ctx.updated_at <= Utc::now());
    }

    #[test]
    fn test_strategy_validation() {
        let ctx = DocumentContext::new("Test Strategy".to_string());

        // Should fail without risk_level
        assert!(ctx.validate_for_type(&DocumentType::Strategy).is_err());

        // Should pass with risk_level
        let ctx_with_risk = ctx.with_risk_level(RiskLevel::Medium);
        assert!(ctx_with_risk
            .validate_for_type(&DocumentType::Strategy)
            .is_ok());
    }

    #[test]
    fn test_initiative_validation() {
        let ctx = DocumentContext::new("Test Initiative".to_string());

        // Should fail without complexity
        assert!(ctx.validate_for_type(&DocumentType::Initiative).is_err());

        // Should pass with complexity
        let ctx_with_complexity = ctx.with_complexity(Complexity::M);
        assert!(ctx_with_complexity
            .validate_for_type(&DocumentType::Initiative)
            .is_ok());
    }

    #[test]
    fn test_adr_validation() {
        let ctx = DocumentContext::new("Test ADR".to_string());

        // Should fail without decision_maker and number
        assert!(ctx.validate_for_type(&DocumentType::Adr).is_err());

        // Should pass with required fields
        let ctx_complete = ctx
            .with_decision_maker("Engineering Team".to_string())
            .with_number(1);
        assert!(ctx_complete.validate_for_type(&DocumentType::Adr).is_ok());
    }

    #[test]
    fn test_vision_and_task_validation() {
        let ctx = DocumentContext::new("Test Document".to_string());

        // Vision and Task have no additional requirements
        assert!(ctx.validate_for_type(&DocumentType::Vision).is_ok());
        assert!(ctx.validate_for_type(&DocumentType::Task).is_ok());
    }

    #[test]
    fn test_builder_pattern() {
        let ctx = DocumentContext::new("Test Document".to_string())
            .with_parent("Parent Document".to_string())
            .with_blocked_by(vec!["Blocker 1".to_string(), "Blocker 2".to_string()])
            .with_stakeholders(vec!["Alice".to_string(), "Bob".to_string()]);

        assert_eq!(ctx.parent_title, Some("Parent Document".to_string()));
        assert_eq!(ctx.blocked_by.len(), 2);
        assert_eq!(ctx.stakeholders.len(), 2);
    }
}
