use crate::domain::documents::types::DocumentType;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::LazyLock;

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

// --- Workspace prefix validation ---

/// Regex for valid workspace prefix: lowercase alphanumeric + hyphens, 2-20 chars,
/// must start and end with alphanumeric
static WORKSPACE_PREFIX_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9][a-z0-9-]{0,18}[a-z0-9]$").unwrap());

/// Single-char prefix pattern (for the 2-char minimum that's just two alphanums)
static WORKSPACE_PREFIX_MIN_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z0-9]{2}$").unwrap());

/// Validate a workspace prefix (the owned folder name in central).
///
/// Rules: lowercase alphanumeric + hyphens, 2-20 chars, must start and end with alphanumeric.
pub fn validate_workspace_prefix(prefix: &str) -> Result<(), ConfigurationError> {
    if prefix.is_empty() {
        return Err(ConfigurationError::InvalidValue(
            "Workspace prefix cannot be empty".to_string(),
        ));
    }
    if prefix.len() < 2 {
        return Err(ConfigurationError::InvalidValue(
            format!(
                "Workspace prefix '{}' is too short (minimum 2 characters)",
                prefix
            ),
        ));
    }
    if prefix.len() > 20 {
        return Err(ConfigurationError::InvalidValue(
            format!(
                "Workspace prefix '{}' is too long (maximum 20 characters)",
                prefix
            ),
        ));
    }
    // For exactly 2 chars, use the min regex (no hyphens possible between start/end)
    let valid = if prefix.len() == 2 {
        WORKSPACE_PREFIX_MIN_RE.is_match(prefix)
    } else {
        WORKSPACE_PREFIX_RE.is_match(prefix)
    };
    if !valid {
        return Err(ConfigurationError::InvalidValue(
            format!(
                "Workspace prefix '{}' is invalid. Must be lowercase alphanumeric + hyphens, start and end with alphanumeric.",
                prefix
            ),
        ));
    }
    Ok(())
}

/// Validate an upstream git URL (SSH or HTTPS format).
pub fn validate_upstream_url(url: &str) -> Result<(), ConfigurationError> {
    if url.is_empty() {
        return Err(ConfigurationError::InvalidValue(
            "Upstream URL cannot be empty".to_string(),
        ));
    }
    // SSH format: git@host:org/repo.git or ssh://git@host/org/repo.git
    let is_ssh = url.starts_with("git@") || url.starts_with("ssh://");
    // HTTPS format: https://host/org/repo.git
    let is_https = url.starts_with("https://") || url.starts_with("http://");
    // File format: file:///path/to/repo (local, useful for development/testing)
    let is_file = url.starts_with("file://");

    if !is_ssh && !is_https && !is_file {
        return Err(ConfigurationError::InvalidValue(
            format!(
                "Upstream URL '{}' is not a valid git remote URL. Expected SSH (git@host:org/repo.git) or HTTPS (https://host/path.git) format.",
                url
            ),
        ));
    }
    Ok(())
}

/// Validate a git commit SHA (40-char hex string).
pub fn validate_commit_sha(sha: &str) -> Result<(), ConfigurationError> {
    if sha.len() != 40 {
        return Err(ConfigurationError::InvalidValue(
            format!(
                "Commit SHA '{}' must be exactly 40 hex characters, got {}",
                sha,
                sha.len()
            ),
        ));
    }
    if !sha.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(ConfigurationError::InvalidValue(
            format!("Commit SHA '{}' contains non-hex characters", sha),
        ));
    }
    Ok(())
}

// --- Config structs ---

/// Workspace identity configuration for multi-workspace sync
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Owned folder name in central repo (e.g., "api"). Also used as workspace identifier.
    pub prefix: String,
    /// Optional team/group label for multi-workspace views (e.g., "platform")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
}

impl WorkspaceConfig {
    pub fn new(prefix: String, team: Option<String>) -> Result<Self, ConfigurationError> {
        validate_workspace_prefix(&prefix)?;
        Ok(Self { prefix, team })
    }
}

/// Sync configuration for upstream central repo
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Central git repo URL (SSH or HTTPS)
    pub upstream_url: String,
    /// SHA of last successful sync commit — updated after each sync
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_commit: Option<String>,
}

impl SyncConfig {
    pub fn new(
        upstream_url: String,
        last_synced_commit: Option<String>,
    ) -> Result<Self, ConfigurationError> {
        validate_upstream_url(&upstream_url)?;
        if let Some(ref sha) = last_synced_commit {
            validate_commit_sha(sha)?;
        }
        Ok(Self {
            upstream_url,
            last_synced_commit,
        })
    }
}

/// Configuration file structure that persists to .metis/config.toml
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigFile {
    pub project: ProjectConfig,
    pub flight_levels: FlightLevelConfig,
    /// Workspace identity for multi-workspace sync (absent = single-workspace mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<WorkspaceConfig>,
    /// Sync configuration for upstream central repo (absent = single-workspace mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sync: Option<SyncConfig>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectConfig {
    pub prefix: String,
}

impl ConfigFile {
    /// Create a new configuration with defaults (single-workspace mode)
    pub fn new(
        prefix: String,
        flight_levels: FlightLevelConfig,
    ) -> Result<Self, ConfigurationError> {
        // Validate prefix format: 2-8 uppercase letters
        if !prefix.chars().all(|c| c.is_ascii_uppercase()) || prefix.len() < 2 || prefix.len() > 8
        {
            return Err(ConfigurationError::InvalidValue(
                "Project prefix must be 2-8 uppercase letters".to_string(),
            ));
        }

        Ok(Self {
            project: ProjectConfig { prefix },
            flight_levels,
            workspace: None,
            sync: None,
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

    /// Get the project prefix (short code prefix, e.g. "METIS")
    pub fn prefix(&self) -> &str {
        &self.project.prefix
    }

    /// Get the flight level configuration
    pub fn flight_levels(&self) -> &FlightLevelConfig {
        &self.flight_levels
    }

    /// Get the workspace configuration (None = single-workspace mode)
    pub fn workspace(&self) -> Option<&WorkspaceConfig> {
        self.workspace.as_ref()
    }

    /// Get the sync configuration (None = single-workspace mode)
    pub fn sync_config(&self) -> Option<&SyncConfig> {
        self.sync.as_ref()
    }

    /// Get the workspace prefix (folder name in central, e.g. "api")
    pub fn workspace_prefix(&self) -> Option<&str> {
        self.workspace.as_ref().map(|w| w.prefix.as_str())
    }

    /// Get the upstream URL
    pub fn upstream_url(&self) -> Option<&str> {
        self.sync.as_ref().map(|s| s.upstream_url.as_str())
    }

    /// Get the last synced commit SHA
    pub fn last_synced_commit(&self) -> Option<&str> {
        self.sync
            .as_ref()
            .and_then(|s| s.last_synced_commit.as_deref())
    }

    /// Check if this is a multi-workspace configuration
    pub fn is_multi_workspace(&self) -> bool {
        self.sync.is_some() && self.workspace.is_some()
    }

    /// Set workspace configuration (validates inputs)
    pub fn set_workspace(
        &mut self,
        workspace_prefix: String,
        team: Option<String>,
    ) -> Result<(), ConfigurationError> {
        self.workspace = Some(WorkspaceConfig::new(workspace_prefix, team)?);
        Ok(())
    }

    /// Set sync configuration (validates inputs)
    pub fn set_sync(&mut self, upstream_url: String) -> Result<(), ConfigurationError> {
        self.sync = Some(SyncConfig::new(upstream_url, None)?);
        Ok(())
    }

    /// Update the last synced commit SHA after a successful sync
    pub fn update_last_synced_commit(
        &mut self,
        commit_sha: &str,
    ) -> Result<(), ConfigurationError> {
        validate_commit_sha(commit_sha)?;
        if let Some(ref mut sync) = self.sync {
            sync.last_synced_commit = Some(commit_sha.to_string());
            Ok(())
        } else {
            Err(ConfigurationError::MissingConfiguration(
                "Cannot update last_synced_commit: no sync configuration present".to_string(),
            ))
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            project: ProjectConfig {
                prefix: "PROJ".to_string(),
            },
            flight_levels: FlightLevelConfig::streamlined(),
            workspace: None,
            sync: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================
    // Existing FlightLevelConfig tests (unchanged)
    // ============================================================

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
        assert!(FlightLevelConfig::new(true, true).is_ok());
        assert!(FlightLevelConfig::new(false, true).is_ok());
        assert!(FlightLevelConfig::new(false, false).is_ok());
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
        assert!(config.workspace().is_none());
        assert!(config.sync_config().is_none());
        assert!(!config.is_multi_workspace());
    }

    #[test]
    fn test_config_file_validation() {
        // Valid prefixes
        assert!(ConfigFile::new("AB".to_string(), FlightLevelConfig::streamlined()).is_ok());
        assert!(ConfigFile::new("ABCDEFGH".to_string(), FlightLevelConfig::streamlined()).is_ok());
        assert!(ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).is_ok());

        // Invalid prefixes
        assert!(ConfigFile::new("A".to_string(), FlightLevelConfig::streamlined()).is_err());
        assert!(
            ConfigFile::new("ABCDEFGHI".to_string(), FlightLevelConfig::streamlined()).is_err()
        );
        assert!(ConfigFile::new("ab".to_string(), FlightLevelConfig::streamlined()).is_err());
        assert!(ConfigFile::new("A1".to_string(), FlightLevelConfig::streamlined()).is_err());
        assert!(ConfigFile::new("A-B".to_string(), FlightLevelConfig::streamlined()).is_err());
    }

    #[test]
    fn test_config_file_save_and_load() {
        use tempfile::NamedTempFile;

        let original_config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::full()).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        original_config.save(&temp_path).unwrap();
        let loaded_config = ConfigFile::load(&temp_path).unwrap();

        assert_eq!(original_config, loaded_config);
        assert_eq!(loaded_config.prefix(), "METIS");
        assert_eq!(loaded_config.flight_levels(), &FlightLevelConfig::full());
    }

    #[test]
    fn test_config_file_toml_format() {
        use tempfile::NamedTempFile;

        let config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        config.save(temp_path).unwrap();

        let content = std::fs::read_to_string(temp_path).unwrap();

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

    // ============================================================
    // Workspace prefix validation tests
    // ============================================================

    #[test]
    fn test_workspace_prefix_valid_simple() {
        assert!(validate_workspace_prefix("api").is_ok());
        assert!(validate_workspace_prefix("sre").is_ok());
        assert!(validate_workspace_prefix("web").is_ok());
    }

    #[test]
    fn test_workspace_prefix_valid_with_hyphens() {
        assert!(validate_workspace_prefix("api-team").is_ok());
        assert!(validate_workspace_prefix("my-cool-service").is_ok());
    }

    #[test]
    fn test_workspace_prefix_valid_with_numbers() {
        assert!(validate_workspace_prefix("api2").is_ok());
        assert!(validate_workspace_prefix("v2-api").is_ok());
        assert!(validate_workspace_prefix("team42").is_ok());
    }

    #[test]
    fn test_workspace_prefix_valid_min_length() {
        assert!(validate_workspace_prefix("ab").is_ok());
        assert!(validate_workspace_prefix("a1").is_ok());
    }

    #[test]
    fn test_workspace_prefix_valid_max_length() {
        // Exactly 20 chars
        assert!(validate_workspace_prefix("abcdefghijklmnopqrst").is_ok());
    }

    #[test]
    fn test_workspace_prefix_invalid_too_short() {
        assert!(validate_workspace_prefix("a").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_empty() {
        assert!(validate_workspace_prefix("").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_too_long() {
        // 21 chars
        assert!(validate_workspace_prefix("abcdefghijklmnopqrstu").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_uppercase() {
        assert!(validate_workspace_prefix("API").is_err());
        assert!(validate_workspace_prefix("Api").is_err());
        assert!(validate_workspace_prefix("aPi").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_spaces() {
        assert!(validate_workspace_prefix("api team").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_special_chars() {
        assert!(validate_workspace_prefix("api_team").is_err());
        assert!(validate_workspace_prefix("api.team").is_err());
        assert!(validate_workspace_prefix("api/team").is_err());
        assert!(validate_workspace_prefix("api@team").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_starts_with_hyphen() {
        assert!(validate_workspace_prefix("-api").is_err());
    }

    #[test]
    fn test_workspace_prefix_invalid_ends_with_hyphen() {
        assert!(validate_workspace_prefix("api-").is_err());
    }

    // ============================================================
    // Upstream URL validation tests
    // ============================================================

    #[test]
    fn test_upstream_url_valid_ssh() {
        assert!(validate_upstream_url("git@github.com:org/repo.git").is_ok());
        assert!(validate_upstream_url("git@gitlab.com:group/subgroup/repo.git").is_ok());
        assert!(validate_upstream_url("ssh://git@github.com/org/repo.git").is_ok());
    }

    #[test]
    fn test_upstream_url_valid_https() {
        assert!(validate_upstream_url("https://github.com/org/repo.git").is_ok());
        assert!(validate_upstream_url("https://gitlab.com/group/subgroup/repo.git").is_ok());
    }

    #[test]
    fn test_upstream_url_valid_http() {
        // HTTP is accepted (user's choice, we don't enforce HTTPS)
        assert!(validate_upstream_url("http://internal-git.company.com/repo.git").is_ok());
    }

    #[test]
    fn test_upstream_url_invalid_empty() {
        assert!(validate_upstream_url("").is_err());
    }

    #[test]
    fn test_upstream_url_invalid_not_a_url() {
        assert!(validate_upstream_url("not-a-url").is_err());
        assert!(validate_upstream_url("just-some-text").is_err());
        assert!(validate_upstream_url("/local/path/repo").is_err());
    }

    #[test]
    fn test_upstream_url_invalid_ftp() {
        assert!(validate_upstream_url("ftp://example.com/repo.git").is_err());
    }

    // ============================================================
    // Commit SHA validation tests
    // ============================================================

    #[test]
    fn test_commit_sha_valid() {
        assert!(validate_commit_sha("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").is_ok());
        assert!(validate_commit_sha("0000000000000000000000000000000000000000").is_ok());
        assert!(validate_commit_sha("ffffffffffffffffffffffffffffffffffffffff").is_ok());
    }

    #[test]
    fn test_commit_sha_invalid_too_short() {
        assert!(validate_commit_sha("abc123").is_err());
        assert!(validate_commit_sha("").is_err());
    }

    #[test]
    fn test_commit_sha_invalid_too_long() {
        assert!(
            validate_commit_sha("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2x").is_err()
        );
    }

    #[test]
    fn test_commit_sha_invalid_non_hex() {
        assert!(
            validate_commit_sha("g1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2").is_err()
        );
        assert!(
            validate_commit_sha("ZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZZ").is_err()
        );
    }

    // ============================================================
    // WorkspaceConfig tests
    // ============================================================

    #[test]
    fn test_workspace_config_new() {
        let ws = WorkspaceConfig::new("api".to_string(), Some("platform".to_string())).unwrap();
        assert_eq!(ws.prefix, "api");
        assert_eq!(ws.team, Some("platform".to_string()));
    }

    #[test]
    fn test_workspace_config_no_team() {
        let ws = WorkspaceConfig::new("api".to_string(), None).unwrap();
        assert_eq!(ws.prefix, "api");
        assert_eq!(ws.team, None);
    }

    #[test]
    fn test_workspace_config_validates_prefix() {
        assert!(WorkspaceConfig::new("API".to_string(), None).is_err());
        assert!(WorkspaceConfig::new("a".to_string(), None).is_err());
        assert!(WorkspaceConfig::new("".to_string(), None).is_err());
    }

    // ============================================================
    // SyncConfig tests
    // ============================================================

    #[test]
    fn test_sync_config_new() {
        let sync = SyncConfig::new(
            "git@github.com:org/repo.git".to_string(),
            None,
        )
        .unwrap();
        assert_eq!(sync.upstream_url, "git@github.com:org/repo.git");
        assert_eq!(sync.last_synced_commit, None);
    }

    #[test]
    fn test_sync_config_with_commit() {
        let sha = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2".to_string();
        let sync = SyncConfig::new(
            "git@github.com:org/repo.git".to_string(),
            Some(sha.clone()),
        )
        .unwrap();
        assert_eq!(sync.last_synced_commit, Some(sha));
    }

    #[test]
    fn test_sync_config_validates_url() {
        assert!(SyncConfig::new("not-a-url".to_string(), None).is_err());
    }

    #[test]
    fn test_sync_config_validates_commit_sha() {
        assert!(SyncConfig::new(
            "git@github.com:org/repo.git".to_string(),
            Some("invalid-sha".to_string()),
        )
        .is_err());
    }

    // ============================================================
    // ConfigFile multi-workspace tests
    // ============================================================

    #[test]
    fn test_config_file_set_workspace() {
        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        assert!(!config.is_multi_workspace());
        assert!(config.workspace_prefix().is_none());

        config
            .set_workspace("api".to_string(), Some("platform".to_string()))
            .unwrap();

        assert_eq!(config.workspace_prefix(), Some("api"));
        assert_eq!(
            config.workspace().unwrap().team,
            Some("platform".to_string())
        );
    }

    #[test]
    fn test_config_file_set_sync() {
        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();

        assert_eq!(
            config.upstream_url(),
            Some("git@github.com:org/repo.git")
        );
        assert!(config.last_synced_commit().is_none());
    }

    #[test]
    fn test_config_file_is_multi_workspace() {
        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        // Neither set — not multi-workspace
        assert!(!config.is_multi_workspace());

        // Only workspace set — not multi-workspace (need both)
        config
            .set_workspace("api".to_string(), None)
            .unwrap();
        assert!(!config.is_multi_workspace());

        // Both set — multi-workspace
        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();
        assert!(config.is_multi_workspace());
    }

    #[test]
    fn test_config_file_update_last_synced_commit() {
        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        // Cannot update without sync config
        assert!(config
            .update_last_synced_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
            .is_err());

        // Set up sync config
        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();

        // Now update should work
        config
            .update_last_synced_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
            .unwrap();
        assert_eq!(
            config.last_synced_commit(),
            Some("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        );

        // Invalid SHA rejected
        assert!(config.update_last_synced_commit("invalid").is_err());
    }

    // ============================================================
    // Serialization roundtrip tests (backward compatibility)
    // ============================================================

    #[test]
    fn test_parse_minimal_config_no_sync_fields() {
        // Legacy config.toml with no workspace/sync sections
        let toml_str = r#"
[project]
prefix = "METIS"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true
"#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.prefix(), "METIS");
        assert!(config.workspace().is_none());
        assert!(config.sync_config().is_none());
        assert!(!config.is_multi_workspace());
    }

    #[test]
    fn test_parse_complete_config_all_fields() {
        let toml_str = r#"
[project]
prefix = "METIS"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true

[workspace]
prefix = "api"
team = "platform"

[sync]
upstream_url = "git@github.com:org/repo.git"
last_synced_commit = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2"
"#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.prefix(), "METIS");
        assert_eq!(config.workspace_prefix(), Some("api"));
        assert_eq!(
            config.workspace().unwrap().team,
            Some("platform".to_string())
        );
        assert_eq!(
            config.upstream_url(),
            Some("git@github.com:org/repo.git")
        );
        assert_eq!(
            config.last_synced_commit(),
            Some("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
        );
        assert!(config.is_multi_workspace());
    }

    #[test]
    fn test_parse_workspace_only_no_sync() {
        let toml_str = r#"
[project]
prefix = "METIS"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true

[workspace]
prefix = "api"
"#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert_eq!(config.workspace_prefix(), Some("api"));
        assert!(config.workspace().unwrap().team.is_none());
        assert!(config.sync_config().is_none());
        assert!(!config.is_multi_workspace());
    }

    #[test]
    fn test_parse_sync_only_no_workspace() {
        let toml_str = r#"
[project]
prefix = "METIS"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true

[sync]
upstream_url = "git@github.com:org/repo.git"
"#;
        let config: ConfigFile = toml::from_str(toml_str).unwrap();
        assert!(config.workspace().is_none());
        assert_eq!(
            config.upstream_url(),
            Some("git@github.com:org/repo.git")
        );
        assert!(config.last_synced_commit().is_none());
    }

    #[test]
    fn test_roundtrip_single_workspace_config() {
        use tempfile::NamedTempFile;

        let config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        config.save(&temp_path).unwrap();
        let loaded = ConfigFile::load(&temp_path).unwrap();
        assert_eq!(config, loaded);

        // Verify no workspace/sync sections in TOML output
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(!content.contains("[workspace]"));
        assert!(!content.contains("[sync]"));
    }

    #[test]
    fn test_roundtrip_multi_workspace_config() {
        use tempfile::NamedTempFile;

        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();
        config
            .set_workspace("api".to_string(), Some("platform".to_string()))
            .unwrap();
        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();
        config
            .update_last_synced_commit("a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2")
            .unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        config.save(&temp_path).unwrap();
        let loaded = ConfigFile::load(&temp_path).unwrap();
        assert_eq!(config, loaded);

        // Verify TOML structure
        let content = std::fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("[workspace]"));
        assert!(content.contains("prefix = \"api\""));
        assert!(content.contains("team = \"platform\""));
        assert!(content.contains("[sync]"));
        assert!(content.contains("upstream_url = \"git@github.com:org/repo.git\""));
        assert!(content
            .contains("last_synced_commit = \"a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2\""));
    }

    #[test]
    fn test_roundtrip_preserves_all_fields() {
        use tempfile::NamedTempFile;

        // Full config with all fields
        let mut config =
            ConfigFile::new("TEST".to_string(), FlightLevelConfig::full()).unwrap();
        config
            .set_workspace("sre-team".to_string(), Some("infrastructure".to_string()))
            .unwrap();
        config
            .set_sync("https://github.com/org/central.git".to_string())
            .unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();

        // Save → load → save → load → compare
        config.save(&temp_path).unwrap();
        let loaded1 = ConfigFile::load(&temp_path).unwrap();
        loaded1.save(&temp_path).unwrap();
        let loaded2 = ConfigFile::load(&temp_path).unwrap();

        assert_eq!(config, loaded1);
        assert_eq!(loaded1, loaded2);
    }

    #[test]
    fn test_legacy_config_roundtrip_no_new_fields_added() {
        use tempfile::NamedTempFile;

        // Simulate a legacy config.toml
        let legacy_toml = r#"[project]
prefix = "PROJ"

[flight_levels]
strategies_enabled = false
initiatives_enabled = true
"#;
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        std::fs::write(temp_path, legacy_toml).unwrap();

        // Load and save
        let config = ConfigFile::load(temp_path).unwrap();
        config.save(temp_path).unwrap();

        // Read back and verify no workspace/sync sections were added
        let content = std::fs::read_to_string(temp_path).unwrap();
        assert!(!content.contains("[workspace]"));
        assert!(!content.contains("[sync]"));
        assert!(!content.contains("upstream_url"));
    }

    #[test]
    fn test_sync_config_without_last_synced_commit_omits_field() {
        use tempfile::NamedTempFile;

        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();
        config
            .set_sync("git@github.com:org/repo.git".to_string())
            .unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        config.save(temp_path).unwrap();

        let content = std::fs::read_to_string(temp_path).unwrap();
        assert!(content.contains("upstream_url"));
        assert!(!content.contains("last_synced_commit"));
    }

    #[test]
    fn test_workspace_config_without_team_omits_field() {
        use tempfile::NamedTempFile;

        let mut config =
            ConfigFile::new("METIS".to_string(), FlightLevelConfig::streamlined()).unwrap();
        config.set_workspace("api".to_string(), None).unwrap();

        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path();
        config.save(temp_path).unwrap();

        let content = std::fs::read_to_string(temp_path).unwrap();
        assert!(content.contains("[workspace]"));
        assert!(content.contains("prefix = \"api\""));
        assert!(!content.contains("team"));
    }
}
