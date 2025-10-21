use crate::domain::documents::types::DocumentType;
use serde::{Deserialize, Serialize};
use std::fmt;

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
    pub fn new(
        strategies_enabled: bool,
        initiatives_enabled: bool,
    ) -> Result<Self, ConfigurationError> {
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
            DocumentType::Task => true,                       // Always allowed
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
    InvalidValue(String),
    MissingConfiguration(String),
}

impl fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigurationError::InvalidConfiguration(msg) => {
                write!(f, "Invalid configuration: {}", msg)
            }
            ConfigurationError::SerializationError(msg) => {
                write!(f, "Serialization error: {}", msg)
            }
            ConfigurationError::InvalidValue(msg) => write!(f, "Invalid value: {}", msg),
            ConfigurationError::MissingConfiguration(msg) => {
                write!(f, "Missing configuration: {}", msg)
            }
        }
    }
}

impl std::error::Error for ConfigurationError {}

/// Configuration file structure that persists to .metis/config.toml
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    pub project: ProjectConfig,
    pub flight_levels: FlightLevelConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub prefix: String,
}

impl ConfigFile {
    /// Create a new configuration with defaults
    pub fn new(prefix: String, flight_levels: FlightLevelConfig) -> Result<Self, ConfigurationError> {
        // Validate prefix format: 2-8 uppercase letters
        if !prefix.chars().all(|c| c.is_ascii_uppercase()) || prefix.len() < 2 || prefix.len() > 8 {
            return Err(ConfigurationError::InvalidValue(
                "Project prefix must be 2-8 uppercase letters".to_string(),
            ));
        }

        Ok(Self {
            project: ProjectConfig { prefix },
            flight_levels,
        })
    }

    /// Load configuration from a TOML file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, ConfigurationError> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            ConfigurationError::SerializationError(format!("Failed to read config file: {}", e))
        })?;

        toml::from_str(&content).map_err(|e| {
            ConfigurationError::SerializationError(format!("Failed to parse TOML: {}", e))
        })
    }

    /// Save configuration to a TOML file
    pub fn save<P: AsRef<std::path::Path>>(&self, path: P) -> Result<(), ConfigurationError> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigurationError::SerializationError(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path.as_ref(), content).map_err(|e| {
            ConfigurationError::SerializationError(format!("Failed to write config file: {}", e))
        })?;

        Ok(())
    }

    /// Create default configuration with given prefix
    pub fn default_with_prefix(prefix: String) -> Result<Self, ConfigurationError> {
        Self::new(prefix, FlightLevelConfig::streamlined())
    }

    /// Get the project prefix
    pub fn prefix(&self) -> &str {
        &self.project.prefix
    }

    /// Get the flight level configuration
    pub fn flight_levels(&self) -> &FlightLevelConfig {
        &self.flight_levels
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                prefix: "PROJ".to_string(),
            },
            flight_levels: FlightLevelConfig::streamlined(),
        }
    }
}

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
        assert_eq!(
            full.get_parent_type(DocumentType::Strategy),
            Some(DocumentType::Vision)
        );
        assert_eq!(
            full.get_parent_type(DocumentType::Initiative),
            Some(DocumentType::Strategy)
        );
        assert_eq!(
            full.get_parent_type(DocumentType::Task),
            Some(DocumentType::Initiative)
        );
        assert_eq!(full.get_parent_type(DocumentType::Adr), None);

        let streamlined = FlightLevelConfig::streamlined();
        assert_eq!(
            streamlined.get_parent_type(DocumentType::Initiative),
            Some(DocumentType::Vision)
        );
        assert_eq!(
            streamlined.get_parent_type(DocumentType::Task),
            Some(DocumentType::Initiative)
        );

        let direct = FlightLevelConfig::direct();
        assert_eq!(
            direct.get_parent_type(DocumentType::Task),
            Some(DocumentType::Vision)
        );
    }

    #[test]
    fn test_enabled_document_types() {
        let full = FlightLevelConfig::full();
        let full_types = full.enabled_document_types();
        assert_eq!(
            full_types,
            vec![
                DocumentType::Vision,
                DocumentType::Strategy,
                DocumentType::Initiative,
                DocumentType::Task,
                DocumentType::Adr
            ]
        );

        let streamlined = FlightLevelConfig::streamlined();
        let streamlined_types = streamlined.enabled_document_types();
        assert_eq!(
            streamlined_types,
            vec![
                DocumentType::Vision,
                DocumentType::Initiative,
                DocumentType::Task,
                DocumentType::Adr
            ]
        );

        let direct = FlightLevelConfig::direct();
        let direct_types = direct.enabled_document_types();
        assert_eq!(
            direct_types,
            vec![DocumentType::Vision, DocumentType::Task, DocumentType::Adr]
        );
    }

    #[test]
    fn test_hierarchy_display() {
        assert_eq!(
            FlightLevelConfig::full().hierarchy_display(),
            "Vision → Strategy → Initiative → Task"
        );
        assert_eq!(
            FlightLevelConfig::streamlined().hierarchy_display(),
            "Vision → Initiative → Task"
        );
        assert_eq!(
            FlightLevelConfig::direct().hierarchy_display(),
            "Vision → Task"
        );
    }

    #[test]
    fn test_serialization() {
        let config = FlightLevelConfig::streamlined();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: FlightLevelConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_file_creation() {
        let config = ConfigFile::new("TEST".to_string(), FlightLevelConfig::streamlined()).unwrap();
        assert_eq!(config.prefix(), "TEST");
        assert_eq!(config.flight_levels(), &FlightLevelConfig::streamlined());
    }

    #[test]
    fn test_config_file_validation() {
        // Valid prefixes
        assert!(ConfigFile::new("AB".to_string(), FlightLevelConfig::streamlined()).is_ok());
        assert!(ConfigFile::new("ABCDEFGH".to_string(), FlightLevelConfig::streamlined()).is_ok());
        assert!(ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).is_ok());

        // Invalid prefixes
        assert!(ConfigFile::new("A".to_string(), FlightLevelConfig::streamlined()).is_err()); // Too short
        assert!(ConfigFile::new("ABCDEFGHI".to_string(), FlightLevelConfig::streamlined()).is_err()); // Too long
        assert!(ConfigFile::new("ab".to_string(), FlightLevelConfig::streamlined()).is_err()); // Lowercase
        assert!(ConfigFile::new("A1".to_string(), FlightLevelConfig::streamlined()).is_err()); // Contains number
        assert!(ConfigFile::new("A-B".to_string(), FlightLevelConfig::streamlined()).is_err()); // Contains hyphen
    }

    #[test]
    fn test_config_file_save_and_load() {
        use tempfile::NamedTempFile;

        let original_config = ConfigFile::new("METIS".to_string(), FlightLevelConfig::full()).unwrap();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Save configuration
        original_config.save(&temp_path).unwrap();

        // Load configuration
        let loaded_config = ConfigFile::load(&temp_path).unwrap();

        // Verify they match
        assert_eq!(original_config, loaded_config);
        assert_eq!(loaded_config.prefix(), "METIS");
        assert_eq!(loaded_config.flight_levels(), &FlightLevelConfig::full());
    }

    #[test]
    fn test_config_file_toml_format() {
        use tempfile::NamedTempFile;

        let config = ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        // Create a temporary file
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();

        // Save configuration
        config.save(temp_path).unwrap();

        // Read raw TOML content
        let content = std::fs::read_to_string(temp_path).unwrap();

        // Verify TOML structure
        assert!(content.contains("[project]"));
        assert!(content.contains("prefix = \"METIS\""));
        assert!(content.contains("[flight_levels]"));
        assert!(content.contains("strategies_enabled = false"));
        assert!(content.contains("initiatives_enabled = true"));
    }

    #[test]
    fn test_config_file_default() {
        let config = ConfigFile::default();
        assert_eq!(config.prefix(), "PROJ");
        assert_eq!(config.flight_levels(), &FlightLevelConfig::streamlined());
    }

    #[test]
    fn test_config_file_default_with_prefix() {
        let config = ConfigFile::default_with_prefix("CUSTOM".to_string()).unwrap();
        assert_eq!(config.prefix(), "CUSTOM");
        assert_eq!(config.flight_levels(), &FlightLevelConfig::streamlined());
    }
}
