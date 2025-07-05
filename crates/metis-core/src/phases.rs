//! Phase transition management for documents

use crate::validation::validate_content;
use crate::{DocumentType, MetisError, Result};
use std::path::Path;
use tokio::fs;

/// Transition a document to a new phase with validation and file updates
pub async fn transition_phase(
    document_path: &Path,
    new_phase: &str,
    force: bool,
) -> Result<String> {
    // Read current document content
    let content = fs::read_to_string(document_path)
        .await
        .map_err(MetisError::Io)?;

    // Validate current document and extract document type
    let validation_result = validate_content(&content)?;
    if !validation_result.is_valid && !force {
        return Err(MetisError::ValidationFailed {
            message: format!(
                "Document validation failed: {:?}",
                validation_result.frontmatter_errors
            ),
        });
    }

    let doc_type = validation_result
        .document_type
        .ok_or_else(|| MetisError::ValidationFailed {
            message: "Cannot determine document type".to_string(),
        })?;

    // Extract current phase from content
    let current_phase = extract_current_phase(&content)?;

    // Check if transition is valid (unless forced)
    if !force && !is_valid_transition(&doc_type, &current_phase, new_phase) {
        return Err(MetisError::ValidationFailed {
            message: format!(
                "Invalid phase transition from '{}' to '{}' for {} document. Valid transitions: {}",
                current_phase,
                new_phase,
                format!("{:?}", doc_type).to_lowercase(),
                get_valid_transitions(&doc_type, &current_phase).join(", ")
            ),
        });
    }

    // Update document content with new phase
    let updated_content = update_phase_in_content(&content, &current_phase, new_phase)?;

    // Write updated content back to file
    fs::write(document_path, &updated_content)
        .await
        .map_err(MetisError::Io)?;

    Ok(updated_content)
}

/// Check if a phase transition is allowed without making changes
pub async fn can_transition_to_phase(document_path: &Path, target_phase: &str) -> Result<bool> {
    // Read current document content
    let content = fs::read_to_string(document_path)
        .await
        .map_err(MetisError::Io)?;

    // Validate current document and extract document type
    let validation_result = validate_content(&content)?;
    if !validation_result.is_valid {
        return Ok(false);
    }

    let doc_type = validation_result
        .document_type
        .ok_or_else(|| MetisError::ValidationFailed {
            message: "Cannot determine document type".to_string(),
        })?;

    // Extract current phase from content
    let current_phase = extract_current_phase(&content)?;

    // Check if transition is valid
    Ok(is_valid_transition(&doc_type, &current_phase, target_phase))
}

/// Extract the current active phase from document content
fn extract_current_phase(content: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut in_frontmatter = false;
    let mut in_tags = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == "---" {
            if !in_frontmatter {
                in_frontmatter = true;
                continue;
            } else {
                break; // End of frontmatter
            }
        }

        if !in_frontmatter {
            continue;
        }

        if trimmed == "tags:" {
            in_tags = true;
            continue;
        }

        if in_tags {
            // Look for active phase tag (not commented out)
            if trimmed.starts_with("- \"#phase/") && !trimmed.starts_with("# ") {
                if let Some(phase_start) = trimmed.find("#phase/") {
                    let phase_part = &trimmed[phase_start + 7..]; // Skip "#phase/"
                    if let Some(end_quote) = phase_part.find('"') {
                        return Ok(phase_part[..end_quote].to_string());
                    }
                }
            }

            // If we hit a line that doesn't start with spaces/dashes, we're out of tags
            if !trimmed.is_empty() && !trimmed.starts_with("-") && !trimmed.starts_with("#") {
                break;
            }
        }
    }

    Err(MetisError::ValidationFailed {
        message: "No active phase tag found in document".to_string(),
    })
}

/// Update the phase in document content using comment/uncomment logic
fn update_phase_in_content(content: &str, current_phase: &str, new_phase: &str) -> Result<String> {
    let lines: Vec<&str> = content.lines().collect();
    let mut updated_lines = Vec::new();
    let mut in_frontmatter = false;
    let mut in_tags = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed == "---" {
            updated_lines.push(line.to_string());
            if !in_frontmatter {
                in_frontmatter = true;
            } else {
                in_frontmatter = false; // End of frontmatter
                in_tags = false;
            }
            continue;
        }

        if !in_frontmatter {
            updated_lines.push(line.to_string());
            continue;
        }

        if trimmed == "tags:" {
            updated_lines.push(line.to_string());
            in_tags = true;
            continue;
        }

        if in_tags {
            // Handle phase tag lines
            if trimmed.contains("#phase/") {
                let is_current_phase = trimmed.contains(&format!("#phase/{}", current_phase));
                let is_target_phase = trimmed.contains(&format!("#phase/{}", new_phase));

                if is_current_phase && !trimmed.starts_with("#") {
                    // Comment out the current phase
                    let spaces = line.len() - line.trim_start().len();
                    updated_lines.push(format!("{}# {}", " ".repeat(spaces), trimmed));
                } else if is_target_phase && trimmed.starts_with("#") {
                    // Uncomment the target phase
                    let uncommented = trimmed.strip_prefix("# ").unwrap_or(trimmed);
                    let spaces = line.len() - line.trim_start().len();
                    updated_lines.push(format!("{}{}", " ".repeat(spaces), uncommented));
                } else {
                    // Keep other phase tags as-is
                    updated_lines.push(line.to_string());
                }
            } else {
                updated_lines.push(line.to_string());
                // If we hit a line that doesn't start with spaces/dashes, we're out of tags
                if !trimmed.is_empty() && !trimmed.starts_with("-") && !trimmed.starts_with("#") {
                    in_tags = false;
                }
            }
        } else {
            updated_lines.push(line.to_string());
        }
    }

    Ok(updated_lines.join("\n"))
}

/// Check if a phase transition is valid according to business rules
fn is_valid_transition(doc_type: &DocumentType, current_phase: &str, target_phase: &str) -> bool {
    let valid_transitions = get_valid_transitions(doc_type, current_phase);
    valid_transitions.contains(&target_phase.to_string())
}

/// Get valid next phases for a document type and current phase
fn get_valid_transitions(doc_type: &DocumentType, current_phase: &str) -> Vec<String> {
    match doc_type {
        DocumentType::Vision => match current_phase {
            "draft" => vec!["review".to_string()],
            "review" => vec!["draft".to_string(), "published".to_string()],
            "published" => vec!["review".to_string()], // Can go back for updates
            _ => vec![],
        },
        DocumentType::Strategy => match current_phase {
            "shaping" => vec!["design".to_string()],
            "design" => vec!["shaping".to_string(), "ready".to_string()],
            "ready" => vec!["design".to_string(), "active".to_string()],
            "active" => vec!["ready".to_string(), "completed".to_string()],
            "completed" => vec!["active".to_string()], // Can reopen if needed
            _ => vec![],
        },
        DocumentType::Initiative => match current_phase {
            "discovery" => vec!["design".to_string()],
            "design" => vec!["discovery".to_string(), "ready".to_string()],
            "ready" => vec!["design".to_string(), "decompose".to_string()],
            "decompose" => vec!["ready".to_string(), "active".to_string()],
            "active" => vec!["decompose".to_string(), "completed".to_string()],
            "completed" => vec!["active".to_string()], // Can reopen if needed
            _ => vec![],
        },
        DocumentType::Task => match current_phase {
            "todo" => vec!["doing".to_string()],
            "doing" => vec!["todo".to_string(), "completed".to_string()],
            "completed" => vec!["doing".to_string()], // Can reopen if needed
            _ => vec![],
        },
        DocumentType::Adr => match current_phase {
            "draft" => vec!["discussion".to_string()],
            "discussion" => vec!["draft".to_string(), "decided".to_string()],
            "decided" => vec!["superseded".to_string()],
            "superseded" => vec![], // Terminal state
            _ => vec![],
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_extract_current_phase() {
        let content = "---
id: test-strategy
level: strategy
status: active
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#strategy\"
  # - \"#phase/shaping\"
  # - \"#phase/design\"
  - \"#phase/active\"
  # - \"#phase/completed\"
---

# Test Strategy

Content here...
";

        let phase = extract_current_phase(content).unwrap();
        assert_eq!(phase, "active");
    }

    #[test]
    fn test_extract_current_phase_no_active() {
        let content = "---
id: test-strategy
level: strategy
status: active
tags:
  - \"#strategy\"
  # - \"#phase/shaping\"
  # - \"#phase/design\"
  # - \"#phase/active\"
---

# Test Strategy
";

        let result = extract_current_phase(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_update_phase_in_content() {
        let content = "---
id: test-strategy
level: strategy
status: active
tags:
  - \"#strategy\"
  # - \"#phase/shaping\"
  # - \"#phase/design\"
  - \"#phase/active\"
  # - \"#phase/completed\"
---

# Test Strategy

Content here...
";

        let updated = update_phase_in_content(content, "active", "completed").unwrap();

        assert!(updated.contains("# - \"#phase/active\""));
        assert!(updated.contains("- \"#phase/completed\""));
        assert!(!updated.contains("# - \"#phase/completed\""));
    }

    #[test]
    fn test_valid_strategy_transitions() {
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "shaping",
            "design"
        ));
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "design",
            "ready"
        ));
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "ready",
            "active"
        ));
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "active",
            "completed"
        ));

        // Backward transitions
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "design",
            "shaping"
        ));
        assert!(is_valid_transition(
            &DocumentType::Strategy,
            "ready",
            "design"
        ));

        // Invalid transitions
        assert!(!is_valid_transition(
            &DocumentType::Strategy,
            "shaping",
            "active"
        ));
        assert!(!is_valid_transition(
            &DocumentType::Strategy,
            "completed",
            "shaping"
        ));
    }

    #[test]
    fn test_valid_task_transitions() {
        assert!(is_valid_transition(&DocumentType::Task, "todo", "doing"));
        assert!(is_valid_transition(
            &DocumentType::Task,
            "doing",
            "completed"
        ));
        assert!(is_valid_transition(
            &DocumentType::Task,
            "completed",
            "doing"
        ));

        // Invalid transitions
        assert!(!is_valid_transition(
            &DocumentType::Task,
            "todo",
            "completed"
        ));
    }

    #[test]
    fn test_valid_adr_transitions() {
        assert!(is_valid_transition(
            &DocumentType::Adr,
            "draft",
            "discussion"
        ));
        assert!(is_valid_transition(
            &DocumentType::Adr,
            "discussion",
            "decided"
        ));
        assert!(is_valid_transition(
            &DocumentType::Adr,
            "decided",
            "superseded"
        ));

        // Backward transition
        assert!(is_valid_transition(
            &DocumentType::Adr,
            "discussion",
            "draft"
        ));

        // Invalid transitions
        assert!(!is_valid_transition(&DocumentType::Adr, "draft", "decided"));
        assert!(!is_valid_transition(
            &DocumentType::Adr,
            "superseded",
            "decided"
        ));
    }

    #[tokio::test]
    async fn test_transition_phase_file() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "---
id: test-strategy
level: strategy
status: active
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#strategy\"
  # - \"#phase/shaping\"
  # - \"#phase/design\"
  - \"#phase/active\"
  # - \"#phase/completed\"
risk_level: medium
---

# Test Strategy

Content here...
";

        temp_file.write_all(content.as_bytes()).unwrap();
        let file_path = temp_file.path();

        // Transition from active to completed
        let result = transition_phase(file_path, "completed", false).await;
        assert!(result.is_ok());

        let updated_content = result.unwrap();
        assert!(updated_content.contains("# - \"#phase/active\""));
        assert!(updated_content.contains("- \"#phase/completed\""));

        // Verify file was actually updated
        let file_content = fs::read_to_string(file_path).await.unwrap();
        assert_eq!(file_content, updated_content);
    }

    #[tokio::test]
    async fn test_invalid_transition_rejected() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "---
id: test-strategy
level: strategy
status: active
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#strategy\"
  - \"#phase/active\"
risk_level: medium
---

# Test Strategy
";

        temp_file.write_all(content.as_bytes()).unwrap();
        let file_path = temp_file.path();

        // Try invalid transition from active to shaping
        let result = transition_phase(file_path, "shaping", false).await;
        assert!(result.is_err());

        // Verify file was not changed
        let file_content = fs::read_to_string(file_path).await.unwrap();
        assert_eq!(file_content, content);
    }

    #[tokio::test]
    async fn test_force_invalid_transition() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "---
id: test-strategy
level: strategy
status: active
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#strategy\"
  - \"#phase/active\"
  # - \"#phase/shaping\"
risk_level: medium
---

# Test Strategy
";

        temp_file.write_all(content.as_bytes()).unwrap();
        let file_path = temp_file.path();

        // Force invalid transition from active to shaping
        let result = transition_phase(file_path, "shaping", true).await;
        assert!(result.is_ok());

        let updated_content = result.unwrap();
        assert!(updated_content.contains("# - \"#phase/active\""));
        assert!(updated_content.contains("- \"#phase/shaping\""));
    }

    #[tokio::test]
    async fn test_can_transition_to_phase() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let content = "---
id: test-strategy
level: strategy
status: active
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#strategy\"
  - \"#phase/active\"
risk_level: medium
---

# Test Strategy
";

        temp_file.write_all(content.as_bytes()).unwrap();
        let file_path = temp_file.path();

        // Can transition to completed
        let can_complete = can_transition_to_phase(file_path, "completed")
            .await
            .unwrap();
        assert!(can_complete);

        // Cannot transition to shaping
        let can_shape = can_transition_to_phase(file_path, "shaping").await.unwrap();
        assert!(!can_shape);
    }
}
