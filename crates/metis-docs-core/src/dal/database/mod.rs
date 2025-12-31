pub mod configuration_repository;
pub mod models;
pub mod repository;
pub mod schema;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("src/dal/database/migrations");

/// Database connection and migration management
pub struct Database {
    connection_string: String,
}

/// Configure SQLite connection for better concurrency
///
/// Sets pragmas to reduce "database is locked" errors when multiple
/// connections access the database simultaneously.
fn configure_connection(connection: &mut SqliteConnection) -> Result<(), diesel::result::Error> {
    // Wait up to 5 seconds when encountering a lock instead of failing immediately
    diesel::sql_query("PRAGMA busy_timeout = 5000").execute(connection)?;

    // Use Write-Ahead Logging for better concurrent read/write performance
    // WAL allows readers and writers to operate simultaneously
    diesel::sql_query("PRAGMA journal_mode = WAL").execute(connection)?;

    // Synchronous NORMAL is safe with WAL and faster than FULL
    diesel::sql_query("PRAGMA synchronous = NORMAL").execute(connection)?;

    Ok(())
}

impl Database {
    /// Create a new database connection and run migrations
    ///
    /// # Arguments
    /// * `connection_string` - SQLite connection string (e.g., ":memory:", "database.db", "file:database.db?mode=rw")
    pub fn new(connection_string: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Run migrations once to ensure the database is set up
        let mut connection = SqliteConnection::establish(connection_string)?;
        configure_connection(&mut connection)?;
        connection.run_pending_migrations(MIGRATIONS)?;

        Ok(Self {
            connection_string: connection_string.to_string(),
        })
    }

    /// Get a new connection to the database
    pub fn get_connection(
        &self,
    ) -> Result<SqliteConnection, Box<dyn std::error::Error + Send + Sync>> {
        let mut connection = SqliteConnection::establish(&self.connection_string)?;
        configure_connection(&mut connection)?;

        // For in-memory databases, we need to run migrations on each connection
        // since each connection is a separate database
        if self.connection_string == ":memory:" {
            connection.run_pending_migrations(MIGRATIONS)?;
        }

        Ok(connection)
    }

    /// Get a document repository with a new connection
    pub fn repository(
        &self,
    ) -> Result<repository::DocumentRepository, Box<dyn std::error::Error + Send + Sync>> {
        let connection = self.get_connection()?;
        Ok(repository::DocumentRepository::new(connection))
    }

    /// Get a document repository (consumes the database) - kept for compatibility
    pub fn into_repository(self) -> repository::DocumentRepository {
        let connection = self.get_connection().expect("Failed to get connection");
        repository::DocumentRepository::new(connection)
    }

    /// Get a configuration repository with a new connection
    pub fn configuration_repository(
        &self,
    ) -> Result<
        configuration_repository::ConfigurationRepository,
        Box<dyn std::error::Error + Send + Sync>,
    > {
        let connection = self.get_connection()?;
        Ok(configuration_repository::ConfigurationRepository::new(
            connection,
        ))
    }
}
