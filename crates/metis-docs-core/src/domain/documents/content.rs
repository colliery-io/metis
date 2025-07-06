use serde::{Deserialize, Serialize};

/// Document content containing the main body and acceptance criteria
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Main content body (without frontmatter)
    pub body: String,
    /// Acceptance criteria section (if present)
    pub acceptance_criteria: Option<String>,
}

impl DocumentContent {
    /// Create new content from body text
    pub fn new(body: &str) -> Self {
        Self {
            body: body.to_string(),
            acceptance_criteria: None,
        }
    }

    /// Create content with both body and acceptance criteria
    pub fn with_acceptance_criteria(body: &str, acceptance_criteria: &str) -> Self {
        Self {
            body: body.to_string(),
            acceptance_criteria: Some(acceptance_criteria.to_string()),
        }
    }

    /// Parse content from markdown, separating main content from acceptance criteria
    pub fn from_markdown(content: &str) -> Self {
        // Look for "## Acceptance Criteria" section
        if let Some(criteria_pos) = content.find("## Acceptance Criteria") {
            let body = content[..criteria_pos].trim().to_string();
            let acceptance_criteria = content[criteria_pos..].trim().to_string();
            Self {
                body,
                acceptance_criteria: Some(acceptance_criteria),
            }
        } else {
            Self::new(content)
        }
    }

    /// Get the full content including acceptance criteria
    pub fn full_content(&self) -> String {
        match &self.acceptance_criteria {
            Some(criteria) => format!("{}\n\n{}", self.body, criteria),
            None => self.body.clone(),
        }
    }

    /// Check if acceptance criteria are present
    pub fn has_acceptance_criteria(&self) -> bool {
        self.acceptance_criteria.is_some()
    }
}