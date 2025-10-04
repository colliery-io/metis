use serde::{Deserialize, Serialize};
use std::fmt;
use crate::domain::documents::types::DocumentType;

/// Flight level configuration defining which levels are enabled
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlightLevelConfig {
    /// Whether strategy level is enabled
    pub strategies_enabled: bool,
    /// Whether initiative level is enabled
    pub initiatives_enabled: bool,
}

impl FlightLevelConfig {
    /// Create a new configuration
    pub fn new(strategies_enabled: bool, initiatives_enabled: bool) -> Result<Self, ConfigurationError> {
        // Validation: If initiatives are disabled, strategies must also be disabled
        if !initiatives_enabled && strategies_enabled {
            return Err(ConfigurationError::InvalidConfiguration(
                "Cannot enable strategies without initiatives - this would create a gap in the hierarchy".to_string()
            ));
        }
        
        Ok(Self {
            strategies_enabled,
            initiatives_enabled,
        })
    }

    /// Full flight levels: Vision → Strategy → Initiative → Task
    pub fn full() -> Self {
        Self {
            strategies_enabled: true,
            initiatives_enabled: true,
        }
    }

    /// Streamlined flight levels: Vision → Initiative → Task
    pub fn streamlined() -> Self {
        Self {
            strategies_enabled: false,
            initiatives_enabled: true,
        }
    }

    /// Direct flight levels: Vision → Task
    pub fn direct() -> Self {
        Self {
            strategies_enabled: false,
            initiatives_enabled: false,
        }
    }

    /// Check if a document type is allowed in this configuration
    pub fn is_document_type_allowed(&self, doc_type: DocumentType) -> bool {
        match doc_type {
            DocumentType::Vision | DocumentType::Adr => true, // Always allowed
            DocumentType::Task => true, // Always allowed
            DocumentType::Strategy => self.strategies_enabled,
            DocumentType::Initiative => self.initiatives_enabled,
        }
    }

    /// Get the parent document type for a given document type in this configuration
    pub fn get_parent_type(&self, doc_type: DocumentType) -> Option<DocumentType> {
        match doc_type {
            DocumentType::Vision | DocumentType::Adr => None, // Top level documents
            DocumentType::Strategy => Some(DocumentType::Vision),
            DocumentType::Initiative => {
                if self.strategies_enabled {
                    Some(DocumentType::Strategy)
                } else {
                    Some(DocumentType::Vision)
                }
            }
            DocumentType::Task => {
                if self.initiatives_enabled {
                    Some(DocumentType::Initiative)
                } else {
                    Some(DocumentType::Vision)
                }
            }
        }
    }

    /// Get the configuration name/preset
    pub fn preset_name(&self) -> &'static str {
        match (self.strategies_enabled, self.initiatives_enabled) {
            (true, true) => "full",
            (false, true) => "streamlined", 
            (false, false) => "direct",
            (true, false) => "invalid", // This shouldn't happen due to validation
        }
    }

    /// Get enabled document types in hierarchical order
    pub fn enabled_document_types(&self) -> Vec<DocumentType> {
        let mut types = vec![DocumentType::Vision];
        
        if self.strategies_enabled {
            types.push(DocumentType::Strategy);
        }
        
        if self.initiatives_enabled {
            types.push(DocumentType::Initiative);
        }
        
        types.push(DocumentType::Task);
        types.push(DocumentType::Adr); // ADRs are always available
        
        types
    }

    /// Get the hierarchy display string
    pub fn hierarchy_display(&self) -> String {
        let mut hierarchy = vec!["Vision"];
        
        if self.strategies_enabled {
            hierarchy.push("Strategy");
        }
        
        if self.initiatives_enabled {
            hierarchy.push("Initiative");
        }
        
        hierarchy.push("Task");
        
        hierarchy.join(" → ")
    }
}

impl Default for FlightLevelConfig {
    fn default() -> Self {
        Self::full()
    }
}

impl fmt::Display for FlightLevelConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.preset_name())
    }
}

/// Configuration validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigurationError {
    InvalidConfiguration(String),
    SerializationError(String),
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::InvalidConfiguration(msg) => write!(f, "Invalid configuration: {}", msg),
            ConfigurationError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ConfigurationError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_configurations() {
        let full = FlightLevelConfig::full();
        assert!(full.strategies_enabled);
        assert!(full.initiatives_enabled);
        assert_eq!(full.preset_name(), "full");

        let streamlined = FlightLevelConfig::streamlined();
        assert!(!streamlined.strategies_enabled);
        assert!(streamlined.initiatives_enabled);
        assert_eq!(streamlined.preset_name(), "streamlined");

        let direct = FlightLevelConfig::direct();
        assert!(!direct.strategies_enabled);
        assert!(!direct.initiatives_enabled);
        assert_eq!(direct.preset_name(), "direct");
    }

    #[test]
    fn test_configuration_validation() {
        // Valid configurations
        assert!(FlightLevelConfig::new(true, true).is_ok());
        assert!(FlightLevelConfig::new(false, true).is_ok());
        assert!(FlightLevelConfig::new(false, false).is_ok());

        // Invalid configuration: strategies enabled but initiatives disabled
        assert!(FlightLevelConfig::new(true, false).is_err());
    }

    #[test]
    fn test_document_type_allowed() {
        let full = FlightLevelConfig::full();
        assert!(full.is_document_type_allowed(DocumentType::Vision));
        assert!(full.is_document_type_allowed(DocumentType::Strategy));
        assert!(full.is_document_type_allowed(DocumentType::Initiative));
        assert!(full.is_document_type_allowed(DocumentType::Task));
        assert!(full.is_document_type_allowed(DocumentType::Adr));

        let streamlined = FlightLevelConfig::streamlined();
        assert!(streamlined.is_document_type_allowed(DocumentType::Vision));
        assert!(!streamlined.is_document_type_allowed(DocumentType::Strategy));
        assert!(streamlined.is_document_type_allowed(DocumentType::Initiative));
        assert!(streamlined.is_document_type_allowed(DocumentType::Task));
        assert!(streamlined.is_document_type_allowed(DocumentType::Adr));

        let direct = FlightLevelConfig::direct();
        assert!(direct.is_document_type_allowed(DocumentType::Vision));
        assert!(!direct.is_document_type_allowed(DocumentType::Strategy));
        assert!(!direct.is_document_type_allowed(DocumentType::Initiative));
        assert!(direct.is_document_type_allowed(DocumentType::Task));
        assert!(direct.is_document_type_allowed(DocumentType::Adr));
    }

    #[test]
    fn test_parent_type_resolution() {
        let full = FlightLevelConfig::full();
        assert_eq!(full.get_parent_type(DocumentType::Vision), None);
        assert_eq!(full.get_parent_type(DocumentType::Strategy), Some(DocumentType::Vision));
        assert_eq!(full.get_parent_type(DocumentType::Initiative), Some(DocumentType::Strategy));
        assert_eq!(full.get_parent_type(DocumentType::Task), Some(DocumentType::Initiative));
        assert_eq!(full.get_parent_type(DocumentType::Adr), None);

        let streamlined = FlightLevelConfig::streamlined();
        assert_eq!(streamlined.get_parent_type(DocumentType::Initiative), Some(DocumentType::Vision));
        assert_eq!(streamlined.get_parent_type(DocumentType::Task), Some(DocumentType::Initiative));

        let direct = FlightLevelConfig::direct();
        assert_eq!(direct.get_parent_type(DocumentType::Task), Some(DocumentType::Vision));
    }

    #[test]
    fn test_enabled_document_types() {
        let full = FlightLevelConfig::full();
        let full_types = full.enabled_document_types();
        assert_eq!(full_types, vec![
            DocumentType::Vision,
            DocumentType::Strategy,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr
        ]);

        let streamlined = FlightLevelConfig::streamlined();
        let streamlined_types = streamlined.enabled_document_types();
        assert_eq!(streamlined_types, vec![
            DocumentType::Vision,
            DocumentType::Initiative,
            DocumentType::Task,
            DocumentType::Adr
        ]);

        let direct = FlightLevelConfig::direct();
        let direct_types = direct.enabled_document_types();
        assert_eq!(direct_types, vec![
            DocumentType::Vision,
            DocumentType::Task,
            DocumentType::Adr
        ]);
    }

    #[test]
    fn test_hierarchy_display() {
        assert_eq!(FlightLevelConfig::full().hierarchy_display(), "Vision → Strategy → Initiative → Task");
        assert_eq!(FlightLevelConfig::streamlined().hierarchy_display(), "Vision → Initiative → Task");
        assert_eq!(FlightLevelConfig::direct().hierarchy_display(), "Vision → Task");
    }

    #[test]
    fn test_serialization() {
        let config = FlightLevelConfig::streamlined();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: FlightLevelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }
}