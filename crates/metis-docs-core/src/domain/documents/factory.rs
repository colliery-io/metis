use super::{
    adr::Adr,
    initiative::Initiative, 
    strategy::Strategy,
    task::Task,
    vision::Vision,
    traits::{Document, DocumentValidationError},
    types::DocumentType,
};
use gray_matter::{Matter, engine::YAML};
use std::path::Path;

/// Factory for creating documents from files
/// Determines document type from frontmatter and creates appropriate document instance
pub struct DocumentFactory;

impl DocumentFactory {
    /// Create a document from a file path
    /// Reads the file, determines type from frontmatter, then creates the appropriate document
    pub async fn from_file<P: AsRef<Path>>(path: P) -> Result<Box<dyn Document>, DocumentValidationError> {
        // First, read the file to extract frontmatter and determine type
        let raw_content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| DocumentValidationError::InvalidContent(format!("Failed to read file: {}", e)))?;
        
        let doc_type = Self::extract_document_type(&raw_content)?;
        
        // Create the appropriate document type
        match doc_type {
            DocumentType::Vision => {
                let vision = Vision::from_file(path).await?;
                Ok(Box::new(vision))
            }
            DocumentType::Strategy => {
                let strategy = Strategy::from_file(path).await?;
                Ok(Box::new(strategy))
            }
            DocumentType::Initiative => {
                let initiative = Initiative::from_file(path).await?;
                Ok(Box::new(initiative))
            }
            DocumentType::Task => {
                let task = Task::from_file(path).await?;
                Ok(Box::new(task))
            }
            DocumentType::Adr => {
                let adr = Adr::from_file(path).await?;
                Ok(Box::new(adr))
            }
        }
    }
    
    /// Create a document from raw content string
    pub fn from_content(raw_content: &str, _filepath: &str) -> Result<Box<dyn Document>, DocumentValidationError> {
        let doc_type = Self::extract_document_type(raw_content)?;
        
        match doc_type {
            DocumentType::Vision => {
                let vision = Vision::from_content(raw_content)?;
                Ok(Box::new(vision))
            }
            DocumentType::Strategy => {
                let strategy = Strategy::from_content(raw_content)?;
                Ok(Box::new(strategy))
            }
            DocumentType::Initiative => {
                let initiative = Initiative::from_content(raw_content)?;
                Ok(Box::new(initiative))
            }
            DocumentType::Task => {
                let task = Task::from_content(raw_content)?;
                Ok(Box::new(task))
            }
            DocumentType::Adr => {
                let adr = Adr::from_content(raw_content)?;
                Ok(Box::new(adr))
            }
        }
    }
    
    /// Extract document type from frontmatter
    fn extract_document_type(raw_content: &str) -> Result<DocumentType, DocumentValidationError> {
        // Parse frontmatter
        let matter = Matter::<YAML>::new();
        let parsed = matter.parse(raw_content);
        
        let frontmatter = parsed.data.ok_or_else(|| {
            DocumentValidationError::MissingRequiredField("frontmatter".to_string())
        })?;
        
        let fm_map = match frontmatter {
            gray_matter::Pod::Hash(map) => map,
            _ => return Err(DocumentValidationError::InvalidContent("Frontmatter must be a hash/map".to_string())),
        };
        
        // Try different field names that might contain the document type
        let type_str = if let Some(gray_matter::Pod::String(s)) = fm_map.get("document_type") {
            s.clone()
        } else if let Some(gray_matter::Pod::String(s)) = fm_map.get("level") {
            s.clone()
        } else if let Some(gray_matter::Pod::String(s)) = fm_map.get("type") {
            s.clone()
        } else {
            return Err(DocumentValidationError::MissingRequiredField("document_type, level, or type".to_string()));
        };
        
        // Parse into DocumentType
        type_str.parse::<DocumentType>().map_err(|_| {
            DocumentValidationError::InvalidContent(format!("Unknown document type: {}", type_str))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_extract_document_type() {
        let vision_content = r#"---
document_type: vision
title: Test Vision
---

# Test Vision
"#;
        
        let doc_type = DocumentFactory::extract_document_type(vision_content)
            .expect("Failed to extract document type");
        assert_eq!(doc_type, DocumentType::Vision);
        
        // Test with "level" field (legacy)
        let strategy_content = r#"---
level: strategy
title: Test Strategy
---

# Test Strategy
"#;
        
        let doc_type = DocumentFactory::extract_document_type(strategy_content)
            .expect("Failed to extract document type");
        assert_eq!(doc_type, DocumentType::Strategy);
    }
    
    #[test]
    fn test_extract_document_type_missing() {
        let content = r#"---
title: Test Document
---

# Test Document
"#;
        
        let result = DocumentFactory::extract_document_type(content);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_extract_document_type_invalid() {
        let content = r#"---
document_type: invalid_type
title: Test Document
---

# Test Document
"#;
        
        let result = DocumentFactory::extract_document_type(content);
        assert!(result.is_err());
    }
}