//! The filesystem object store: content as sharded files under a root.
//!
//! Layout: `<root>/<ab>/<full-key>` where `ab` is the first two hex chars of
//! the key (so a directory never holds the whole corpus). Writes go to a temp
//! file in the same shard directory and are atomically `rename`d into place, so
//! a reader never sees a partial object. The connection argument is ignored.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use diesel_dualdb::DualConnection;

use super::{content_key, ObjectError, ObjectStore};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Stores objects as files under `root`, sharded by key prefix.
#[derive(Debug, Clone)]
pub struct FsObjectStore {
    root: PathBuf,
}

impl FsObjectStore {
    /// Create the store, ensuring `root` exists.
    pub fn new(root: PathBuf) -> Result<Self, ObjectError> {
        std::fs::create_dir_all(&root)?;
        Ok(Self { root })
    }

    fn shard_dir(&self, key: &str) -> PathBuf {
        self.root.join(&key[..2])
    }

    fn object_path(&self, key: &str) -> PathBuf {
        self.shard_dir(key).join(key)
    }
}

impl ObjectStore for FsObjectStore {
    fn get(&self, _conn: &mut DualConnection, key: &str) -> Result<Option<Vec<u8>>, ObjectError> {
        match std::fs::read(self.object_path(key)) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn put(&self, _conn: &mut DualConnection, content: &[u8]) -> Result<String, ObjectError> {
        let key = content_key(content);
        let final_path = self.object_path(&key);
        // Immutable content-addressed store: if it already exists, it is byte
        // identical — nothing to do.
        if final_path.exists() {
            return Ok(key);
        }
        let shard = self.shard_dir(&key);
        std::fs::create_dir_all(&shard)?;

        let tmp = shard.join(format!(
            ".tmp-{}-{}",
            std::process::id(),
            TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&tmp, content)?;
        // Atomic publish. If a concurrent writer beat us to it, rename still
        // yields the same correct bytes.
        std::fs::rename(&tmp, &final_path)?;
        Ok(key)
    }

    fn delete(&self, _conn: &mut DualConnection, key: &str) -> Result<(), ObjectError> {
        match std::fs::remove_file(self.object_path(key)) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }

    fn list(&self, _conn: &mut DualConnection) -> Result<Vec<String>, ObjectError> {
        let mut keys = Vec::new();
        collect_keys(&self.root, &mut keys)?;
        Ok(keys)
    }
}

/// Walk shard directories collecting object file names (skipping temp files).
fn collect_keys(root: &Path, out: &mut Vec<String>) -> Result<(), ObjectError> {
    let shards = match std::fs::read_dir(root) {
        Ok(rd) => rd,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(e.into()),
    };
    for shard in shards {
        let shard = shard?;
        if !shard.file_type()?.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(shard.path())? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                if !name.starts_with(".tmp-") {
                    out.push(name.to_string());
                }
            }
        }
    }
    Ok(())
}
