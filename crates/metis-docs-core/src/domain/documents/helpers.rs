use super::traits::DocumentValidationError;
use super::types::Tag;
use gray_matter;
use chrono::{DateTime, Utc};

/// Helper methods for parsing frontmatter
pub struct FrontmatterParser;

impl FrontmatterParser {
    pub fn extract_string(map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str) -> Result<String, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::String(s)) => Ok(s.clone()),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!("{} must be a string", key))),
            None => Err(DocumentValidationError::MissingRequiredField(key.to_string())),
        }
    }

    pub fn extract_bool(map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str) -> Result<bool, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Boolean(b)) => Ok(*b),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!("{} must be a boolean", key))),
            None => Err(DocumentValidationError::MissingRequiredField(key.to_string())),
        }
    }

    pub fn extract_integer(map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str) -> Result<i64, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Integer(i)) => Ok(*i),
            Some(_) => Err(DocumentValidationError::InvalidContent(format!("{} must be an integer", key))),
            None => Err(DocumentValidationError::MissingRequiredField(key.to_string())),
        }
    }

    pub fn extract_datetime(map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str) -> Result<DateTime<Utc>, DocumentValidationError> {
        let date_str = Self::extract_string(map, key)?;
        DateTime::parse_from_rfc3339(&date_str)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(|_| DocumentValidationError::InvalidContent(format!("Invalid datetime format for {}", key)))
    }

    pub fn extract_tags(map: &std::collections::HashMap<String, gray_matter::Pod>) -> Result<Vec<Tag>, DocumentValidationError> {
        match map.get("tags") {
            Some(gray_matter::Pod::Array(arr)) => {
                let mut tags = Vec::new();
                for item in arr {
                    if let gray_matter::Pod::String(tag_str) = item {
                        if let Ok(tag) = tag_str.parse::<Tag>() {
                            tags.push(tag);
                        }
                    }
                }
                Ok(tags)
            }
            Some(_) => Err(DocumentValidationError::InvalidContent("tags must be an array".to_string())),
            None => Err(DocumentValidationError::MissingRequiredField("tags".to_string())),
        }
    }

    pub fn extract_string_array(map: &std::collections::HashMap<String, gray_matter::Pod>, key: &str) -> Result<Vec<String>, DocumentValidationError> {
        match map.get(key) {
            Some(gray_matter::Pod::Array(arr)) => {
                let mut strings = Vec::new();
                for item in arr {
                    if let gray_matter::Pod::String(s) = item {
                        strings.push(s.clone());
                    }
                }
                Ok(strings)
            }
            Some(_) => Err(DocumentValidationError::InvalidContent(format!("{} must be an array", key))),
            None => Err(DocumentValidationError::MissingRequiredField(key.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use gray_matter::Pod;
    use super::super::types::{Phase, Tag};

    fn create_test_map() -> HashMap<String, Pod> {
        let mut map = HashMap::new();
        map.insert("string_field".to_string(), Pod::String("test_value".to_string()));
        map.insert("bool_field".to_string(), Pod::Boolean(true));
        map.insert("integer_field".to_string(), Pod::Integer(42));
        map.insert("date_field".to_string(), Pod::String("2025-01-01T12:00:00Z".to_string()));
        map.insert("tags".to_string(), Pod::Array(vec![
            Pod::String("#phase/draft".to_string()),
            Pod::String("#vision".to_string()),
            Pod::String("urgent".to_string()),
        ]));
        map.insert("string_array".to_string(), Pod::Array(vec![
            Pod::String("item1".to_string()),
            Pod::String("item2".to_string()),
        ]));
        map
    }

    #[test]
    fn test_extract_string() {
        let map = create_test_map();
        
        assert_eq!(FrontmatterParser::extract_string(&map, "string_field").unwrap(), "test_value");
        
        // Test missing field
        assert!(FrontmatterParser::extract_string(&map, "missing_field").is_err());
        
        // Test wrong type
        assert!(FrontmatterParser::extract_string(&map, "bool_field").is_err());
    }

    #[test]
    fn test_extract_bool() {
        let map = create_test_map();
        
        assert!(FrontmatterParser::extract_bool(&map, "bool_field").unwrap());
        
        // Test missing field
        assert!(FrontmatterParser::extract_bool(&map, "missing_field").is_err());
        
        // Test wrong type
        assert!(FrontmatterParser::extract_bool(&map, "string_field").is_err());
    }

    #[test]
    fn test_extract_integer() {
        let map = create_test_map();
        
        assert_eq!(FrontmatterParser::extract_integer(&map, "integer_field").unwrap(), 42);
        
        // Test missing field
        assert!(FrontmatterParser::extract_integer(&map, "missing_field").is_err());
        
        // Test wrong type
        assert!(FrontmatterParser::extract_integer(&map, "string_field").is_err());
    }

    #[test]
    fn test_extract_datetime() {
        let map = create_test_map();
        
        let dt = FrontmatterParser::extract_datetime(&map, "date_field").unwrap();
        assert_eq!(dt.to_rfc3339(), "2025-01-01T12:00:00+00:00");
        
        // Test missing field
        assert!(FrontmatterParser::extract_datetime(&map, "missing_field").is_err());
        
        // Test invalid format
        let mut bad_map = HashMap::new();
        bad_map.insert("bad_date".to_string(), Pod::String("not-a-date".to_string()));
        assert!(FrontmatterParser::extract_datetime(&bad_map, "bad_date").is_err());
    }

    #[test]
    fn test_extract_tags() {
        let map = create_test_map();
        
        let tags = FrontmatterParser::extract_tags(&map).unwrap();
        assert_eq!(tags.len(), 3);
        assert!(tags.contains(&Tag::Phase(Phase::Draft)));
        assert!(tags.contains(&Tag::Label("vision".to_string())));
        assert!(tags.contains(&Tag::Label("urgent".to_string())));
        
        // Test missing tags field
        let empty_map = HashMap::new();
        assert!(FrontmatterParser::extract_tags(&empty_map).is_err());
        
        // Test wrong type
        let mut bad_map = HashMap::new();
        bad_map.insert("tags".to_string(), Pod::String("not-an-array".to_string()));
        assert!(FrontmatterParser::extract_tags(&bad_map).is_err());
    }

    #[test]
    fn test_extract_string_array() {
        let map = create_test_map();
        
        let strings = FrontmatterParser::extract_string_array(&map, "string_array").unwrap();
        assert_eq!(strings, vec!["item1", "item2"]);
        
        // Test missing field
        assert!(FrontmatterParser::extract_string_array(&map, "missing_field").is_err());
        
        // Test wrong type
        assert!(FrontmatterParser::extract_string_array(&map, "string_field").is_err());
    }

    #[test]
    fn test_extract_tags_with_invalid_tags() {
        let mut map = HashMap::new();
        map.insert("tags".to_string(), Pod::Array(vec![
            Pod::String("#phase/draft".to_string()),
            Pod::Integer(123), // Invalid - not a string
            Pod::String("#valid-tag".to_string()),
        ]));
        
        // Should still work, just ignoring invalid entries
        let tags = FrontmatterParser::extract_tags(&map).unwrap();
        assert_eq!(tags.len(), 2);
        assert!(tags.contains(&Tag::Phase(Phase::Draft)));
        assert!(tags.contains(&Tag::Label("valid-tag".to_string())));
    }
}