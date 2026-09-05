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

    /// Header that introduces the acceptance criteria section. Stored separately from
    /// `acceptance_criteria` so round-tripping through `from_markdown`/`full_content`
    /// never duplicates it.
    const ACCEPTANCE_CRITERIA_HEADER: &'static str = "## Acceptance Criteria";

    /// Parse content from markdown, separating main content from acceptance criteria
    pub fn from_markdown(content: &str) -> Self {
        // Look for "## Acceptance Criteria" section
        if let Some(header_pos) = content.find(Self::ACCEPTANCE_CRITERIA_HEADER) {
            let body = content[..header_pos].trim().to_string();
            let after_header = &content[header_pos + Self::ACCEPTANCE_CRITERIA_HEADER.len()..];
            let acceptance_criteria = after_header.trim().to_string();
            Self {
                body,
                acceptance_criteria: Some(acceptance_criteria),
            }
        } else {
            Self::new(content)
        }
    }

    /// Get the full content including acceptance criteria, with the header rendered exactly once
    pub fn full_content(&self) -> String {
        match &self.acceptance_criteria {
            Some(criteria) => format!(
                "{}\n\n{}\n\n{}",
                self.body,
                Self::ACCEPTANCE_CRITERIA_HEADER,
                criteria
            ),
            None => self.body.clone(),
        }
    }

    /// Check if acceptance criteria are present
    pub fn has_acceptance_criteria(&self) -> bool {
        self.acceptance_criteria.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_markdown_and_full_content_round_trip_without_duplicating_header() {
        let markdown = "# Title\n\nBody text\n\n## Acceptance Criteria\n\n- [ ] one\n- [ ] two";

        let content = DocumentContent::from_markdown(markdown);
        assert_eq!(content.full_content(), markdown);
        assert_eq!(
            content.full_content().matches("## Acceptance Criteria").count(),
            1
        );

        // Serializing content that was itself parsed from markdown (e.g. an edit-then-save
        // cycle) must stay stable and never re-introduce the header a second time.
        let reparsed = DocumentContent::from_markdown(&content.full_content());
        assert_eq!(reparsed.full_content(), content.full_content());
    }
}
