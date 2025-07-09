use crate::domain::documents::types::DocumentType;
use crate::{Vision, Strategy, Initiative, Task, Adr, MetisError};
use crate::Result;
use std::path::Path;

/// Service for validating documents and detecting their types
pub struct DocumentValidationService;

/// Result of document validation
#[derive(Debug)]
pub struct ValidationResult {
    pub document_type: DocumentType,
    pub is_valid: bool,
    pub errors: Vec<String>,
}

impl DocumentValidationService {
    /// Create a new document validation service
    pub fn new() -> Self {
        Self
    }

    /// Validate a document file and detect its type
    pub async fn validate_document<P: AsRef<Path>>(&self, file_path: P) -> Result<ValidationResult> {
        let file_path = file_path.as_ref();
        
        // Check if file exists
        if !file_path.exists() {
            return Err(MetisError::NotFound("File does not exist".to_string()));
        }

        if !file_path.is_file() {
            return Err(MetisError::NotFound("Path is not a file".to_string()));
        }

        // Try to parse as each document type and collect results
        let mut validation_results = Vec::new();
        
        // Try Vision
        match Vision::from_file(file_path).await {
            Ok(_vision) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Vision,
                    is_valid: true,
                    errors: vec![],
                });
            }
            Err(e) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Vision,
                    is_valid: false,
                    errors: vec![format!("Vision validation failed: {}", e)],
                });
            }
        }
        
        // Try Strategy
        match Strategy::from_file(file_path).await {
            Ok(_strategy) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Strategy,
                    is_valid: true,
                    errors: vec![],
                });
            }
            Err(e) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Strategy,
                    is_valid: false,
                    errors: vec![format!("Strategy validation failed: {}", e)],
                });
            }
        }
        
        // Try Initiative
        match Initiative::from_file(file_path).await {
            Ok(_initiative) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Initiative,
                    is_valid: true,
                    errors: vec![],
                });
            }
            Err(e) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Initiative,
                    is_valid: false,
                    errors: vec![format!("Initiative validation failed: {}", e)],
                });
            }
        }
        
        // Try Task
        match Task::from_file(file_path).await {
            Ok(_task) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Task,
                    is_valid: true,
                    errors: vec![],
                });
            }
            Err(e) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Task,
                    is_valid: false,
                    errors: vec![format!("Task validation failed: {}", e)],
                });
            }
        }
        
        // Try ADR
        match Adr::from_file(file_path).await {
            Ok(_adr) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Adr,
                    is_valid: true,
                    errors: vec![],
                });
            }
            Err(e) => {
                validation_results.push(ValidationResult {
                    document_type: DocumentType::Adr,
                    is_valid: false,
                    errors: vec![format!("ADR validation failed: {}", e)],
                });
            }
        }
        
        // Find the first valid result
        if let Some(valid_result) = validation_results.iter().find(|r| r.is_valid) {
            return Ok(ValidationResult {
                document_type: valid_result.document_type,
                is_valid: true,
                errors: vec![],
            });
        }
        
        // If no valid results, return combined errors
        let all_errors: Vec<String> = validation_results
            .into_iter()
            .flat_map(|r| r.errors)
            .collect();
        
        Ok(ValidationResult {
            document_type: DocumentType::Vision, // Default, since we couldn't determine
            is_valid: false,
            errors: all_errors,
        })
    }

    /// Validate a document and return just the document type (simpler interface)
    pub async fn detect_document_type<P: AsRef<Path>>(&self, file_path: P) -> Result<DocumentType> {
        let result = self.validate_document(file_path).await?;
        
        if result.is_valid {
            Ok(result.document_type)
        } else {
            Err(MetisError::InvalidDocument(
                format!("Could not determine document type: {}", result.errors.join("; "))
            ))
        }
    }

    /// Validate a document of a specific expected type
    pub async fn validate_document_as_type<P: AsRef<Path>>(
        &self,
        file_path: P,
        expected_type: DocumentType,
    ) -> Result<bool> {
        let file_path = file_path.as_ref();
        
        match expected_type {
            DocumentType::Vision => {
                match Vision::from_file(file_path).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DocumentType::Strategy => {
                match Strategy::from_file(file_path).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DocumentType::Initiative => {
                match Initiative::from_file(file_path).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DocumentType::Task => {
                match Task::from_file(file_path).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
            DocumentType::Adr => {
                match Adr::from_file(file_path).await {
                    Ok(_) => Ok(true),
                    Err(_) => Ok(false),
                }
            }
        }
    }

    /// Check if a document is valid without loading the full document
    pub async fn is_valid_document<P: AsRef<Path>>(&self, file_path: P) -> bool {
        self.validate_document(file_path).await
            .map(|result| result.is_valid)
            .unwrap_or(false)
    }
}

impl Default for DocumentValidationService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[tokio::test]
    async fn test_validate_valid_vision_document() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("vision.md");
        
        // Create a valid vision document
        let vision_content = r##"---
id: test-vision
title: Test Vision
level: vision
created_at: 2023-01-01T00:00:00Z
updated_at: 2023-01-01T00:00:00Z
archived: false
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
---

# Test Vision

This is a test vision document.
"##;
        fs::write(&file_path, vision_content).unwrap();

        let service = DocumentValidationService::new();
        let result = service.validate_document(&file_path).await.unwrap();
        
        assert!(result.is_valid);
        assert_eq!(result.document_type, DocumentType::Vision);
        assert!(result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_invalid_document() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("invalid.md");
        
        // Create an invalid document
        let invalid_content = r##"# Invalid Document

This has no frontmatter.
"##;
        fs::write(&file_path, invalid_content).unwrap();

        let service = DocumentValidationService::new();
        let result = service.validate_document(&file_path).await.unwrap();
        
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[tokio::test]
    async fn test_detect_document_type() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("vision.md");
        
        // Create a valid vision document
        let vision_content = r##"---
id: test-vision
title: Test Vision
level: vision
created_at: 2023-01-01T00:00:00Z
updated_at: 2023-01-01T00:00:00Z
archived: false
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
---

# Test Vision

This is a test vision document.
"##;
        fs::write(&file_path, vision_content).unwrap();

        let service = DocumentValidationService::new();
        let doc_type = service.detect_document_type(&file_path).await.unwrap();
        
        assert_eq!(doc_type, DocumentType::Vision);
    }

    #[tokio::test]
    async fn test_validate_document_as_type() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("vision.md");
        
        // Create a valid vision document
        let vision_content = r##"---
id: test-vision
title: Test Vision
level: vision
created_at: 2023-01-01T00:00:00Z
updated_at: 2023-01-01T00:00:00Z
archived: false
tags:
  - "#vision"
  - "#phase/draft"
exit_criteria_met: false
---

# Test Vision

This is a test vision document.
"##;
        fs::write(&file_path, vision_content).unwrap();

        let service = DocumentValidationService::new();
        
        // Should be valid as vision
        assert!(service.validate_document_as_type(&file_path, DocumentType::Vision).await.unwrap());
        
        // Should not be valid as strategy
        assert!(!service.validate_document_as_type(&file_path, DocumentType::Strategy).await.unwrap());
    }

    #[tokio::test]
    async fn test_validate_nonexistent_file() {
        let service = DocumentValidationService::new();
        let result = service.validate_document("/nonexistent/file.md").await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), MetisError::NotFound(_)));
    }
}