use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use std::collections::HashMap;
use crate::dal::database::models::Configuration;
use crate::dal::database::schema::configuration;
use crate::domain::configuration::{FlightLevelConfig, ConfigurationError};
use crate::Result;

/// Repository for managing configuration data
pub struct ConfigurationRepository {
    connection: SqliteConnection,
    cache: Option<HashMap<String, String>>,
}

impl ConfigurationRepository {
    pub fn new(connection: SqliteConnection) -> Self {
        Self {
            connection,
            cache: None,
        }
    }

    /// Load all configuration into cache
    pub fn load_cache(&mut self) -> Result<()> {
        let configs: Vec<Configuration> = configuration::table
            .load(&mut self.connection)
            .map_err(crate::MetisError::Database)?;

        let mut cache = HashMap::new();
        for config in configs {
            cache.insert(config.key, config.value);
        }
        self.cache = Some(cache);
        Ok(())
    }

    /// Get configuration value by key
    pub fn get(&mut self, key: &str) -> Result<Option<String>> {
        // Load cache if not already loaded
        if self.cache.is_none() {
            self.load_cache()?;
        }

        Ok(self.cache.as_ref().unwrap().get(key).cloned())
    }

    /// Set configuration value
    pub fn set(&mut self, key: &str, value: &str) -> Result<()> {
        let now = chrono::Utc::now().timestamp() as f64;
        let config = Configuration {
            key: key.to_string(),
            value: value.to_string(),
            updated_at: now,
        };

        diesel::insert_into(configuration::table)
            .values(&config)
            .on_conflict(configuration::key)
            .do_update()
            .set((
                configuration::value.eq(value),
                configuration::updated_at.eq(now),
            ))
            .execute(&mut self.connection)
            .map_err(crate::MetisError::Database)?;

        // Update cache if loaded
        if let Some(ref mut cache) = self.cache {
            cache.insert(key.to_string(), value.to_string());
        }

        Ok(())
    }

    /// Get flight level configuration
    pub fn get_flight_level_config(&mut self) -> Result<FlightLevelConfig> {
        let json = self.get("flight_levels")?
            .unwrap_or_else(|| r#"{"strategies_enabled":false,"initiatives_enabled":true}"#.to_string());

        serde_json::from_str(&json)
            .map_err(|e| crate::MetisError::ConfigurationError(
                ConfigurationError::SerializationError(e.to_string())
            ))
    }

    /// Set flight level configuration
    pub fn set_flight_level_config(&mut self, config: &FlightLevelConfig) -> Result<()> {
        let json = serde_json::to_string(config)
            .map_err(|e| crate::MetisError::ConfigurationError(
                ConfigurationError::SerializationError(e.to_string())
            ))?;

        self.set("flight_levels", &json)
    }

    /// Get all configuration as a map
    pub fn get_all(&mut self) -> Result<HashMap<String, String>> {
        if self.cache.is_none() {
            self.load_cache()?;
        }
        Ok(self.cache.as_ref().unwrap().clone())
    }

    /// Delete configuration by key
    pub fn delete(&mut self, key: &str) -> Result<bool> {
        let deleted_rows = diesel::delete(configuration::table.filter(configuration::key.eq(key)))
            .execute(&mut self.connection)
            .map_err(crate::MetisError::Database)?;

        // Update cache if loaded
        if let Some(ref mut cache) = self.cache {
            cache.remove(key);
        }

        Ok(deleted_rows > 0)
    }

    /// Get project prefix for short codes
    pub fn get_project_prefix(&mut self) -> Result<Option<String>> {
        self.get("project_prefix")
    }

    /// Set project prefix for short codes (validates 2-8 uppercase letters)
    pub fn set_project_prefix(&mut self, prefix: &str) -> Result<()> {
        // Validate prefix format: 2-8 uppercase letters
        if !prefix.chars().all(|c| c.is_ascii_uppercase()) || prefix.len() < 2 || prefix.len() > 8 {
            return Err(crate::MetisError::ConfigurationError(
                ConfigurationError::InvalidValue(
                    "Project prefix must be 2-8 uppercase letters".to_string()
                )
            ));
        }

        self.set("project_prefix", prefix)
    }

    /// Get next short code number for a document type and increment the counter
    pub fn get_next_short_code_number(&mut self, doc_type: &str) -> Result<u32> {
        let counter_key = format!("short_code_counter_{}", doc_type.to_lowercase());
        let current_value = self.get(&counter_key)?
            .unwrap_or_else(|| "0".to_string())
            .parse::<u32>()
            .unwrap_or(0);
        
        let next_value = current_value + 1;
        self.set(&counter_key, &next_value.to_string())?;
        
        Ok(next_value)
    }

    /// Generate a short code for a document type (PREFIX-TYPE-NNNN)
    pub fn generate_short_code(&mut self, doc_type: &str) -> Result<String> {
        let prefix = self.get_project_prefix()?
            .ok_or_else(|| crate::MetisError::ConfigurationError(
                ConfigurationError::MissingConfiguration("project_prefix".to_string())
            ))?;

        let type_letter = match doc_type.to_lowercase().as_str() {
            "vision" => "V",
            "strategy" => "S", 
            "initiative" => "I",
            "task" => "T",
            "adr" => "A",
            _ => return Err(crate::MetisError::ConfigurationError(
                ConfigurationError::InvalidValue(
                    format!("Unknown document type: {}", doc_type)
                )
            )),
        };

        let number = self.get_next_short_code_number(doc_type)?;
        Ok(format!("{}-{}-{:04}", prefix, type_letter, number))
    }

    /// Clear all configuration (for testing)
    #[cfg(test)]
    pub fn clear_all(&mut self) -> Result<()> {
        diesel::delete(configuration::table)
            .execute(&mut self.connection)
            .map_err(crate::MetisError::Database)?;

        self.cache = Some(HashMap::new());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dal::database::Database;

    fn setup_test_repo() -> ConfigurationRepository {
        let db = Database::new(":memory:").expect("Failed to create test database");
        let conn = db.get_connection().expect("Failed to get connection");
        ConfigurationRepository::new(conn)
    }

    #[test]
    fn test_basic_configuration_crud() {
        let mut repo = setup_test_repo();

        // Set a value
        repo.set("test_key", "test_value").unwrap();

        // Get the value
        let value = repo.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Update the value
        repo.set("test_key", "updated_value").unwrap();
        let value = repo.get("test_key").unwrap();
        assert_eq!(value, Some("updated_value".to_string()));

        // Delete the value
        let deleted = repo.delete("test_key").unwrap();
        assert!(deleted);

        // Verify it's gone
        let value = repo.get("test_key").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn test_flight_level_config() {
        let mut repo = setup_test_repo();

        // Should have default configuration (streamlined)
        let config = repo.get_flight_level_config().unwrap();
        assert_eq!(config, FlightLevelConfig::streamlined());

        // Set streamlined configuration
        let streamlined = FlightLevelConfig::streamlined();
        repo.set_flight_level_config(&streamlined).unwrap();

        // Verify it was saved
        let loaded_config = repo.get_flight_level_config().unwrap();
        assert_eq!(loaded_config, streamlined);

        // Set direct configuration
        let direct = FlightLevelConfig::direct();
        repo.set_flight_level_config(&direct).unwrap();

        // Verify it was saved
        let loaded_config = repo.get_flight_level_config().unwrap();
        assert_eq!(loaded_config, direct);
    }

    #[test]
    fn test_cache_functionality() {
        let mut repo = setup_test_repo();

        // Set multiple values
        repo.set("key1", "value1").unwrap();
        repo.set("key2", "value2").unwrap();

        // Get all should load cache
        let all = repo.get_all().unwrap();
        assert_eq!(all.get("key1"), Some(&"value1".to_string()));
        assert_eq!(all.get("key2"), Some(&"value2".to_string()));

        // Subsequent gets should use cache
        let value = repo.get("key1").unwrap();
        assert_eq!(value, Some("value1".to_string()));
    }

    #[test]
    fn test_nonexistent_key() {
        let mut repo = setup_test_repo();

        let value = repo.get("nonexistent").unwrap();
        assert_eq!(value, None);

        let deleted = repo.delete("nonexistent").unwrap();
        assert!(!deleted);
    }
}