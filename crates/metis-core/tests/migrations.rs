//! Verifies the generated migrations apply and the schema is usable on both
//! backends. The SQLite arm runs in-memory with no setup; the Postgres arm runs
//! only when `DUALDB_PG_URL` is set (otherwise it self-skips).

use diesel::prelude::*;
use diesel_dualdb::types::{Json, Timestamp, Uuid};
use diesel_dualdb::DualConnection;
use metis_core::db;
use metis_core::schema::projects;

#[diesel_dualdb::test(pg, sqlite)]
fn migrations_apply_and_project_round_trips(conn: &mut DualConnection) {
    db::run_migrations(conn).expect("migrations apply");

    let id = uuid::Uuid::new_v4();
    let now = chrono::Utc::now();
    let config = serde_json::json!({ "types": {} });

    diesel::insert_into(projects::table)
        .values((
            projects::id.eq(Uuid(id)),
            projects::slug.eq("acme"),
            projects::name.eq("Acme"),
            projects::config.eq(Json(config.clone())),
            projects::created_at.eq(Timestamp(now)),
            projects::updated_at.eq(Timestamp(now)),
        ))
        .execute(conn)
        .expect("insert project");

    let (slug, stored): (String, Json<serde_json::Value>) = projects::table
        .select((projects::slug, projects::config))
        .filter(projects::id.eq(Uuid(id)))
        .first(conn)
        .expect("read project back");

    assert_eq!(slug, "acme");
    assert_eq!(stored.0, config);

    // Unique constraint on slug is enforced.
    let dup = diesel::insert_into(projects::table)
        .values((
            projects::id.eq(Uuid(uuid::Uuid::new_v4())),
            projects::slug.eq("acme"),
            projects::name.eq("Acme II"),
            projects::config.eq(Json(serde_json::json!({}))),
            projects::created_at.eq(Timestamp(now)),
            projects::updated_at.eq(Timestamp(now)),
        ))
        .execute(conn);
    assert!(dup.is_err(), "duplicate slug must be rejected");
}
