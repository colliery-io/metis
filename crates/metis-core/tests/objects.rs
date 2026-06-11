//! ObjectStore conformance + GC tests.
//!
//! The conformance suite runs against both backends (db-blob and filesystem) on
//! both database backends (SQLite always; Postgres when `DUALDB_PG_URL` is set),
//! so behavior is identical regardless of where bytes land. GC and the
//! crash-window contract get dedicated tests.

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use diesel_dualdb::DualConnection;
use metis_core::db;
use metis_core::objects::{content_key, gc, DbBlobObjectStore, FsObjectStore, ObjectStore};
use tempfile::TempDir;

// ----- conformance suite ----------------------------------------------------

fn conformance(conn: &mut DualConnection, store: &dyn ObjectStore) {
    // round-trip
    let key = store.put(conn, b"hello world").unwrap();
    assert_eq!(key, content_key(b"hello world"));
    assert_eq!(
        store.get(conn, &key).unwrap().as_deref(),
        Some(&b"hello world"[..])
    );

    // idempotent put: same content -> same key, no duplicate
    let key2 = store.put(conn, b"hello world").unwrap();
    assert_eq!(key, key2);
    assert_eq!(store.list(conn).unwrap().len(), 1);

    // distinct content -> distinct key
    let other = store.put(conn, b"different").unwrap();
    assert_ne!(other, key);
    assert_eq!(store.list(conn).unwrap().len(), 2);

    // missing key
    let absent = content_key(b"nope, never stored");
    assert_eq!(store.get(conn, &absent).unwrap(), None);

    // delete present then absent (idempotent)
    store.delete(conn, &key).unwrap();
    assert_eq!(store.get(conn, &key).unwrap(), None);
    store.delete(conn, &key).unwrap(); // deleting absent is fine

    // empty content is storable and distinct from null
    let empty = store.put(conn, b"").unwrap();
    assert_eq!(store.get(conn, &empty).unwrap(), Some(Vec::new()));
}

#[diesel_dualdb::test(pg, sqlite)]
fn db_blob_conformance(conn: &mut DualConnection) {
    db::reset(conn).unwrap();
    conformance(conn, &DbBlobObjectStore::new());
}

#[diesel_dualdb::test(pg, sqlite)]
fn fs_conformance(conn: &mut DualConnection) {
    db::reset(conn).unwrap();
    let dir = TempDir::new().unwrap();
    let store = FsObjectStore::new(dir.path().join("objects")).unwrap();
    conformance(conn, &store);
}

// ----- crash-window contract ------------------------------------------------

// An object written but never referenced by a committed row is a harmless
// orphan, and GC reclaims it. (For the filesystem backend this is the literal
// crash window: object on disk, row never written.)
#[diesel_dualdb::test(pg, sqlite)]
fn fs_orphan_is_durable_then_reclaimable(conn: &mut DualConnection) {
    db::reset(conn).unwrap();
    let dir = TempDir::new().unwrap();
    let store = FsObjectStore::new(dir.path().join("objects")).unwrap();

    let orphan = store.put(conn, b"orphaned body").unwrap();
    // durable immediately, before any row references it
    assert!(store.get(conn, &orphan).unwrap().is_some());

    // no work_item or event references it -> GC reclaims it
    let report = gc(conn, &store, false).unwrap();
    assert_eq!(report.deleted, vec![orphan.clone()]);
    assert_eq!(store.get(conn, &orphan).unwrap(), None);
}

// ----- GC reachability ------------------------------------------------------

#[diesel_dualdb::test(pg, sqlite)]
fn gc_keeps_live_and_historical_drops_orphan(conn: &mut DualConnection) {
    use metis_core::schema::{events, projects, work_items};
    db::reset(conn).unwrap();

    let store = DbBlobObjectStore::new();
    let now = chrono::Utc::now();

    // Three objects: a live body, a historical body, and an orphan.
    let live = store.put(conn, b"current body").unwrap();
    let historical = store.put(conn, b"previous body").unwrap();
    let orphan = store.put(conn, b"unreferenced").unwrap();

    // A project and a work item whose content_key is the live object.
    let project_id = uuid::Uuid::new_v4();
    diesel::insert_into(projects::table)
        .values((
            projects::id.eq(Uuid(project_id)),
            projects::slug.eq("p"),
            projects::name.eq("P"),
            projects::config.eq(Json(serde_json::json!({}))),
            projects::created_at.eq(Timestamp(now)),
            projects::updated_at.eq(Timestamp(now)),
        ))
        .execute(conn)
        .unwrap();

    let item_id = uuid::Uuid::new_v4();
    diesel::insert_into(work_items::table)
        .values((
            work_items::id.eq(Uuid(item_id)),
            work_items::project_id.eq(Uuid(project_id)),
            work_items::short_code.eq("P-T-0001"),
            work_items::seq.eq(1),
            work_items::item_type.eq("task"),
            work_items::phase.eq("todo"),
            work_items::title.eq("t"),
            work_items::content_key.eq(Some(&live)),
            work_items::created_by.eq(Uuid(uuid::Uuid::new_v4())),
            work_items::archived.eq(false),
            work_items::created_at.eq(Timestamp(now)),
            work_items::updated_at.eq(Timestamp(now)),
        ))
        .execute(conn)
        .unwrap();

    // An edit event recording the prior (historical) content key.
    diesel::insert_into(events::table)
        .values((
            events::id.eq(Uuid(uuid::Uuid::new_v4())),
            events::item_id.eq(Uuid(item_id)),
            events::actor_user.eq(Uuid(uuid::Uuid::new_v4())),
            events::kind.eq("edit"),
            events::payload.eq(Json(serde_json::json!({
                "prev_content_key": historical,
                "new_content_key": live,
            }))),
            events::created_at.eq(Timestamp(now)),
        ))
        .execute(conn)
        .unwrap();

    // Dry run: reports the orphan, deletes nothing.
    let dry = gc(conn, &store, true).unwrap();
    assert_eq!(dry.deleted, vec![orphan.clone()]);
    assert!(dry.dry_run);
    assert_eq!(store.list(conn).unwrap().len(), 3);

    // Real run: orphan gone, live + historical retained.
    let report = gc(conn, &store, false).unwrap();
    assert_eq!(report.deleted, vec![orphan.clone()]);
    assert_eq!(report.scanned, 3);
    assert_eq!(report.referenced, 2);
    assert_eq!(store.get(conn, &orphan).unwrap(), None);
    assert!(store.get(conn, &live).unwrap().is_some());
    assert!(store.get(conn, &historical).unwrap().is_some());
}
