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

impl Database {
    /// Create a new database connection and run migrations
    ///
    /// # Arguments
    /// * `connection_string` - SQLite connection string (e.g., ":memory:", "database.db", "file:database.db?mode=rw")
    pub fn new(connection_string: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Run migrations once to ensure the database is set up
        let mut connection = SqliteConnection::establish(connection_string)?;
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
}
