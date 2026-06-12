//! Full-text search (the per-backend divergence) and saved views, on both
//! backends.

use diesel_dualdb::DualConnection;
use metis_core::actor::Actor;
use metis_core::db;
use metis_core::objects::DbBlobObjectStore;
use metis_core::service::item::{self, ItemEdit, NewItem};
use metis_core::service::query::{self, ItemFilter};
use metis_core::service::{project, user, view};

fn setup(conn: &mut DualConnection) -> (DbBlobObjectStore, Actor) {
    db::reset(conn).unwrap();
    let u = user::create_user(conn, "dev", "Dev", None, true).unwrap();
    project::create(conn, "metis", "Metis", None).unwrap();
    (DbBlobObjectStore::new(), Actor::human(u.id.0))
}

fn mk(
    conn: &mut DualConnection,
    store: &DbBlobObjectStore,
    actor: &Actor,
    title: &str,
    body: &str,
) -> String {
    item::create(
        conn,
        store,
        actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: title.into(),
            body: body.into(),
            ..Default::default()
        },
    )
    .unwrap()
    .short_code
}

#[diesel_dualdb::test(pg, sqlite)]
fn fts_matches_title_and_body(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    let widget = mk(
        conn,
        &store,
        &actor,
        "Widget pipeline",
        "needs a flux capacitor",
    );
    let _other = mk(conn, &store, &actor, "Unrelated chore", "paint the shed");

    // body term
    let hits = query::list(
        conn,
        &ItemFilter {
            q: Some("flux".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].short_code, widget);

    // title term
    let hits = query::list(
        conn,
        &ItemFilter {
            q: Some("pipeline".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].short_code, widget);

    // no match
    let hits = query::list(
        conn,
        &ItemFilter {
            q: Some("nonexistentxyz".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(hits.is_empty());
}

#[diesel_dualdb::test(pg, sqlite)]
fn fts_reflects_edits_and_excludes_archived(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    let sc = mk(conn, &store, &actor, "alpha", "original body");

    // edit body -> new term searchable
    item::edit(
        conn,
        &store,
        &actor,
        &sc,
        &ItemEdit {
            body: Some("rewritten kryptonite".into()),
            ..Default::default()
        },
    )
    .unwrap();
    let hits = query::list(
        conn,
        &ItemFilter {
            q: Some("kryptonite".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(hits.len(), 1);

    // archived items don't surface in search results
    item::archive(conn, &store, &actor, &sc).unwrap();
    let hits = query::list(
        conn,
        &ItemFilter {
            q: Some("kryptonite".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert!(hits.is_empty(), "archived item must not appear in search");
}

#[diesel_dualdb::test(pg, sqlite)]
fn fts_respects_structured_filters(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    // two items share a search term but differ by type
    mk(conn, &store, &actor, "shared term apple", "");
    item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "bug".into(),
            title: "shared term apple bug".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let all = query::list(
        conn,
        &ItemFilter {
            q: Some("apple".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(all.len(), 2);

    let tasks_only = query::list(
        conn,
        &ItemFilter {
            q: Some("apple".into()),
            item_type: Some("task".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(tasks_only.len(), 1);
    assert_eq!(tasks_only[0].item_type, "task");
}

#[diesel_dualdb::test(pg, sqlite)]
fn saved_views_crud_and_run(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    mk(conn, &store, &actor, "a task", "");
    item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "bug".into(),
            title: "a bug".into(),
            ..Default::default()
        },
    )
    .unwrap();

    // create a view that selects only bugs
    let v = view::create(
        conn,
        &actor,
        "metis",
        "Open bugs",
        ItemFilter {
            item_type: Some("bug".into()),
            ..Default::default()
        },
        true,
    )
    .unwrap();
    assert_eq!(v.name, "Open bugs");
    assert_eq!(v.project, "metis");

    // list shows it
    assert_eq!(view::list(conn, "metis").unwrap().len(), 1);

    // run applies the stored query (scoped to the view's project)
    let items = view::run(conn, v.id).unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0].item_type, "bug");

    // delete
    view::delete(conn, v.id).unwrap();
    assert!(view::list(conn, "metis").unwrap().is_empty());
    assert!(view::get(conn, v.id).is_err());
}
