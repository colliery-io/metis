pub mod services;

use crate::dal::Database;
use crate::{MetisError, Result};
use std::path::Path;

/// Application layer coordinator
/// Manages services and provides high-level application operations
pub struct Application {
    database: Database,
}

impl Application {
    /// Create a new application instance
    pub fn new(database: Database) -> Self {
        Self { database }
    }

    /// Execute a database operation
    pub fn with_database<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut services::DatabaseService) -> R,
    {
        let repository = self
            .database
            .repository()
            .expect("Failed to get database repository");
        let mut service = services::DatabaseService::new(repository);
        f(&mut service)
    }

    /// Execute a sync operation
    pub fn with_sync<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut services::SyncService) -> R,
    {
        let repository = self
            .database
            .repository()
            .expect("Failed to get database repository");
        let mut db_service = services::DatabaseService::new(repository);
        let mut sync_service = services::SyncService::new(&mut db_service);
        f(&mut sync_service)
    }

    /// Convenience method to sync a directory
    ///
    /// Automatically handles:
    /// - Database corruption recovery
    /// - Configuration sync from config.toml
    /// - Counter recovery when needed
    /// - File synchronization
    pub async fn sync_directory<P: AsRef<Path>>(
        self,
        dir_path: P,
    ) -> Result<Vec<services::synchronization::SyncResult>> {
        let workspace_path = dir_path.as_ref().to_path_buf();
        let db_path = workspace_path.join("metis.db");

        // Step 1: Check if recovery is needed (DB missing or corrupt)
        if services::workspace::ConfigurationRecoveryService::needs_recovery(&workspace_path) {
            tracing::warn!("Database recovery needed, initiating full recovery");

            // Recreate database if needed
            if !db_path.exists() {
                tracing::info!("Creating new database at {}", db_path.display());
                let _ = Database::new(db_path.to_str().unwrap())
                    .map_err(|e| MetisError::FileSystem(e.to_string()))?;
            }

            // Run full recovery
            let report = services::workspace::ConfigurationRecoveryService::recover_configuration(
                &workspace_path,
                &db_path,
            )?;

            if report.had_recovery_actions() {
                tracing::info!(
                    "Recovery complete: config_created={}, prefix_synced={}, flight_levels_synced={}, counters_recovered={}",
                    report.config_file_created,
                    report.prefix_synced,
                    report.flight_levels_synced,
                    report.counters_recovered
                );
            }
        } else {
            // Step 2: Normal path - just sync config.toml to DB (lightweight)
            match services::workspace::ConfigurationRecoveryService::sync_config_to_database(
                &workspace_path,
                &db_path,
            ) {
                Ok(synced) => {
                    if synced {
                        tracing::info!("Synced configuration from config.toml to database");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to sync configuration: {}, continuing with file sync", e);
                }
            }
        }

        // Step 3: Perform normal file synchronization
        let mut db_service = services::DatabaseService::new(self.database.into_repository());
        let mut sync_service =
            services::SyncService::new(&mut db_service).with_workspace_dir(&workspace_path);
        sync_service.sync_directory(dir_path).await
    }

    /// Get access to the underlying database
    pub fn database(&mut self) -> &mut Database {
        &mut self.database
    }
}
