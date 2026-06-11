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
use diesel::prelude::*;
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
    postgres_down: &'static str,
    sqlite_down: &'static str,
}

/// All migrations, in application order. Generated SQL is embedded at build
/// time so the binary is self-contained (no migrations directory to ship).
const MIGRATIONS: &[Migration] = &[Migration {
    name: "0001_init",
    postgres_up: include_str!("../schema/generated/migrations-postgres/0001_init/up.sql"),
    sqlite_up: include_str!("../schema/generated/migrations-sqlite/0001_init/up.sql"),
    postgres_down: include_str!("../schema/generated/migrations-postgres/0001_init/down.sql"),
    sqlite_down: include_str!("../schema/generated/migrations-sqlite/0001_init/down.sql"),
}];

/// Connect to `database_url`, returning a pool of dual connections.
///
/// The backend is detected from the URL scheme (see [`detect_backend`]).
/// SQLite gets a **single-connection** pool: it is a single-writer database, so
/// a multi-connection pool only invites lock contention and read-after-write
/// visibility races across connections. Postgres uses the default pool size.
pub fn connect(database_url: &str) -> Result<Pool, DbError> {
    let pool = match detect_backend(database_url) {
        Some(Backend::Sqlite) => Pool::builder().max_size(1).connect(database_url)?,
        _ => Pool::connect(database_url)?,
    };
    Ok(pool)
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

/// Apply migrations only if the schema is not already present.
///
/// The initial migration uses bare `CREATE TABLE`, so re-running it on a
/// populated database errors. This probes for a core table and migrates only
/// when it is absent — the idempotent entry point a long-running server uses on
/// startup. (A versioned-migration table will replace this once a second
/// migration exists.)
pub fn ensure_migrated(conn: &mut DualConnection) -> Result<(), DbError> {
    use crate::schema::projects;
    let present = projects::table
        .select(projects::id)
        .limit(1)
        .load::<diesel_dualdb::types::Uuid>(conn)
        .is_ok();
    if present {
        Ok(())
    } else {
        run_migrations(conn)
    }
}

/// Connect and ensure migrations are applied, returning the ready pool.
pub fn connect_and_migrate(database_url: &str) -> Result<Pool, DbError> {
    let pool = connect(database_url)?;
    {
        let mut conn = pool.get()?;
        ensure_migrated(&mut conn)?;
    }
    Ok(pool)
}

/// Drop and recreate the schema. Idempotent (down uses `DROP TABLE IF EXISTS`),
/// so it is safe on a fresh database too.
///
/// Intended for tests, which share one Postgres database across the suite (the
/// `#[diesel_dualdb::test]` macro does not isolate the PG arm). Call this at the
/// start of each test and run the PG arm single-threaded. Not for production
/// use — it destroys data.
pub fn reset(conn: &mut DualConnection) -> Result<(), DbError> {
    let backend = backend_of(conn);
    for migration in MIGRATIONS.iter().rev() {
        let down = match backend {
            Backend::Postgres => migration.postgres_down,
            Backend::Sqlite => migration.sqlite_down,
        };
        conn.batch_execute(down)?;
    }
    run_migrations(conn)
}

/// Which backend a live connection is using.
fn backend_of(conn: &DualConnection) -> Backend {
    match conn {
        DualConnection::Pg(_) => Backend::Postgres,
        DualConnection::Sqlite(_) => Backend::Sqlite,
    }
}
