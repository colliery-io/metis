//! Garbage collection for unreferenced objects.
//!
//! An object is reachable if its key appears in `work_items.content_key` (a live
//! body) or anywhere in an `events` row payload (a historical body recorded when
//! an item was edited). GC lists the store, subtracts the reachable set, and
//! deletes the rest.
//!
//! The reference scan is deliberately **conservative**: it treats any 64-hex
//! string found anywhere in any event payload as a reference, so a future event
//! shape can't accidentally make GC reclaim a still-referenced object. Deleting
//! a live object is the only real data-loss risk here, hence the conservatism
//! and the [`dry_run`](GcReport) mode.

use std::collections::BTreeSet;

use diesel::prelude::*;
use diesel_dualdb::types::Json;
use diesel_dualdb::DualConnection;

use super::{is_content_key, ObjectError, ObjectStore};
use crate::schema::{events, work_items};

/// Outcome of a GC pass.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GcReport {
    /// Total objects found in the store.
    pub scanned: usize,
    /// Objects reachable from the database.
    pub referenced: usize,
    /// Keys deleted (or that would be deleted, when `dry_run`).
    pub deleted: Vec<String>,
    /// Whether this was a dry run (nothing actually removed).
    pub dry_run: bool,
}

/// Run garbage collection over `store` using `conn` to find references.
///
/// With `dry_run = true`, nothing is deleted; the report lists what *would* be.
pub fn gc(
    conn: &mut DualConnection,
    store: &dyn ObjectStore,
    dry_run: bool,
) -> Result<GcReport, ObjectError> {
    let referenced = referenced_keys(conn)?;
    let all = store.list(conn)?;
    let scanned = all.len();

    let mut deleted = Vec::new();
    for key in all {
        if !referenced.contains(&key) {
            if !dry_run {
                store.delete(conn, &key)?;
            }
            deleted.push(key);
        }
    }

    Ok(GcReport {
        scanned,
        referenced: referenced.len(),
        deleted,
        dry_run,
    })
}

/// Collect every object key reachable from the database.
fn referenced_keys(conn: &mut DualConnection) -> Result<BTreeSet<String>, ObjectError> {
    let mut keys = BTreeSet::new();

    // Live bodies.
    let live: Vec<Option<String>> = work_items::table
        .select(work_items::content_key)
        .load::<Option<String>>(conn)?;
    for key in live.into_iter().flatten() {
        keys.insert(key);
    }

    // Historical bodies recorded in event payloads.
    let payloads: Vec<Json<serde_json::Value>> =
        events::table.select(events::payload).load(conn)?;
    for payload in payloads {
        collect_content_keys(&payload.0, &mut keys);
    }

    Ok(keys)
}

/// Recursively gather any 64-hex content keys appearing as string values.
fn collect_content_keys(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::String(s) => {
            if is_content_key(s) {
                out.insert(s.clone());
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                collect_content_keys(item, out);
            }
        }
        serde_json::Value::Object(map) => {
            for v in map.values() {
                collect_content_keys(v, out);
            }
        }
        _ => {}
    }
}
