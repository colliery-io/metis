//! Content-addressed object store for work item bodies (METIS-T-0128).
//!
//! Structure lives in the relational schema; item *content* lives here, behind
//! a small `key => bytes` abstraction with pluggable backends. Keys are the
//! SHA-256 of the content, so objects are **immutable**: editing an item writes
//! a new object and repoints `work_items.content_key`. That buys deduplication,
//! an append-only store, and body history for free (an `events` row records the
//! prior key, so any past version stays retrievable).
//!
//! ## Backends
//! - [`db_blob::DbBlobObjectStore`] — the `objects` table in the same database
//!   (default; shares the caller's transaction).
//! - [`fs::FsObjectStore`] — sharded files under a root directory.
//! - s3 — planned; [`ObjectStoreConfig::S3`] parses but is not yet implemented.
//!
//! ## Write ordering contract
//! Always write the object **before** committing the row that references it.
//! For db-blob inside a transaction this is naturally atomic; for the
//! filesystem backend the object is durable immediately. A crash between the
//! object write and the row commit leaves at most an orphaned, immutable object
//! — harmless, and reclaimed by [`gc`]. There is therefore no path that leaves
//! a row pointing at a missing object.

use diesel_dualdb::DualConnection;
use sha2::{Digest, Sha256};

pub mod db_blob;
pub mod fs;
mod gc;

pub use db_blob::DbBlobObjectStore;
pub use fs::FsObjectStore;
pub use gc::{gc, GcReport};

/// Compute the content-addressed key (lowercase hex SHA-256) for some bytes.
pub fn content_key(content: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content);
    hex_lower(&hasher.finalize())
}

/// Whether a string is a well-formed content key (64 lowercase hex chars).
pub fn is_content_key(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Errors from object-store operations.
#[derive(Debug, thiserror::Error)]
pub enum ObjectError {
    /// Filesystem I/O failed.
    #[error("object store io error: {0}")]
    Io(#[from] std::io::Error),
    /// A database operation failed (db-blob backend).
    #[error("object store db error: {0}")]
    Db(#[from] diesel::result::Error),
    /// A backend was configured but is not yet implemented.
    #[error("object store backend not yet supported: {0}")]
    Unsupported(&'static str),
}

/// A content-addressed blob store. Methods take a [`DualConnection`] so the
/// db-blob backend can share the caller's transaction; backends that don't need
/// it (filesystem, s3) ignore it.
pub trait ObjectStore {
    /// Fetch the bytes for `key`, or `None` if absent.
    fn get(&self, conn: &mut DualConnection, key: &str) -> Result<Option<Vec<u8>>, ObjectError>;

    /// Store `content`, returning its content key. Idempotent: storing the same
    /// bytes twice yields the same key and is a no-op the second time.
    fn put(&self, conn: &mut DualConnection, content: &[u8]) -> Result<String, ObjectError>;

    /// Remove `key`. Removing an absent key is not an error.
    fn delete(&self, conn: &mut DualConnection, key: &str) -> Result<(), ObjectError>;

    /// List every key currently in the store.
    fn list(&self, conn: &mut DualConnection) -> Result<Vec<String>, ObjectError>;
}

/// Declarative backend selection (e.g. from server config).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectStoreConfig {
    /// Blobs in the `objects` table (default).
    DbBlob,
    /// Blobs as sharded files under a root directory.
    Filesystem {
        /// Root directory for the object tree.
        root: std::path::PathBuf,
    },
    /// S3-compatible store (planned; not yet implemented).
    S3 {
        /// Bucket name.
        bucket: String,
    },
}

/// A boxed object store, so callers can hold any backend uniformly.
pub type BoxedObjectStore = Box<dyn ObjectStore + Send + Sync>;

impl ObjectStoreConfig {
    /// Build the configured backend. Returns [`ObjectError::Unsupported`] for
    /// backends that parse but aren't implemented yet (s3).
    pub fn build(&self) -> Result<BoxedObjectStore, ObjectError> {
        match self {
            ObjectStoreConfig::DbBlob => Ok(Box::new(DbBlobObjectStore::new())),
            ObjectStoreConfig::Filesystem { root } => {
                Ok(Box::new(FsObjectStore::new(root.clone())?))
            }
            ObjectStoreConfig::S3 { .. } => Err(ObjectError::Unsupported("s3")),
        }
    }
}

/// Lowercase hex encoding (no external dep).
fn hex_lower(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut s = String::with_capacity(bytes.len() * 2);
    for &b in bytes {
        s.push(HEX[(b >> 4) as usize] as char);
        s.push(HEX[(b & 0xf) as usize] as char);
    }
    s
}
