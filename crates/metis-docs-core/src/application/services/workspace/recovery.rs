use crate::application::services::SyncService;
use crate::dal::database::configuration_repository::ConfigurationRepository;
use crate::domain::configuration::ConfigFile;
use crate::{Database, MetisError, Result};
use diesel::{sqlite::SqliteConnection, Connection};
use std::path::Path;

/// Service for recovering workspace configuration from filesystem
pub struct ConfigurationRecoveryService;

impl ConfigurationRecoveryService {
    /// Recover configuration from config.toml file to database
    ///
    /// This should be called when:
    /// - Database is missing or corrupt (automatic via sync)
    /// - Migrating from old version without config.toml
    /// - User explicitly requests recovery
    ///
    /// # Arguments
    /// * `workspace_dir` - Path to .metis directory
    /// * `db_path` - Path to database file
    ///
    /// # Recovery Process
    /// 1. Load config.toml (or create from DB if missing - migration path)
    /// 2. Sync configuration to database
    /// 3. Scan filesystem for short codes and recover counters
    ///
    /// NOTE: This does NOT sync files - caller should run sync after recovery
    pub fn recover_configuration<P: AsRef<Path>>(
        workspace_dir: P,
        db_path: P,
    ) -> Result<RecoveryReport> {
        let workspace_dir = workspace_dir.as_ref();
        let db_path = db_path.as_ref();
        let config_file_path = workspace_dir.join("config.toml");

        let mut report = RecoveryReport::new();

        // Step 1: Load or create config.toml
        let config_file = if config_file_path.exists() {
            tracing::info!("Loading configuration from {}", config_file_path.display());
            ConfigFile::load(&config_file_path).map_err(|e| MetisError::ConfigurationError(e))?
        } else {
            // Migration path: create config.toml from existing DB if it exists
            tracing::warn!("config.toml not found, attempting migration from database");
            report.config_file_created = true;

            let config_file = Self::create_config_from_database(&config_file_path, db_path)?;
            tracing::info!(
                "Created config.toml from database at {}",
                config_file_path.display()
            );
            config_file
        };

        // Step 2: Sync configuration to database
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(db_path.to_str().unwrap()).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        // Sync prefix
        let db_prefix = config_repo.get_project_prefix()?;
        if db_prefix.as_ref().map(|s| s.as_str()) != Some(config_file.prefix()) {
            tracing::info!(
                "Syncing prefix to database: {} -> {}",
                db_prefix.as_deref().unwrap_or("none"),
                config_file.prefix()
            );
            config_repo.set_project_prefix(config_file.prefix())?;
            report.prefix_synced = true;
        }

        // Sync flight levels
        let db_flight_levels = config_repo.get_flight_level_config()?;
        if &db_flight_levels != config_file.flight_levels() {
            tracing::info!(
                "Syncing flight levels to database: {} -> {}",
                db_flight_levels.preset_name(),
                config_file.flight_levels().preset_name()
            );
            config_repo.set_flight_level_config(config_file.flight_levels())?;
            report.flight_levels_synced = true;
        }

        // Step 3: Recover counters from filesystem
        tracing::info!("Recovering short code counters from filesystem");
        let counters = Self::recover_counters(workspace_dir, &mut config_repo)?;
        report.counters_recovered = counters;

        Ok(report)
    }

    /// Sync config.toml to database (lightweight operation, safe to call frequently)
    ///
    /// This is called on every normal sync operation to keep config in sync
    pub fn sync_config_to_database<P: AsRef<Path>>(workspace_dir: P, db_path: P) -> Result<bool> {
        let workspace_dir = workspace_dir.as_ref();
        let db_path = db_path.as_ref();
        let config_file_path = workspace_dir.join("config.toml");

        // Guard: config.toml must exist for normal sync
        if !config_file_path.exists() {
            tracing::warn!(
                "config.toml not found at {}, skipping config sync",
                config_file_path.display()
            );
            return Ok(false);
        }

        let config_file =
            ConfigFile::load(&config_file_path).map_err(|e| MetisError::ConfigurationError(e))?;

        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(db_path.to_str().unwrap()).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        let mut synced = false;

        // Sync prefix if different
        let db_prefix = config_repo.get_project_prefix()?;
        if db_prefix.as_ref().map(|s| s.as_str()) != Some(config_file.prefix()) {
            config_repo.set_project_prefix(config_file.prefix())?;
            synced = true;
        }

        // Sync flight levels if different
        let db_flight_levels = config_repo.get_flight_level_config()?;
        if &db_flight_levels != config_file.flight_levels() {
            config_repo.set_flight_level_config(config_file.flight_levels())?;
            synced = true;
        }

        Ok(synced)
    }

    /// Create config.toml from existing database (migration path)
    fn create_config_from_database(config_file_path: &Path, db_path: &Path) -> Result<ConfigFile> {
        let mut config_repo = ConfigurationRepository::new(
            SqliteConnection::establish(db_path.to_str().unwrap()).map_err(|e| {
                MetisError::ConfigurationError(
                    crate::domain::configuration::ConfigurationError::InvalidValue(e.to_string()),
                )
            })?,
        );

        let prefix = config_repo
            .get_project_prefix()?
            .unwrap_or_else(|| "PROJ".to_string());
        let flight_levels = config_repo.get_flight_level_config()?;

        let config_file = ConfigFile::new(prefix, flight_levels)
            .map_err(|e| MetisError::ConfigurationError(e))?;

        config_file
            .save(config_file_path)
            .map_err(|e| MetisError::ConfigurationError(e))?;

        Ok(config_file)
    }

    /// Recover counters from filesystem by scanning all documents
    fn recover_counters(
        workspace_dir: &Path,
        config_repo: &mut ConfigurationRepository,
    ) -> Result<usize> {
        // Use SyncService's counter recovery method
        // We need a temporary database service to create the sync service
        use crate::application::services::database::DatabaseService;

        // Create a temporary connection for the sync service
        let db = Database::new(workspace_dir.join("metis.db").to_str().unwrap())
            .map_err(|e| MetisError::FileSystem(e.to_string()))?;
        let mut db_service = DatabaseService::new(db.into_repository());
        let sync_service = SyncService::new(&mut db_service);

        let counters = sync_service.recover_counters_from_filesystem(workspace_dir)?;

        let mut recovered_count = 0;
        for (doc_type, max_counter) in counters {
            if config_repo.set_counter_if_lower(&doc_type, max_counter)? {
                recovered_count += 1;
            }
        }

        tracing::info!("Recovered {} counter(s)", recovered_count);
        Ok(recovered_count)
    }

    /// Check if database needs recovery
    ///
    /// Returns true if:
    /// - Database file doesn't exist
    /// - Database file exists but is unreadable/corrupt
    pub fn needs_recovery(workspace_dir: &Path) -> bool {
        let db_path = workspace_dir.join("metis.db");

        // Check if DB exists
        if !db_path.exists() {
            tracing::warn!(
                "Database not found at {}, recovery needed",
                db_path.display()
            );
            return true;
        }

        // Try to open database
        match Database::new(db_path.to_str().unwrap()) {
            Ok(_) => false, // Database is readable
            Err(e) => {
                tracing::error!("Database corrupt or unreadable: {}, recovery needed", e);
                true
            }
        }
    }
}

/// Report of what was recovered during configuration recovery
#[derive(Debug, Clone, Default)]
pub struct RecoveryReport {
    /// Whether config.toml was created (migration)
    pub config_file_created: bool,
    /// Whether project prefix was synced to DB
    pub prefix_synced: bool,
    /// Whether flight levels were synced to DB
    pub flight_levels_synced: bool,
    /// Number of counters that were recovered
    pub counters_recovered: usize,
}

impl RecoveryReport {
    fn new() -> Self {
        Self::default()
    }

    /// Check if any recovery actions were taken
    pub fn had_recovery_actions(&self) -> bool {
        self.config_file_created
            || self.prefix_synced
            || self.flight_levels_synced
            || self.counters_recovered > 0
    }
}
