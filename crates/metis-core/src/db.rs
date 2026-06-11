//! Connection pooling and migrations for the dual (Postgres/SQLite) backend.
//!
//! Metis 3.0 runs one query codebase against either backend via
//! [`diesel_dualdb`]. The backend is chosen by the connection URL: a
//! `postgres://` URL selects Postgres (team deployments), anything else selects
//! SQLite (solo/local mode).
//!
//! Migrations are applied by executing the per-backend SQL emitted by the
//! schema generator. Each [`Migration`] carries the Postgres and SQLite `up`
//! statements; [`run_migrations`] picks the right arm for the live connection.

use diesel::connection::SimpleConnection;
use diesel_dualdb::pool::Backend;
use diesel_dualdb::{DualConnection, Pool};

pub use diesel_dualdb::pool::{detect_backend, Error as PoolError};

/// Errors from connecting or migrating.
#[derive(Debug, thiserror::Error)]
pub enum DbError {
    /// The pool could not be created or a connection could not be checked out.
    #[error("connection pool error: {0}")]
    Pool(#[from] PoolError),
    /// A migration statement failed to execute.
    #[error("migration error: {0}")]
    Migration(#[from] diesel::result::Error),
}

/// One ordered migration step, with the SQL for each backend.
struct Migration {
    name: &'static str,
    postgres_up: &'static str,
    sqlite_up: &'static str,
}

/// All migrations, in application order. Generated SQL is embedded at build
/// time so the binary is self-contained (no migrations directory to ship).
const MIGRATIONS: &[Migration] = &[Migration {
    name: "0001_init",
    postgres_up: include_str!("../schema/generated/migrations-postgres/0001_init/up.sql"),
    sqlite_up: include_str!("../schema/generated/migrations-sqlite/0001_init/up.sql"),
}];

/// Connect to `database_url`, returning a pool of dual connections.
///
/// The backend is detected from the URL scheme (see [`detect_backend`]).
pub fn connect(database_url: &str) -> Result<Pool, DbError> {
    Ok(Pool::connect(database_url)?)
}

/// Apply all pending migrations on a single connection.
///
/// This is idempotent only insofar as the underlying DDL is — the initial
/// schema uses bare `CREATE TABLE`, so it must run against a fresh database.
/// A versioned-migration table is intentionally deferred until a second
/// migration exists and the need is real.
pub fn run_migrations(conn: &mut DualConnection) -> Result<(), DbError> {
    let backend = backend_of(conn);
    for migration in MIGRATIONS {
        let sql = match backend {
            Backend::Postgres => migration.postgres_up,
            Backend::Sqlite => migration.sqlite_up,
        };
        conn.batch_execute(sql).map_err(|e| {
            tracing::error!(migration = migration.name, error = %e, "migration failed");
            e
        })?;
    }
    Ok(())
}

/// Connect and immediately apply migrations, returning the ready pool.
pub fn connect_and_migrate(database_url: &str) -> Result<Pool, DbError> {
    let pool = connect(database_url)?;
    {
        let mut conn = pool.get()?;
        run_migrations(&mut conn)?;
    }
    Ok(pool)
}

/// Which backend a live connection is using.
fn backend_of(conn: &DualConnection) -> Backend {
    match conn {
        DualConnection::Pg(_) => Backend::Postgres,
        DualConnection::Sqlite(_) => Backend::Sqlite,
    }
}
