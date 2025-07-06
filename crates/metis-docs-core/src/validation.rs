//! Document validation functions

use crate::{DocumentType, Result};
use serde_yaml::Value;
use std::path::Path;
use tokio::fs;

/// Result of document validation containing detailed error information
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub document_type: Option<DocumentType>,
    pub frontmatter_errors: Vec<String>,
}

impl ValidationResult {
    pub fn valid(document_type: DocumentType) -> Self {
        Self {
            is_valid: true,
            document_type: Some(document_type),
            frontmatter_errors: Vec::new(),
        }
    }

    pub fn invalid(errors: Vec<String>) -> Self {
        Self {
            is_valid: false,
            document_type: None,
            frontmatter_errors: errors,
        }
    }

    pub fn add_error(&mut self, error: String) {
        self.frontmatter_errors.push(error);
        self.is_valid = false;
    }
}

/// Validate a document from file path
pub async fn validate(document_path: &Path) -> Result<ValidationResult> {
    let content = fs::read_to_string(document_path)
        .await
        .map_err(crate::MetisError::Io)?;
    validate_content(&content)
}

/// Validate a document from content string
pub fn validate_content(content: &str) -> Result<ValidationResult> {
    // Extract frontmatter manually
    let frontmatter = match extract_frontmatter(content) {
        Some(fm) => fm,
        None => {
            return Ok(ValidationResult::invalid(vec![
                "No frontmatter found in document".to_string(),
            ]));
        }
    };

    let mut result = ValidationResult {
        is_valid: true,
        document_type: None,
        frontmatter_errors: Vec::new(),
    };

    // Extract and validate document type
    let doc_type = match extract_document_type(&frontmatter) {
        Ok(dt) => {
            result.document_type = Some(dt.clone());
            dt
        }
        Err(error) => {
            result.add_error(error);
            return Ok(result);
        }
    };

    // Validate required fields for the document type
    validate_required_fields(&frontmatter, &doc_type, &mut result);

    // Validate phase from tags
    validate_phase_from_tags(&frontmatter, &doc_type, &mut result);

    Ok(result)
}

/// Extract frontmatter from document content
fn extract_frontmatter(content: &str) -> Option<Value> {
    if !content.starts_with("---") {
        return None;
    }

    let mut lines = content.lines();
    lines.next(); // Skip first "---"

    let mut frontmatter_lines = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        frontmatter_lines.push(line);
    }

    let frontmatter_str = frontmatter_lines.join("\n");
    serde_yaml::from_str(&frontmatter_str).ok()
}

/// Extract document type from frontmatter
fn extract_document_type(frontmatter: &Value) -> std::result::Result<DocumentType, String> {
    let level = frontmatter
        .get("level")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing 'level' field in frontmatter".to_string())?;

    match level {
        "vision" => Ok(DocumentType::Vision),
        "strategy" => Ok(DocumentType::Strategy),
        "initiative" => Ok(DocumentType::Initiative),
        "task" => Ok(DocumentType::Task),
        "adr" => Ok(DocumentType::Adr),
        _ => Err(format!(
            "Invalid document level: '{}'. Must be one of: vision, strategy, initiative, task, adr",
            level
        )),
    }
}

/// Validate required fields for a specific document type
fn validate_required_fields(
    frontmatter: &Value,
    doc_type: &DocumentType,
    result: &mut ValidationResult,
) {
    // Check common required fields
    let required_common = ["id", "level", "status", "created_at", "updated_at"];
    for field in &required_common {
        if frontmatter.get(field).is_none() {
            result.add_error(format!("Missing required field: '{}'", field));
        }
    }

    // Check document-type specific required fields
    match doc_type {
        DocumentType::Strategy => {
            if frontmatter.get("risk_level").is_none() {
                result.add_error("Strategy documents require 'risk_level' field".to_string());
            } else if let Some(risk) = frontmatter.get("risk_level").and_then(|v| v.as_str()) {
                if !["low", "medium", "high", "critical"].contains(&risk) {
                    result.add_error(format!(
                        "Invalid risk_level '{}'. Must be: low, medium, high, or critical",
                        risk
                    ));
                }
            }
        }
        DocumentType::Initiative => {
            if frontmatter.get("estimated_complexity").is_none() {
                result.add_error(
                    "Initiative documents require 'estimated_complexity' field".to_string(),
                );
            } else if let Some(complexity) = frontmatter
                .get("estimated_complexity")
                .and_then(|v| v.as_str())
            {
                if !["S", "M", "L", "XL"].contains(&complexity) {
                    result.add_error(format!(
                        "Invalid complexity '{}'. Must be: S, M, L, or XL",
                        complexity
                    ));
                }
            }
        }
        DocumentType::Adr => {
            if frontmatter.get("decision_maker").is_none() {
                result.add_error("ADR documents require 'decision_maker' field".to_string());
            }
            // Note: number field is optional in our templates, so not requiring it
        }
        DocumentType::Vision | DocumentType::Task => {
            // No additional required fields for these types
        }
    }
}

/// Validate phase from tags array
fn validate_phase_from_tags(
    frontmatter: &Value,
    doc_type: &DocumentType,
    result: &mut ValidationResult,
) {
    let tags = match frontmatter.get("tags").and_then(|v| v.as_sequence()) {
        Some(tags) => tags,
        None => {
            result.add_error("Missing 'tags' field in frontmatter".to_string());
            return;
        }
    };

    // Extract phase tags
    let phase_tags: Vec<String> = tags
        .iter()
        .filter_map(|tag| tag.as_str())
        .filter(|tag| tag.starts_with("#phase/"))
        .map(|tag| tag.strip_prefix("#phase/").unwrap().to_string())
        .collect();

    if phase_tags.is_empty() {
        result.add_error("Document must have at least one #phase/ tag".to_string());
        return;
    }

    if phase_tags.len() > 1 {
        result.add_error(format!(
            "Document has multiple active phase tags: {}. Only one phase should be active.",
            phase_tags.join(", ")
        ));
        return;
    }

    let current_phase = &phase_tags[0];

    // Validate phase against document type
    let valid_phases = match doc_type {
        DocumentType::Vision => vec!["draft", "review", "published"],
        DocumentType::Strategy => vec!["shaping", "design", "ready", "active", "completed"],
        DocumentType::Initiative => vec![
            "discovery",
            "design",
            "ready",
            "decompose",
            "active",
            "completed",
        ],
        DocumentType::Task => vec!["todo", "doing", "completed"],
        DocumentType::Adr => vec!["draft", "discussion", "decided", "superseded"],
    };

    if !valid_phases.contains(&current_phase.as_str()) {
        result.add_error(format!(
            "Invalid phase '{}' for {} document. Valid phases: {}",
            current_phase,
            format!("{:?}", doc_type).to_lowercase(),
            valid_phases.join(", ")
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_strategy_document() {
        let content = "---
id: strategy-test
level: strategy
status: shaping
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
parent: 
blocked_by: 
tags:
  - \"#strategy\"
  - \"#phase/shaping\"
exit_criteria_met: false
success_metrics: []
risk_level: high
stakeholders: []
---

# Test Strategy

Content here...
";

        let result = validate_content(content).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.document_type, Some(DocumentType::Strategy));
        assert!(result.frontmatter_errors.is_empty());
    }

    #[test]
    fn test_validate_missing_frontmatter() {
        let content = "# Just some markdown without frontmatter";

        let result = validate_content(content).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.document_type, None);
        assert_eq!(
            result.frontmatter_errors,
            vec!["No frontmatter found in document"]
        );
    }

    #[test]
    fn test_validate_invalid_level() {
        let content = "---
id: test-doc
level: invalid_level
status: draft
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
tags:
  - \"#test\"
---

# Test Document
";

        let result = validate_content(content).unwrap();
        assert!(!result.is_valid);
        assert_eq!(result.document_type, None);
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("Invalid document level")));
    }

    #[test]
    fn test_validate_missing_required_fields() {
        let content = "---
level: strategy
---

# Incomplete Strategy
";

        let result = validate_content(content).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("Missing required field: 'id'")));
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("Missing required field: 'status'")));
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("Strategy documents require 'risk_level'")));
    }

    #[test]
    fn test_validate_invalid_phase_tags() {
        let content = "---
id: strategy-test
level: strategy
status: shaping
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
risk_level: high
tags:
  - \"#strategy\"
  - \"#phase/invalid_phase\"
---

# Test Strategy
";

        let result = validate_content(content).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("Invalid phase 'invalid_phase'")));
    }

    #[test]
    fn test_validate_multiple_phase_tags() {
        let content = "---
id: strategy-test
level: strategy
status: shaping
created_at: 2025-07-03T12:00:00Z
updated_at: 2025-07-03T12:00:00Z
risk_level: high
tags:
  - \"#strategy\"
  - \"#phase/shaping\"
  - \"#phase/design\"
---

# Test Strategy
";

        let result = validate_content(content).unwrap();
        assert!(!result.is_valid);
        assert!(result
            .frontmatter_errors
            .iter()
            .any(|e| e.contains("multiple active phase tags")));
    }
}
