//! Exit criteria validation for documents

use crate::{MetisError, Result};
use std::path::Path;
use tokio::fs;

/// Result of exit criteria validation containing detailed completion status
#[derive(Debug, Clone)]
pub struct ExitCriteriaResult {
    pub met: bool,
    pub total_criteria: usize,
    pub completed_criteria: usize,
    pub missing_criteria: Vec<String>,
}

impl ExitCriteriaResult {
    /// Create a new result with no criteria found
    pub fn empty() -> Self {
        Self {
            met: true, // No criteria means all criteria are met
            total_criteria: 0,
            completed_criteria: 0,
            missing_criteria: Vec::new(),
        }
    }
}

/// Validate exit criteria from a file path
pub async fn validate_exit_criteria(document_path: &Path) -> Result<ExitCriteriaResult> {
    let content = fs::read_to_string(document_path)
        .await
        .map_err(MetisError::Io)?;
    validate_exit_criteria_content(&content)
}

/// Validate exit criteria from document content
pub fn validate_exit_criteria_content(content: &str) -> Result<ExitCriteriaResult> {
    let criteria = parse_exit_criteria(content);

    if criteria.is_empty() {
        return Ok(ExitCriteriaResult::empty());
    }

    let completed_criteria = criteria.iter().filter(|c| c.completed).count();
    let missing_criteria: Vec<String> = criteria
        .iter()
        .filter(|c| !c.completed)
        .map(|c| c.text.clone())
        .collect();

    Ok(ExitCriteriaResult {
        met: missing_criteria.is_empty(),
        total_criteria: criteria.len(),
        completed_criteria,
        missing_criteria,
    })
}

/// Individual exit criterion with completion status
#[derive(Debug, Clone)]
struct ExitCriterion {
    text: String,
    completed: bool,
}

/// Parse exit criteria from markdown content
fn parse_exit_criteria(content: &str) -> Vec<ExitCriterion> {
    let lines: Vec<&str> = content.lines().collect();
    let mut criteria = Vec::new();
    let mut in_exit_criteria = false;

    for line in lines {
        let trimmed = line.trim();

        // Look for the exit criteria section
        if trimmed.starts_with("## Exit Criteria") {
            in_exit_criteria = true;
            continue;
        }

        // Stop parsing if we hit another section
        if in_exit_criteria && trimmed.starts_with("##") && !trimmed.starts_with("## Exit Criteria")
        {
            break;
        }

        // Parse checkbox items
        if in_exit_criteria {
            if let Some(criterion) = parse_checkbox_line(trimmed) {
                criteria.push(criterion);
            }
        }
    }

    criteria
}

/// Parse a single line for checkbox format
fn parse_checkbox_line(line: &str) -> Option<ExitCriterion> {
    let trimmed = line.trim();

    // Check for incomplete checkbox: - [ ]
    if let Some(text) = trimmed.strip_prefix("- [ ]") {
        return Some(ExitCriterion {
            text: text.trim().to_string(),
            completed: false,
        });
    }

    // Check for completed checkbox: - [x] or - [X]
    if let Some(text) = trimmed.strip_prefix("- [x]") {
        return Some(ExitCriterion {
            text: text.trim().to_string(),
            completed: true,
        });
    }

    if let Some(text) = trimmed.strip_prefix("- [X]") {
        return Some(ExitCriterion {
            text: text.trim().to_string(),
            completed: true,
        });
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_parse_checkbox_line() {
        // Incomplete checkbox
        let incomplete = parse_checkbox_line("- [ ] This is incomplete");
        assert!(incomplete.is_some());
        let criterion = incomplete.unwrap();
        assert_eq!(criterion.text, "This is incomplete");
        assert!(!criterion.completed);

        // Completed checkbox (lowercase x)
        let completed = parse_checkbox_line("- [x] This is complete");
        assert!(completed.is_some());
        let criterion = completed.unwrap();
        assert_eq!(criterion.text, "This is complete");
        assert!(criterion.completed);

        // Completed checkbox (uppercase X)
        let completed_upper = parse_checkbox_line("- [X] This is also complete");
        assert!(completed_upper.is_some());
        let criterion = completed_upper.unwrap();
        assert_eq!(criterion.text, "This is also complete");
        assert!(criterion.completed);

        // Not a checkbox
        let not_checkbox = parse_checkbox_line("- This is just a list item");
        assert!(not_checkbox.is_none());

        // Not a checkbox either
        let also_not = parse_checkbox_line("Some regular text");
        assert!(also_not.is_none());
    }

    #[test]
    fn test_parse_exit_criteria() {
        let content = "---
id: test-doc
level: task
---

# Test Document

Some content here.

## Exit Criteria

- [ ] First incomplete criterion
- [x] Second completed criterion
- [ ] Third incomplete criterion
- [X] Fourth completed criterion (uppercase)

## Some Other Section

- [ ] This should not be parsed
";

        let criteria = parse_exit_criteria(content);
        assert_eq!(criteria.len(), 4);

        assert_eq!(criteria[0].text, "First incomplete criterion");
        assert!(!criteria[0].completed);

        assert_eq!(criteria[1].text, "Second completed criterion");
        assert!(criteria[1].completed);

        assert_eq!(criteria[2].text, "Third incomplete criterion");
        assert!(!criteria[2].completed);

        assert_eq!(criteria[3].text, "Fourth completed criterion (uppercase)");
        assert!(criteria[3].completed);
    }

    #[test]
    fn test_validate_exit_criteria_content() {
        let content = "---
id: test-doc
level: task
---

# Test Document

## Exit Criteria

- [x] Complete criterion
- [ ] Incomplete criterion 1  
- [ ] Incomplete criterion 2
";

        let result = validate_exit_criteria_content(content).unwrap();
        assert!(!result.met);
        assert_eq!(result.total_criteria, 3);
        assert_eq!(result.completed_criteria, 1);
        assert_eq!(result.missing_criteria.len(), 2);
        assert_eq!(result.missing_criteria[0], "Incomplete criterion 1");
        assert_eq!(result.missing_criteria[1], "Incomplete criterion 2");
    }

    #[test]
    fn test_validate_all_criteria_complete() {
        let content = "---
id: test-doc
level: task
---

# Test Document

## Exit Criteria

- [x] First criterion
- [x] Second criterion
- [X] Third criterion
";

        let result = validate_exit_criteria_content(content).unwrap();
        assert!(result.met);
        assert_eq!(result.total_criteria, 3);
        assert_eq!(result.completed_criteria, 3);
        assert!(result.missing_criteria.is_empty());
    }

    #[test]
    fn test_validate_no_exit_criteria() {
        let content = "---
id: test-doc
level: task
---

# Test Document

Some content without exit criteria.

## Some Other Section

Regular content here.
";

        let result = validate_exit_criteria_content(content).unwrap();
        assert!(result.met); // No criteria means all criteria are met
        assert_eq!(result.total_criteria, 0);
        assert_eq!(result.completed_criteria, 0);
        assert!(result.missing_criteria.is_empty());
    }

    #[test]
    fn test_validate_empty_exit_criteria_section() {
        let content = "---
id: test-doc
level: task
---

# Test Document

## Exit Criteria

## Next Section

Some other content.
";

        let result = validate_exit_criteria_content(content).unwrap();
        assert!(result.met);
        assert_eq!(result.total_criteria, 0);
        assert_eq!(result.completed_criteria, 0);
        assert!(result.missing_criteria.is_empty());
    }

    #[test]
    fn test_mixed_content_in_exit_criteria() {
        let content = "---
id: test-doc
level: task
---

# Test Document

## Exit Criteria

Here are the criteria:

- [x] Complete this task
- [ ] Review the implementation

Some explanatory text here.

- [x] Write tests
- [ ] Update documentation

More text.
";

        let result = validate_exit_criteria_content(content).unwrap();
        assert!(!result.met);
        assert_eq!(result.total_criteria, 4);
        assert_eq!(result.completed_criteria, 2);
        assert_eq!(result.missing_criteria.len(), 2);
        assert_eq!(result.missing_criteria[0], "Review the implementation");
        assert_eq!(result.missing_criteria[1], "Update documentation");
    }

    #[tokio::test]
    async fn test_validate_exit_criteria_from_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "---
id: test-doc
level: task
---

# Test Document

## Exit Criteria

- [x] Criterion 1
- [ ] Criterion 2
- [x] Criterion 3
";

        temp_file.write_all(content.as_bytes()).unwrap();
        let file_path = temp_file.path();

        let result = validate_exit_criteria(file_path).await.unwrap();
        assert!(!result.met);
        assert_eq!(result.total_criteria, 3);
        assert_eq!(result.completed_criteria, 2);
        assert_eq!(result.missing_criteria.len(), 1);
        assert_eq!(result.missing_criteria[0], "Criterion 2");
    }
}
