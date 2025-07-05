//! Document update operations for surgical content modifications

use crate::{validate_content, MetisError, Result};
use chrono::Utc;
use gray_matter;
use std::fs;
use std::path::Path;

/// Update specific document sections using markdown heading navigation
pub async fn update_document_content(
    document_path: &Path,
    section_heading: &str,
    new_content: &str,
) -> Result<()> {
    // Read the document file
    let raw_content = fs::read_to_string(document_path).map_err(MetisError::Io)?;

    // Parse frontmatter and content
    let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&raw_content);
    let frontmatter_yaml = if let Some(data) = parsed.data {
        // Convert Pod to YAML string for reconstruction
        match data {
            gray_matter::Pod::Hash(_) => {
                // Extract the original frontmatter YAML from the raw content
                extract_frontmatter_yaml(&raw_content)?
            }
            _ => String::new(),
        }
    } else {
        String::new()
    };

    let content = parsed.content;

    // Update the section content
    let updated_content = update_section_in_content(&content, section_heading, new_content)?;

    // Reconstruct the document
    let final_document = if frontmatter_yaml.is_empty() {
        updated_content
    } else {
        format!("---\n{}\n---\n\n{}", frontmatter_yaml, updated_content)
    };

    // Validate the updated document
    validate_content(&final_document).map_err(|e| MetisError::ValidationFailed {
        message: format!("Updated document failed validation: {}", e),
    })?;

    // Write the updated document back atomically
    write_file_atomic(document_path, &final_document)?;

    Ok(())
}

/// Update exit criteria checkbox completion status
pub async fn update_exit_criterion(
    document_path: &Path,
    criterion_text: &str,
    completed: bool,
) -> Result<()> {
    // Read the document file
    let raw_content = fs::read_to_string(document_path).map_err(MetisError::Io)?;

    // Parse frontmatter and content
    let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&raw_content);
    let frontmatter_yaml = if parsed.data.is_some() {
        extract_frontmatter_yaml(&raw_content)?
    } else {
        String::new()
    };

    let content = parsed.content;

    // Update the checkbox in content
    let updated_content = update_checkbox_in_content(&content, criterion_text, completed)?;

    // Check if all criteria are now complete
    let all_complete = check_all_criteria_complete(&updated_content)?;

    // Update frontmatter with new exit_criteria_met status and timestamp
    let updated_frontmatter = update_frontmatter_exit_criteria(&frontmatter_yaml, all_complete)?;

    // Reconstruct the document
    let final_document = if updated_frontmatter.is_empty() {
        updated_content
    } else {
        format!("---\n{}\n---\n\n{}", updated_frontmatter, updated_content)
    };

    // Validate the updated document
    validate_content(&final_document).map_err(|e| MetisError::ValidationFailed {
        message: format!("Updated document failed validation: {}", e),
    })?;

    // Write the updated document back atomically
    write_file_atomic(document_path, &final_document)?;

    Ok(())
}

/// Update blocked_by relationships in document frontmatter
pub async fn update_blocked_by(document_path: &Path, blocked_by: Vec<String>) -> Result<()> {
    // Read the document file
    let raw_content = fs::read_to_string(document_path).map_err(MetisError::Io)?;

    // Parse frontmatter and content
    let parsed = gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&raw_content);
    let frontmatter_yaml = if parsed.data.is_some() {
        extract_frontmatter_yaml(&raw_content)?
    } else {
        return Err(MetisError::ValidationFailed {
            message: "Document has no frontmatter to update".to_string(),
        });
    };

    let content = parsed.content;

    // Validate blocked_by references
    validate_blocked_by_references(&blocked_by)?;

    // Update frontmatter with new blocked_by and timestamp
    let updated_frontmatter = update_frontmatter_blocked_by(&frontmatter_yaml, &blocked_by)?;

    // Reconstruct the document
    let final_document = format!("---\n{}\n---\n\n{}", updated_frontmatter, content);

    // Validate the updated document
    validate_content(&final_document).map_err(|e| MetisError::ValidationFailed {
        message: format!("Updated document failed validation: {}", e),
    })?;

    // Write the updated document back atomically
    write_file_atomic(document_path, &final_document)?;

    Ok(())
}

/// Update a section within content text, preserving structure
fn update_section_in_content(
    content: &str,
    section_heading: &str,
    new_content: &str,
) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let target_heading = format!("## {}", section_heading);

    // Find the section start
    let section_start = lines.iter().position(|line| {
        line.trim() == target_heading
    }).ok_or_else(|| MetisError::ValidationFailed {
        message: format!("H2 section '## {}' not found in document. Provide heading text only (e.g., 'Problem Statement' for '## Problem Statement')", section_heading),
    })?;

    // Find the section end (next ## heading or end of document)
    let section_end = lines[section_start + 1..]
        .iter()
        .position(|line| line.trim_start().starts_with("## "))
        .map(|pos| section_start + 1 + pos)
        .unwrap_or(lines.len());

    // Build the updated content
    let mut updated_lines = Vec::new();

    // Add content before the section
    updated_lines.extend_from_slice(&lines[..section_start + 1]);

    // Add empty line after heading if new content exists
    if !new_content.trim().is_empty() {
        updated_lines.push("");

        // Add the new content
        for line in new_content.lines() {
            updated_lines.push(line);
        }
    }

    // Add content after the section
    if section_end < lines.len() {
        updated_lines.push(""); // Empty line before next section
        updated_lines.extend_from_slice(&lines[section_end..]);
    }

    Ok(updated_lines.join("\n"))
}

/// Extract original frontmatter YAML from raw content
fn extract_frontmatter_yaml(raw_content: &str) -> Result<String> {
    if !raw_content.starts_with("---\n") {
        return Ok(String::new());
    }

    let content_after_first_marker = &raw_content[4..]; // Skip "---\n"
    if let Some(end_pos) = content_after_first_marker.find("\n---\n") {
        Ok(content_after_first_marker[..end_pos].to_string())
    } else {
        Err(MetisError::ValidationFailed {
            message: "Invalid frontmatter format - missing closing marker".to_string(),
        })
    }
}

/// Update checkbox state in content for a specific criterion
fn update_checkbox_in_content(
    content: &str,
    criterion_text: &str,
    completed: bool,
) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the Exit Criteria section
    let exit_criteria_start = lines
        .iter()
        .position(|line| line.trim() == "## Exit Criteria")
        .ok_or_else(|| MetisError::ValidationFailed {
            message: "Exit Criteria section not found in document".to_string(),
        })?;

    // Find the end of the Exit Criteria section
    let exit_criteria_end = lines[exit_criteria_start + 1..]
        .iter()
        .position(|line| line.trim_start().starts_with("## "))
        .map(|pos| exit_criteria_start + 1 + pos)
        .unwrap_or(lines.len());

    // Find matching criteria within the section
    let mut matching_lines = Vec::new();
    for (i, line) in lines[exit_criteria_start + 1..exit_criteria_end]
        .iter()
        .enumerate()
    {
        if is_checkbox_line(line) && line.to_lowercase().contains(&criterion_text.to_lowercase()) {
            matching_lines.push(exit_criteria_start + 1 + i);
        }
    }

    // Check for ambiguous matches
    if matching_lines.is_empty() {
        return Err(MetisError::ValidationFailed {
            message: format!("No exit criteria found matching text: '{}'", criterion_text),
        });
    }

    if matching_lines.len() > 1 {
        return Err(MetisError::ValidationFailed {
            message: format!(
                "Multiple exit criteria match text '{}' - please be more specific",
                criterion_text
            ),
        });
    }

    // Update the matching line
    let target_line_idx = matching_lines[0];
    let updated_lines: Vec<String> = lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            if i == target_line_idx {
                update_checkbox_state(line, completed)
            } else {
                line.to_string()
            }
        })
        .collect();

    Ok(updated_lines.join("\n"))
}

/// Check if a line contains a checkbox
fn is_checkbox_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("- [ ]") || trimmed.starts_with("- [x]") || trimmed.starts_with("- [X]")
}

/// Update the checkbox state in a single line
fn update_checkbox_state(line: &str, completed: bool) -> String {
    let checkbox = if completed { "[x]" } else { "[ ]" };

    // Replace any checkbox pattern with the desired state
    if line.trim_start().starts_with("- [ ]") {
        line.replacen("- [ ]", &format!("- {}", checkbox), 1)
    } else if line.trim_start().starts_with("- [x]") {
        line.replacen("- [x]", &format!("- {}", checkbox), 1)
    } else if line.trim_start().starts_with("- [X]") {
        line.replacen("- [X]", &format!("- {}", checkbox), 1)
    } else {
        line.to_string()
    }
}

/// Check if all exit criteria in the content are complete
fn check_all_criteria_complete(content: &str) -> Result<bool> {
    let lines: Vec<&str> = content.lines().collect();

    // Find the Exit Criteria section
    let exit_criteria_start = lines
        .iter()
        .position(|line| line.trim() == "## Exit Criteria");

    if exit_criteria_start.is_none() {
        // No exit criteria section, consider complete
        return Ok(true);
    }

    let start = exit_criteria_start.unwrap();
    let exit_criteria_end = lines[start + 1..]
        .iter()
        .position(|line| line.trim_start().starts_with("## "))
        .map(|pos| start + 1 + pos)
        .unwrap_or(lines.len());

    // Check all checkbox lines in the section
    for line in &lines[start + 1..exit_criteria_end] {
        if is_checkbox_line(line) && line.trim_start().starts_with("- [ ]") {
            return Ok(false); // Found incomplete criterion
        }
    }

    Ok(true) // All criteria are complete
}

/// Update frontmatter with exit criteria status and timestamp
fn update_frontmatter_exit_criteria(frontmatter_yaml: &str, all_complete: bool) -> Result<String> {
    if frontmatter_yaml.is_empty() {
        return Ok(String::new());
    }

    // Parse YAML
    let mut frontmatter: serde_yaml::Value =
        serde_yaml::from_str(frontmatter_yaml).map_err(|e| MetisError::ValidationFailed {
            message: format!("Failed to parse frontmatter YAML: {}", e),
        })?;

    // Update fields
    if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
        // Update exit_criteria_met
        map.insert(
            serde_yaml::Value::String("exit_criteria_met".to_string()),
            serde_yaml::Value::Bool(all_complete),
        );

        // Update updated_at timestamp
        let now = Utc::now();
        map.insert(
            serde_yaml::Value::String("updated_at".to_string()),
            serde_yaml::Value::String(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );
    }

    // Serialize back to YAML
    serde_yaml::to_string(&frontmatter).map_err(|e| MetisError::ValidationFailed {
        message: format!("Failed to serialize frontmatter YAML: {}", e),
    })
}

/// Validate blocked_by reference format
fn validate_blocked_by_references(blocked_by: &[String]) -> Result<()> {
    for reference in blocked_by {
        if !is_valid_wiki_link_reference(reference) {
            return Err(MetisError::ValidationFailed {
                message: format!(
                    "Invalid reference format '{}'. Expected format: '[[Document Title]]'",
                    reference
                ),
            });
        }
    }
    Ok(())
}

/// Check if a string is a valid wiki-link reference
fn is_valid_wiki_link_reference(reference: &str) -> bool {
    if reference.is_empty() {
        return false;
    }

    // Must start with [[ and end with ]]
    if !reference.starts_with("[[") || !reference.ends_with("]]") {
        return false;
    }

    // Extract the inner content
    let inner = &reference[2..reference.len() - 2];

    // Must not be empty and should not contain invalid characters
    !inner.is_empty() && !inner.contains("[[") && !inner.contains("]]")
}

/// Update frontmatter with new blocked_by list and timestamp
fn update_frontmatter_blocked_by(frontmatter_yaml: &str, blocked_by: &[String]) -> Result<String> {
    // Parse YAML
    let mut frontmatter: serde_yaml::Value =
        serde_yaml::from_str(frontmatter_yaml).map_err(|e| MetisError::ValidationFailed {
            message: format!("Failed to parse frontmatter YAML: {}", e),
        })?;

    // Update fields
    if let serde_yaml::Value::Mapping(ref mut map) = frontmatter {
        // Update blocked_by field
        if blocked_by.is_empty() {
            // Empty blocked_by should be represented as empty array or null
            map.insert(
                serde_yaml::Value::String("blocked_by".to_string()),
                serde_yaml::Value::Sequence(Vec::new()),
            );
        } else {
            let blocked_by_values: Vec<serde_yaml::Value> = blocked_by
                .iter()
                .map(|s| serde_yaml::Value::String(s.clone()))
                .collect();
            map.insert(
                serde_yaml::Value::String("blocked_by".to_string()),
                serde_yaml::Value::Sequence(blocked_by_values),
            );
        }

        // Update updated_at timestamp
        let now = Utc::now();
        map.insert(
            serde_yaml::Value::String("updated_at".to_string()),
            serde_yaml::Value::String(now.format("%Y-%m-%dT%H:%M:%SZ").to_string()),
        );
    }

    // Serialize back to YAML
    serde_yaml::to_string(&frontmatter).map_err(|e| MetisError::ValidationFailed {
        message: format!("Failed to serialize frontmatter YAML: {}", e),
    })
}

/// Write file atomically using a temporary file
fn write_file_atomic(path: &Path, content: &str) -> Result<()> {
    let temp_path = path.with_extension("tmp");

    // Write to temporary file first
    fs::write(&temp_path, content).map_err(MetisError::Io)?;

    // Atomic rename
    fs::rename(&temp_path, path).map_err(MetisError::Io)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_document(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(content.as_bytes()).unwrap();
        temp_file.flush().unwrap();
        temp_file
    }

    #[test]
    fn test_update_section_in_content_basic() {
        let content = r#"# Document Title

## Introduction

This is the introduction.

## Implementation

Old implementation details.

## Conclusion

This is the conclusion."#;

        let result = update_section_in_content(
            content,
            "Implementation",
            "New implementation details.\n\nWith multiple paragraphs.",
        );

        assert!(result.is_ok());
        let updated = result.unwrap();

        assert!(updated.contains("New implementation details."));
        assert!(updated.contains("With multiple paragraphs."));
        assert!(updated.contains("This is the introduction."));
        assert!(updated.contains("This is the conclusion."));
        assert!(!updated.contains("Old implementation details."));
    }

    #[test]
    fn test_update_section_in_content_last_section() {
        let content = r#"# Document Title

## Introduction

This is the introduction.

## Conclusion

Old conclusion."#;

        let result =
            update_section_in_content(content, "Conclusion", "New conclusion with more details.");

        assert!(result.is_ok());
        let updated = result.unwrap();

        assert!(updated.contains("New conclusion with more details."));
        assert!(updated.contains("This is the introduction."));
        assert!(!updated.contains("Old conclusion."));
    }

    #[test]
    fn test_update_section_in_content_empty_content() {
        let content = r#"# Document Title

## Introduction

This is the introduction.

## Implementation

Old implementation details.

## Conclusion

This is the conclusion."#;

        let result = update_section_in_content(content, "Implementation", "");

        assert!(result.is_ok());
        let updated = result.unwrap();

        assert!(!updated.contains("Old implementation details."));
        assert!(updated.contains("This is the introduction."));
        assert!(updated.contains("This is the conclusion."));

        // Section heading should still be present
        assert!(updated.contains("## Implementation"));
    }

    #[test]
    fn test_update_section_in_content_missing_section() {
        let content = r#"# Document Title

## Introduction

This is the introduction."#;

        let result = update_section_in_content(content, "NonExistent", "New content");

        assert!(result.is_err());
        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("H2 section '## NonExistent' not found"));
        } else {
            panic!("Expected ValidationFailed error");
        }
    }

    #[test]
    fn test_extract_frontmatter_yaml() {
        let content = r##"---
id: test-document
level: strategy
status: active
tags:
  - "#strategy"
  - "#active"
---

# Test Document

Content here."##;

        let result = extract_frontmatter_yaml(content);
        assert!(result.is_ok());

        let yaml = result.unwrap();
        assert!(yaml.contains("id: test-document"));
        assert!(yaml.contains("level: strategy"));
        assert!(yaml.contains("status: active"));
        assert!(!yaml.contains("# Test Document"));
    }

    #[test]
    fn test_extract_frontmatter_yaml_no_frontmatter() {
        let content = r#"# Test Document

Just content, no frontmatter."#;

        let result = extract_frontmatter_yaml(content);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn test_extract_frontmatter_yaml_invalid() {
        let content = r#"---
id: test-document
level: strategy
# Missing closing marker"#;

        let result = extract_frontmatter_yaml(content);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_update_document_content_full() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Problem Statement

This is the old problem statement.

## Solution Approach

This section needs updating.

## Exit Criteria

- [ ] Criterion 1
- [ ] Criterion 2"##;

        let temp_file = create_test_document(document_content);

        let result = update_document_content(
            temp_file.path(),
            "Solution Approach",
            "This is the updated solution approach.\n\nWith more detailed implementation notes.",
        )
        .await;

        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify updates
        assert!(updated_content.contains("This is the updated solution approach."));
        assert!(updated_content.contains("With more detailed implementation notes."));
        assert!(!updated_content.contains("This section needs updating."));

        // Verify other sections preserved
        assert!(updated_content.contains("This is the old problem statement."));
        assert!(updated_content.contains("- [ ] Criterion 1"));

        // Verify frontmatter preserved
        assert!(updated_content.contains("id: test-document"));
        assert!(updated_content.contains("level: strategy"));
    }

    #[tokio::test]
    async fn test_update_document_content_nonexistent_file() {
        let result =
            update_document_content(Path::new("/nonexistent/file.md"), "Section", "Content").await;

        assert!(result.is_err());
    }

    #[test]
    fn test_is_checkbox_line() {
        assert!(is_checkbox_line("- [ ] Incomplete task"));
        assert!(is_checkbox_line("- [x] Complete task"));
        assert!(is_checkbox_line("- [X] Complete task"));
        assert!(is_checkbox_line("  - [ ] Indented task"));

        assert!(!is_checkbox_line("- Regular list item"));
        assert!(!is_checkbox_line("Not a list"));
        assert!(!is_checkbox_line("- [invalid] Bad checkbox"));
    }

    #[test]
    fn test_update_checkbox_state() {
        assert_eq!(
            update_checkbox_state("- [ ] Complete this task", true),
            "- [x] Complete this task"
        );
        assert_eq!(
            update_checkbox_state("- [x] Complete this task", false),
            "- [ ] Complete this task"
        );
        assert_eq!(
            update_checkbox_state("- [X] Complete this task", false),
            "- [ ] Complete this task"
        );
        assert_eq!(
            update_checkbox_state("  - [ ] Indented task", true),
            "  - [x] Indented task"
        );
    }

    #[test]
    fn test_check_all_criteria_complete() {
        let content_all_complete = r#"# Document

## Exit Criteria

- [x] First criterion
- [x] Second criterion
- [x] Third criterion"#;

        let content_incomplete = r#"# Document

## Exit Criteria

- [x] First criterion
- [ ] Second criterion
- [x] Third criterion"#;

        let content_no_exit_criteria = r#"# Document

## Other Section

Some content."#;

        assert!(check_all_criteria_complete(content_all_complete).unwrap());
        assert!(!check_all_criteria_complete(content_incomplete).unwrap());
        assert!(check_all_criteria_complete(content_no_exit_criteria).unwrap());
    }

    #[test]
    fn test_update_checkbox_in_content() {
        let content = r#"# Document

## Exit Criteria

- [ ] First criterion to complete
- [x] Second criterion already done
- [ ] Third criterion pending"#;

        let result = update_checkbox_in_content(content, "First criterion", true);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert!(updated.contains("- [x] First criterion to complete"));
        assert!(updated.contains("- [x] Second criterion already done"));
        assert!(updated.contains("- [ ] Third criterion pending"));
    }

    #[test]
    fn test_update_checkbox_in_content_partial_match() {
        let content = r#"# Document

## Exit Criteria

- [ ] Complete the implementation
- [ ] Complete the testing
- [ ] Complete the documentation"#;

        let result = update_checkbox_in_content(content, "implementation", true);
        assert!(result.is_ok());

        let updated = result.unwrap();
        assert!(updated.contains("- [x] Complete the implementation"));
        assert!(updated.contains("- [ ] Complete the testing"));
        assert!(updated.contains("- [ ] Complete the documentation"));
    }

    #[test]
    fn test_update_checkbox_in_content_ambiguous_match() {
        let content = r#"# Document

## Exit Criteria

- [ ] Complete the implementation phase
- [ ] Complete the implementation testing
- [ ] Complete the documentation"#;

        let result = update_checkbox_in_content(content, "implementation", true);
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Multiple exit criteria match"));
        } else {
            panic!("Expected ValidationFailed error for ambiguous match");
        }
    }

    #[test]
    fn test_update_checkbox_in_content_no_match() {
        let content = r#"# Document

## Exit Criteria

- [ ] First criterion
- [ ] Second criterion"#;

        let result = update_checkbox_in_content(content, "nonexistent", true);
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("No exit criteria found matching"));
        } else {
            panic!("Expected ValidationFailed error for no match");
        }
    }

    #[test]
    fn test_update_checkbox_in_content_no_exit_criteria() {
        let content = r#"# Document

## Some Other Section

No exit criteria here."#;

        let result = update_checkbox_in_content(content, "anything", true);
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Exit Criteria section not found"));
        } else {
            panic!("Expected ValidationFailed error for missing section");
        }
    }

    #[tokio::test]
    async fn test_update_exit_criterion_full() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
exit_criteria_met: false
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Problem Statement

This is the problem statement.

## Exit Criteria

- [ ] Define the solution approach
- [ ] Complete implementation 
- [x] Write documentation

## Conclusion

Final thoughts."##;

        let temp_file = create_test_document(document_content);

        let result = update_exit_criterion(temp_file.path(), "solution approach", true).await;

        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify checkbox was updated
        assert!(updated_content.contains("- [x] Define the solution approach"));
        assert!(updated_content.contains("- [ ] Complete implementation"));
        assert!(updated_content.contains("- [x] Write documentation"));

        // Verify other content preserved
        assert!(updated_content.contains("This is the problem statement"));
        assert!(updated_content.contains("Final thoughts"));

        // Verify frontmatter updated (exit_criteria_met should still be false since not all complete)
        assert!(updated_content.contains("exit_criteria_met: false"));
        assert!(updated_content.contains("id: test-document"));

        // Verify updated_at was changed (should be current date)
        let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert!(updated_content.contains(&format!("updated_at: {}T", current_date)));
    }

    #[tokio::test]
    async fn test_update_exit_criterion_all_complete() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
exit_criteria_met: false
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Exit Criteria

- [x] First criterion
- [ ] Second criterion
- [x] Third criterion"##;

        let temp_file = create_test_document(document_content);

        // Complete the last remaining criterion
        let result = update_exit_criterion(temp_file.path(), "Second criterion", true).await;

        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify all checkboxes are complete
        assert!(updated_content.contains("- [x] First criterion"));
        assert!(updated_content.contains("- [x] Second criterion"));
        assert!(updated_content.contains("- [x] Third criterion"));

        // Verify exit_criteria_met is now true
        assert!(updated_content.contains("exit_criteria_met: true"));
    }

    #[test]
    fn test_is_valid_wiki_link_reference() {
        // Valid references
        assert!(is_valid_wiki_link_reference("[[Document Title]]"));
        assert!(is_valid_wiki_link_reference(
            "[[Complex Document Title with Spaces]]"
        ));
        assert!(is_valid_wiki_link_reference("[[Document-With-Hyphens]]"));
        assert!(is_valid_wiki_link_reference(
            "[[Document_With_Underscores]]"
        ));

        // Invalid references
        assert!(!is_valid_wiki_link_reference(""));
        assert!(!is_valid_wiki_link_reference("Document Title"));
        assert!(!is_valid_wiki_link_reference("[Document Title]"));
        assert!(!is_valid_wiki_link_reference("[[]]"));
        assert!(!is_valid_wiki_link_reference(
            "[[Document [[Nested]] Title]]"
        ));
        assert!(!is_valid_wiki_link_reference("Document Title]]"));
        assert!(!is_valid_wiki_link_reference("[[Document Title"));
    }

    #[test]
    fn test_validate_blocked_by_references() {
        // Valid references
        let valid_refs = vec![
            "[[Document One]]".to_string(),
            "[[Document Two]]".to_string(),
            "[[Another Document]]".to_string(),
        ];
        assert!(validate_blocked_by_references(&valid_refs).is_ok());

        // Empty list should be valid
        let empty_refs: Vec<String> = vec![];
        assert!(validate_blocked_by_references(&empty_refs).is_ok());

        // Invalid references
        let invalid_refs = vec![
            "[[Valid Document]]".to_string(),
            "Invalid Document".to_string(), // Missing brackets
        ];
        assert!(validate_blocked_by_references(&invalid_refs).is_err());

        let nested_refs = vec!["[[Document [[Nested]] Title]]".to_string()];
        assert!(validate_blocked_by_references(&nested_refs).is_err());
    }

    #[tokio::test]
    async fn test_update_blocked_by_basic() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
blocked_by: []
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Problem Statement

This is the problem statement."##;

        let temp_file = create_test_document(document_content);

        let new_blocked_by = vec![
            "[[Dependency One]]".to_string(),
            "[[Dependency Two]]".to_string(),
        ];

        let result = update_blocked_by(temp_file.path(), new_blocked_by).await;
        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify blocked_by was updated
        assert!(updated_content.contains("[[Dependency One]]"));
        assert!(updated_content.contains("[[Dependency Two]]"));

        // Verify other content preserved
        assert!(updated_content.contains("This is the problem statement"));
        assert!(updated_content.contains("id: test-document"));
        assert!(updated_content.contains("level: strategy"));

        // Verify updated_at was changed (should be current date)
        let current_date = chrono::Utc::now().format("%Y-%m-%d").to_string();
        assert!(updated_content.contains(&format!("updated_at: {}T", current_date)));

        // Verify frontmatter can be parsed again
        let parsed_again =
            gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&updated_content);
        assert!(parsed_again.data.is_some());
    }

    #[tokio::test]
    async fn test_update_blocked_by_clear_blockers() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
blocked_by:
  - "[[Old Dependency One]]"
  - "[[Old Dependency Two]]"
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Problem Statement

This is the problem statement."##;

        let temp_file = create_test_document(document_content);

        // Clear all blockers
        let empty_blocked_by: Vec<String> = vec![];

        let result = update_blocked_by(temp_file.path(), empty_blocked_by).await;
        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify blocked_by was cleared
        assert!(updated_content.contains("blocked_by: []"));
        assert!(!updated_content.contains("Old Dependency One"));
        assert!(!updated_content.contains("Old Dependency Two"));

        // Verify other content preserved
        assert!(updated_content.contains("This is the problem statement"));
        assert!(updated_content.contains("id: test-document"));

        // Verify frontmatter can be parsed again
        let parsed_again =
            gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&updated_content);
        assert!(parsed_again.data.is_some());
    }

    #[tokio::test]
    async fn test_update_blocked_by_replace_existing() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
blocked_by:
  - "[[Old Dependency]]"
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy

## Problem Statement

This is the problem statement."##;

        let temp_file = create_test_document(document_content);

        let new_blocked_by = vec![
            "[[New Dependency One]]".to_string(),
            "[[New Dependency Two]]".to_string(),
            "[[New Dependency Three]]".to_string(),
        ];

        let result = update_blocked_by(temp_file.path(), new_blocked_by).await;
        assert!(result.is_ok());

        // Read the updated file
        let updated_content = fs::read_to_string(temp_file.path()).unwrap();

        // Verify old blocked_by was replaced
        assert!(!updated_content.contains("Old Dependency"));
        assert!(updated_content.contains("[[New Dependency One]]"));
        assert!(updated_content.contains("[[New Dependency Two]]"));
        assert!(updated_content.contains("[[New Dependency Three]]"));

        // Verify other content preserved
        assert!(updated_content.contains("This is the problem statement"));

        // Verify frontmatter can be parsed again
        let parsed_again =
            gray_matter::Matter::<gray_matter::engine::YAML>::new().parse(&updated_content);
        assert!(parsed_again.data.is_some());
    }

    #[tokio::test]
    async fn test_update_blocked_by_invalid_reference() {
        let document_content = r##"---
id: test-document
level: strategy
status: active
blocked_by: []
created_at: 2025-07-03T18:00:00Z
updated_at: 2025-07-03T18:00:00Z
---

# Test Strategy"##;

        let temp_file = create_test_document(document_content);

        let invalid_blocked_by = vec![
            "[[Valid Dependency]]".to_string(),
            "Invalid Dependency".to_string(), // Missing brackets
        ];

        let result = update_blocked_by(temp_file.path(), invalid_blocked_by).await;
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Invalid reference format"));
            assert!(message.contains("Invalid Dependency"));
        } else {
            panic!("Expected ValidationFailed error for invalid reference");
        }
    }

    #[tokio::test]
    async fn test_update_blocked_by_no_frontmatter() {
        let document_content = r##"# Test Strategy

## Problem Statement

This document has no frontmatter."##;

        let temp_file = create_test_document(document_content);

        let blocked_by = vec!["[[Some Dependency]]".to_string()];

        let result = update_blocked_by(temp_file.path(), blocked_by).await;
        assert!(result.is_err());

        if let Err(MetisError::ValidationFailed { message }) = result {
            assert!(message.contains("Document has no frontmatter to update"));
        } else {
            panic!("Expected ValidationFailed error for missing frontmatter");
        }
    }
}
