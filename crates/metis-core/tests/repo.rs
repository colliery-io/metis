//! Repo URL normalization (unit) and registry/resolve/briefing on both backends.

use diesel_dualdb::DualConnection;
use metis_core::actor::Actor;
use metis_core::db;
use metis_core::objects::DbBlobObjectStore;
use metis_core::repo::normalize_git_url;
use metis_core::service::item::{self, NewItem};
use metis_core::service::{project, repo, user};

#[test]
fn normalization_collapses_url_forms() {
    let canonical = "github.com/org/Repo";
    for url in [
        "git@github.com:org/Repo.git",
        "git@github.com:org/Repo",
        "https://github.com/org/Repo.git",
        "https://github.com/org/Repo",
        "https://user@github.com/org/Repo.git",
        "ssh://git@github.com/org/Repo",
        "ssh://git@github.com:22/org/Repo.git",
        "git://github.com/org/Repo.git",
        "github.com/org/Repo",
        "  https://github.com/org/Repo/  ",
    ] {
        assert_eq!(normalize_git_url(url), canonical, "failed for {url:?}");
    }

    // host is lowercased; distinct repos stay distinct
    assert_eq!(
        normalize_git_url("git@GitHub.com:org/repo"),
        "github.com/org/repo"
    );
    assert_ne!(
        normalize_git_url("git@github.com:org/a"),
        normalize_git_url("git@github.com:org/b")
    );
}

fn setup(conn: &mut DualConnection) -> (DbBlobObjectStore, Actor) {
    db::reset(conn).unwrap();
    let u = user::create_user(conn, "dev", "Dev", None, true).unwrap();
    project::create(conn, "metis", "Metis", None).unwrap();
    (DbBlobObjectStore::new(), Actor::human(u.id.0))
}

#[diesel_dualdb::test(pg, sqlite)]
fn register_and_resolve(conn: &mut DualConnection) {
    setup(conn);
    repo::register(
        conn,
        "metis",
        "core",
        vec!["git@github.com:colliery-io/metis.git".into()],
    )
    .unwrap();

    // any spelling of the same repo resolves
    let ctx = repo::resolve(conn, "https://github.com/colliery-io/metis").unwrap();
    let ctx = ctx.expect("should resolve");
    assert_eq!(ctx.project, "metis");
    assert_eq!(ctx.repo, "core");

    // unregistered remote -> None
    assert!(repo::resolve(conn, "git@github.com:someone/else.git")
        .unwrap()
        .is_none());

    // list
    assert_eq!(repo::list(conn, "metis").unwrap().len(), 1);
}

#[diesel_dualdb::test(pg, sqlite)]
fn briefing_buckets_by_phase(conn: &mut DualConnection) {
    let (store, actor) = setup(conn);
    repo::register(
        conn,
        "metis",
        "core",
        vec!["git@github.com:org/metis.git".into()],
    )
    .unwrap();

    let mk = |conn: &mut DualConnection, title: &str| {
        item::create(
            conn,
            &store,
            &actor,
            "metis",
            &NewItem {
                item_type: "task".into(),
                title: title.into(),
                repos: vec!["core".into()],
                ..Default::default()
            },
        )
        .unwrap()
    };

    // one stays in backlog, one -> todo (ready), one -> active
    mk(conn, "backlog one");
    let r = mk(conn, "ready one");
    let a = mk(conn, "active one");
    item::transition(conn, &store, &actor, &r.short_code, Some("todo"), false).unwrap();
    item::transition(conn, &store, &actor, &a.short_code, Some("todo"), false).unwrap();
    item::transition(conn, &store, &actor, &a.short_code, Some("active"), false).unwrap();

    // an item NOT touching the repo must not appear
    item::create(
        conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "unrelated".into(),
            ..Default::default()
        },
    )
    .unwrap();

    let b = repo::briefing(conn, "core").unwrap();
    assert_eq!(b.project, "metis");
    assert_eq!(b.repo, "core");
    assert_eq!(b.active.len(), 1, "one active");
    assert_eq!(b.active[0].short_code, a.short_code);
    assert_eq!(b.ready.len(), 1, "one ready");
    assert_eq!(b.ready[0].short_code, r.short_code);
}
