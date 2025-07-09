pub mod services;

use crate::dal::Database;
use crate::Result;
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
        let repository = self.database.repository().expect("Failed to get database repository");
        let mut service = services::DatabaseService::new(repository);
        f(&mut service)
    }

    /// Execute a sync operation
    pub fn with_sync<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce(&mut services::SyncService) -> R,
    {
        let repository = self.database.repository().expect("Failed to get database repository");
        let mut db_service = services::DatabaseService::new(repository);
        let mut sync_service = services::SyncService::new(&mut db_service);
        f(&mut sync_service)
    }

    /// Convenience method to sync a directory
    pub async fn sync_directory<P: AsRef<Path>>(mut self, dir_path: P) -> Result<Vec<services::synchronization::SyncResult>> {
        let mut db_service = services::DatabaseService::new(self.database.into_repository());
        let mut sync_service = services::SyncService::new(&mut db_service);
        sync_service.sync_directory(dir_path).await
    }

    /// Get access to the underlying database
    pub fn database(&mut self) -> &mut Database {
        &mut self.database
    }
}