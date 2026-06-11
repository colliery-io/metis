//! The db-blob object store: content lives in the `objects` table.
//!
//! This is the default backend. Because it operates on the connection passed to
//! each call, it naturally participates in whatever transaction the caller has
//! open — giving true write atomicity with the referencing row.

use diesel::prelude::*;
use diesel_dualdb::types::{Bytes, Timestamp};
use diesel_dualdb::DualConnection;

use super::{content_key, ObjectError, ObjectStore};
use crate::schema::objects;

/// Stores objects as rows in the `objects` table.
#[derive(Debug, Default, Clone, Copy)]
pub struct DbBlobObjectStore;

impl DbBlobObjectStore {
    /// Construct the db-blob store. It is stateless; the connection is supplied
    /// per call.
    pub fn new() -> Self {
        Self
    }
}

impl ObjectStore for DbBlobObjectStore {
    fn get(&self, conn: &mut DualConnection, key: &str) -> Result<Option<Vec<u8>>, ObjectError> {
        let row: Option<Bytes> = objects::table
            .select(objects::content)
            .filter(objects::object_key.eq(key))
            .first::<Bytes>(conn)
            .optional()?;
        Ok(row.map(|b| b.0))
    }

    fn put(&self, conn: &mut DualConnection, content: &[u8]) -> Result<String, ObjectError> {
        let key = content_key(content);

        // Check-then-insert rather than ON CONFLICT: the unified MultiBackend
        // doesn't support upsert clauses, and — more importantly — a failed
        // INSERT (unique violation) would poison the caller's open transaction
        // on Postgres. The fast path avoids the error in the common dedup case.
        let exists = objects::table
            .find(&key)
            .select(objects::object_key)
            .first::<String>(conn)
            .optional()?
            .is_some();
        if exists {
            return Ok(key);
        }

        let insert = diesel::insert_into(objects::table)
            .values((
                objects::object_key.eq(&key),
                objects::content.eq(Bytes(content.to_vec())),
                objects::byte_size.eq(content.len() as i64),
                objects::created_at.eq(Timestamp(chrono::Utc::now())),
            ))
            .execute(conn);

        match insert {
            Ok(_) => Ok(key),
            // A concurrent writer inserted the same (byte-identical) content
            // between our check and insert — harmless for a content-addressed
            // store.
            Err(diesel::result::Error::DatabaseError(
                diesel::result::DatabaseErrorKind::UniqueViolation,
                _,
            )) => Ok(key),
            Err(e) => Err(e.into()),
        }
    }

    fn delete(&self, conn: &mut DualConnection, key: &str) -> Result<(), ObjectError> {
        diesel::delete(objects::table.filter(objects::object_key.eq(key))).execute(conn)?;
        Ok(())
    }

    fn list(&self, conn: &mut DualConnection) -> Result<Vec<String>, ObjectError> {
        Ok(objects::table
            .select(objects::object_key)
            .load::<String>(conn)?)
    }
}
