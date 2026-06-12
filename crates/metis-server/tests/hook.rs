//! Session-start hook: end-to-end against a temp git repo + temp SQLite DB, plus
//! user-config parsing. Verifies the briefing context, the unregistered notice,
//! and silent behavior outside a Metis environment.

use std::path::Path;
use std::process::Command;

use metis_core::actor::Actor;
use metis_core::objects::DbBlobObjectStore;
use metis_core::service::item::{self, NewItem};
use metis_core::service::{project, repo, user};
use metis_core::{db, Pool};
use metis_server::hook;
use metis_server::user_config::UserConfig;

fn git(cwd: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .arg("-C")
        .arg(cwd)
        .args(args)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "git {args:?} failed");
}

fn init_repo(cwd: &Path, remote: &str) {
    git(cwd, &["init", "-q"]);
    git(cwd, &["remote", "add", "origin", remote]);
}

/// Seed a DB with a project + repo + an active item, return a UserConfig pointing at it.
fn seed_db(db_path: &str, remote_for_repo: &str) -> Pool {
    let pool = db::connect_and_migrate(db_path).unwrap();
    let store = DbBlobObjectStore::new();
    let mut conn = pool.get().unwrap();
    let u = user::create_user(&mut conn, "dev", "Dev", None, true).unwrap();
    let actor = Actor::human(u.id.0);
    project::create(&mut conn, "metis", "Metis", None).unwrap();
    repo::register(
        &mut conn,
        "metis",
        "core",
        vec![remote_for_repo.to_string()],
    )
    .unwrap();
    let it = item::create(
        &mut conn,
        &store,
        &actor,
        "metis",
        &NewItem {
            item_type: "task".into(),
            title: "wire the thing".into(),
            repos: vec!["core".into()],
            ..Default::default()
        },
    )
    .unwrap();
    item::transition(
        &mut conn,
        &store,
        &actor,
        &it.short_code,
        Some("todo"),
        false,
    )
    .unwrap();
    item::transition(
        &mut conn,
        &store,
        &actor,
        &it.short_code,
        Some("active"),
        false,
    )
    .unwrap();
    pool
}

#[test]
fn registered_repo_yields_briefing_context() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("metis.db").to_string_lossy().into_owned();
    // repo registered under one URL spelling; checkout uses another
    seed_db(&db_path, "git@github.com:org/metis.git");

    let checkout = tempfile::TempDir::new().unwrap();
    init_repo(checkout.path(), "https://github.com/org/metis.git");

    let cfg = UserConfig {
        database_url: Some(db_path),
        ..Default::default()
    };
    let out = hook::session_start(checkout.path(), &cfg)
        .unwrap()
        .expect("some context");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert_eq!(v["hookSpecificOutput"]["hookEventName"], "SessionStart");
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("project `metis`"), "names the project");
    assert!(ctx.contains("METIS-T-0001"), "lists the active item: {ctx}");
    assert!(ctx.contains("wire the thing"));
    assert!(ctx.contains("transition_phase"), "lists MCP tools");
}

#[test]
fn unregistered_repo_yields_notice() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("metis.db").to_string_lossy().into_owned();
    seed_db(&db_path, "git@github.com:org/metis.git");

    let checkout = tempfile::TempDir::new().unwrap();
    init_repo(checkout.path(), "git@github.com:someone/unrelated.git");

    let cfg = UserConfig {
        database_url: Some(db_path),
        ..Default::default()
    };
    let out = hook::session_start(checkout.path(), &cfg)
        .unwrap()
        .expect("notice");
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    let ctx = v["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(ctx.contains("not registered with Metis"), "got: {ctx}");
}

#[test]
fn no_config_is_silent() {
    let checkout = tempfile::TempDir::new().unwrap();
    init_repo(checkout.path(), "git@github.com:org/metis.git");
    let out = hook::session_start(checkout.path(), &UserConfig::default()).unwrap();
    assert!(out.is_none(), "no database_url → silent");
}

#[test]
fn non_git_dir_is_silent() {
    let dir = tempfile::TempDir::new().unwrap();
    let db_path = dir.path().join("metis.db").to_string_lossy().into_owned();
    seed_db(&db_path, "git@github.com:org/metis.git");
    let plain = tempfile::TempDir::new().unwrap(); // no git init
    let cfg = UserConfig {
        database_url: Some(db_path),
        ..Default::default()
    };
    let out = hook::session_start(plain.path(), &cfg).unwrap();
    assert!(out.is_none(), "no git remote → silent");
}

#[test]
fn user_config_parses() {
    let cfg = UserConfig::parse(
        r#"
        database_url = "postgres://metis:metis@localhost/metis"
        token = "mtk_abc"
        default_project = "metis"
        unknown_future_field = "ignored"
        "#,
    )
    .unwrap();
    assert_eq!(
        cfg.database_url.as_deref(),
        Some("postgres://metis:metis@localhost/metis")
    );
    assert_eq!(cfg.token.as_deref(), Some("mtk_abc"));
    assert_eq!(cfg.default_project.as_deref(), Some("metis"));
}
