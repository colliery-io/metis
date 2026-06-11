//! Service-layer integration tests on both backends: item lifecycle, short-code
//! allocation, body history, transition gating, queries, and audit events.

use diesel::prelude::*;
use diesel_dualdb::DualConnection;
use metis_core::actor::Actor;
use metis_core::db;
use metis_core::objects::DbBlobObjectStore;
use metis_core::service::item::{self, ItemEdit, NewItem};
use metis_core::service::query::{self, ItemFilter};
use metis_core::service::{project, user};

fn setup(conn: &mut DualConnection) -> (DbBlobObjectStore, Actor) {
    db::reset(conn).unwrap();
    let store = DbBlobObjectStore::new();
    let u = user::create_user(conn, "dylan", "Dylan", None, true).unwrap();
    project::create(conn, "metis", "Metis", None).unwrap();
    (store, Actor::agent(u.id.0, "claude-code"))
}

#[diesel_dualdb::test(pg, sqlite)]
fn create_allocates_sequential_short_codes(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);

    let mk = |conn: &mut DualConnection, title: &str| {
        item::create(
            conn,
            &store,
            &actor,
            "metis",
            &NewItem {
                item_type: "task".into(),
                title: title.into(),
                body: format!("# {title}"),
                exit_criteria: vec!["done".into()],
                ..Default::default()
            },
        )
        .unwrap()
    };

    let a = mk(conn, "first");
    let b = mk(conn, "second");
    assert_eq!(a.short_code, "METIS-T-0001");
    assert_eq!(b.short_code, "METIS-T-0002");
    assert_eq!(a.phase, "backlog"); // task initial phase
    assert_eq!(a.body, "# first");

    // Per-(project,type) sequence: an initiative starts its own numbering.
    let i = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "initiative".into(),
            title: "init".into(),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(i.short_code, "METIS-I-0001");
}

#[diesel_dualdb::test(pg, sqlite)]
fn edit_body_records_history_and_dedups(conn: &mut DualConnection) {
    use metis_core::objects::{content_key, ObjectStore};
    let (store, actor) = setup(conn);

    let item = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "t".into(),
            body: "v1".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let v1_key = content_key(b"v1");
    assert_eq!(item.content_key.as_deref(), Some(v1_key.as_str()));

    let edited = item::edit(
        conn,
        &store,
        &actor,
        &item.short_code,
        &ItemEdit {
            body: Some("v2".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(edited.body, "v2");
    assert_eq!(
        edited.content_key.as_deref(),
        Some(content_key(b"v2").as_str())
    );

    // The prior body object is still retrievable (history) — recorded in events,
    // so GC keeps it.
    assert_eq!(
        store.get(conn, &v1_key).unwrap().as_deref(),
        Some(&b"v1"[..]),
        "previous body must remain retrievable"
    );
}

#[diesel_dualdb::test(pg, sqlite)]
fn optimistic_concurrency_precondition(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    let item = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "t".into(),
            body: "v1".into(),
            ..Default::default()
        },
    )
    .unwrap();

    // Stale expected key -> Conflict.
    let err = item::edit(
        conn,
        &store,
        &actor,
        &item.short_code,
        &ItemEdit {
            body: Some("v2".into()),
            expected_content_key: Some(Some("deadbeef".into())),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        metis_core::service::ServiceError::Conflict(_)
    ));

    // Correct expected key -> succeeds.
    item::edit(
        conn,
        &store,
        &actor,
        &item.short_code,
        &ItemEdit {
            body: Some("v2".into()),
            expected_content_key: Some(item.content_key.clone()),
            ..Default::default()
        },
    )
    .unwrap();
}

#[diesel_dualdb::test(pg, sqlite)]
fn transition_enforces_gate_and_force_overrides(conn: &mut DualConnection) {
    use metis_core::workflow::transition::TransitionError;
    let (store, actor) = setup(conn);
    let item = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "t".into(),
            exit_criteria: vec!["ship it".into()],
            ..Default::default()
        },
    )
    .unwrap();

    // backlog -> todo blocked: exit criterion unmet.
    let err =
        item::transition(conn, &store, &actor, &item.short_code, Some("todo"), false).unwrap_err();
    assert!(matches!(
        err,
        metis_core::service::ServiceError::Transition(TransitionError::ExitCriteriaNotMet { .. })
    ));

    // Mark the criterion met, then it advances.
    item::edit(
        conn,
        &store,
        &actor,
        &item.short_code,
        &ItemEdit {
            exit_criteria: Some(vec![metis_core::models::ExitCriterion {
                ordinal: 0,
                text: "ship it".into(),
                met: true,
            }]),
            ..Default::default()
        },
    )
    .unwrap();
    let moved =
        item::transition(conn, &store, &actor, &item.short_code, Some("todo"), false).unwrap();
    assert_eq!(moved.phase, "todo");

    // force skips ahead ignoring the gate.
    let forced = item::transition(
        conn,
        &store,
        &actor,
        &item.short_code,
        Some("completed"),
        true,
    )
    .unwrap();
    assert_eq!(forced.phase, "completed");
}

#[diesel_dualdb::test(pg, sqlite)]
fn parent_constraints_enforced(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);

    // initiative requires a vision parent or root (allow_root true) — root ok.
    let init = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "initiative".into(),
            title: "I".into(),
            ..Default::default()
        },
    )
    .unwrap();

    // task may parent under an initiative.
    let task = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "T".into(),
            parent: Some(init.short_code.clone()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(task.parent.as_deref(), Some(init.short_code.as_str()));

    // task may NOT parent under another task.
    let err = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "bad".into(),
            parent: Some(task.short_code.clone()),
            ..Default::default()
        },
    )
    .unwrap_err();
    assert!(matches!(
        err,
        metis_core::service::ServiceError::Validation(_)
    ));
}

#[diesel_dualdb::test(pg, sqlite)]
fn query_filters(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    for (ty, title, tags) in [
        ("task", "a", vec!["x"]),
        ("task", "b", vec!["y"]),
        ("bug", "c", vec!["x"]),
    ] {
        item::create(
            conn,
            &store,
            &actor,
            "metis",
            &NewItem {
                item_type: ty.into(),
                title: title.into(),
                tags: tags.into_iter().map(String::from).collect(),
                ..Default::default()
            },
        )
        .unwrap();
    }

    let all = query::list(
        conn,
        &ItemFilter {
            project: Some("metis".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(all.len(), 3);

    let tasks = query::list(
        conn,
        &ItemFilter {
            project: Some("metis".into()),
            item_type: Some("task".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(tasks.len(), 2);

    let tagged_x = query::list(
        conn,
        &ItemFilter {
            tag: Some("x".into()),
            ..Default::default()
        },
    )
    .unwrap();
    assert_eq!(tagged_x.len(), 2);
}

#[diesel_dualdb::test(pg, sqlite)]
fn mutations_write_events_with_actor(conn: &mut DualConnection) {
    use metis_core::schema::events;
    let (store, actor) = setup(conn);
    let item = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "t".into(),
            ..Default::default()
        },
    )
    .unwrap();
    item::archive(conn, &store, &actor, &item.short_code).unwrap();

    // created + archived events, both carrying the agent attribution.
    let agents: Vec<Option<String>> = events::table
        .select(events::actor_agent)
        .load::<Option<String>>(conn)
        .unwrap();
    assert!(agents.len() >= 2);
    assert!(agents.iter().all(|a| a.as_deref() == Some("claude-code")));
}

#[diesel_dualdb::test(pg, sqlite)]
fn links_between_items(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    let a = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "a".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let b = item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "b".into(),
            ..Default::default()
        },
    )
    .unwrap();

    item::add_link(conn, &actor, &a.short_code, "blocks", &b.short_code).unwrap();
    let detail = item::get(conn, &store, &a.short_code).unwrap();
    assert_eq!(detail.links.len(), 1);
    assert_eq!(detail.links[0].kind, "blocks");
    assert_eq!(detail.links[0].target, b.short_code);

    // unknown kind rejected
    assert!(item::add_link(conn, &actor, &a.short_code, "frobnicates", &b.short_code).is_err());
}
